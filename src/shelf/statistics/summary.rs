//! Summary endpoint computation for `/api/reading/summary`.

use std::collections::HashMap;

use chrono::NaiveDate;

use crate::server::api::responses::reading::{
    HeatmapConfig, ReadingOverview, ReadingStreaks, ReadingSummaryData, ResolvedRange, StreakData,
};
use crate::shelf::statistics::compute::scaling::round_pages;
use crate::shelf::statistics::compute::sessions;
use crate::shelf::statistics::queries::ReadingSummaryQuery;
use crate::shelf::statistics::shared;
use crate::shelf::time_config::TimeConfig;
use crate::store::memory::ReadingData;

/// Compute the full summary response from reading data and a validated query.
pub fn summary(reading_data: &ReadingData, query: ReadingSummaryQuery) -> ReadingSummaryData {
    let time_config = shared::resolve_time_config(&reading_data.time_config, query.tz);
    let tz_name = time_config
        .timezone
        .map(|tz| tz.to_string())
        .unwrap_or_else(|| "local".to_string());

    let stats = shared::filter_stats_by_scope(&reading_data.stats_data, query.scope);
    let (page_stats, resolved_from, resolved_to) = shared::filter_and_resolve_range(
        &stats.page_stats,
        query.range.as_ref().map(|r| (r.from, r.to)),
        &time_config,
    );

    // Page counts accumulate as floats (each page_stat contributes its scaling
    // factor, e.g. 1.5) and are rounded once per day to avoid per-page rounding error.
    let mut daily_read_time: HashMap<NaiveDate, i64> = HashMap::new();
    let mut daily_scaled_pages: HashMap<NaiveDate, f64> = HashMap::new();
    let mut total_read_time: i64 = 0;

    for stat in &page_stats {
        let date = time_config.date_for_timestamp(stat.start_time);
        let factor = reading_data.page_scaling.factor_for_book_id(stat.id_book);
        total_read_time += stat.duration;
        *daily_read_time.entry(date).or_insert(0) += stat.duration;
        *daily_scaled_pages.entry(date).or_insert(0.0) += factor;
    }

    let daily_page_reads: HashMap<NaiveDate, i64> = daily_scaled_pages
        .iter()
        .map(|(&date, &v)| (date, round_pages(v)))
        .collect();
    let total_page_reads: i64 = daily_page_reads.values().sum();

    let longest_reading_time_in_day_sec = daily_read_time.values().copied().max().unwrap_or(0);
    let most_pages_in_day = daily_page_reads.values().copied().max().unwrap_or(0);

    let (average_session_duration_sec, longest_session_duration_sec) =
        sessions::session_metrics(&page_stats);
    let session_count = sessions::aggregate_session_durations(&page_stats).len() as i64;

    let (total_completions, items_completed) =
        shared::count_completions_in_range(&stats, &resolved_from, &resolved_to);

    let streaks = compute_streaks(&daily_read_time, &time_config);
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

    // Streak ending today or yesterday counts as "current".
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
