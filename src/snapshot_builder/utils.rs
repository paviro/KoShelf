//! Utility functions for snapshot building.

use chrono::Datelike;

use super::SnapshotBuilder;
use crate::models::StatisticsData;
use std::collections::HashMap;

/// Format a duration in seconds to a human-readable string (e.g., "2d 5h 30m")
pub(crate) fn format_duration(seconds: i64, translations: &crate::i18n::Translations) -> String {
    if seconds <= 0 {
        return format!("0{}", translations.get("units.m"));
    }
    let total_minutes = seconds / 60;
    let days = total_minutes / (24 * 60);
    let hours = (total_minutes % (24 * 60)) / 60;
    let mins = total_minutes % 60;
    let mut parts: Vec<String> = Vec::new();
    if days > 0 {
        parts.push(format!("{}{}", days, translations.get("units.d")));
    }
    if hours > 0 {
        parts.push(format!("{}{}", hours, translations.get("units.h")));
    }
    if mins > 0 || parts.is_empty() {
        parts.push(format!("{}{}", mins, translations.get("units.m")));
    }
    parts.join(" ")
}

/// Format an ISO date (YYYY-MM-DD) to a human-readable format (e.g., "7 Mar" or "7. März")
pub(crate) fn format_day_month(iso: &str, translations: &crate::i18n::Translations) -> String {
    if let Ok(date) = chrono::NaiveDate::parse_from_str(iso, "%Y-%m-%d") {
        let current_year = chrono::Utc::now().year();
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
        iso.to_string()
    }
}

/// Parse completion end date into `(year, year_month)` where `year_month` is `YYYY-MM`.
pub(crate) fn completion_year_and_month(end_date: &str) -> Option<(i32, String)> {
    let year_str = end_date.get(0..4)?;
    let year_month = end_date.get(0..7)?.to_string();
    let year = year_str.parse::<i32>().ok()?;
    Some((year, year_month))
}

/// Count completion entries grouped by completion year.
pub(crate) fn completion_counts_by_year(stats_data: &StatisticsData) -> HashMap<i32, i64> {
    let mut completion_counts: HashMap<i32, i64> = HashMap::new();

    for book in &stats_data.books {
        let Some(completions) = &book.completions else {
            continue;
        };

        for completion in &completions.entries {
            if let Some((year, _)) = completion_year_and_month(&completion.end_date) {
                *completion_counts.entry(year).or_insert(0) += 1;
            }
        }
    }

    completion_counts
}

impl SnapshotBuilder {
    /// Get current version from Cargo.toml
    pub(crate) fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Get current datetime as formatted string
    pub(crate) fn get_last_updated(&self) -> String {
        self.time_config.now_formatted()
    }
}
