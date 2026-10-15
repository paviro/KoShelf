//! Reconcile readable document identities before deciding whether records disappeared.
use super::batch::IngestStats;
use super::{batch::ingest_items_with_metadata_indices, library::LibrarySyncResult};
use crate::source::scanner::CollectedItem;
use crate::{
    app::config::SiteConfig,
    pipeline::media::{self, MediaDirs},
    source::metadata::MetadataIndices,
    source::{
        koreader::{calculate_partial_md5, merge::normalize_partial_md5},
        scanner::{CollectionOptions, scan_paths},
    },
    store::{bundles::State, sqlite::repo::LibraryRepository},
};
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

/// Try sources in preference order; a failed source must not veto a usable copy.
async fn ingest_candidates(
    id: &str,
    items: &[CollectedItem],
    saved: bool,
    config: &SiteConfig,
    repo: &LibraryRepository,
    media: &MediaDirs,
    indices: std::sync::Arc<MetadataIndices>,
) -> Result<(IngestStats, bool)> {
    let mut stats = IngestStats::default();
    for item in items {
        let result = ingest_items_with_metadata_indices(
            std::slice::from_ref(item),
            config,
            repo,
            media,
            indices.clone(),
        )
        .await?;
        let succeeded = result.errors == 0
            && result.skipped_unread == 0
            && result.skipped_duplicates == 0
            && repo.get_item(id).await?.is_some_and(|i| !i.is_archived())
            && repo.find_book_path_by_id(id).await?.as_deref() == item.path.to_str();
        stats.merge(result);
        if succeeded {
            return Ok((stats, true));
        }
        if saved {
            media::remove_item_files_by_id(id, &media.files_dir)?;
            let _metadata = repo.metadata_guard().await;
            repo.set_archived(id, true).await?;
        }
    }
    Ok((stats, false))
}

pub(crate) async fn sync_locked(
    config: &SiteConfig,
    repo: &LibraryRepository,
    media: &MediaDirs,
    paths: Option<&HashSet<PathBuf>>,
) -> Result<LibrarySyncResult> {
    let store = repo.bundles.as_ref().expect("persistent sync");
    repo.configure_metadata(&config.metadata_location);
    *repo
        .runtime_config
        .write()
        .expect("runtime configuration lock") = Some(config.clone());
    let records = store.records()?;
    // Reconcile committed generations without undoing a successful source write
    // whose capture is pending: that cache still names the previous generation.
    let mut restored = 0;
    // Validate managed enumeration before making any cache changes. An unreadable
    // or corrupt capture is not an explicitly removed record.
    let present = store.present_ids()?;
    let metadata = repo.metadata_guard().await;
    for bundle in &records {
        if repo.managed_revision(&bundle.record.id).await?.as_deref()
            != Some(bundle.revision()?.as_str())
        {
            repo.project_bundle(bundle, &config.time_config).await?;
            restored += 1;
        }
    }
    drop(metadata);
    let prepared = super::inventory::collect(config, repo, paths).await?;
    let records = records
        .into_iter()
        .filter(|b| prepared.includes(&b.record.id))
        .collect::<Vec<_>>();
    let complete = prepared.complete;
    let indices = prepared.indices.clone();
    let inventory = &prepared.sources;
    let documents = prepared.documents.clone();
    let sidecar_candidates = prepared.sidecars.clone();
    let mut live = HashSet::new();
    let mut candidates: BTreeMap<String, Vec<CollectedItem>> = BTreeMap::new();
    for item in documents {
        let Some(source) = inventory.get(&item.path) else {
            continue;
        };
        let checksum = source.checksum.clone();
        let known: Vec<_> = records
            .iter()
            .filter(|b| b.record.checksum == checksum)
            .collect();
        if known.len() > 1 {
            log::warn!("Conflicting managed identities for {}", item.path.display());
            // The document is readable, even though we cannot choose its identity.
            // Preserve these records instead of treating ambiguity as absence.
            live.extend(known.iter().map(|bundle| bundle.record.id.clone()));
            continue;
        }
        let existing = known.first().map(|b| b.record.id.as_str());
        let resolution = indices.resolve(&item.path, item.format, &checksum, existing);
        let id = existing
            .map(str::to_string)
            .or_else(|| {
                resolution
                    .selected
                    .as_ref()
                    .and_then(|c| c.parse().ok())
                    .and_then(|m| {
                        m.partial_md5_checksum
                            .as_deref()
                            .and_then(normalize_partial_md5)
                    })
            })
            .unwrap_or(checksum);
        live.insert(id.clone());
        candidates.entry(id).or_default().push(item);
    }
    // Lookup no longer mutates fingerprints. Full scans must explicitly free
    // displaced paths too, before replacement workers can claim their unique key.
    for bundle in &records {
        if complete
            && bundle.record.state == State::Available
            && inventory
                .get(&bundle.record.book_path)
                .is_some_and(|source| {
                    source.identity.as_deref() != Some(bundle.record.id.as_str())
                        || source.checksum != bundle.record.checksum
                })
        {
            media::remove_item_files_by_id(&bundle.record.id, &media.files_dir)?;
            let _metadata = repo.metadata_guard().await;
            repo.set_archived(&bundle.record.id, true).await?;
            restored += 1;
        }
    }
    // Membership comes from readable ebooks and saved records. Snapshots of
    // unreadable ebooks are archived below. With neither source nor capture,
    // even an old SQLite projection must disappear; no tombstone is needed.
    for id in repo.load_all_item_ids().await? {
        if prepared.includes(&id) && !live.contains(&id) && !present.contains(&id) {
            repo.delete_item(&id).await?;
            media::remove_item_files_by_id(&id, &media.files_dir)?;
            restored += 1;
        }
    }
    // Keep the ordinary single-source batch parallel. Only duplicate identities
    // need ordered attempts, with the established source preferred.
    let mut items = Vec::new();
    let mut duplicates = Vec::new();
    for (id, mut sources) in candidates {
        if sources.len() == 1 {
            items.extend(sources);
        } else {
            log::warn!("Duplicate live identity {id}: trying sources in preference order");
            let established = repo.find_book_path_by_id(&id).await?.map(PathBuf::from);
            sources
                .sort_by_key(|item| (established.as_ref() != Some(&item.path), item.path.clone()));
            duplicates.push((id, sources));
        }
    }
    let mut stats =
        ingest_items_with_metadata_indices(&items, config, repo, media, indices.clone()).await?;
    for (id, sources) in duplicates {
        let saved = present.contains(&id);
        let (result, _) =
            ingest_candidates(&id, &sources, saved, config, repo, media, indices.clone()).await?;
        stats.merge(result);
    }
    // A blocked candidate does not make a vanished document available. Let
    // its surviving sidecar retain the established identity instead.
    for id in live.clone() {
        if let Some(path) = repo.find_book_path_by_id(&id).await?
            && matches!(std::fs::metadata(path), Err(e) if e.kind() == std::io::ErrorKind::NotFound)
        {
            live.remove(&id);
        }
    }
    let mut attempted = HashSet::new();
    for item in sidecar_candidates {
        let checksum = item.sidecar_checksum.as_deref().unwrap_or_default();
        let id = records
            .iter()
            .find(|b| b.record.checksum == checksum)
            .map_or(checksum, |b| b.record.id.as_str())
            .to_string();
        if live.contains(&id) || !attempted.insert(item.path.clone()) {
            continue;
        }
        let result = ingest_items_with_metadata_indices(
            std::slice::from_ref(&item),
            config,
            repo,
            media,
            indices.clone(),
        )
        .await?;
        // Failed reads are not evidence that a source disappeared.
        if result.errors > 0 || (result.skipped_unread == 0 && result.skipped_duplicates == 0) {
            live.insert(id);
        }
        stats.merge(result);
    }
    stats.upserted += restored;
    stats.stats_invalidated += restored;
    let mut removed = 0;
    for bundle in store.records()? {
        let id = &bundle.record.id.clone();
        if !prepared.includes(id) || !complete || live.contains(id) {
            continue;
        }
        media::remove_item_files_by_id(id, &media.files_dir)?;
        let _metadata = repo.metadata_guard().await;
        let mut current = bundle;
        if current.record.state == State::Available {
            current = store.transition(id, State::Archived)?;
            repo.project_bundle(&current, &config.time_config).await?;
            removed += 1;
        }
        if current.record.state == State::Archived {
            media::remove_item_files_by_id(id, &media.files_dir)?;
        }
    }
    if let Err(e) = store.expire() {
        log::warn!("Retired file cleanup pending: {e:#}");
    }
    // Covers are served from the media cache. Restore copies lost with the cache
    // (archived books cannot regenerate theirs from the document).
    let _metadata = repo.metadata_guard().await;
    for bundle in store.records()? {
        let dest = media.covers_dir.join(format!("{}.webp", bundle.record.id));
        if let Some(cover) = bundle.cover()
            && !dest.try_exists()?
        {
            crate::store::bundles::link_or_copy(&cover, &dest)?;
        }
    }
    Ok(LibrarySyncResult {
        changed: stats.upserted,
        removed,
        ingest_stats: Some(stats),
        ..Default::default()
    })
}

