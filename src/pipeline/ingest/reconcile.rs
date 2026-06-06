use log::warn;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::pipeline::ingest::metadata::{MetadataIndices, locate_metadata_path};
use crate::pipeline::media;
use crate::shelf::models::LibraryItemFormat;
use crate::source::scanner::CollectedItem;
use crate::source::{
    FileFingerprint, ItemFingerprints, ReconcileAction, classify_reconcile_action,
};
use crate::store::sqlite::repo::rows::FingerprintRow;

#[derive(Debug, Default)]
pub(super) struct LibrarySyncPlan {
    pub unchanged: u64,
    pub changed: u64,
    pub added: u64,
    pub removed: u64,
    pub items_to_ingest: Vec<CollectedItem>,
    pub item_ids_to_delete: Vec<String>,
    pub file_links_to_sync: Vec<FileLinkSync>,
}

#[derive(Debug, Clone)]
pub(super) struct FileLinkSync {
    pub item_id: String,
    pub format: LibraryItemFormat,
    pub book_path: PathBuf,
}

pub(super) fn build_library_sync_plan(
    fs_items: &[CollectedItem],
    stored_fingerprints: &[FingerprintRow],
    metadata_indices: &MetadataIndices,
    covers_dir: &Path,
    is_internal_server: bool,
) -> LibrarySyncPlan {
    let stored_by_book_path: HashMap<&str, _> = stored_fingerprints
        .iter()
        .map(|fp| (fp.book_path.as_str(), fp))
        .collect();

    let fs_path_set: HashSet<String> = fs_items
        .iter()
        .map(|item| item.path.to_string_lossy().into_owned())
        .collect();
    let fs_item_by_path: HashMap<String, CollectedItem> = fs_items
        .iter()
        .cloned()
        .map(|item| (item.path.to_string_lossy().into_owned(), item))
        .collect();

    let mut plan = LibrarySyncPlan::default();

    for fp in stored_fingerprints {
        if !fs_path_set.contains(&fp.book_path) {
            plan.item_ids_to_delete.push(fp.item_id.clone());
            plan.removed += 1;
            continue;
        }

        let book_path = Path::new(&fp.book_path);

        let current_book_fp = match FileFingerprint::capture(book_path) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to capture fingerprint for {:?}: {}", book_path, e);
                push_existing_item(&mut plan.items_to_ingest, &fs_item_by_path, fp, book_path);
                plan.changed += 1;
                continue;
            }
        };

        let current_metadata_path = if let Some(ref stored_meta) = fp.metadata_path {
            Some(PathBuf::from(stored_meta))
        } else {
            fs_item_by_path
                .get(&fp.book_path)
                .map(|item| item.format)
                .or_else(|| LibraryItemFormat::from_path(book_path))
                .and_then(|fmt| locate_metadata_path(metadata_indices, book_path, fmt))
        };

        let current_metadata_fp = match &current_metadata_path {
            Some(meta_path) => match FileFingerprint::capture_optional(meta_path) {
                Ok(fp) => fp,
                Err(e) => {
                    warn!(
                        "Failed to capture metadata fingerprint {:?}: {}",
                        meta_path, e
                    );
                    None
                }
            },
            None => None,
        };

        let current = ItemFingerprints {
            book_file: current_book_fp,
            metadata_file: current_metadata_fp,
        };
        let stored = stored_item_fingerprints(fp, book_path);
        let metadata_appeared = fp.metadata_path.is_none() && current.metadata_file.is_some();

        match classify_reconcile_action(Some(&stored), Some(&current)) {
            ReconcileAction::Unchanged if !metadata_appeared => {
                let cover_path = covers_dir.join(format!("{}.webp", fp.item_id));
                if media::cover_needs_generation(book_path, &cover_path) {
                    push_existing_item(&mut plan.items_to_ingest, &fs_item_by_path, fp, book_path);
                    plan.changed += 1;
                } else {
                    if is_internal_server
                        && let Some(format) = fs_item_by_path
                            .get(&fp.book_path)
                            .map(|item| item.format)
                            .or_else(|| LibraryItemFormat::from_path(book_path))
                    {
                        plan.file_links_to_sync.push(FileLinkSync {
                            item_id: fp.item_id.clone(),
                            format,
                            book_path: book_path.to_path_buf(),
                        });
                    }
                    plan.unchanged += 1;
                }
            }
            ReconcileAction::Unchanged | ReconcileAction::Reparse(_) => {
                push_existing_item(&mut plan.items_to_ingest, &fs_item_by_path, fp, book_path);
                plan.changed += 1;
            }
            _ => {
                plan.unchanged += 1;
            }
        }
    }

    for item in fs_items {
        let path_str = item.path.to_string_lossy();
        if !stored_by_book_path.contains_key(path_str.as_ref()) {
            plan.items_to_ingest.push(item.clone());
            plan.added += 1;
        }
    }

    plan
}

