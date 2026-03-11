use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::domain::reading::ReadingService;
use crate::server::ServerState;

use super::shared::{
    ContentTypeQuery, parse_content_type, parse_validated_year, runtime_snapshot,
    validate_month_key, validate_week_key, validate_year_key,
};

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

    let payload = ReadingService::activity_weeks(&snapshot, content_type);

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

    let payload = ReadingService::activity_week(&snapshot, content_type, week_key.as_str());

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
    let payload =
        ReadingService::activity_year_daily(&snapshot, content_type, year.as_str(), year_value);

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
    let payload =
        ReadingService::activity_year_summary(&snapshot, content_type, year.as_str(), year_value);

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

    let payload = ReadingService::activity_months(&snapshot, content_type);

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

    let payload = ReadingService::activity_month(&snapshot, content_type, month_key.as_str());

    (StatusCode::OK, Json(payload)).into_response()
}
