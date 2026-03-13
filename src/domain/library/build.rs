//! Library build/update pipeline types and implementation.
//!
//! `LibraryBuildPipeline` orchestrates populating `library.sqlite` from
//! scanned source files.  It supports two modes:
//!
//! - **Full**: clears the database and rebuilds from scratch (used on first
//!   startup, temp-DB mode, or corruption recovery).
//! - **Incremental**: compares file fingerprints to detect added, changed, and
//!   removed items, then applies targeted upserts/deletes.

use anyhow::{Context, Result};
use log::{debug, warn};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use std::path::Path;

use super::collision::detect_collisions;
use super::item_mapping::{
    capture_fingerprint_row, derive_in_book_folder_metadata_path, map_annotations_to_rows,
    map_item_to_row,
};
use crate::infra::sources::fingerprints::{
    FileFingerprint, ItemFingerprints, ReconcileAction, classify_reconcile_action,
};
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::infra::sqlite::library_repo::rows::FingerprintRow;
use crate::models::{BookStatus, LibraryItem};
use crate::time_config::TimeConfig;

// ── Standalone upsert ────────────────────────────────────────────────────

/// Upsert a single library item (row + annotations + fingerprint).
///
/// `metadata_path` is the resolved KOReader metadata file path, or `None`
/// for items without metadata.  Used by the watcher's targeted rebuild.
pub async fn upsert_single_item(
    repo: &LibraryRepository,
    item: &LibraryItem,
    metadata_path: Option<&Path>,
    use_stable_page_metadata: bool,
    time_config: &TimeConfig,
) -> Result<()> {
    let row = map_item_to_row(item, use_stable_page_metadata, time_config);

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

// ── Data types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryBuildMode {
    #[default]
    Full,
    Incremental,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LibraryBuildResult {
    pub scanned_files: u64,
    pub upserted_items: u64,
    pub removed_items: u64,
    pub collision_count: u64,
}

// ── Pipeline ────────────────────────────────────────────────────────────

pub struct LibraryBuildPipeline<'a> {
    repo: &'a LibraryRepository,
    include_unread: bool,
    use_stable_page_metadata: bool,
    time_config: &'a TimeConfig,
}

impl<'a> LibraryBuildPipeline<'a> {
    pub fn new(
        repo: &'a LibraryRepository,
        include_unread: bool,
        use_stable_page_metadata: bool,
        time_config: &'a TimeConfig,
    ) -> Self {
        Self {
            repo,
            include_unread,
            use_stable_page_metadata,
            time_config,
        }
    }

    /// Run the build pipeline in the specified mode.
    ///
    /// `items` should be the raw scan result from `scan_library()`.
    pub async fn build(
        &self,
        mode: LibraryBuildMode,
        items: Vec<LibraryItem>,
    ) -> Result<LibraryBuildResult> {
        let scanned_files = items.len() as u64;
        let items = self.filter_unread(items);

        match mode {
            LibraryBuildMode::Full => self.build_full(scanned_files, items).await,
            LibraryBuildMode::Incremental => self.build_incremental(scanned_files, items).await,
        }
    }

    // ── Full build ──────────────────────────────────────────────────────

    async fn build_full(
        &self,
        scanned_files: u64,
        items: Vec<LibraryItem>,
    ) -> Result<LibraryBuildResult> {
        debug!(
            "Running full library build ({} items after filtering)",
            items.len()
        );

        self.repo
            .clear_all()
            .await
            .context("Failed to clear library database before full build")?;

        let now = self.time_config.now_rfc3339();
        let collision_result = detect_collisions(items, &now);
        let collision_count = collision_result.diagnostics.len() as u64;

        if collision_count > 0 {
            warn!(
                "Detected {} canonical-ID collisions during library build",
                collision_count
            );
        }

        // Persist collision diagnostics.
        for diag in &collision_result.diagnostics {
            self.repo
                .upsert_collision_diagnostic(diag)
                .await
                .context("Failed to persist collision diagnostic")?;
        }

        let mut upserted = 0u64;
        for item in &collision_result.winners {
            self.upsert_item(item).await?;
            upserted += 1;
        }

        debug!(
            "Full library build complete: {} upserted, {} collisions",
            upserted, collision_count
        );

        Ok(LibraryBuildResult {
            scanned_files,
            upserted_items: upserted,
            removed_items: 0,
            collision_count,
        })
    }

    // ── Incremental build ───────────────────────────────────────────────

