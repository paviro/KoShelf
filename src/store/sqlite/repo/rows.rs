//! Row types written to the library SQLite cache.
//!
//! Read-path queries use contract types directly via `FromRow`.
//! These types are used for the write path only, except `FingerprintRow`
//! which is also read back for incremental build reconciliation.

#[derive(Debug, Clone)]
pub struct LibraryItemRow {
    pub id: String,
    pub file_path: String,
    pub format: String,
    pub content_type: String,
    pub title: String,
    pub authors_json: String,
    pub series_json: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub subjects_json: String,
    pub identifiers_json: String,
    pub status: String,
    pub progress_percentage: Option<f64>,
    pub rating: Option<i32>,
    pub review_note: Option<String>,
    pub doc_pages: Option<i32>,
    pub pagemap_doc_pages: Option<i32>,
    pub has_synthetic_pagination: bool,
    pub parser_pages: Option<i32>,
    pub cover_url: String,
    pub search_base_path: String,
    pub annotation_count: i32,
    pub bookmark_count: i32,
    pub highlight_count: i32,
    pub partial_md5_checksum: Option<String>,
    pub hidden_flow_pages: Option<i32>,
    pub reader_presentation: Option<String>,
    pub last_open_at: Option<String>,
    pub total_reading_time_sec: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct AnnotationRow {
    pub id: String,
    pub item_id: String,
    pub annotation_kind: String,
    pub lua_index: i32,
    pub chapter: Option<String>,
    pub datetime: Option<String>,
    pub datetime_updated: Option<String>,
    pub pageno: Option<i32>,
    pub text: Option<String>,
    pub note: Option<String>,
    pub pos0: Option<String>,
    pub pos1: Option<String>,
    pub color: Option<String>,
    pub drawer: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FingerprintRow {
    pub item_id: String,
    pub book_path: String,
    pub book_size_bytes: i64,
    pub book_modified_unix_ms: i64,
    pub metadata_path: Option<String>,
    pub metadata_size_bytes: Option<i64>,
    pub metadata_modified_unix_ms: Option<i64>,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct CollisionDiagnosticRow {
    pub canonical_id: String,
    pub file_path: String,
    pub winner_item_id: String,
    pub reason: String,
    pub detected_at: String,
}
