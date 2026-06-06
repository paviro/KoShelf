use anyhow::Result;
use log::{info, warn};
use std::sync::Arc;
use std::time::Instant;

use crate::app::config::SiteConfig;
use crate::pipeline::ingest::batch::{IngestStats, ingest_items_with_metadata_indices};
use crate::pipeline::ingest::cleanup::delete_item_and_media;
use crate::pipeline::ingest::metadata::MetadataIndices;
use crate::pipeline::ingest::reconcile::build_library_sync_plan;
use crate::pipeline::media::{self, MediaDirs};
use crate::source::scanner::{CollectionOptions, collect_paths};
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
    let start = Instant::now();
    let fs_items = collect_paths(
        &config.library_paths,
        &CollectionOptions {
            kobo_db_path: config.kobo_db_path.clone(),
        },
    )
    .await;
    let stored_fingerprints = repo.load_all_fingerprints().await?;
    let metadata_indices = Arc::new(MetadataIndices::new(&config.metadata_location)?);

    let plan = build_library_sync_plan(
        &fs_items,
        &stored_fingerprints,
        metadata_indices.as_ref(),
        &media_dirs.covers_dir,
        config.is_internal_server,
    );

    for link in &plan.file_links_to_sync {
        if let Err(e) = media::sync_item_file_symlink(
            &link.item_id,
            link.format.extension(),
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
        Some(
            ingest_items_with_metadata_indices(
                &plan.items_to_ingest,
                config,
                repo,
                media_dirs,
                metadata_indices,
            )
            .await?,
        )
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
