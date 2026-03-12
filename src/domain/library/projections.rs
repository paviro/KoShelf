//! Maps `LibraryItemRow` / `AnnotationRow` → contract types for API responses.
//!
//! This is the read-path counterpart to `item_mapping.rs` (which maps
//! `LibraryItem` → rows for the write path).

use crate::contracts::library::{
    LibraryAnnotation, LibraryContentType, LibraryDetailItem, LibraryIdentifier, LibraryListItem,
    LibrarySeries, LibraryStatus,
};
use crate::infra::sqlite::library_repo::rows::{AnnotationRow, LibraryItemRow};
use crate::models::Identifier;

pub fn row_to_list_item(row: &LibraryItemRow) -> LibraryListItem {
    LibraryListItem {
        id: row.id.clone(),
        title: row.title.clone(),
        authors: parse_json_string_vec(&row.authors_json),
        series: parse_series(row.series_name.as_deref(), row.series_index.as_deref()),
        status: parse_status(&row.status),
        progress_percentage: row.progress_percentage,
        rating: row.rating.map(|r| r as u32),
        annotation_count: row.annotation_count as usize,
        cover_url: row.cover_url.clone(),
        content_type: parse_content_type(&row.content_type),
    }
}

pub fn row_to_detail_item(row: &LibraryItemRow) -> LibraryDetailItem {
    LibraryDetailItem {
        id: row.id.clone(),
        title: row.title.clone(),
        authors: parse_json_string_vec(&row.authors_json),
        series: parse_series(row.series_name.as_deref(), row.series_index.as_deref()),
        status: parse_status(&row.status),
        progress_percentage: row.progress_percentage,
        rating: row.rating.map(|r| r as u32),
        cover_url: row.cover_url.clone(),
        content_type: parse_content_type(&row.content_type),
        language: row.language.clone(),
        publisher: row.publisher.clone(),
        description: row.description.clone(),
        review_note: row.review_note.clone(),
        pages: row.pages.map(|p| p as u32),
        search_base_path: row.search_base_path.clone(),
        subjects: parse_json_string_vec(&row.subjects_json),
        identifiers: parse_identifiers(&row.identifiers_json),
    }
}

pub fn annotation_row_to_contract(row: &AnnotationRow) -> LibraryAnnotation {
    LibraryAnnotation {
        chapter: row.chapter.clone(),
        datetime: row.datetime.clone(),
        pageno: row.pageno.map(|p| p as u32),
        text: row.text.clone(),
        note: row.note.clone(),
    }
}

fn parse_json_string_vec(json: &str) -> Vec<String> {
    serde_json::from_str(json).unwrap_or_default()
}

fn parse_series(name: Option<&str>, index: Option<&str>) -> Option<LibrarySeries> {
    name.map(|n| LibrarySeries {
        name: n.to_string(),
        index: index.map(ToString::to_string),
    })
}

fn parse_status(status: &str) -> LibraryStatus {
    match status {
        "reading" => LibraryStatus::Reading,
        "complete" => LibraryStatus::Complete,
        "abandoned" => LibraryStatus::Abandoned,
        _ => LibraryStatus::Unknown,
    }
}

fn parse_content_type(ct: &str) -> LibraryContentType {
    match ct {
        "comic" => LibraryContentType::Comic,
        _ => LibraryContentType::Book,
    }
}

