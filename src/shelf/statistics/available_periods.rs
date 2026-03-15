//! Available-periods endpoint computation for `/api/reading/available-periods`.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use super::compute::scaling::PageScaling;
use super::queries::{DateRange, PeriodGroupBy, PeriodSource, ReadingAvailablePeriodsQuery};
use super::shared;
use crate::server::api::responses::reading::{PeriodEntry, ReadingAvailablePeriodsData};
use crate::shelf::time_config::TimeConfig;
use crate::source::koreader::types::StatisticsData;
use crate::store::memory::ReadingData;

/// Compute the available-periods response from reading data and a validated query.
pub fn available_periods(
    reading_data: &ReadingData,
    query: ReadingAvailablePeriodsQuery,
) -> ReadingAvailablePeriodsData {
    let time_config = shared::resolve_time_config(&reading_data.time_config, query.tz);
    let stats = shared::filter_stats_by_scope(&reading_data.stats_data, query.scope);

    let periods = match query.source {
        PeriodSource::ReadingData => reading_data_periods(
            &stats,
            &time_config,
            query.group_by,
            query.range.as_ref(),
            &reading_data.page_scaling,
        ),
        PeriodSource::Completions => {
            completions_periods(&stats, query.group_by, query.range.as_ref())
        }
    };

    let latest_key = periods.last().map(|p| p.key.clone());

    ReadingAvailablePeriodsData {
        source: query.source.as_str().to_string(),
        group_by: query.group_by.as_str().to_string(),
        periods,
        latest_key,
    }
}

/// Accumulator for reading-data period stats.
struct PeriodBucket {
    reading_time_sec: i64,
    scaled_pages: f64,
    completions: i64,
}

impl PeriodBucket {
    fn new() -> Self {
        Self {
            reading_time_sec: 0,
            scaled_pages: 0.0,
            completions: 0,
        }
    }
}

/// Compute periods from reading activity (page stats).
fn reading_data_periods(
    stats: &StatisticsData,
    time_config: &TimeConfig,
    group_by: PeriodGroupBy,
    range: Option<&DateRange>,
    page_scaling: &PageScaling,
) -> Vec<PeriodEntry> {
    let (page_stats, resolved_from, resolved_to) = shared::filter_and_resolve_range(
        &stats.page_stats,
        range.map(|r| (r.from, r.to)),
        time_config,
    );

    if page_stats.is_empty() {
        return Vec::new();
    }

    let mut buckets: BTreeMap<String, PeriodBucket> = BTreeMap::new();
    for stat in &page_stats {
        let date = time_config.date_for_timestamp(stat.start_time);
        let key = period_bucket_key(date, group_by);
        let bucket = buckets.entry(key).or_insert_with(PeriodBucket::new);
        bucket.reading_time_sec += stat.duration;
        bucket.scaled_pages += page_scaling.factor_for_book_id(stat.id_book);
    }

    for book in &stats.books {
        let Some(ref completions) = book.completions else {
            continue;
        };
        for entry in &completions.entries {
            if let Ok(end_date) = NaiveDate::parse_from_str(&entry.end_date, "%Y-%m-%d")
                && end_date >= resolved_from
                && end_date <= resolved_to
            {
                let key = period_bucket_key(end_date, group_by);
                if let Some(bucket) = buckets.get_mut(&key) {
                    bucket.completions += 1;
                } else {
                    // Completion in a period with no page stats — still include it.
                    let mut b = PeriodBucket::new();
                    b.completions = 1;
                    buckets.insert(key, b);
                }
            }
        }
    }

    buckets
        .into_iter()
        .map(|(key, bucket)| {
            let (start_date, end_date) = period_date_bounds(&key, group_by);
            PeriodEntry {
                key,
                start_date,
                end_date,
                reading_time_sec: Some(bucket.reading_time_sec),
                pages_read: Some(super::compute::scaling::round_pages(bucket.scaled_pages)),
                completions: Some(bucket.completions),
            }
        })
        .collect()
}