fn push_existing_item(
    items_to_ingest: &mut Vec<CollectedItem>,
    fs_item_by_path: &HashMap<String, CollectedItem>,
    fp: &FingerprintRow,
    book_path: &Path,
) {
    if let Some(item) = fs_item_by_path.get(&fp.book_path) {
        items_to_ingest.push(item.clone());
    } else if let Some(item) = regular_collected_item(book_path.to_path_buf()) {
        items_to_ingest.push(item);
    }
}

fn regular_collected_item(path: PathBuf) -> Option<CollectedItem> {
    LibraryItemFormat::from_path(&path).map(|format| CollectedItem {
        path,
        format,
        kobo_hints: None,
    })
}

fn stored_item_fingerprints(fp: &FingerprintRow, book_path: &Path) -> ItemFingerprints {
    let stored_book_fp = FileFingerprint {
        path: book_path.to_path_buf(),
        size_bytes: fp.book_size_bytes as u64,
        modified_unix_ms: fp.book_modified_unix_ms as u64,
    };
    let stored_metadata_fp = match (
        &fp.metadata_path,
        fp.metadata_size_bytes,
        fp.metadata_modified_unix_ms,
    ) {
        (Some(meta_path), Some(size), Some(modified)) => Some(FileFingerprint {
            path: PathBuf::from(meta_path),
            size_bytes: size as u64,
            modified_unix_ms: modified as u64,
        }),
        _ => None,
    };

    ItemFingerprints {
        book_file: stored_book_fp,
        metadata_file: stored_metadata_fp,
    }
}

#[cfg(test)]
mod tests {
    use super::build_library_sync_plan;
    use crate::pipeline::ingest::metadata::MetadataIndices;
    use crate::shelf::models::LibraryItemFormat;
    use crate::source::FileFingerprint;
    use crate::source::scanner::{CollectedItem, MetadataLocation};
    use crate::store::sqlite::repo::rows::FingerprintRow;
    use std::path::Path;

    fn collected(path: &Path) -> CollectedItem {
        CollectedItem {
            path: path.to_path_buf(),
            format: LibraryItemFormat::Epub,
            kobo_hints: None,
        }
    }

