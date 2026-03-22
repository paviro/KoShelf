//! Repository layer for the library SQLite cache.
//!
//! Provides typed read/write operations against `library.sqlite` tables.
//! Read queries return contract types directly via `FromRow`.
//! Write operations use dedicated row types from `rows`.

mod read;
mod write;

pub mod rows;

use sqlx::SqlitePool;

/// SQLite-backed repository for library items, annotations, and fingerprints.
#[derive(Clone)]
pub struct LibraryRepository {
    pool: SqlitePool,
    use_stable_page_metadata: bool,
}

impl LibraryRepository {
    pub fn new(pool: SqlitePool, use_stable_page_metadata: bool) -> Self {
        Self {
            pool,
            use_stable_page_metadata,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::LibraryRepository;
    use super::rows::{AnnotationRow, FingerprintRow, LibraryItemRow};
    use crate::store::sqlite::{open_library_pool_in_memory, run_library_migrations};

    pub async fn test_repo() -> LibraryRepository {
        let pool = open_library_pool_in_memory()
            .await
            .expect("pool should open");
        run_library_migrations(&pool)
            .await
            .expect("migrations should run");
        LibraryRepository::new(pool, true)
    }

    pub fn sample_item(id: &str) -> LibraryItemRow {
        LibraryItemRow {
            id: id.to_string(),
            file_path: format!("/books/{id}.epub"),
            format: "epub".to_string(),
            content_type: "book".to_string(),
            title: format!("Book {id}"),
            authors_json: r#"["Jane Doe"]"#.to_string(),
            series_json: None,
            description: None,
            language: Some("en".to_string()),
            publisher: None,
            subjects_json: "[]".to_string(),
            identifiers_json: "[]".to_string(),
            status: "reading".to_string(),
            progress_percentage: Some(0.42),
            rating: Some(4),
            review_note: None,
            doc_pages: Some(300),
            pagemap_doc_pages: None,
            has_synthetic_pagination: false,
            parser_pages: Some(300),
            cover_url: format!("/assets/covers/{id}.webp"),
            search_base_path: "/books".to_string(),
            annotation_count: 0,
            bookmark_count: 0,
            highlight_count: 0,
            partial_md5_checksum: Some(id.to_string()),
            hidden_flow_pages: None,
            reader_presentation: None,
            last_open_at: None,
            total_reading_time_sec: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    pub fn sample_annotation(item_id: &str, kind: &str, lua_index: i32) -> AnnotationRow {
        AnnotationRow {
            id: uuid::Uuid::new_v4().to_string(),
            item_id: item_id.to_string(),
            annotation_kind: kind.to_string(),
            lua_index,
            chapter: Some("Chapter 1".to_string()),
            datetime: Some("2026-01-15T10:00:00Z".to_string()),
            datetime_updated: None,
            pageno: Some(42),
            text: Some("highlighted text".to_string()),
            note: None,
            pos0: None,
            pos1: None,
            color: None,
            drawer: None,
        }
    }

    pub fn sample_fingerprint(item_id: &str) -> FingerprintRow {
        FingerprintRow {
            item_id: item_id.to_string(),
            book_path: format!("/books/{item_id}.epub"),
            book_size_bytes: 1024,
            book_modified_unix_ms: 1700000000000,
            metadata_path: Some(format!("/books/{item_id}.sdr/metadata.epub.lua")),
            metadata_size_bytes: Some(512),
            metadata_modified_unix_ms: Some(1700000001000),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }
}
