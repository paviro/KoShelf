//! Row types returned by and written to the library SQLite cache.
//!
//! These stay in `infra` so the repository layer is self-contained — domain and
//! contract layers map to/from these types at their own boundaries.

#[derive(Debug, Clone)]
pub struct LibraryItemRow {
    pub id: String,
    pub file_path: String,
    pub format: String,
    pub content_type: String,
    pub title: String,
    pub title_sort: String,
    pub primary_author_sort: String,
    pub authors_json: String,
    pub series_name: Option<String>,
    pub series_index: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub subjects_json: String,
    pub identifiers_json: String,
    pub status: String,
    pub progress_percentage: Option<f64>,
    pub rating: Option<i32>,
    pub review_note: Option<String>,
    pub pages: Option<i32>,
    pub cover_url: String,
    pub search_base_path: String,
    pub annotation_count: i32,
    pub bookmark_count: i32,
    pub highlight_count: i32,
    pub partial_md5_checksum: Option<String>,
    pub last_open_at: Option<String>,
    pub total_reading_time_sec: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct AnnotationRow {
    pub item_id: String,
    pub annotation_kind: String,
    pub ordinal: i32,
    pub chapter: Option<String>,
    pub datetime: Option<String>,
    pub pageno: Option<i32>,
    pub text: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
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