/// Confirm that both document and sidecar are absent. An incomplete inventory
/// cannot authorize deletion of the last saved capture.
pub(crate) async fn confirm_absent(
    config: &SiteConfig,
    repo: &LibraryRepository,
    id: &str,
) -> Result<bool> {
    let store = repo.bundles.as_ref().expect("persistent removal");
    let Some(bundle) = store.get(id)? else {
        return Ok(false);
    };
    if bundle.record.state != State::Archived {
        return Ok(false);
    }
    let scan = scan_paths(
        &config.library_paths,
        &CollectionOptions {
            kobo_db_path: config.kobo_db_path.clone(),
        },
        &[std::path::absolute(&store.root)?],
    )
    .await;
    let indices = repo.metadata_indices()?;
    let discovery = crate::source::sidecars::discover(
        scan.sidecars
            .iter()
            .map(|p| p.as_path())
            .chain(indices.sidecars()),
        &indices,
        &scan.items.iter().map(|i| i.path.as_path()).collect(),
    );
    if !scan.complete
        || !discovery.complete
        || discovery.sidecars.iter().any(|s| {
            s.item.sidecar_checksum.as_deref() == Some(&bundle.record.checksum)
                || s.item.sidecar_checksum.as_deref() == Some(id)
        })
    {
        return Ok(false);
    }
    let resolved = indices.resolve(
        &bundle.record.book_path,
        bundle.record.format,
        &bundle.record.checksum,
        Some(id),
    );
    if resolved.selected.is_some() || resolved.blocked || !resolved.issues.is_empty() {
        return Ok(false);
    }
    for item in scan.items.into_iter().chain(discovery.present) {
        let checksum = match calculate_partial_md5(&item.path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if checksum == bundle.record.checksum {
            return Ok(false);
        }
        // Only metadata naming this identity (selectable or conflicting) can revive it.
        if indices
            .resolve(&item.path, item.format, &checksum, None)
            .identities
            .contains(id)
        {
            return Ok(false);
        }
    }
    Ok(true)
}