    async fn build_incremental(
        &self,
        scanned_files: u64,
        items: Vec<LibraryItem>,
    ) -> Result<LibraryBuildResult> {
        debug!(
            "Running incremental library build ({} items after filtering)",
            items.len()
        );

        // Load stored fingerprints and index by book_path for lookup.
        let stored_fps = self
            .repo
            .load_all_fingerprints()
            .await
            .context("Failed to load stored fingerprints")?;

        let stored_by_path: HashMap<String, FingerprintRow> = stored_fps
            .into_iter()
            .map(|fp| (fp.book_path.clone(), fp))
            .collect();

        // Track which stored paths we've seen so we can detect removals.
        let mut seen_paths: HashSet<String> = HashSet::new();
        let mut upserted = 0u64;

        // First pass: collision detection on the full item set.
        let now = self.time_config.now_rfc3339();
        let collision_result = detect_collisions(items, &now);
        let collision_count = collision_result.diagnostics.len() as u64;

        // Update collision diagnostics.
        self.repo
            .clear_collision_diagnostics()
            .await
            .context("Failed to clear stale collision diagnostics")?;
        for diag in &collision_result.diagnostics {
            self.repo
                .upsert_collision_diagnostic(diag)
                .await
                .context("Failed to persist collision diagnostic")?;
        }

        // Second pass: reconcile each winning item against stored fingerprints.
        for item in &collision_result.winners {
            let book_path = item.file_path.to_string_lossy().to_string();
            seen_paths.insert(book_path.clone());

            let current_fp = self.capture_item_fingerprints(item);
            let previous_fp = stored_by_path
                .get(&book_path)
                .map(fingerprint_row_to_item_fingerprints);

            let action = classify_reconcile_action(previous_fp.as_ref(), current_fp.as_ref());

            match action {
                ReconcileAction::Added | ReconcileAction::Reparse(_) => {
                    self.upsert_item(item).await?;
                    upserted += 1;
                }
                ReconcileAction::Unchanged => {}
                ReconcileAction::Removed => {
                    // Shouldn't happen for items we just scanned, but handle gracefully.
                    warn!(
                        "Unexpected Removed action for scanned item {:?}",
                        item.file_path
                    );
                }
            }
        }

        // Third pass: remove items whose stored book_path wasn't seen in the scan.
        let mut removed = 0u64;
        for (stored_path, fp_row) in &stored_by_path {
            if !seen_paths.contains(stored_path.as_str()) {
                self.repo
                    .delete_item(&fp_row.item_id)
                    .await
                    .context("Failed to delete removed item")?;
                removed += 1;
            }
        }

        if removed > 0 {
            debug!("Removed {} items no longer present in library", removed);
        }

        debug!(
            "Incremental library build complete: {} upserted, {} removed, {} collisions",
            upserted, removed, collision_count
        );

        Ok(LibraryBuildResult {
            scanned_files,
            upserted_items: upserted,
            removed_items: removed,
            collision_count,
        })
    }

    // ── Shared helpers ──────────────────────────────────────────────────

    /// Upsert a single item: item row, annotations, and fingerprint.
    async fn upsert_item(&self, item: &LibraryItem) -> Result<()> {
        let metadata_path = derive_in_book_folder_metadata_path(item);
        upsert_single_item(
            self.repo,
            item,
            metadata_path.as_deref(),
            self.use_stable_page_metadata,
            self.time_config,
        )
        .await
    }

    /// Filter items based on the `include_unread` setting.
    fn filter_unread(&self, items: Vec<LibraryItem>) -> Vec<LibraryItem> {
        items
            .into_iter()
            .filter(|item| {
                if item.koreader_metadata.is_some() {
                    return true;
                }
                self.include_unread && item.status() == BookStatus::Unknown
            })
            .collect()
    }

    /// Capture current fingerprints for an item (book file + optional metadata file).
    fn capture_item_fingerprints(&self, item: &LibraryItem) -> Option<ItemFingerprints> {
        let book_fp = FileFingerprint::capture(&item.file_path).ok()?;
        let metadata_path = derive_in_book_folder_metadata_path(item);
        let metadata_fp =
            metadata_path.and_then(|p| FileFingerprint::capture_optional(&p).ok().flatten());

        Some(ItemFingerprints {
            book_file: book_fp,
            metadata_file: metadata_fp,
        })
    }
}

/// Convert a stored `FingerprintRow` back to `ItemFingerprints` for comparison.
fn fingerprint_row_to_item_fingerprints(row: &FingerprintRow) -> ItemFingerprints {
    ItemFingerprints {
        book_file: FileFingerprint {
            path: PathBuf::from(&row.book_path),
            size_bytes: row.book_size_bytes as u64,
            modified_unix_ms: row.book_modified_unix_ms as u64,
        },
        metadata_file: row.metadata_path.as_ref().map(|path| FileFingerprint {
            path: PathBuf::from(path),
            size_bytes: row.metadata_size_bytes.unwrap_or(0) as u64,
            modified_unix_ms: row.metadata_modified_unix_ms.unwrap_or(0) as u64,
        }),
    }
}
