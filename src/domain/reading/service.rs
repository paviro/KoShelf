use std::collections::HashMap;

use chrono::NaiveDate;

use crate::contracts::calendar::{ActivityMonthResponse, ActivityMonthsResponse};
use crate::contracts::common::ContentTypeFilter;
use crate::contracts::reading::{
    HeatmapConfig, ReadingOverview, ReadingStreaks, ReadingSummaryData, ResolvedRange, StreakData,
};
use crate::contracts::recap::{CompletionYearResponse, CompletionYearsResponse};
use crate::contracts::statistics::{
    ActivityWeekResponse, ActivityWeeksResponse, ActivityYearDailyResponse,
    ActivityYearSummaryResponse,
};
use crate::domain::reading::queries::ReadingSummaryQuery;
use crate::domain::reading::{calendar, completions, metrics};
use crate::koreader::session;
use crate::models::{ContentType, PageStat};
use crate::runtime::{ContractSnapshot, ReadingData};
use crate::time_config::TimeConfig;

#[derive(Debug, Default, Clone, Copy)]
pub struct ReadingService;

impl ReadingService {
    // ── Legacy snapshot-based methods ────────────────────────────────────

    pub fn activity_weeks(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
    ) -> ActivityWeeksResponse {
        metrics::activity_weeks(snapshot, content_type)
    }

    pub fn activity_week(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        week_key: &str,
    ) -> ActivityWeekResponse {
        metrics::activity_week(snapshot, content_type, week_key)
    }

    pub fn activity_year_daily(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        year_key: &str,
        year_value: i32,
    ) -> ActivityYearDailyResponse {
        metrics::activity_year_daily(snapshot, content_type, year_key, year_value)
    }

    pub fn activity_year_summary(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        year_key: &str,
        year_value: i32,
    ) -> ActivityYearSummaryResponse {
        metrics::activity_year_summary(snapshot, content_type, year_key, year_value)
    }

    pub fn activity_months(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
    ) -> ActivityMonthsResponse {
        calendar::activity_months(snapshot, content_type)
    }

    pub fn activity_month(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        month_key: &str,
    ) -> ActivityMonthResponse {
        calendar::activity_month(snapshot, content_type, month_key)
    }

    pub fn completion_years(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
    ) -> CompletionYearsResponse {
        completions::completion_years(snapshot, content_type)
    }

    pub fn completion_year(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        year_key: &str,
        year_value: i32,
    ) -> CompletionYearResponse {
        completions::completion_year(snapshot, content_type, year_key, year_value)
    }

    // ── New on-demand reading endpoints ─────────────────────────────────

