use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::collections::BTreeMap;

use crate::contracts::calendar::{
    ActivityMonthResponse, ActivityMonthsResponse, CalendarMonthlyStats,
};
use crate::contracts::common::{ApiMeta, ContentTypeFilter};
use crate::contracts::statistics::{
    ActivityHeatmapConfig, ActivityOverview, ActivityStreaks, ActivityWeekResponse,
    ActivityWeeksResponse, ActivityYearDailyResponse, ActivityYearSummaryResponse, YearlySummary,
};
use crate::models::{StreakInfo, WeeklyStats};
use crate::server::ServerState;

use super::shared::{
    ContentTypeQuery, fallback_meta, parse_content_type, parse_validated_year, resolve_week_bounds,
    runtime_snapshot, validate_month_key, validate_week_key, validate_year_key,
};

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

fn empty_activity_months_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
) -> ActivityMonthsResponse {
    ActivityMonthsResponse {
        meta,
        content_type,
        months: vec![],
    }
}

fn empty_calendar_monthly_stats() -> CalendarMonthlyStats {
    CalendarMonthlyStats {
        items_read: 0,
        pages_read: 0,
        time_read: 0,
        days_read_pct: 0,
    }
}

fn empty_activity_month_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
) -> ActivityMonthResponse {
    ActivityMonthResponse {
        meta,
        content_type,
        events: vec![],
        items: BTreeMap::new(),
        stats: empty_calendar_monthly_stats(),
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

pub async fn activity_weeks(
    State(state): State<ServerState>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let payload = snapshot
        .activity_weeks
        .get(content_type.as_str())
        .cloned()
        .unwrap_or_else(|| empty_activity_weeks_response(fallback_meta(&snapshot), content_type));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn activity_week(
    State(state): State<ServerState>,
    Path(week_key): Path<String>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };
    let week_key = match validate_week_key(&week_key) {
        Ok(week_key) => week_key,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let meta = snapshot
        .activity_weeks
        .get(content_type.as_str())
        .map(|weeks| weeks.meta.clone())
        .unwrap_or_else(|| fallback_meta(&snapshot));
    let payload = snapshot
        .activity_weeks_by_key
        .get(content_type.as_str())
        .and_then(|weeks| weeks.get(week_key.as_str()))
        .cloned()
        .unwrap_or_else(|| empty_activity_week_response(meta, content_type, week_key.as_str()));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn activity_year_daily(
    State(state): State<ServerState>,
    Path(year): Path<String>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };
    let year = match validate_year_key(&year) {
        Ok(year) => year,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let year_value = match parse_validated_year(&year) {
        Ok(value) => value,
        Err(error) => return error.into_response(),
    };
    let meta = snapshot
        .activity_weeks
        .get(content_type.as_str())
        .map(|weeks| weeks.meta.clone())
        .unwrap_or_else(|| fallback_meta(&snapshot));
    let payload = snapshot
        .activity_year_daily
        .get(content_type.as_str())
        .and_then(|years| years.get(year.as_str()))
        .cloned()
        .unwrap_or_else(|| empty_activity_year_daily_response(meta, content_type, year_value));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn activity_year_summary(
    State(state): State<ServerState>,
    Path(year): Path<String>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };
    let year = match validate_year_key(&year) {
        Ok(year) => year,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let year_value = match parse_validated_year(&year) {
        Ok(value) => value,
        Err(error) => return error.into_response(),
    };
    let meta = snapshot
        .activity_weeks
        .get(content_type.as_str())
        .map(|weeks| weeks.meta.clone())
        .unwrap_or_else(|| fallback_meta(&snapshot));
    let payload = snapshot
        .activity_year_summary
        .get(content_type.as_str())
        .and_then(|years| years.get(year.as_str()))
        .cloned()
        .unwrap_or_else(|| empty_activity_year_summary_response(meta, content_type, year_value));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn activity_months(
    State(state): State<ServerState>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let payload = snapshot
        .activity_months
        .get(content_type.as_str())
        .cloned()
        .unwrap_or_else(|| empty_activity_months_response(fallback_meta(&snapshot), content_type));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn activity_month(
    State(state): State<ServerState>,
    Path(month_key): Path<String>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };
    let month_key = match validate_month_key(&month_key) {
        Ok(month_key) => month_key,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let meta = snapshot
        .activity_months
        .get(content_type.as_str())
        .map(|months| months.meta.clone())
        .unwrap_or_else(|| fallback_meta(&snapshot));
    let payload = snapshot
        .activity_months_by_key
        .get(content_type.as_str())
        .and_then(|months| months.get(month_key.as_str()))
        .cloned()
        .unwrap_or_else(|| empty_activity_month_response(meta, content_type));

    (StatusCode::OK, Json(payload)).into_response()
}
