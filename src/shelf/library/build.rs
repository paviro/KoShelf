//! Library item persistence helpers.
//!
//! `upsert_captured_item` handles writing a single library item (row +
//! annotations + capture provenance + fingerprint) to the database.  Used by the ingest pipeline
//! and by the watcher's targeted rebuild.

use anyhow::{Context, Result};

use crate::shelf::library::item_mapping::{
    capture_fingerprint_row, map_annotations_to_rows, map_item_to_row,
};
use crate::shelf::models::LibraryItem;
use crate::shelf::time_config::TimeConfig;
use crate::store::sqlite::repo::LibraryRepository;

/// Persist the reading state and its source provenance atomically.
pub(crate) async fn upsert_captured_item(
    repo: &LibraryRepository,
    item: &LibraryItem,
    captured: Option<&crate::source::metadata::CapturedMetadata>,
    checksum: &str,
    writable: bool,
    time_config: &TimeConfig,
) -> Result<()> {
    let row = map_item_to_row(item, time_config);
    let annotations = map_annotations_to_rows(&item.id, item, time_config);
    let mut fp =
        capture_fingerprint_row(item, None, time_config).context("capture book fingerprint")?;
    if let Some(captured) = captured {
        fp.metadata_path = Some(captured.path.to_string_lossy().into_owned());
        fp.metadata_size_bytes = Some(i64::try_from(captured.bytes.len())?);
        fp.metadata_modified_unix_ms = Some(i64::try_from(captured.modified_ns / 1_000_000)?);
    }
    repo.persist_captured_item(&row, &annotations, &fp, captured, checksum, writable)
        .await
}
