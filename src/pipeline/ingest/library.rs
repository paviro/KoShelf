use anyhow::Result;
use log::{info, warn};
use std::time::Instant;
use std::{collections::HashSet, path::PathBuf};

use crate::app::config::SiteConfig;
use crate::pipeline::ingest::batch::{IngestStats, ingest_items_with_metadata_indices};
use crate::pipeline::ingest::cleanup::delete_item_and_media;
use crate::pipeline::ingest::reconcile::build_library_sync_plan;
use crate::pipeline::media::{self, MediaDirs};
use crate::store::sqlite::repo::LibraryRepository;

/// Summary of a library sync: what changed since the last run.
#[derive(Debug, Default)]
pub(crate) struct LibrarySyncResult {
    pub unchanged: u64,
    pub changed: u64,
    pub added: u64,
    pub removed: u64,
    pub ingest_stats: Option<IngestStats>,
}

/// Sync the library DB to match the current filesystem state.
///
/// Handles both first run (empty DB -> everything is new) and subsequent
/// runs (detect added/removed/modified books via fingerprint comparison).
pub(crate) async fn sync_library(
    config: &SiteConfig,
    repo: &LibraryRepository,
    media_dirs: &MediaDirs,
) -> Result<LibrarySyncResult> {
    reconcile(config, repo, media_dirs, None).await
}

pub(crate) async fn reconcile_paths(
    config: &SiteConfig,
    repo: &LibraryRepository,
    media_dirs: &MediaDirs,
    paths: &HashSet<PathBuf>,
) -> Result<LibrarySyncResult> {
    reconcile(config, repo, media_dirs, Some(paths)).await
}

async fn reconcile(
    config: &SiteConfig,
    repo: &LibraryRepository,
    media_dirs: &MediaDirs,
    paths: Option<&HashSet<PathBuf>>,
) -> Result<LibrarySyncResult> {
    let _lifecycle = repo.lifecycle_gate.lock().await;
    if repo.bundles.is_some() {
        return super::preservation::sync_locked(config, repo, media_dirs, paths).await;
    }
    let start = Instant::now();
    let inventory = super::inventory::collect(config, repo, paths).await?;
    let stored_fingerprints = repo
        .load_all_fingerprints()
        .await?
        .into_iter()
        .filter(|fp| inventory.includes(&fp.item_id))
        .collect::<Vec<_>>();
    let complete = inventory.complete;
    let metadata_indices = inventory.indices.clone();
    let fs_items = inventory
        .documents
        .into_iter()
        .chain(inventory.sidecars)
        .collect::<Vec<_>>();
    let mut plan = build_library_sync_plan(
        &fs_items,
        &stored_fingerprints,
        metadata_indices.as_ref(),
        &media_dirs.covers_dir,
        config.is_internal_server,
    );

    plan.item_ids_to_delete.retain(|id| {
        !inventory
            .sources
            .all()
            .any(|source| source.identity.as_ref() == Some(id))
    });
    plan.removed = plan.item_ids_to_delete.len() as u64;
    if !complete {
        plan.item_ids_to_delete.clear();
        plan.removed = 0;
    }
    plan.items_to_ingest
        .sort_by_key(|item| item.sidecar_checksum.is_some());
    for link in &plan.file_links_to_sync {
        if let Err(e) = media::sync_item_file_symlink(
            &link.item_id,
            &link.format.extension_for_path(&link.book_path),
            &link.book_path,
            &media_dirs.files_dir,
        ) {
            warn!("Failed to create file symlink for {}: {}", link.item_id, e);
        }
    }

    for item_id in &plan.item_ids_to_delete {
        delete_item_and_media(
            repo,
            item_id,
            media_dirs,
            config.is_internal_server,
            "book removed from disk",
        )
        .await;
    }

    let ingest_stats = if plan.items_to_ingest.is_empty() {
        None
    } else {
        let split = plan
            .items_to_ingest
            .partition_point(|i| i.sidecar_checksum.is_none());
        let mut stats = ingest_items_with_metadata_indices(
            &plan.items_to_ingest[..split],
            config,
            repo,
            media_dirs,
            metadata_indices.clone(),
        )
        .await?;
        for item in &plan.items_to_ingest[split..] {
            stats.merge(
                ingest_items_with_metadata_indices(
                    std::slice::from_ref(item),
                    config,
                    repo,
                    media_dirs,
                    metadata_indices.clone(),
                )
                .await?,
            );
        }
        Some(stats)
    };

    if plan.items_to_ingest.is_empty() && plan.item_ids_to_delete.is_empty() {
        info!(
            "Library unchanged ({} items, checked in {} ms)",
            plan.unchanged,
            start.elapsed().as_millis(),
        );
    }

    Ok(LibrarySyncResult {
        unchanged: plan.unchanged,
        changed: plan.changed,
        added: plan.added,
        removed: plan.removed,
        ingest_stats,
    })
}
