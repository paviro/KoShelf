//! Library item persistence helpers.
//!
//! `upsert_single_item` handles writing a single library item (row +
//! annotations + fingerprint) to the database.  Used by the ingest pipeline
//! and by the watcher's targeted rebuild.

use anyhow::{Context, Result};
use std::path::Path;

use super::item_mapping::{capture_fingerprint_row, map_annotations_to_rows, map_item_to_row};
use crate::models::LibraryItem;
use crate::store::sqlite::repo::LibraryRepository;
use crate::time_config::TimeConfig;

/// Upsert a single library item (row + annotations + fingerprint).
///
/// `metadata_path` is the resolved KOReader metadata file path, or `None`
/// for items without metadata.
pub async fn upsert_single_item(
    repo: &LibraryRepository,
    item: &LibraryItem,
    metadata_path: Option<&Path>,
    time_config: &TimeConfig,
) -> Result<()> {
    let row = map_item_to_row(item, time_config);

    repo.upsert_item(&row)
        .await
        .context("Failed to upsert library item")?;

    let annotation_rows = map_annotations_to_rows(&item.id, item, time_config);
    repo.replace_annotations(&item.id, &annotation_rows)
        .await
        .context("Failed to replace annotations")?;

    if let Some(fp_row) = capture_fingerprint_row(item, metadata_path, time_config) {
        repo.upsert_fingerprint(&fp_row)
            .await
            .context("Failed to upsert fingerprint")?;
    }

    Ok(())
}
