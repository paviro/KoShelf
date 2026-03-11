use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::domain::reading::ReadingService;
use crate::server::ServerState;

use super::shared::{
    ContentTypeQuery, parse_content_type, parse_validated_year, runtime_snapshot, validate_year_key,
};

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

    let payload = ReadingService::completion_years(&snapshot, content_type);

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
    let payload =
        ReadingService::completion_year(&snapshot, content_type, year.as_str(), year_value);

    (StatusCode::OK, Json(payload)).into_response()
}
