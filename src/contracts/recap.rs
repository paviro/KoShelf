use serde::{Deserialize, Serialize};

use super::common::{ApiMeta, ContentTypeFilter};
use super::library::LibraryContentType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionYearsResponse {
    pub meta: ApiMeta,
    pub content_type: ContentTypeFilter,
    pub available_years: Vec<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapSummaryResponse {
    pub total_items: usize,
    pub total_time_seconds: i64,
    pub total_time_days: i64,
    pub total_time_hours: i64,
    pub longest_session_hours: i64,
    pub longest_session_minutes: i64,
    pub average_session_hours: i64,
    pub average_session_minutes: i64,
    pub active_days: usize,
    pub active_days_percentage: f64,
    pub longest_streak: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_month_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_month_time_display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapItemResponse {
    pub title: String,
    pub authors: Vec<String>,
    pub start_date: String,
    pub end_date: String,
    pub reading_time: i64,
    pub session_count: i64,
    pub pages_read: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_cover: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<LibraryContentType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapMonthResponse {
    pub month_key: String,
    pub month_label: String,
    pub items_finished: usize,
    pub read_time: i64,
    pub items: Vec<RecapItemResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapShareAssets {
    pub story_url: String,
    pub square_url: String,
    pub banner_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionYearResponse {
    pub meta: ApiMeta,
    pub content_type: ContentTypeFilter,
    pub year: i32,
    pub summary: RecapSummaryResponse,
    pub months: Vec<RecapMonthResponse>,
    pub items: Vec<RecapItemResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_assets: Option<RecapShareAssets>,
}
