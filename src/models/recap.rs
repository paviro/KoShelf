use serde::Serialize;

use super::ContentType;

/// Recap view item for a single completed book entry enriched with optional Book data
#[derive(Debug, Clone, Serialize)]
pub struct RecapItem {
    pub title: String,
    pub authors: Vec<String>,
    pub start_date: String,
    pub end_date: String,
    pub start_display: String,
    pub end_display: String,
    pub reading_time: i64,
    pub reading_time_display: String,
    pub session_count: i64,
    pub pages_read: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book_cover: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<ContentType>,
    pub star_display: [bool; 5],
}

/// Recap view month summary and entries
#[derive(Debug, Clone, Serialize)]
pub struct MonthRecap {
    pub month_key: String,          // YYYY-MM
    pub month_label: String,        // e.g. March
    pub books_finished: usize,      // number of completions in this month
    pub hours_read_seconds: i64,    // total reading time in month from daily activity
    pub hours_read_display: String, // formatted e.g. "12h 30m"
    pub items: Vec<RecapItem>,      // enriched completion entries (sorted by end date)
}

/// Aggregated yearly statistics for the recap header
#[derive(Debug, Clone, Serialize)]
pub struct YearlySummary {
    pub total_books: usize,
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
    pub best_month_name: Option<String>,
    pub best_month_time_display: Option<String>,
}
