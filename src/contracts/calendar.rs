use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::common::{ApiMeta, Scoped};
use super::library::LibraryContentType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarMonthsResponse {
    pub meta: ApiMeta,
    pub months: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEventResponse {
    pub start: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    pub total_read_time: i64,
    pub total_pages_read: i64,
    pub item_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarItemResponse {
    pub title: String,
    pub authors: Vec<String>,
    pub content_type: LibraryContentType,
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_cover: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarMonthlyStats {
    pub books_read: usize,
    pub pages_read: i64,
    pub time_read: i64,
    pub days_read_pct: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarMonthResponse {
    pub meta: ApiMeta,
    pub events: Vec<CalendarEventResponse>,
    pub items: BTreeMap<String, CalendarItemResponse>,
    pub stats: Scoped<CalendarMonthlyStats>,
}
