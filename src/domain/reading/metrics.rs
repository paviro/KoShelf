use chrono::{Duration as ChronoDuration, NaiveDate};

use crate::contracts::common::{ApiMeta, ContentTypeFilter};
use crate::contracts::statistics::{
    ActivityHeatmapConfig, ActivityOverview, ActivityStreaks, ActivityWeekResponse,
    ActivityWeeksResponse, ActivityYearDailyResponse, ActivityYearSummaryResponse, YearlySummary,
};
use crate::domain::meta::fallback_meta;
use crate::models::{StreakInfo, WeeklyStats};
use crate::runtime::ContractSnapshot;

pub fn activity_weeks(
    snapshot: &ContractSnapshot,
    content_type: ContentTypeFilter,
) -> ActivityWeeksResponse {
    snapshot
        .activity_weeks
        .get(content_type.as_str())
        .cloned()
        .unwrap_or_else(|| empty_activity_weeks_response(fallback_meta(snapshot), content_type))
}

pub fn activity_week(
    snapshot: &ContractSnapshot,
    content_type: ContentTypeFilter,
    week_key: &str,
) -> ActivityWeekResponse {
    let meta = snapshot
        .activity_weeks
        .get(content_type.as_str())
        .map(|weeks| weeks.meta.clone())
        .unwrap_or_else(|| fallback_meta(snapshot));

    snapshot
        .activity_weeks_by_key
        .get(content_type.as_str())
        .and_then(|weeks| weeks.get(week_key))
        .cloned()
        .unwrap_or_else(|| empty_activity_week_response(meta, content_type, week_key))
}

pub fn activity_year_daily(
    snapshot: &ContractSnapshot,
    content_type: ContentTypeFilter,
    year_key: &str,
    year_value: i32,
) -> ActivityYearDailyResponse {
    let meta = snapshot
        .activity_weeks
        .get(content_type.as_str())
        .map(|weeks| weeks.meta.clone())
        .unwrap_or_else(|| fallback_meta(snapshot));

    snapshot
        .activity_year_daily
        .get(content_type.as_str())
        .and_then(|years| years.get(year_key))
        .cloned()
        .unwrap_or_else(|| empty_activity_year_daily_response(meta, content_type, year_value))
}

pub fn activity_year_summary(
    snapshot: &ContractSnapshot,
    content_type: ContentTypeFilter,
    year_key: &str,
    year_value: i32,
) -> ActivityYearSummaryResponse {
    let meta = snapshot
        .activity_weeks
        .get(content_type.as_str())
        .map(|weeks| weeks.meta.clone())
        .unwrap_or_else(|| fallback_meta(snapshot));

    snapshot
        .activity_year_summary
        .get(content_type.as_str())
        .and_then(|years| years.get(year_key))
        .cloned()
        .unwrap_or_else(|| empty_activity_year_summary_response(meta, content_type, year_value))
}

fn empty_activity_overview() -> ActivityOverview {
    ActivityOverview {
        total_read_time: 0,
        total_page_reads: 0,
        longest_read_time_in_day: 0,
        most_pages_in_day: 0,
        average_session_duration: None,
        longest_session_duration: None,
        total_completions: 0,
        items_completed: 0,
        most_completions: 0,
    }
}

fn empty_activity_streaks() -> ActivityStreaks {
    ActivityStreaks {
        longest: StreakInfo::new(0, None, None),
        current: StreakInfo::new(0, None, None),
    }
}

fn empty_activity_weeks_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
) -> ActivityWeeksResponse {
    ActivityWeeksResponse {
        meta,
        content_type,
        available_years: vec![],
        available_weeks: vec![],
        overview: empty_activity_overview(),
        streaks: empty_activity_streaks(),
        heatmap_config: ActivityHeatmapConfig {
            max_scale_seconds: None,
        },
    }
}

fn empty_activity_week_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    week_key: &str,
) -> ActivityWeekResponse {
    let (start_date, end_date) = resolve_week_bounds(week_key);

    ActivityWeekResponse {
        meta,
        content_type,
        week_key: week_key.to_string(),
        stats: WeeklyStats {
            start_date,
            end_date,
            read_time: 0,
            pages_read: 0,
            longest_session_duration: None,
            average_session_duration: None,
        },
        daily_activity: vec![],
    }
}

fn empty_activity_year_daily_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    year: i32,
) -> ActivityYearDailyResponse {
    ActivityYearDailyResponse {
        meta,
        content_type,
        year,
        daily_activity: vec![],
        config: Some(ActivityHeatmapConfig {
            max_scale_seconds: None,
        }),
    }
}

fn empty_activity_year_summary_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    year: i32,
) -> ActivityYearSummaryResponse {
    ActivityYearSummaryResponse {
        meta,
        content_type,
        year,
        summary: YearlySummary { completed_count: 0 },
        monthly_aggregates: vec![],
        config: Some(ActivityHeatmapConfig {
            max_scale_seconds: None,
        }),
    }
}

fn resolve_week_bounds(week_key: &str) -> (String, String) {
    let start = NaiveDate::parse_from_str(week_key, "%Y-%m-%d")
        .ok()
        .unwrap_or_else(|| {
            NaiveDate::from_ymd_opt(1970, 1, 1).expect("constant date should be valid")
        });
    let end = start + ChronoDuration::days(6);

    (
        start.format("%Y-%m-%d").to_string(),
        end.format("%Y-%m-%d").to_string(),
    )
}