fn parse_identifiers(json: &str) -> Vec<LibraryIdentifier> {
    let stored: Vec<Identifier> = serde_json::from_str(json).unwrap_or_default();
    stored
        .into_iter()
        .map(|id| {
            let display_scheme = id.display_scheme();
            let url = id.url();
            LibraryIdentifier {
                scheme: id.scheme,
                value: id.value,
                display_scheme,
                url,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::sqlite::library_repo::rows::LibraryItemRow;

    fn sample_row() -> LibraryItemRow {
        LibraryItemRow {
            id: "abc123".to_string(),
            file_path: "/books/test.epub".to_string(),
            format: "epub".to_string(),
            content_type: "book".to_string(),
            title: "Test Book".to_string(),
            title_sort: "test book".to_string(),
            primary_author_sort: "doe".to_string(),
            authors_json: r#"["Jane Doe","John Smith"]"#.to_string(),
            series_name: Some("Test Series".to_string()),
            series_index: Some("2".to_string()),
            description: Some("A test book".to_string()),
            language: Some("en".to_string()),
            publisher: Some("Test Publisher".to_string()),
            subjects_json: r#"["Fiction","Sci-Fi"]"#.to_string(),
            identifiers_json: r#"[{"scheme":"isbn","value":"1234567890"}]"#.to_string(),
            status: "reading".to_string(),
            progress_percentage: Some(0.42),
            rating: Some(4),
            review_note: Some("Great book".to_string()),
            pages: Some(300),
            cover_url: "/assets/covers/abc123.webp".to_string(),
            search_base_path: "/books".to_string(),
            annotation_count: 5,
            bookmark_count: 2,
            highlight_count: 3,
            partial_md5_checksum: Some("abc123".to_string()),
            last_open_at: None,
            total_reading_time_sec: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn row_to_list_item_maps_all_fields() {
        let row = sample_row();
        let item = row_to_list_item(&row);

        assert_eq!(item.id, "abc123");
        assert_eq!(item.title, "Test Book");
        assert_eq!(item.authors, vec!["Jane Doe", "John Smith"]);
        assert_eq!(item.series.as_ref().unwrap().name, "Test Series");
        assert_eq!(item.series.as_ref().unwrap().index.as_deref(), Some("2"));
        assert_eq!(item.status, LibraryStatus::Reading);
        assert_eq!(item.progress_percentage, Some(0.42));
        assert_eq!(item.rating, Some(4));
        assert_eq!(item.annotation_count, 5);
        assert_eq!(item.content_type, LibraryContentType::Book);
    }

    #[test]
    fn row_to_detail_item_maps_extended_fields() {
        let row = sample_row();
        let item = row_to_detail_item(&row);

        assert_eq!(item.language.as_deref(), Some("en"));
        assert_eq!(item.publisher.as_deref(), Some("Test Publisher"));
        assert_eq!(item.description.as_deref(), Some("A test book"));
        assert_eq!(item.review_note.as_deref(), Some("Great book"));
        assert_eq!(item.pages, Some(300));
        assert_eq!(item.subjects, vec!["Fiction", "Sci-Fi"]);
        assert_eq!(item.identifiers.len(), 1);
        assert_eq!(item.identifiers[0].scheme, "isbn");
        assert_eq!(item.identifiers[0].display_scheme, "ISBN");
        assert!(item.identifiers[0].url.is_some());
    }

    #[test]
    fn parse_status_handles_all_variants() {
        assert_eq!(parse_status("reading"), LibraryStatus::Reading);
        assert_eq!(parse_status("complete"), LibraryStatus::Complete);
        assert_eq!(parse_status("abandoned"), LibraryStatus::Abandoned);
        assert_eq!(parse_status("unknown"), LibraryStatus::Unknown);
        assert_eq!(parse_status("garbage"), LibraryStatus::Unknown);
    }

    #[test]
    fn parse_content_type_handles_book_and_comic() {
        assert_eq!(parse_content_type("book"), LibraryContentType::Book);
        assert_eq!(parse_content_type("comic"), LibraryContentType::Comic);
        assert_eq!(parse_content_type("other"), LibraryContentType::Book);
    }

    #[test]
    fn annotation_row_maps_correctly() {
        let row = AnnotationRow {
            item_id: "abc".to_string(),
            annotation_kind: "highlight".to_string(),
            ordinal: 0,
            chapter: Some("Chapter 1".to_string()),
            datetime: Some("2026-01-15T10:00:00Z".to_string()),
            pageno: Some(42),
            text: Some("highlighted text".to_string()),
            note: Some("my note".to_string()),
        };

        let annotation = annotation_row_to_contract(&row);
        assert_eq!(annotation.chapter.as_deref(), Some("Chapter 1"));
        assert_eq!(annotation.pageno, Some(42));
        assert_eq!(annotation.text.as_deref(), Some("highlighted text"));
        assert_eq!(annotation.note.as_deref(), Some("my note"));
    }
}
