use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Calendar event representing a book reading session (optimized structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub start: String, // ISO date: yyyy-mm-dd
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>, // ISO date: yyyy-mm-dd (optional, for single-day events)
    pub total_read_time: i64, // Total seconds read for this book
    pub total_pages_read: i64, // Total pages read for this book
    pub book_id: String, // Reference to book metadata
}

/// Book metadata for calendar events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarBook {
    pub title: String,
    pub authors: Vec<String>,
    pub color: String, // Color for the event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book_path: Option<String>, // Relative path to the book detail page, if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book_cover: Option<String>, // Relative path to the book cover image, if available
}

/// Complete calendar data structure with optimized format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarMonthData {
    pub events: Vec<CalendarEvent>,
    pub books: BTreeMap<String, CalendarBook>,
    pub stats: MonthlyStats,
}

/// Map of "YYYY-MM" to its monthly calendar data payload
pub type CalendarMonths = BTreeMap<String, CalendarMonthData>;

impl CalendarEvent {
    /// Create a new calendar event for a book's reading period
    pub fn new(
        start_date: String,
        end_date: Option<String>,
        total_read_time: i64,
        total_pages_read: i64,
        book_id: String,
    ) -> Self {
        Self {
            start: start_date,
            end: end_date,
            total_read_time,
            total_pages_read,
            book_id,
        }
    }
}

impl CalendarBook {
    /// Create a new calendar book metadata entry
    pub fn new(
        title: String,
        authors: Vec<String>,
        book_path: Option<String>,
        book_cover: Option<String>,
    ) -> Self {
        // Generate a consistent color based on book title
        let color = Self::generate_color(&title);

        Self {
            title,
            authors,
            color,
            book_path,
            book_cover,
        }
    }

    /// Generate a consistent color for a book based on its title
    fn generate_color(title: &str) -> String {
        // Define a set of pleasant colors for calendar events
        let colors = [
            "#3B82F6", // Blue
            "#10B981", // Green
            "#F59E0B", // Yellow
            "#EF4444", // Red
            "#8B5CF6", // Purple
            "#F97316", // Orange
            "#06B6D4", // Cyan
            "#84CC16", // Lime
            "#EC4899", // Pink
            "#6366F1", // Indigo
        ];

        // Simple hash function to get consistent color based on title
        let mut hash: u32 = 0;
        for byte in title.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }

        let index = (hash as usize) % colors.len();
        colors[index].to_string()
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
