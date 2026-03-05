use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::contracts::common::{ApiMeta, ContentTypeFilter};
use crate::contracts::recap::{
    CompletionYearResponse, CompletionYearsResponse, RecapSummaryResponse,
};
use crate::server::ServerState;

use super::shared::{
    ContentTypeQuery, fallback_meta, parse_content_type, parse_validated_year, runtime_snapshot,
    validate_year_key,
};

fn empty_recap_summary_response() -> RecapSummaryResponse {
    RecapSummaryResponse {
        total_items: 0,
        total_time_seconds: 0,
        total_time_days: 0,
        total_time_hours: 0,
        longest_session_hours: 0,
        longest_session_minutes: 0,
        average_session_hours: 0,
        average_session_minutes: 0,
        active_days: 0,
        active_days_percentage: 0.0,
        longest_streak: 0,
        best_month_name: None,
    }
}

fn empty_completion_years_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
) -> CompletionYearsResponse {
    CompletionYearsResponse {
        meta,
        content_type,
        available_years: vec![],
        latest_year: None,
    }
}

fn empty_completion_year_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    year: i32,
) -> CompletionYearResponse {
    CompletionYearResponse {
        meta,
        content_type,
        year,
        summary: empty_recap_summary_response(),
        months: vec![],
        items: vec![],
        share_assets: None,
    }
}

pub async fn completion_years(
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
        .completion_years
        .get(content_type.as_str())
        .cloned()
        .unwrap_or_else(|| empty_completion_years_response(fallback_meta(&snapshot), content_type));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn completion_year(
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
        .completion_years
        .get(content_type.as_str())
        .map(|years| years.meta.clone())
        .unwrap_or_else(|| fallback_meta(&snapshot));
    let payload = snapshot
        .completion_years_by_key
        .get(content_type.as_str())
        .and_then(|years| years.get(year.as_str()))
        .cloned()
        .unwrap_or_else(|| empty_completion_year_response(meta, content_type, year_value));

    (StatusCode::OK, Json(payload)).into_response()
}
