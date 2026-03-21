use serde::{Deserialize, Serialize};
use sqlx::types::Json;

use crate::shelf::models::ReaderPresentation as LibraryReaderPresentation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum LibraryContentType {
    Book,
    Comic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum LibraryStatus {
    Reading,
    Complete,
    Abandoned,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibrarySeries {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalIdentifier {
    pub scheme: String,
    pub value: String,
    pub display_scheme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

// ── Types queried directly from the database via FromRow ──────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LibraryListItem {
    pub id: String,
    pub title: String,
    #[sqlx(rename = "authors_json")]
    pub authors: Json<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(rename = "series_json")]
    pub series: Option<Json<LibrarySeries>>,
    pub status: LibraryStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<i32>,
    #[serde(default)]
    pub annotation_count: i32,
    pub cover_url: String,
    pub content_type: LibraryContentType,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LibraryDetailItem {
    pub id: String,
    pub title: String,
    #[sqlx(rename = "authors_json")]
    pub authors: Json<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(rename = "series_json")]
    pub series: Option<Json<LibrarySeries>>,
    pub status: LibraryStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<i32>,
    pub cover_url: String,
    pub content_type: LibraryContentType,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<i32>,
    pub search_base_path: String,
    #[sqlx(rename = "subjects_json")]
    pub subjects: Json<Vec<String>>,
    #[sqlx(rename = "identifiers_json")]
    pub identifiers: Json<Vec<ExternalIdentifier>>,
    /// Populated from DB but not serialized directly; moved to `LibraryDetailData`
    /// when the `reader_presentation` include token is present.
    #[serde(skip)]
    pub reader_presentation: Option<Json<LibraryReaderPresentation>>,
    /// Used internally for statistics lookup; not exposed in API responses.
    #[serde(skip)]
    pub partial_md5_checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LibraryAnnotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pageno: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos0: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drawer: Option<String>,
}

// ── Response wrappers ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryListData {
    pub items: Vec<LibraryListItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDetailData {
    pub item: LibraryDetailItem,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlights: Option<Vec<LibraryAnnotation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmarks: Option<Vec<LibraryAnnotation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<LibraryDetailStatistics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completions: Option<LibraryCompletions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reader_presentation: Option<Json<LibraryReaderPresentation>>,
}

// ── Statistics (non-DB, mapped in service layer) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_open_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlights: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_reading_time_sec: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibrarySessionStats {
    pub session_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_session_duration_sec: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longest_session_duration_sec: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_read_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading_speed: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDetailStatistics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_stats: Option<LibraryItemStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_stats: Option<LibrarySessionStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryCompletionEntry {
    pub start_date: String,
    pub end_date: String,
    pub reading_time_sec: i64,
    pub session_count: i64,
    pub pages_read: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryCompletions {
    pub entries: Vec<LibraryCompletionEntry>,
    pub total_completions: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_completion_date: Option<String>,
}
