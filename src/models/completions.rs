use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

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

    /// Average reading speed in pages per hour
    pub fn average_speed(&self) -> Option<f64> {
        if self.reading_time > 0 && self.pages_read > 0 {
            Some(self.pages_read as f64 / (self.reading_time as f64 / 3600.0))
        } else {
            None
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

    /// Get a nicely formatted date range display (e.g. "Jun 05" or "Jun 05 – Jun 12 2025")
    pub fn date_range_display(&self, translations: &crate::i18n::Translations) -> String {
        if self.start_date == self.end_date {
            Self::format_date_display(&self.start_date, translations)
        } else {
            format!(
                "{} – {}",
                Self::format_date_display(&self.start_date, translations),
                Self::format_date_display(&self.end_date, translations)
            )
        }
    }

    /// Helper: format a date string (YYYY-MM-DD) into a shorter, reader-friendly form.
    /// If the date is in the current year it returns e.g. "Jun 05", otherwise "Jun 05 2024".
    pub fn format_date_display(date_str: &str, translations: &crate::i18n::Translations) -> String {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let current_year = Utc::now().year();
            let locale = translations.locale();
            
            // Get appropriate format_string from translations
            let format_key = if date.year() == current_year {
                "datetime-short-current-year-format"
            } else {
                "datetime-short-with-year-format"
            };
            let format_str = translations.get(format_key);
            
            date.format_localized(&format_str, locale).to_string()
        } else {
            date_str.to_string()
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
