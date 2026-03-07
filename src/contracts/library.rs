use serde::{Deserialize, Serialize};

use super::common::ApiMeta;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibraryContentType {
    Book,
    Comic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
pub struct LibraryListItem {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<LibrarySeries>,
    pub status: LibraryStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    #[serde(default)]
    pub annotation_count: usize,
    pub cover_url: String,
    pub content_type: LibraryContentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryListResponse {
    pub meta: ApiMeta,
    pub items: Vec<LibraryListItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryIdentifier {
    pub scheme: String,
    pub value: String,
    pub display_scheme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDetailItem {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<LibrarySeries>,
    pub status: LibraryStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    pub cover_url: String,
    pub content_type: LibraryContentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<u32>,
    pub search_base_path: String,
    pub subjects: Vec<String>,
    pub identifiers: Vec<LibraryIdentifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryAnnotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter: Option<String>,
    /// RFC3339 instant normalized from KOReader's timezone-less wall-clock metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pageno: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryCompletionEntry {
    pub start_date: String,
    pub end_date: String,
    pub reading_time: i64,
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
    pub total_read_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibrarySessionStats {
    pub session_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_session_duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longest_session_duration: Option<i64>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completions: Option<LibraryCompletions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDetailResponse {
    pub meta: ApiMeta,
    pub item: LibraryDetailItem,
    pub highlights: Vec<LibraryAnnotation>,
    pub bookmarks: Vec<LibraryAnnotation>,
    pub statistics: LibraryDetailStatistics,
}