/// Compute periods from completion data only.
fn completions_periods(
    stats: &StatisticsData,
    group_by: PeriodGroupBy,
    range: Option<&super::queries::DateRange>,
) -> Vec<PeriodEntry> {
    let mut buckets: BTreeMap<String, i64> = BTreeMap::new();

    for book in &stats.books {
        let Some(ref completions) = book.completions else {
            continue;
        };
        for entry in &completions.entries {
            let Ok(end_date) = NaiveDate::parse_from_str(&entry.end_date, "%Y-%m-%d") else {
                continue;
            };
            if let Some(r) = range
                && (end_date < r.from || end_date > r.to)
            {
                continue;
            }
            let key = period_bucket_key(end_date, group_by);
            *buckets.entry(key).or_insert(0) += 1;
        }
    }

    buckets
        .into_iter()
        .map(|(key, count)| {
            let (start_date, end_date) = period_date_bounds(&key, group_by);
            PeriodEntry {
                key,
                start_date,
                end_date,
                reading_time_sec: None,
                pages_read: None,
                completions: Some(count),
            }
        })
        .collect()
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Map a date to its period bucket key string.
fn period_bucket_key(date: NaiveDate, group_by: PeriodGroupBy) -> String {
    match group_by {
        PeriodGroupBy::Week => shared::bucket_key_week(date),
        PeriodGroupBy::Month => shared::bucket_key_month(date),
        PeriodGroupBy::Year => shared::bucket_key_year(date),
    }
}

/// Compute the inclusive start and end dates for a period key.
fn period_date_bounds(key: &str, group_by: PeriodGroupBy) -> (String, String) {
    match group_by {
        PeriodGroupBy::Week => {
            // Key is Monday date: YYYY-MM-DD
            let monday =
                NaiveDate::parse_from_str(key, "%Y-%m-%d").expect("valid week key expected");
            let sunday = monday + chrono::Duration::days(6);
            (
                shared::bucket_key_day(monday),
                shared::bucket_key_day(sunday),
            )
        }
        PeriodGroupBy::Month => {
            // Key is YYYY-MM
            let parts: Vec<&str> = key.split('-').collect();
            let year: i32 = parts[0].parse().expect("valid year in month key");
            let month: u32 = parts[1].parse().expect("valid month in month key");
            let first = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month start");
            let last = if month == 12 {
                NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("valid next year")
                    - chrono::Duration::days(1)
            } else {
                NaiveDate::from_ymd_opt(year, month + 1, 1).expect("valid next month")
                    - chrono::Duration::days(1)
            };
            (shared::bucket_key_day(first), shared::bucket_key_day(last))
        }
        PeriodGroupBy::Year => {
            // Key is YYYY
            let year: i32 = key.parse().expect("valid year key");
            let first = NaiveDate::from_ymd_opt(year, 1, 1).expect("valid year start");
            let last = NaiveDate::from_ymd_opt(year, 12, 31).expect("valid year end");
            (shared::bucket_key_day(first), shared::bucket_key_day(last))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn period_bucket_key_week_returns_monday() {
        // 2026-03-12 is a Thursday
        let date = NaiveDate::from_ymd_opt(2026, 3, 12).unwrap();
        assert_eq!(period_bucket_key(date, PeriodGroupBy::Week), "2026-03-09");
    }

    #[test]
    fn period_bucket_key_month_returns_yyyy_mm() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 12).unwrap();
        assert_eq!(period_bucket_key(date, PeriodGroupBy::Month), "2026-03");
    }

    #[test]
    fn period_bucket_key_year_returns_yyyy() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 12).unwrap();
        assert_eq!(period_bucket_key(date, PeriodGroupBy::Year), "2026");
    }

    #[test]
    fn period_date_bounds_week() {
        let (start, end) = period_date_bounds("2026-03-09", PeriodGroupBy::Week);
        assert_eq!(start, "2026-03-09");
        assert_eq!(end, "2026-03-15");
    }

    #[test]
    fn period_date_bounds_month_regular() {
        let (start, end) = period_date_bounds("2026-03", PeriodGroupBy::Month);
        assert_eq!(start, "2026-03-01");
        assert_eq!(end, "2026-03-31");
    }

    #[test]
    fn period_date_bounds_month_february_non_leap() {
        let (start, end) = period_date_bounds("2026-02", PeriodGroupBy::Month);
        assert_eq!(start, "2026-02-01");
        assert_eq!(end, "2026-02-28");
    }

    #[test]
    fn period_date_bounds_month_december() {
        let (start, end) = period_date_bounds("2026-12", PeriodGroupBy::Month);
        assert_eq!(start, "2026-12-01");
        assert_eq!(end, "2026-12-31");
    }

    #[test]
    fn period_date_bounds_year() {
        let (start, end) = period_date_bounds("2026", PeriodGroupBy::Year);
        assert_eq!(start, "2026-01-01");
        assert_eq!(end, "2026-12-31");
    }
}
