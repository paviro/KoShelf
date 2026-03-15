use anyhow::{Context, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFingerprint {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub modified_unix_ms: u64,
}

impl FileFingerprint {
    pub fn capture(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to read file metadata for {path:?}"))?;
        Self::from_metadata(path, &metadata)
    }

    pub fn capture_optional(path: impl AsRef<Path>) -> Result<Option<Self>> {
        let path = path.as_ref();
        match fs::metadata(path) {
            Ok(metadata) => Self::from_metadata(path, &metadata).map(Some),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => {
                Err(error).with_context(|| format!("Failed to read file metadata for {path:?}"))
            }
        }
    }

    fn from_metadata(path: &Path, metadata: &fs::Metadata) -> Result<Self> {
        let modified_unix_ms = u64::try_from(
            metadata
                .modified()
                .with_context(|| format!("Failed to read modified timestamp for {path:?}"))?
                .duration_since(UNIX_EPOCH)
                .with_context(|| {
                    format!("File modified timestamp is before unix epoch for {path:?}")
                })?
                .as_millis(),
        )
        .with_context(|| format!("Modified timestamp overflows u64 milliseconds for {path:?}"))?;

        Ok(Self {
            path: path.to_path_buf(),
            size_bytes: metadata.len(),
            modified_unix_ms,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemFingerprints {
    pub book_file: FileFingerprint,
    pub metadata_file: Option<FileFingerprint>,
}

impl ItemFingerprints {
    pub fn change_from(&self, previous: &Self) -> FingerprintChange {
        let book_changed = self.book_file != previous.book_file;
        let metadata_changed = self.metadata_file != previous.metadata_file;

        match (book_changed, metadata_changed) {
            (false, false) => FingerprintChange::Unchanged,
            (true, false) => FingerprintChange::BookChanged,
            (false, true) => FingerprintChange::MetadataChanged,
            (true, true) => FingerprintChange::BookAndMetadataChanged,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FingerprintChange {
    Unchanged,
    BookChanged,
    MetadataChanged,
    BookAndMetadataChanged,
}

impl FingerprintChange {
    pub fn reparse_scope(self) -> Option<ReparseScope> {
        match self {
            Self::Unchanged => None,
            Self::BookChanged => Some(ReparseScope::BookOnly),
            Self::MetadataChanged => Some(ReparseScope::MetadataOnly),
            Self::BookAndMetadataChanged => Some(ReparseScope::Full),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReparseScope {
    BookOnly,
    MetadataOnly,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconcileAction {
    Added,
    Removed,
    Unchanged,
    Reparse(ReparseScope),
}

pub fn classify_reconcile_action(
    previous: Option<&ItemFingerprints>,
    current: Option<&ItemFingerprints>,
) -> ReconcileAction {
    match (previous, current) {
        (None, Some(_)) => ReconcileAction::Added,
        (Some(_), None) => ReconcileAction::Removed,
        (None, None) => ReconcileAction::Unchanged,
        (Some(previous), Some(current)) => match current.change_from(previous).reparse_scope() {
            Some(scope) => ReconcileAction::Reparse(scope),
            None => ReconcileAction::Unchanged,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FileFingerprint, FingerprintChange, ItemFingerprints, ReconcileAction, ReparseScope,
        classify_reconcile_action,
    };
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn fingerprint(path: &str, size_bytes: u64, modified_unix_ms: u64) -> FileFingerprint {
        FileFingerprint {
            path: PathBuf::from(path),
            size_bytes,
            modified_unix_ms,
        }
    }

    fn item_fingerprints(
        book_path: &str,
        book_size_bytes: u64,
        book_modified_unix_ms: u64,
        metadata: Option<(&str, u64, u64)>,
    ) -> ItemFingerprints {
        ItemFingerprints {
            book_file: fingerprint(book_path, book_size_bytes, book_modified_unix_ms),
            metadata_file: metadata.map(|(path, size_bytes, modified_unix_ms)| {
                fingerprint(path, size_bytes, modified_unix_ms)
            }),
        }
    }

    #[test]
    fn capture_optional_returns_none_for_missing_file() {
        let temp_dir = tempdir().expect("temp directory should be created");
        let missing_path = temp_dir.path().join("missing.epub");

        let fingerprint = FileFingerprint::capture_optional(&missing_path)
            .expect("optional capture should not fail for missing files");

        assert_eq!(fingerprint, None);
    }

    #[test]
    fn capture_reads_path_size_and_modified_timestamp() {
        let temp_dir = tempdir().expect("temp directory should be created");
        let book_path = temp_dir.path().join("book.epub");
        fs::write(&book_path, b"book bytes").expect("test file should be written");

        let fingerprint =
            FileFingerprint::capture(&book_path).expect("fingerprint capture should succeed");

        assert_eq!(fingerprint.path, book_path);
        assert_eq!(fingerprint.size_bytes, 10);
        assert!(fingerprint.modified_unix_ms > 0);
    }

    #[test]
    fn item_fingerprints_detect_change_scope() {
        let previous = item_fingerprints(
            "books/book.epub",
            100,
            1000,
            Some(("books/book.sdr/metadata.epub.lua", 20, 2000)),
        );

        let unchanged = previous.clone();
        let book_only = item_fingerprints(
            "books/book.epub",
            101,
            1000,
            Some(("books/book.sdr/metadata.epub.lua", 20, 2000)),
        );
        let metadata_only = item_fingerprints(
            "books/book.epub",
            100,
            1000,
            Some(("books/book.sdr/metadata.epub.lua", 21, 2000)),
        );
        let both_changed = item_fingerprints(
            "books/book-renamed.epub",
            101,
            1100,
            Some(("books/book-renamed.sdr/metadata.epub.lua", 21, 2100)),
        );

        assert_eq!(
            unchanged.change_from(&previous),
            FingerprintChange::Unchanged
        );
        assert_eq!(
            book_only.change_from(&previous),
            FingerprintChange::BookChanged
        );
        assert_eq!(
            metadata_only.change_from(&previous),
            FingerprintChange::MetadataChanged
        );
        assert_eq!(
            both_changed.change_from(&previous),
            FingerprintChange::BookAndMetadataChanged
        );
    }

    #[test]
    fn classify_reconcile_action_maps_change_to_reparse_scope() {
        let previous = item_fingerprints(
            "books/book.epub",
            100,
            1000,
            Some(("books/book.sdr/metadata.epub.lua", 20, 2000)),
        );

        assert_eq!(
            classify_reconcile_action(None, Some(&previous)),
            ReconcileAction::Added
        );
        assert_eq!(
            classify_reconcile_action(Some(&previous), None),
            ReconcileAction::Removed
        );
        assert_eq!(
            classify_reconcile_action(Some(&previous), Some(&previous)),
            ReconcileAction::Unchanged
        );

        let book_only = item_fingerprints(
            "books/book.epub",
            101,
            1000,
            Some(("books/book.sdr/metadata.epub.lua", 20, 2000)),
        );
        assert_eq!(
            classify_reconcile_action(Some(&previous), Some(&book_only)),
            ReconcileAction::Reparse(ReparseScope::BookOnly)
        );

        let metadata_only = item_fingerprints("books/book.epub", 100, 1000, None);
        assert_eq!(
            classify_reconcile_action(Some(&previous), Some(&metadata_only)),
            ReconcileAction::Reparse(ReparseScope::MetadataOnly)
        );

        let both_changed = item_fingerprints(
            "books/book-renamed.epub",
            101,
            1100,
            Some(("books/book-renamed.sdr/metadata.epub.lua", 21, 2100)),
        );
        assert_eq!(
            classify_reconcile_action(Some(&previous), Some(&both_changed)),
            ReconcileAction::Reparse(ReparseScope::Full)
        );
    }
}