    pub fn summary(reading_data: &ReadingData, query: ReadingSummaryQuery) -> ReadingSummaryData {
        let time_config = resolve_time_config(&reading_data.time_config, query.tz);
        let tz_name = time_config
            .timezone
            .map(|tz| tz.to_string())
            .unwrap_or_else(|| "local".to_string());

        // Filter page_stats by scope (content type).
        let stats = match query.scope {
            ContentTypeFilter::All => reading_data.stats_data.clone(),
            ContentTypeFilter::Books => reading_data
                .stats_data
                .filtered_by_content_type(ContentType::Book),
            ContentTypeFilter::Comics => reading_data
                .stats_data
                .filtered_by_content_type(ContentType::Comic),
        };

        // Filter page_stats by date range and resolve the effective range.
        let (page_stats, resolved_from, resolved_to) = filter_and_resolve_range(
            &stats.page_stats,
            query.range.as_ref().map(|r| (r.from, r.to)),
            &time_config,
        );

        // Compute daily aggregates.
        let mut daily_read_time: HashMap<NaiveDate, i64> = HashMap::new();
        let mut daily_page_reads: HashMap<NaiveDate, i64> = HashMap::new();
        let mut total_read_time: i64 = 0;
        let mut total_page_reads: i64 = 0;

        for stat in &page_stats {
            let date = time_config.date_for_timestamp(stat.start_time);
            total_read_time += stat.duration;
            total_page_reads += 1;
            *daily_read_time.entry(date).or_insert(0) += stat.duration;
            *daily_page_reads.entry(date).or_insert(0) += 1;
        }

        let longest_reading_time_in_day_sec = daily_read_time.values().copied().max().unwrap_or(0);
        let most_pages_in_day = daily_page_reads.values().copied().max().unwrap_or(0);

        // Session metrics.
        let (average_session_duration_sec, longest_session_duration_sec) =
            session::session_metrics(&page_stats);
        let session_count = session::aggregate_session_durations(&page_stats).len() as i64;

        // Completion counts within the resolved range.
        let (total_completions, items_completed) =
            count_completions_in_range(&stats, &resolved_from, &resolved_to);

        // Streaks (computed from daily activity within range).
        let streaks = compute_streaks(&daily_read_time, &time_config);

        // Heatmap config.
        let heatmap_config = HeatmapConfig {
            max_scale_sec: reading_data.heatmap_scale_max.map(|v| v as i64),
        };

        ReadingSummaryData {
            range: ResolvedRange {
                from: resolved_from.format("%Y-%m-%d").to_string(),
                to: resolved_to.format("%Y-%m-%d").to_string(),
                tz: tz_name,
            },
            overview: ReadingOverview {
                reading_time_sec: total_read_time,
                pages_read: total_page_reads,
                sessions: session_count,
                completions: total_completions,
                items_completed,
                longest_reading_time_in_day_sec,
                most_pages_in_day,
                average_session_duration_sec,
                longest_session_duration_sec,
            },
            streaks,
            heatmap_config,
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Build a `TimeConfig` with an optional per-request timezone override.
fn resolve_time_config(base: &TimeConfig, tz_override: Option<chrono_tz::Tz>) -> TimeConfig {
    match tz_override {
        Some(tz) => TimeConfig::new(Some(tz), base.day_start_minutes),
        None => base.clone(),
    }
}

/// Filter page stats by date range and determine the resolved from/to dates.
/// Returns (filtered_stats, resolved_from, resolved_to).
fn filter_and_resolve_range(
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
fn count_completions_in_range(
    stats: &crate::models::StatisticsData,
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

/// Compute current and longest reading streaks from daily read time data.
fn compute_streaks(
    daily_read_time: &HashMap<NaiveDate, i64>,
    time_config: &TimeConfig,
) -> ReadingStreaks {
    if daily_read_time.is_empty() {
        return ReadingStreaks {
            current: StreakData {
                days: 0,
                start_date: None,
                end_date: None,
            },
            longest: StreakData {
                days: 0,
                start_date: None,
                end_date: None,
            },
        };
    }

    let mut sorted_dates: Vec<NaiveDate> = daily_read_time.keys().copied().collect();
    sorted_dates.sort();

    let today = time_config.today_date();

    // Build streaks.
    let mut streaks: Vec<(i64, NaiveDate, NaiveDate)> = Vec::new();
    let mut streak_start = sorted_dates[0];
    let mut streak_len = 1i64;

    for i in 1..sorted_dates.len() {
        if sorted_dates[i] == sorted_dates[i - 1] + chrono::Duration::days(1) {
            streak_len += 1;
        } else {
            streaks.push((streak_len, streak_start, sorted_dates[i - 1]));
            streak_start = sorted_dates[i];
            streak_len = 1;
        }
    }
    streaks.push((streak_len, streak_start, *sorted_dates.last().unwrap()));

    // Longest streak.
    let longest = streaks
        .iter()
        .max_by_key(|&&(len, _, _)| len)
        .map(|&(len, start, end)| StreakData {
            days: len,
            start_date: Some(start.format("%Y-%m-%d").to_string()),
            end_date: Some(end.format("%Y-%m-%d").to_string()),
        })
        .unwrap_or(StreakData {
            days: 0,
            start_date: None,
            end_date: None,
        });

    // Current streak: the streak that ends today or yesterday.
    let last_reading_date = *sorted_dates.last().unwrap();
    let days_since_last = (today - last_reading_date).num_days();

    let current = if days_since_last <= 1 {
        streaks
            .iter()
            .find(|&&(_, _, end)| end == last_reading_date)
            .map(|&(len, start, _)| StreakData {
                days: len,
                start_date: Some(start.format("%Y-%m-%d").to_string()),
                end_date: None,
            })
            .unwrap_or(StreakData {
                days: 0,
                start_date: None,
                end_date: None,
            })
    } else {
        StreakData {
            days: 0,
            start_date: None,
            end_date: None,
        }
    };

    ReadingStreaks { current, longest }
}
