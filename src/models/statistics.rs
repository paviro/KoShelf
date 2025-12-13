use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::ContentType;
use super::completions::BookCompletions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatBook {
    #[serde(skip_serializing)]
    pub id: i64,
    #[serde(skip_serializing)]
    pub title: String,
    #[serde(skip_serializing)]
    pub authors: String,
    pub notes: Option<i64>,
    pub last_open: Option<i64>,
    pub highlights: Option<i64>,
    pub pages: Option<i64>,
    #[serde(skip_serializing)]
    pub md5: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<ContentType>,
    pub total_read_time: Option<i64>,
    #[serde(skip_serializing)]
    pub total_read_pages: Option<i64>,
    pub completions: Option<BookCompletions>,
}

/// Additional statistics calculated for a book from its reading sessions
#[derive(Debug, Clone, Serialize)]
pub struct BookSessionStats {
    pub session_count: i64,
    pub average_session_duration: Option<i64>, // in seconds
    pub longest_session_duration: Option<i64>, // in seconds
    pub last_read_date: Option<String>,
    pub reading_speed: Option<f64>, // pages per hour
}

/// Data structure representing a page stat entry from the statistics database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageStat {
    pub id_book: i64,
    pub page: i64,
    pub start_time: i64,
    pub duration: i64,
}

/// Main container for KoReader statistics data
#[derive(Debug, Clone)]
pub struct StatisticsData {
    pub books: Vec<StatBook>,
    pub page_stats: Vec<PageStat>,
    pub stats_by_md5: HashMap<String, StatBook>,
}

impl StatisticsData {
    /// Tag each `StatBook` with its content type using a MD5 lookup map.
    ///
    /// Any books not found in the map will keep `content_type = None`.
    pub fn tag_content_types(&mut self, md5_to_content_type: &HashMap<String, ContentType>) {
        for b in &mut self.books {
            b.content_type = md5_to_content_type.get(&b.md5).copied();
        }
        for (md5, b) in &mut self.stats_by_md5 {
            b.content_type = md5_to_content_type.get(md5).copied();
        }
    }

    /// Return a cloned `StatisticsData` containing only entries of the given content type.
    ///
    /// This filters:
    /// - `books` by `content_type`
    /// - `page_stats` by the remaining `id_book` set
    /// - `stats_by_md5` by the remaining MD5 set
    pub fn filtered_by_content_type(&self, content_type: ContentType) -> Self {
        let books: Vec<StatBook> = self
            .books
            .iter()
            .filter(|b| b.content_type == Some(content_type))
            .cloned()
            .collect();

        // Keep deterministic ordering the same as the original slice.
        // (No sorting here; the upstream order is preserved.)

        let ids_to_keep: std::collections::HashSet<i64> = books.iter().map(|b| b.id).collect();
        let page_stats: Vec<PageStat> = self
            .page_stats
            .iter()
            .filter(|ps| ids_to_keep.contains(&ps.id_book))
            .cloned()
            .collect();

        let mut stats_by_md5: HashMap<String, StatBook> = HashMap::new();
        for b in &books {
            stats_by_md5.insert(b.md5.clone(), b.clone());
        }

        Self {
            books,
            page_stats,
            stats_by_md5,
        }
    }
}

/// Streak information with date ranges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakInfo {
    pub days: i64,
    pub start_date: Option<String>, // ISO format yyyy-mm-dd
    pub end_date: Option<String>,   // ISO format yyyy-mm-dd
}

impl StreakInfo {
    pub fn new(days: i64, start_date: Option<String>, end_date: Option<String>) -> Self {
        Self {
            days,
            start_date,
            end_date,
        }
    }

    /// Format the date range for display
    pub fn date_range_display(&self, translations: &crate::i18n::Translations) -> Option<String> {
        match (&self.start_date, &self.end_date) {
            (Some(start), Some(end)) => {
                if start == end {
                    Self::format_date_display(start, translations)
                        .to_string()
                        .into()
                } else {
                    Some(format!(
                        "{} - {}",
                        Self::format_date_display(start, translations),
                        Self::format_date_display(end, translations)
                    ))
                }
            }
            (Some(start), None) => Some(format!(
                "{} - now",
                Self::format_date_display(start, translations)
            )),
            _ => None,
        }
    }

    /// Format a date string for display (convert from YYYY-MM-DD to more readable format)
    pub fn format_date_display(date_str: &str, translations: &crate::i18n::Translations) -> String {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let current_year = Utc::now().year();
            let locale = translations.locale();

            // Get appropriate format string from translations
            let format_key = if date.year() == current_year {
                "datetime.short-current-year"
            } else {
                "datetime.short-with-year"
            };
            let format_str = translations.get(format_key);

            date.format_localized(&format_str, locale).to_string()
        } else {
            date_str.to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingStats {
    // Overall stats
    pub total_read_time: i64, // seconds
    pub total_page_reads: i64,
    pub longest_read_time_in_day: i64, // seconds
    pub most_pages_in_day: i64,

    // Session stats (across all books)
    pub average_session_duration: Option<i64>, // seconds
    pub longest_session_duration: Option<i64>, // seconds

    // Completion stats (across all books)
    pub total_completions: i64,
    pub books_completed: i64, // Number of unique books completed at least once
    pub most_completions: i64, // Most times a single book was completed

    // Streak stats
    pub longest_streak: StreakInfo,
    pub current_streak: StreakInfo,

    // Weekly stats
    pub weeks: Vec<WeeklyStats>,

    // Daily activity data for heatmap
    pub daily_activity: Vec<DailyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyStats {
    pub start_date: String, // ISO format yyyy-mm-dd
    pub end_date: String,   // ISO format yyyy-mm-dd
    pub read_time: i64,     // seconds
    pub pages_read: i64,
    pub avg_pages_per_day: f64,
    pub avg_read_time_per_day: f64,            // seconds
    pub longest_session_duration: Option<i64>, // seconds
    pub average_session_duration: Option<i64>, // seconds
}

/// Daily reading stats for the activity heatmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,   // ISO format yyyy-mm-dd
    pub read_time: i64, // seconds
    pub pages_read: i64,
}
