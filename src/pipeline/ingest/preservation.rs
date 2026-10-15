//! Reconcile readable ebook identities before deciding whether records disappeared.
use super::batch::IngestStats;
use super::sources::{Source, SourceIndex};
use super::{batch::ingest_items, library::LibrarySyncResult};
use crate::source::scanner::CollectedItem;
use crate::{
    app::config::SiteConfig,
    pipeline::media::{self, MediaDirs},
    source::metadata::MetadataIndices,
    source::{
        koreader::{calculate_partial_md5, merge::normalize_partial_md5},
        scanner::{CollectionOptions, scan_paths},
    },
    store::{
        bundles::{Bundle, State},
        sqlite::repo::LibraryRepository,
    },
};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::PathBuf;

fn source_index(repo: &LibraryRepository) -> Result<std::sync::MutexGuard<'_, SourceIndex>> {
    repo.source_index
        .lock()
        .map_err(|_| anyhow::anyhow!("source index lock poisoned"))
}

fn inspect_source(
    mut item: CollectedItem,
    records: &[Bundle],
    indices: &MetadataIndices,
) -> Result<Source> {
    item.path = std::path::absolute(&item.path)?;
    let checksum = calculate_partial_md5(&item.path)?;
    Ok(identify_source(item, checksum, records, indices))
}

fn identify_source(
    item: CollectedItem,
    checksum: String,
    records: &[Bundle],
    indices: &MetadataIndices,
) -> Source {
    let known: Vec<_> = records
        .iter()
        .filter(|b| b.record.checksum == checksum)
        .collect();
    let existing = (known.len() == 1).then(|| known[0].record.id.as_str());
    let resolution = indices.resolve(&item.path, item.format, &checksum, existing);
    let identity = if known.len() > 1 || resolution.blocked {
        None
    } else {
        Some(
            existing
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
                .unwrap_or_else(|| checksum.clone()),
        )
    };
    Source {
        item,
        checksum,
        identity,
    }
}

/// Reconcile only observed sources related to this batch. The inventory is never
/// a substitute for verifying bytes and metadata immediately before ingestion.
pub(crate) async fn reconcile_sources(
    config: &SiteConfig,
    repo: &LibraryRepository,
    media: &MediaDirs,
    changed: &[CollectedItem],
    deleted: &[String],
) -> Result<IngestStats> {
    let _lifecycle = repo.lifecycle_gate.lock().await;
    let store = repo.bundles.as_ref().expect("persistent reconciliation");
    let metadata = repo.metadata_guard().await;
    repo.configure_metadata(&config.metadata_location);
    *repo
        .runtime_config
        .write()
        .expect("runtime configuration lock") = Some(config.clone());
    let records = store.records()?;
    let indices = repo.metadata_indices()?;
    let mut affected = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for item in changed {
        paths.insert(std::path::absolute(&item.path)?);
    }
    for path in deleted {
        paths.insert(std::path::absolute(path)?);
    }
    // Snapshot old associations before registering any replacements or moves.
    for path in &paths {
        if let Some(old) = source_index(repo)?.remove(path)
            && let Some(id) = old.identity
        {
            affected.insert(id);
        }
        for bundle in &records {
            if bundle.record.book_path == *path {
                affected.insert(bundle.record.id.clone());
            }
        }
    }
    for item in changed {
        match inspect_source(item.clone(), &records, &indices) {
            Ok(source) => {
                if let Some(id) = &source.identity {
                    affected.insert(id.clone());
                }
                source_index(repo)?.insert(source);
            }
            Err(e) => log::warn!("Cannot inspect {}: {e:#}", item.path.display()),
        }
    }
    let mut stats = IngestStats::default();
    let mut visited = HashSet::new();
    let mut selections = Vec::new();
    while let Some(id) = affected.pop_first() {
        if !visited.insert(id.clone()) {
            continue;
        }
        let bundle = records.iter().find(|b| b.record.id == id);
        let candidates =
            source_index(repo)?.candidates(&id, bundle.map(|b| b.record.checksum.as_str()));
        let mut valid = Vec::new();
        for candidate in candidates {
            source_index(repo)?.remove(&candidate.item.path);
            match inspect_source(candidate.item, &records, &indices) {
                Ok(source) => {
                    if let Some(current) = &source.identity {
                        if current == &id {
                            valid.push(source.clone());
                        } else {
                            affected.insert(current.clone());
                        }
                    }
                    source_index(repo)?.insert(source);
                }
                Err(e) => log::warn!("Discarding stale source: {e:#}"),
            }
        }
        let established = repo.find_book_path_by_id(&id).await?.map(PathBuf::from);
        valid.sort_by(|a, b| {
            let a_established = established.as_ref() == Some(&a.item.path);
            let b_established = established.as_ref() == Some(&b.item.path);
            b_established
                .cmp(&a_established)
                .then(a.item.path.cmp(&b.item.path))
        });
        let displaced = established
            .as_ref()
            .is_some_and(|path| !valid.iter().any(|s| &s.item.path == path));
        // Unlink first: capture or projection failures must not expose replacement bytes.
        if displaced {
            media::remove_item_files_by_id(&id, &media.files_dir)?;
            let was_available = repo
                .get_item(&id)
                .await?
                .is_some_and(|item| !item.is_archived);
            repo.set_archived(&id, true).await?;
            if was_available {
                stats.upserted += 1;
                stats.stats_invalidated += 1;
            }
        }
        selections.push((id, valid, bundle.is_some()));
    }
    drop(metadata);
    // All displaced links are invalidated before any replacement is published.
    for (id, valid, saved) in selections {
        let items: Vec<_> = valid.into_iter().map(|source| source.item).collect();
        let (result, available) =
            ingest_candidates(&id, &items, saved, config, repo, media).await?;
        stats.merge(result);
        if !available && saved && archive_identity_locked(config, repo, &id, media).await? {
            stats.upserted += 1;
            stats.stats_invalidated += 1;
        }
    }
    Ok(stats)
}

