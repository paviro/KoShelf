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
pub struct LibraryDetailStatistics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_stats: Option<crate::models::StatBook>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_stats: Option<crate::models::BookSessionStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completions: Option<crate::models::BookCompletions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDetailResponse {
    pub meta: ApiMeta,
    pub item: LibraryDetailItem,
    pub highlights: Vec<LibraryAnnotation>,
    pub bookmarks: Vec<LibraryAnnotation>,
    pub statistics: LibraryDetailStatistics,
}
