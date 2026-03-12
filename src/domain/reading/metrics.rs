//! Metrics endpoint computation for `/api/reading/metrics`.

use std::collections::{BTreeMap, HashMap};

use chrono::{Datelike, NaiveDate};

use crate::contracts::reading::{MetricPoint, ReadingMetricsData};
use crate::domain::reading::queries::{MetricsGroupBy, ReadingMetric, ReadingMetricsQuery};
use crate::domain::reading::shared;
use crate::models::PageStat;
use crate::runtime::ReadingData;
use crate::time_config::TimeConfig;

/// Default time gap that separates two reading events into different sessions (seconds).
const SESSION_GAP_SECONDS: i64 = 300;

/// Compute the metrics response from reading data and a validated query.
pub fn metrics(reading_data: &ReadingData, query: ReadingMetricsQuery) -> ReadingMetricsData {
    let time_config = shared::resolve_time_config(&reading_data.time_config, query.tz);
    let stats = shared::filter_stats_by_scope(&reading_data.stats_data, query.scope);
    let (page_stats, resolved_from, resolved_to) = shared::filter_and_resolve_range(
        &stats.page_stats,
        query.range.as_ref().map(|r| (r.from, r.to)),
        &time_config,
    );

    let all_keys = all_bucket_keys(resolved_from, resolved_to, query.group_by);

    let points = match query.metric {
        ReadingMetric::ReadingTimeSec => {
            let mut buckets: BTreeMap<String, i64> = BTreeMap::new();
            for stat in &page_stats {
                let date = time_config.date_for_timestamp(stat.start_time);
                let key = bucket_key(date, query.group_by);
                *buckets.entry(key).or_insert(0) += stat.duration;
            }
            fill_points(&all_keys, &buckets)
        }
        ReadingMetric::PagesRead => {
            let mut buckets: BTreeMap<String, i64> = BTreeMap::new();
            for stat in &page_stats {
                let date = time_config.date_for_timestamp(stat.start_time);
                let key = bucket_key(date, query.group_by);
                *buckets.entry(key).or_insert(0) += 1;
            }
            fill_points(&all_keys, &buckets)
        }
        ReadingMetric::Sessions => {
            let sessions = sessions_with_dates(&page_stats, &time_config);
            let mut buckets: BTreeMap<String, i64> = BTreeMap::new();
            for (date, _) in &sessions {
                let key = bucket_key(*date, query.group_by);
                *buckets.entry(key).or_insert(0) += 1;
            }
            fill_points(&all_keys, &buckets)
        }
        ReadingMetric::Completions => {
            let mut buckets: BTreeMap<String, i64> = BTreeMap::new();
            for book in &stats.books {
                let Some(ref completions) = book.completions else {
                    continue;
                };
                for entry in &completions.entries {
                    if let Ok(end_date) = NaiveDate::parse_from_str(&entry.end_date, "%Y-%m-%d")
                        && end_date >= resolved_from
                        && end_date <= resolved_to
                    {
                        let key = bucket_key(end_date, query.group_by);
                        *buckets.entry(key).or_insert(0) += 1;
                    }
                }
            }
            fill_points(&all_keys, &buckets)
        }
        ReadingMetric::AverageSessionDurationSec => {
            let sessions = sessions_with_dates(&page_stats, &time_config);
            let mut bucket_sessions: BTreeMap<String, Vec<i64>> = BTreeMap::new();
            for (date, duration) in &sessions {
                let key = bucket_key(*date, query.group_by);
                bucket_sessions.entry(key).or_default().push(*duration);
            }
            let mut buckets: BTreeMap<String, i64> = BTreeMap::new();
            for (key, durations) in &bucket_sessions {
                let total: i64 = durations.iter().sum();
                buckets.insert(key.clone(), total / durations.len() as i64);
            }
            fill_points(&all_keys, &buckets)
        }
        ReadingMetric::LongestSessionDurationSec => {
            let sessions = sessions_with_dates(&page_stats, &time_config);
            let mut bucket_sessions: BTreeMap<String, Vec<i64>> = BTreeMap::new();
            for (date, duration) in &sessions {
                let key = bucket_key(*date, query.group_by);
                bucket_sessions.entry(key).or_default().push(*duration);
            }
            let mut buckets: BTreeMap<String, i64> = BTreeMap::new();
            for (key, durations) in &bucket_sessions {
                buckets.insert(key.clone(), durations.iter().copied().max().unwrap_or(0));
            }
            fill_points(&all_keys, &buckets)
        }
    };

    ReadingMetricsData {
        metric: query.metric.as_str().to_string(),
        group_by: query.group_by.as_str().to_string(),
        scope: query.scope.as_str().to_string(),
        points,
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Map a date to its bucket key string for the given grouping.
fn bucket_key(date: NaiveDate, group_by: MetricsGroupBy) -> String {
    match group_by {
        MetricsGroupBy::Day => shared::bucket_key_day(date),
        MetricsGroupBy::Week => shared::bucket_key_week(date),
        MetricsGroupBy::Month => shared::bucket_key_month(date),
    }
}

/// Generate all contiguous bucket keys covering `[from, to]`.
fn all_bucket_keys(from: NaiveDate, to: NaiveDate, group_by: MetricsGroupBy) -> Vec<String> {
    let mut keys = Vec::new();
    match group_by {
        MetricsGroupBy::Day => {
            let mut date = from;
            while date <= to {
                keys.push(shared::bucket_key_day(date));
                date += chrono::Duration::days(1);
            }
        }
        MetricsGroupBy::Week => {
            let mut monday = shared::week_monday(from);
            let end_monday = shared::week_monday(to);
            while monday <= end_monday {
                keys.push(monday.format("%Y-%m-%d").to_string());
                monday += chrono::Duration::days(7);
            }
        }
        MetricsGroupBy::Month => {
            let mut year = from.year();
            let mut month = from.month();
            loop {
                keys.push(format!("{year:04}-{month:02}"));
                if year == to.year() && month == to.month() {
                    break;
                }
                if month == 12 {
                    year += 1;
                    month = 1;
                } else {
                    month += 1;
                }
            }
        }
    }
    keys
}

/// Produce a filled point series: every key in `all_keys` gets a value
/// (from `buckets` if present, otherwise 0).
fn fill_points(all_keys: &[String], buckets: &BTreeMap<String, i64>) -> Vec<MetricPoint> {
    all_keys
        .iter()
        .map(|key| MetricPoint {
            key: key.clone(),
            value: buckets.get(key).copied().unwrap_or(0),
        })
        .collect()
}

/// Compute reading sessions with their logical start dates.
///
/// Groups page stats by book, identifies sessions using a 5-minute gap threshold,
/// and returns `(session_start_date, session_duration)` pairs.
fn sessions_with_dates(page_stats: &[PageStat], time_config: &TimeConfig) -> Vec<(NaiveDate, i64)> {
    let mut by_book: HashMap<i64, Vec<PageStat>> = HashMap::new();
    for stat in page_stats.iter().filter(|s| s.duration > 0) {
        by_book.entry(stat.id_book).or_default().push(stat.clone());
    }

    let mut result = Vec::new();
    for book_stats in by_book.values() {
        let mut sorted = book_stats.clone();
        sorted.sort_by_key(|s| s.start_time);

        let mut session_start = sorted[0].start_time;
        let mut session_duration = sorted[0].duration;
        let mut last_end = sorted[0].start_time + sorted[0].duration;

        for stat in &sorted[1..] {
            if stat.start_time - last_end <= SESSION_GAP_SECONDS {
                session_duration += stat.duration;
            } else {
                result.push((
                    time_config.date_for_timestamp(session_start),
                    session_duration,
                ));
                session_start = stat.start_time;
                session_duration = stat.duration;
            }
            last_end = stat.start_time + stat.duration;
        }
        result.push((
            time_config.date_for_timestamp(session_start),
            session_duration,
        ));
    }

    result
}