/// Try sources in preference order; a failed source must not veto a usable copy.
async fn ingest_candidates(
    id: &str,
    items: &[CollectedItem],
    saved: bool,
    config: &SiteConfig,
    repo: &LibraryRepository,
    media: &MediaDirs,
) -> Result<(IngestStats, bool)> {
    let mut stats = IngestStats::default();
    for item in items {
        let result = ingest_items(std::slice::from_ref(item), config, repo, media).await?;
        let succeeded = result.errors == 0
            && result.skipped_unread == 0
            && result.skipped_duplicates == 0
            && repo.get_item(id).await?.is_some_and(|i| !i.is_archived)
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

pub(crate) async fn sync(
    config: &SiteConfig,
    repo: &LibraryRepository,
    media: &MediaDirs,
) -> Result<LibrarySyncResult> {
    let _lifecycle = repo.lifecycle_gate.lock().await;
    sync_locked(config, repo, media).await
}

pub(crate) async fn sync_locked(
    config: &SiteConfig,
    repo: &LibraryRepository,
    media: &MediaDirs,
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
    let scan = scan_paths(
        &config.library_paths,
        &CollectionOptions {
            kobo_db_path: config.kobo_db_path.clone(),
        },
        &[std::path::absolute(&store.root)?],
    )
    .await;
    if !scan.complete {
        log::info!("Library scan incomplete; books without readable sources will be archived");
    }
    let indices = repo.metadata_indices()?;
    let mut inventory = SourceIndex::default();
    let mut live = HashSet::new();
    let mut candidates: BTreeMap<String, Vec<CollectedItem>> = BTreeMap::new();
    for item in scan.items {
        let checksum = match calculate_partial_md5(&item.path) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Cannot read {}: {e:#}", item.path.display());
                continue;
            }
        };
        inventory.insert(identify_source(
            item.clone(),
            checksum.clone(),
            &records,
            &indices,
        ));
        let known: Vec<_> = records
            .iter()
            .filter(|b| b.record.checksum == checksum)
            .collect();
        if known.len() > 1 {
            log::warn!("Conflicting managed identities for {}", item.path.display());
            // The ebook is readable, even though we cannot choose its identity.
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
        if bundle.record.state == State::Available
            && inventory
                .get(&bundle.record.book_path)
                .is_none_or(|source| {
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
    *source_index(repo)? = inventory;
    // Membership comes from readable ebooks and saved records. Snapshots of
    // unreadable ebooks are archived below. With neither source nor capture,
    // even an old SQLite projection must disappear; no tombstone is needed.
    for id in repo.load_all_item_ids().await? {
        if !live.contains(&id) && !present.contains(&id) {
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
    let mut stats = ingest_items(&items, config, repo, media).await?;
    for (id, sources) in duplicates {
        let saved = present.contains(&id);
        let (result, _) = ingest_candidates(&id, &sources, saved, config, repo, media).await?;
        stats.merge(result);
    }
    stats.upserted += restored;
    stats.stats_invalidated += restored;
    let mut removed = 0;
    for bundle in store.records()? {
        let id = &bundle.record.id.clone();
        if live.contains(id)
            && repo
                .get_item(id)
                .await?
                .is_some_and(|item| !item.is_archived)
        {
            continue;
        }
        media::remove_item_files_by_id(id, &media.files_dir)?;
        let _metadata = repo.metadata_guard().await;
        let (mut current, recaptured) = recapture(config, repo, &indices, bundle).await?;
        if recaptured {
            stats.upserted += 1;
            stats.stats_invalidated += 1;
        }
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
    // (archived books cannot regenerate theirs from the ebook).
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

/// Refresh the saved metadata of a book whose ebook is gone. Callers hold the
/// metadata gate. Returns the current bundle and whether a new capture was saved.
async fn recapture(
    config: &SiteConfig,
    repo: &LibraryRepository,
    indices: &MetadataIndices,
    bundle: Bundle,
) -> Result<(Bundle, bool)> {
    let store = repo.bundles.as_ref().expect("persistent sync");
    let id = bundle.record.id.clone();
    let resolved = indices.resolve(
        &bundle.record.book_path,
        bundle.record.format,
        &bundle.record.checksum,
        Some(&id),
    );
    for issue in &resolved.issues {
        log::warn!("{id}: {issue}");
    }
    if let Some(capture) = resolved.selected
        && !resolved.blocked
        && bundle
            .record
            .source
            .as_ref()
            .is_none_or(|s| s.revision != capture.revision)
    {
        let mut item = bundle.item()?;
        item.koreader_metadata = Some(capture.parse()?);
        capture.verify()?;
        match store.capture(
            &item,
            &bundle.record.checksum,
            Some(&capture),
            None,
            bundle.record.state,
        ) {
            Ok(saved) => {
                repo.project_bundle(&saved, &config.time_config).await?;
                return Ok((saved, true));
            }
            Err(e) => log::warn!("Archive capture pending for {id}: {e:#}"),
        }
    }
    Ok((bundle, false))
}

/// Incremental counterpart of the archive pass in [`sync_locked`]: refresh the
/// saved metadata of archived books after a sidecar without an ebook changed.
/// Returns how many captures were saved.
pub(crate) async fn recapture_archived(
    config: &SiteConfig,
    repo: &LibraryRepository,
) -> Result<u64> {
    let Some(store) = &repo.bundles else {
        return Ok(0);
    };
    let _lifecycle = repo.lifecycle_gate.lock().await;
    let _metadata = repo.metadata_guard().await;
    let indices = repo.metadata_indices()?;
    let mut saved = 0;
    for bundle in store.records()? {
        if bundle.record.state == State::Archived
            && recapture(config, repo, &indices, bundle).await?.1
        {
            saved += 1;
        }
    }
    Ok(saved)
}

/// Caller holds the lifecycle gate. Remove links before fallible storage work.
async fn archive_identity_locked(
    config: &SiteConfig,
    repo: &LibraryRepository,
    id: &str,
    media: &MediaDirs,
) -> Result<bool> {
    media::remove_item_files_by_id(id, &media.files_dir)?;
    let _metadata = repo.metadata_guard().await;
    let mut changed = repo
        .get_item(id)
        .await?
        .is_some_and(|item| !item.is_archived);
    repo.set_archived(id, true).await?;
    let store = repo.bundles.as_ref().expect("persistent reconciliation");
    match store.get(id)? {
        Some(bundle) => {
            // The ebook can disappear in the same batch as a sidecar update.
            // Follow the live source before projecting the archived capture.
            let indices = repo.metadata_indices()?;
            let (bundle, recaptured) = recapture(config, repo, &indices, bundle).await?;
            changed |= recaptured;
            if bundle.record.state == State::Available {
                changed = true;
                let archived = store.transition(id, State::Archived)?;
                repo.project_bundle(&archived, &config.time_config).await?;
            }
        }
        None => log::warn!("No saved record for {id}; archived cached projection only"),
    }
    Ok(changed)
}

/// Reject removal only when a fresh scan finds a readable returning ebook or
/// metadata naming this identity. Offline sources do not prevent removal.
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
    for item in scan.items {
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
