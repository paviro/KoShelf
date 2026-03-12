//! Pure mapping functions from `LibraryItem` → repository row types.
//!
//! These functions apply the merge-precedence rules already encoded in the
//! `LibraryItem` model methods and produce flattened row types ready for
//! SQLite upsert.  No I/O happens here.

use crate::infra::sources::fingerprints::FileFingerprint;
use crate::infra::sqlite::library_repo::rows::{AnnotationRow, FingerprintRow, LibraryItemRow};
use crate::models::{LibraryItem, LibraryItemFormat};
use crate::time_config::TimeConfig;

/// Map a `LibraryItem` to a `LibraryItemRow` for upsert.
pub fn map_item_to_row(
    item: &LibraryItem,
    use_stable_page_metadata: bool,
    time_config: &TimeConfig,
) -> LibraryItemRow {
    let now = time_config.now_rfc3339();
    let content_type = item.content_type();

    let search_base_path = match content_type {
        crate::models::ContentType::Book => "/books",
        crate::models::ContentType::Comic => "/comics",
    };

    LibraryItemRow {
        id: item.id.clone(),
        file_path: item.file_path.to_string_lossy().to_string(),
        format: format_str(item.format).to_string(),
        content_type: content_type.to_string(),
        title: item.book_info.title.clone(),
        title_sort: sort_key_title(&item.book_info.title),
        primary_author_sort: sort_key_author(&item.book_info.authors),
        authors_json: serde_json::to_string(&item.book_info.authors).unwrap_or_default(),
        series_name: item.series().cloned(),
        series_index: item.series_number().cloned(),
        description: item.book_info.description.clone(),
        language: item.language().cloned(),
        publisher: item.publisher().cloned(),
        subjects_json: serde_json::to_string(item.subjects()).unwrap_or_default(),
        identifiers_json: serde_json::to_string(&item.identifiers()).unwrap_or_default(),
        status: item.status().to_string(),
        progress_percentage: item.progress_percentage(),
        rating: item.rating().map(|r| r as i32),
        review_note: item.review_note().cloned(),
        pages: item
            .doc_pages_with_stable_metadata(use_stable_page_metadata)
            .map(|p| p as i32),
        cover_url: format!("/assets/covers/{}.webp", item.id),
        search_base_path: search_base_path.to_string(),
        annotation_count: item.annotation_count() as i32,
        bookmark_count: item.bookmark_count() as i32,
        highlight_count: item.highlight_count() as i32,
        partial_md5_checksum: item
            .koreader_metadata
            .as_ref()
            .and_then(|m| m.partial_md5_checksum.clone()),
        last_open_at: None,
        total_reading_time_sec: None,
        created_at: now.clone(),
        updated_at: now,
    }
}

/// Map a `LibraryItem`'s annotations to `AnnotationRow` entries.
pub fn map_annotations_to_rows(
    item_id: &str,
    item: &LibraryItem,
    time_config: &TimeConfig,
) -> Vec<AnnotationRow> {
    let annotations = item.annotations();
    let mut rows = Vec::with_capacity(annotations.len());

    // Separate into highlights and bookmarks, preserving ordinal within each kind.
    let mut highlight_ordinal: i32 = 0;
    let mut bookmark_ordinal: i32 = 0;

    for annotation in annotations {
        let (kind, ordinal) = if annotation.is_highlight() {
            let ord = highlight_ordinal;
            highlight_ordinal += 1;
            ("highlight", ord)
        } else {
            let ord = bookmark_ordinal;
            bookmark_ordinal += 1;
            ("bookmark", ord)
        };

        let datetime = annotation
            .datetime
            .as_deref()
            .and_then(|v| time_config.normalize_naive_datetime_to_rfc3339(v));

        rows.push(AnnotationRow {
            item_id: item_id.to_string(),
            annotation_kind: kind.to_string(),
            ordinal,
            chapter: annotation.chapter.clone(),
            datetime,
            pageno: annotation.pageno.map(|p| p as i32),
            text: annotation.text.clone(),
            note: annotation.note.clone(),
        });
    }

    rows
}

/// Build a `FingerprintRow` for a `LibraryItem` by capturing current file metadata.
///
/// `metadata_path` should be the resolved path to the KOReader metadata file
/// (if one was found during scanning).  Returns `None` if the book file's
/// metadata cannot be read.
pub fn capture_fingerprint_row(
    item: &LibraryItem,
    metadata_path: Option<&std::path::Path>,
    time_config: &TimeConfig,
) -> Option<FingerprintRow> {
    let book_fp = FileFingerprint::capture(&item.file_path).ok()?;

    let metadata_fp =
        metadata_path.and_then(|p| FileFingerprint::capture_optional(p).ok().flatten());

    let now = time_config.now_rfc3339();

    Some(FingerprintRow {
        item_id: item.id.clone(),
        book_path: book_fp.path.to_string_lossy().to_string(),
        book_size_bytes: book_fp.size_bytes as i64,
        book_modified_unix_ms: book_fp.modified_unix_ms as i64,
        metadata_path: metadata_fp
            .as_ref()
            .map(|fp| fp.path.to_string_lossy().to_string()),
        metadata_size_bytes: metadata_fp.as_ref().map(|fp| fp.size_bytes as i64),
        metadata_modified_unix_ms: metadata_fp.as_ref().map(|fp| fp.modified_unix_ms as i64),
        updated_at: now,
    })
}

/// Derive the InBookFolder metadata path for an item: `{book_stem}.sdr/{metadata_filename}`.
///
/// Returns `None` when the item has no KOReader metadata or the path cannot be
/// constructed.  Only valid for the default `InBookFolder` metadata location.
pub fn derive_in_book_folder_metadata_path(item: &LibraryItem) -> Option<std::path::PathBuf> {
    item.koreader_metadata.as_ref()?;
    let parent = item.file_path.parent()?;
    let stem = item.file_path.file_stem()?.to_str()?;
    let sdr_dir = parent.join(format!("{}.sdr", stem));
    Some(sdr_dir.join(item.format.metadata_filename()))
}

fn format_str(format: LibraryItemFormat) -> &'static str {
    match format {
        LibraryItemFormat::Epub => "epub",
        LibraryItemFormat::Fb2 => "fb2",
        LibraryItemFormat::Mobi => "mobi",
        LibraryItemFormat::Cbz => "cbz",
        LibraryItemFormat::Cbr => "cbr",
    }
}

/// Generate a sort key for titles: lowercase, strip leading articles.
fn sort_key_title(title: &str) -> String {
    let lower = title.to_lowercase();
    let trimmed = lower.trim();
    for prefix in &["the ", "a ", "an "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return rest.to_string();
        }
    }
    trimmed.to_string()
}

/// Generate a sort key for authors: first author, lowercase.
fn sort_key_author(authors: &[String]) -> String {
    authors
        .first()
        .map(|a| a.to_lowercase())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_key_title_strips_leading_articles() {
        assert_eq!(sort_key_title("The Great Gatsby"), "great gatsby");
        assert_eq!(sort_key_title("A Tale of Two Cities"), "tale of two cities");
        assert_eq!(sort_key_title("An Introduction"), "introduction");
        assert_eq!(sort_key_title("Dune"), "dune");
    }

    #[test]
    fn sort_key_author_uses_first_author_lowercase() {
        assert_eq!(
            sort_key_author(&["J.R.R. Tolkien".to_string(), "Other".to_string()]),
            "j.r.r. tolkien"
        );
        assert_eq!(sort_key_author(&[]), "");
    }
}
