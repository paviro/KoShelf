//! Pure mapping functions from `LibraryItem` → repository row types.
//!
//! These functions apply the merge-precedence rules already encoded in the
//! `LibraryItem` model methods and produce flattened row types ready for
//! SQLite upsert.  No I/O happens here.

use crate::server::api::responses::library::{ExternalIdentifier, LibrarySeries};
use crate::shelf::models::{ContentType, LibraryItem, LibraryItemFormat};
use crate::shelf::time_config::TimeConfig;
use crate::source::fingerprints::FileFingerprint;
use crate::store::sqlite::repo::rows::{AnnotationRow, FingerprintRow, LibraryItemRow};
use std::path::Path;

/// Map a `LibraryItem` to a `LibraryItemRow` for upsert.
pub fn map_item_to_row(item: &LibraryItem, time_config: &TimeConfig) -> LibraryItemRow {
    let now = time_config.now_rfc3339();
    let content_type = item.content_type();

    let search_base_path = match content_type {
        ContentType::Book => "/books",
        ContentType::Comic => "/comics",
    };

    let series_json = item.series().map(|name| {
        let series = LibrarySeries {
            name: name.clone(),
            index: item.series_number().cloned(),
        };
        serde_json::to_string(&series).unwrap_or_default()
    });

    let identifiers: Vec<ExternalIdentifier> = item
        .identifiers()
        .into_iter()
        .map(|id| {
            let display_scheme = id.display_scheme();
            let url = id.url();
            ExternalIdentifier {
                scheme: id.scheme,
                value: id.value,
                display_scheme,
                url,
            }
        })
        .collect();

    LibraryItemRow {
        id: item.id.clone(),
        file_path: item.file_path.to_string_lossy().to_string(),
        format: format_str(item.format).to_string(),
        content_type: content_type.to_string(),
        title: item.book_info.title.clone(),
        authors_json: serde_json::to_string(&item.book_info.authors).unwrap_or_default(),
        series_json,
        description: item.book_info.description.clone(),
        language: item.language().cloned(),
        publisher: item.publisher().cloned(),
        subjects_json: serde_json::to_string(item.subjects()).unwrap_or_default(),
        identifiers_json: serde_json::to_string(&identifiers).unwrap_or_default(),
        status: item.status().to_string(),
        progress_percentage: item.progress_percentage(),
        rating: item.rating().map(|r| r as i32),
        review_note: item.review_note().cloned(),
        doc_pages: item
            .koreader_metadata
            .as_ref()
            .and_then(|m| m.doc_pages)
            .map(|p| p as i32),
        pagemap_doc_pages: item.stable_display_page_total().map(|p| p as i32),
        has_synthetic_pagination: item.synthetic_scaling_page_total().is_some(),
        parser_pages: item.book_info.pages.map(|p| p as i32),
        cover_url: format!("/assets/covers/{}.webp", item.id),
        search_base_path: search_base_path.to_string(),
        annotation_count: item.annotation_count() as i32,
        bookmark_count: item.bookmark_count() as i32,
        highlight_count: item.highlight_count() as i32,
        partial_md5_checksum: item
            .koreader_metadata
            .as_ref()
            .and_then(|m| m.partial_md5_checksum.clone()),
        reader_presentation: item
            .koreader_metadata
            .as_ref()
            .and_then(|m| m.reader_presentation.as_ref())
            .and_then(|presentation| serde_json::to_string(presentation).ok()),
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
            pos0: annotation.pos0.clone(),
            pos1: annotation.pos1.clone(),
            color: annotation.color.clone(),
            drawer: annotation.drawer.clone(),
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
    metadata_path: Option<&Path>,
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

fn format_str(format: LibraryItemFormat) -> &'static str {
    match format {
        LibraryItemFormat::Epub => "epub",
        LibraryItemFormat::Fb2 => "fb2",
        LibraryItemFormat::Mobi => "mobi",
        LibraryItemFormat::Cbz => "cbz",
        LibraryItemFormat::Cbr => "cbr",
    }
}
