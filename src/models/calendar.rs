use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::ContentType;

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
