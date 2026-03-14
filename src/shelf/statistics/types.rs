//! Reading domain types for calendar and recap views.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::shelf::models::ContentType;

// ── Calendar types ───────────────────────────────────────────────────────

/// Calendar event representing a reading session (optimized structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub start: String, // ISO date: yyyy-mm-dd
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>, // ISO date: yyyy-mm-dd (optional, for single-day events)
    pub total_read_time: i64, // Total seconds read for this item
    pub total_pages_read: i64, // Total pages read for this item
    pub item_id: String, // Reference to item metadata
}

/// Item metadata for calendar events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarItem {
    pub title: String,
    pub authors: Vec<String>,
    pub content_type: ContentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>, // Canonical item identifier for detail navigation, if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_cover: Option<String>, // Relative path to the item cover image, if available
}

/// Complete calendar data structure with optimized format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarMonthData {
    pub events: Vec<CalendarEvent>,
    pub books: BTreeMap<String, CalendarItem>,
    /// Pre-calculated monthly statistics across all content types
    pub stats: MonthlyStats,
    /// Pre-calculated monthly statistics for books only
    pub stats_books: MonthlyStats,
    /// Pre-calculated monthly statistics for comics only
    pub stats_comics: MonthlyStats,
}

/// Map of "YYYY-MM" to its monthly calendar data payload
pub type CalendarMonths = BTreeMap<String, CalendarMonthData>;

impl CalendarEvent {
    /// Create a new calendar event for an item's reading period
    pub fn new(
        start_date: String,
        end_date: Option<String>,
        total_read_time: i64,
        total_pages_read: i64,
        item_id: String,
    ) -> Self {
        Self {
            start: start_date,
            end: end_date,
            total_read_time,
            total_pages_read,
            item_id,
        }
    }
}

impl CalendarItem {
    /// Create a new calendar item metadata entry
    pub fn new(
        title: String,
        authors: Vec<String>,
        content_type: ContentType,
        item_id: Option<String>,
        item_cover: Option<String>,
    ) -> Self {
        Self {
            title,
            authors,
            content_type,
            item_id,
            item_cover,
        }
    }
}

/// Pre-calculated monthly reading statistics for the calendar view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyStats {
    pub books_read: usize, // Number of unique books read in the month
    pub pages_read: i64,   // Total pages read in the month
    pub time_read: i64,    // Total time read in seconds in the month
    pub days_read_pct: u8, // Percentage of days in the month with any reading activity (0-100)
}

// ── Recap types ──────────────────────────────────────────────────────────

/// Recap view item for a single completed item entry enriched with optional LibraryItem data
#[derive(Debug, Clone, Serialize)]
pub struct RecapItem {
    pub title: String,
    pub authors: Vec<String>,
    pub start_date: String,
    pub end_date: String,
    pub reading_time: i64,
    pub session_count: i64,
    pub pages_read: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calendar_length_days: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_cover: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<ContentType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_session_duration: Option<i64>,
}

/// Recap view month summary and entries
#[derive(Debug, Clone, Serialize)]
pub struct MonthRecap {
    pub month_key: String,       // YYYY-MM
    pub books_finished: usize,   // number of completions in this month
    pub hours_read_seconds: i64, // total reading time in month from daily activity
    pub items: Vec<RecapItem>,   // enriched completion entries (sorted by end date)
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
    pub active_days_percentage: u8,
    pub longest_streak: i64,
    pub best_month: Option<String>, // YYYY-MM
}
