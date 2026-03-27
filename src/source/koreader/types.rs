//! KOReader statistics and completion data types.
//!
//! These types represent the shape of data from KOReader's statistics database
//! and the completion analysis derived from it.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::shelf::models::ContentType;

// ── Completions ──────────────────────────────────────────────────────────

/// Represents a single reading completion/cycle for a book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadCompletion {
    pub start_date: String, // ISO format yyyy-mm-dd
    pub end_date: String,   // ISO format yyyy-mm-dd
    pub reading_time: i64,  // Total seconds spent reading for this completion
    pub session_count: i64, // Number of reading sessions in this completion
    pub pages_read: i64,
}

impl ReadCompletion {
    pub fn new(
        start_date: String,
        end_date: String,
        reading_time: i64,
        session_count: i64,
        pages_read: i64,
    ) -> Self {
        Self {
            start_date,
            end_date,
            reading_time,
            session_count,
            pages_read,
        }
    }

    /// Average session duration in seconds
    pub fn avg_session_duration(&self) -> Option<i64> {
        if self.session_count > 0 {
            Some(self.reading_time / self.session_count)
        } else {
            None
        }
    }

    /// Calendar length (number of days spanned by the completion)
    pub fn calendar_length_days(&self) -> Option<i64> {
        if let (Ok(start), Ok(end)) = (
            NaiveDate::parse_from_str(&self.start_date, "%Y-%m-%d"),
            NaiveDate::parse_from_str(&self.end_date, "%Y-%m-%d"),
        ) {
            Some((end - start).num_days().abs() + 1)
        } else {
            None
        }
    }
}

/// Container for all reading completions of a book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookCompletions {
    pub entries: Vec<ReadCompletion>,
    pub total_completions: usize,
    pub last_completion_date: Option<String>,
}

impl BookCompletions {
    pub fn new(entries: Vec<ReadCompletion>) -> Self {
        let total_completions = entries.len();
        let last_completion_date = entries.last().map(|c| c.end_date.clone());

        Self {
            entries,
            total_completions,
            last_completion_date,
        }
    }

    /// Check if this book has been completed at least once
    pub fn has_completions(&self) -> bool {
        !self.entries.is_empty()
    }
}

// ── Statistics ────────────────────────────────────────────────────────────

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            b.content_type = md5_to_content_type
                .get(&b.md5)
                .copied()
                .or_else(|| md5_to_content_type.get(&b.md5.to_lowercase()).copied());
        }
        for (md5, b) in &mut self.stats_by_md5 {
            b.content_type = md5_to_content_type
                .get(md5)
                .copied()
                .or_else(|| md5_to_content_type.get(&md5.to_lowercase()).copied());
        }
    }

    /// Subtract hidden flow pages from book page counts.
    ///
    /// When a user has enabled KOReader's handmade flows and marked sections
    /// (e.g. appendices) as hidden, those pages are skipped during reading and
    /// excluded from KOReader's own progress calculation.  This method applies
    /// the same adjustment to `StatBook.pages` so that completion detection
    /// uses the effective (linear) page count.
    pub fn apply_hidden_flow_adjustments(&mut self, hidden_pages_by_md5: &HashMap<String, i32>) {
        if hidden_pages_by_md5.is_empty() {
            return;
        }
        for book in &mut self.books {
            Self::adjust_book_pages(book, hidden_pages_by_md5);
        }
        for book in self.stats_by_md5.values_mut() {
            Self::adjust_book_pages(book, hidden_pages_by_md5);
        }
    }

    fn adjust_book_pages(book: &mut StatBook, hidden_pages_by_md5: &HashMap<String, i32>) {
        if let Some((&hidden, pages)) = hidden_pages_by_md5.get(&book.md5).zip(book.pages.as_mut())
        {
            let adjusted = (*pages).saturating_sub(hidden as i64);
            if adjusted > 0 {
                *pages = adjusted;
            }
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

        let ids_to_keep: HashSet<i64> = books.iter().map(|b| b.id).collect();
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