    fn stored_row(item_id: &str, book_fp: &FileFingerprint) -> FingerprintRow {
        FingerprintRow {
            item_id: item_id.to_string(),
            book_path: book_fp.path.to_string_lossy().into_owned(),
            book_size_bytes: book_fp.size_bytes as i64,
            book_modified_unix_ms: book_fp.modified_unix_ms as i64,
            metadata_path: None,
            metadata_size_bytes: None,
            metadata_modified_unix_ms: None,
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn reconcile_unchanged_item_is_ignored() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join("Book.epub");
        std::fs::write(&book_path, b"book").expect("book write");
        let book_fp = FileFingerprint::capture(&book_path).expect("book fp");
        let cover_dir = dir.path().join("covers");
        std::fs::create_dir_all(&cover_dir).expect("cover dir");
        std::fs::write(
            cover_dir.join("0123456789abcdef0123456789abcdef.webp"),
            b"cover",
        )
        .expect("cover write");
        let indices = MetadataIndices::new(&MetadataLocation::InBookFolder).expect("indices");

        let plan = build_library_sync_plan(
            &[collected(&book_path)],
            &[stored_row("0123456789abcdef0123456789abcdef", &book_fp)],
            &indices,
            &cover_dir,
            false,
        );

        assert_eq!(plan.unchanged, 1);
        assert!(plan.items_to_ingest.is_empty());
        assert!(plan.item_ids_to_delete.is_empty());
    }

    #[test]
    fn reconcile_changed_book_file_schedules_ingest() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join("Book.epub");
        std::fs::write(&book_path, b"book").expect("book write");
        let mut row = stored_row(
            "0123456789abcdef0123456789abcdef",
            &FileFingerprint::capture(&book_path).expect("book fp"),
        );
        row.book_size_bytes += 1;
        let cover_dir = dir.path().join("covers");
        std::fs::create_dir_all(&cover_dir).expect("cover dir");
        let indices = MetadataIndices::new(&MetadataLocation::InBookFolder).expect("indices");

        let plan = build_library_sync_plan(
            &[collected(&book_path)],
            &[row],
            &indices,
            &cover_dir,
            false,
        );

        assert_eq!(plan.changed, 1);
        assert_eq!(plan.items_to_ingest.len(), 1);
        assert_eq!(plan.items_to_ingest[0].path, book_path);
    }

    #[test]
    fn reconcile_metadata_appeared_after_no_stored_metadata_schedules_ingest() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join("Book.epub");
        std::fs::write(&book_path, b"book").expect("book write");
        let metadata_path = dir.path().join("Book.sdr").join("metadata.epub.lua");
        std::fs::create_dir_all(metadata_path.parent().expect("metadata parent"))
            .expect("metadata dir");
        std::fs::write(&metadata_path, "return {}\n").expect("metadata write");

        let row = stored_row(
            "0123456789abcdef0123456789abcdef",
            &FileFingerprint::capture(&book_path).expect("book fp"),
        );
        let cover_dir = dir.path().join("covers");
        std::fs::create_dir_all(&cover_dir).expect("cover dir");
        let indices = MetadataIndices::new(&MetadataLocation::InBookFolder).expect("indices");

        let plan = build_library_sync_plan(
            &[collected(&book_path)],
            &[row],
            &indices,
            &cover_dir,
            false,
        );

        assert_eq!(plan.changed, 1);
        assert_eq!(plan.items_to_ingest.len(), 1);
    }

    #[test]
    fn reconcile_missing_cover_schedules_ingest() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join("Book.epub");
        std::fs::write(&book_path, b"book").expect("book write");
        let row = stored_row(
            "0123456789abcdef0123456789abcdef",
            &FileFingerprint::capture(&book_path).expect("book fp"),
        );
        let cover_dir = dir.path().join("covers");
        std::fs::create_dir_all(&cover_dir).expect("cover dir");
        let indices = MetadataIndices::new(&MetadataLocation::InBookFolder).expect("indices");

        let plan = build_library_sync_plan(
            &[collected(&book_path)],
            &[row],
            &indices,
            &cover_dir,
            false,
        );

        assert_eq!(plan.changed, 1);
        assert_eq!(plan.items_to_ingest.len(), 1);
    }

    #[test]
    fn reconcile_removed_book_schedules_cleanup() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join("Book.epub");
        std::fs::write(&book_path, b"book").expect("book write");
        let row = stored_row(
            "0123456789abcdef0123456789abcdef",
            &FileFingerprint::capture(&book_path).expect("book fp"),
        );
        std::fs::remove_file(&book_path).expect("remove book");
        let cover_dir = dir.path().join("covers");
        std::fs::create_dir_all(&cover_dir).expect("cover dir");
        let indices = MetadataIndices::new(&MetadataLocation::InBookFolder).expect("indices");

        let plan = build_library_sync_plan(&[], &[row], &indices, &cover_dir, false);

        assert_eq!(plan.removed, 1);
        assert_eq!(
            plan.item_ids_to_delete,
            vec!["0123456789abcdef0123456789abcdef"]
        );
    }
}
