//! Reusable helpers shared across reading endpoint implementations.

use chrono::{Datelike, NaiveDate};

use crate::contracts::common::ContentTypeFilter;
use crate::contracts::library::LibraryContentType;
use crate::models::{ContentType, PageStat, StatisticsData};
use crate::time_config::TimeConfig;

/// Build a `TimeConfig` with an optional per-request timezone override.
pub fn resolve_time_config(base: &TimeConfig, tz_override: Option<chrono_tz::Tz>) -> TimeConfig {
    match tz_override {
        Some(tz) => TimeConfig::new(Some(tz), base.day_start_minutes),
        None => base.clone(),
    }
}

/// Filter `StatisticsData` by the requested content-type scope.
pub fn filter_stats_by_scope(
    stats_data: &StatisticsData,
    scope: ContentTypeFilter,
) -> StatisticsData {
    match scope {
        ContentTypeFilter::All => stats_data.clone(),
        ContentTypeFilter::Books => stats_data.filtered_by_content_type(ContentType::Book),
        ContentTypeFilter::Comics => stats_data.filtered_by_content_type(ContentType::Comic),
    }
}

/// Filter page stats by date range and determine the resolved from/to dates.
/// Returns (filtered_stats, resolved_from, resolved_to).
pub fn filter_and_resolve_range(
    page_stats: &[PageStat],
    range: Option<(NaiveDate, NaiveDate)>,
    time_config: &TimeConfig,
) -> (Vec<PageStat>, NaiveDate, NaiveDate) {
    let valid_stats: Vec<PageStat> = page_stats
        .iter()
        .filter(|s| s.duration > 0)
        .cloned()
        .collect();

    match range {
        Some((from, to)) => {
            let filtered: Vec<PageStat> = valid_stats
                .into_iter()
                .filter(|s| {
                    let date = time_config.date_for_timestamp(s.start_time);
                    date >= from && date <= to
                })
                .collect();
            (filtered, from, to)
        }
        None => {
            // Default to full available range.
            let today = time_config.today_date();
            if valid_stats.is_empty() {
                return (valid_stats, today, today);
            }
            let mut min_date = today;
            let mut max_date = today;
            for s in &valid_stats {
                let date = time_config.date_for_timestamp(s.start_time);
                if date < min_date {
                    min_date = date;
                }
                if date > max_date {
                    max_date = date;
                }
            }
            (valid_stats, min_date, max_date)
        }
    }
}

/// Count completions whose end_date falls within [from, to].
/// Returns (total_completion_count, distinct_items_completed).
pub fn count_completions_in_range(
    stats: &StatisticsData,
    from: &NaiveDate,
    to: &NaiveDate,
) -> (i64, i64) {
    let mut total = 0i64;
    let mut items_with_completion = 0i64;

    for book in &stats.books {
        let Some(ref completions) = book.completions else {
            continue;
        };
        let mut book_has_completion_in_range = false;
        for entry in &completions.entries {
            if let Ok(end_date) = NaiveDate::parse_from_str(&entry.end_date, "%Y-%m-%d")
                && end_date >= *from
                && end_date <= *to
            {
                total += 1;
                book_has_completion_in_range = true;
            }
        }
        if book_has_completion_in_range {
            items_with_completion += 1;
        }
    }

    (total, items_with_completion)
}

// ── Bucket key helpers ──────────────────────────────────────────────────────

/// Return the Monday of the ISO week containing `date`.
pub fn week_monday(date: NaiveDate) -> NaiveDate {
    date - chrono::Duration::days(date.weekday().num_days_from_monday() as i64)
}

/// Bucket key for day grouping: `YYYY-MM-DD`.
pub fn bucket_key_day(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Bucket key for week grouping: `YYYY-MM-DD` (Monday of that week).
pub fn bucket_key_week(date: NaiveDate) -> String {
    week_monday(date).format("%Y-%m-%d").to_string()
}

/// Bucket key for month grouping: `YYYY-MM`.
pub fn bucket_key_month(date: NaiveDate) -> String {
    format!("{:04}-{:02}", date.year(), date.month())
}

/// Bucket key for year grouping: `YYYY`.
pub fn bucket_key_year(date: NaiveDate) -> String {
    format!("{:04}", date.year())
}

// ── Content helpers ─────────────────────────────────────────────────────────

/// Parse a comma/semicolon-separated author string into trimmed author names.
pub fn parse_authors(authors_str: &str) -> Vec<String> {
    if authors_str.is_empty() {
        Vec::new()
    } else {
        authors_str
            .split(&[',', ';'])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Map an optional internal `ContentType` to the API `LibraryContentType`.
pub fn to_library_content_type(ct: Option<ContentType>) -> LibraryContentType {
    match ct {
        Some(ContentType::Comic) => LibraryContentType::Comic,
        _ => LibraryContentType::Book,
    }
}
