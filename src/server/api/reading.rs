use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::contracts::common::{ApiResponse, ResponseMeta};
use crate::domain::reading::ReadingService;
use crate::server::ServerState;

use super::shared::{
    ApiResponseError, ReadingAvailablePeriodsParams, ReadingCalendarParams,
    ReadingCompletionsParams, ReadingMetricsParams, ReadingSummaryParams,
    parse_reading_available_periods_query, parse_reading_calendar_query,
    parse_reading_completions_query, parse_reading_metrics_query, parse_reading_summary_query,
};

pub async fn reading_summary(
    State(state): State<ServerState>,
    Query(params): Query<ReadingSummaryParams>,
) -> Response {
    let query = match parse_reading_summary_query(&params) {
        Ok(q) => q,
        Err(error) => return error.into_response(),
    };

    let reading_data = match state.reading_data_store.get() {
        Some(rd) => rd,
        None => return ApiResponseError::internal_server_error().into_response(),
    };

    let data = ReadingService::summary(&reading_data, query);

    (
        StatusCode::OK,
        Json(ApiResponse {
            data,
            meta: ResponseMeta::now(),
        }),
    )
        .into_response()
}

pub async fn reading_metrics(
    State(state): State<ServerState>,
    Query(params): Query<ReadingMetricsParams>,
) -> Response {
    let query = match parse_reading_metrics_query(&params) {
        Ok(q) => q,
        Err(error) => return error.into_response(),
    };

    let reading_data = match state.reading_data_store.get() {
        Some(rd) => rd,
        None => return ApiResponseError::internal_server_error().into_response(),
    };

    let data = ReadingService::metrics(&reading_data, query);

    (
        StatusCode::OK,
        Json(ApiResponse {
            data,
            meta: ResponseMeta::now(),
        }),
    )
        .into_response()
}

pub async fn reading_available_periods(
    State(state): State<ServerState>,
    Query(params): Query<ReadingAvailablePeriodsParams>,
) -> Response {
    let query = match parse_reading_available_periods_query(&params) {
        Ok(q) => q,
        Err(error) => return error.into_response(),
    };

    let reading_data = match state.reading_data_store.get() {
        Some(rd) => rd,
        None => return ApiResponseError::internal_server_error().into_response(),
    };

    let data = ReadingService::available_periods(&reading_data, query);

    (
        StatusCode::OK,
        Json(ApiResponse {
            data,
            meta: ResponseMeta::now(),
        }),
    )
        .into_response()
}

pub async fn reading_calendar(
    State(state): State<ServerState>,
    Query(params): Query<ReadingCalendarParams>,
) -> Response {
    let query = match parse_reading_calendar_query(&params) {
        Ok(q) => q,
        Err(error) => return error.into_response(),
    };

    let reading_data = match state.reading_data_store.get() {
        Some(rd) => rd,
        None => return ApiResponseError::internal_server_error().into_response(),
    };

    let data = ReadingService::calendar(&reading_data, query);

    (
        StatusCode::OK,
        Json(ApiResponse {
            data,
            meta: ResponseMeta::now(),
        }),
    )
        .into_response()
}

pub async fn reading_completions(
    State(state): State<ServerState>,
    Query(params): Query<ReadingCompletionsParams>,
) -> Response {
    let query = match parse_reading_completions_query(&params) {
        Ok(q) => q,
        Err(error) => return error.into_response(),
    };

    let reading_data = match state.reading_data_store.get() {
        Some(rd) => rd,
        None => return ApiResponseError::internal_server_error().into_response(),
    };

    let data = ReadingService::completions(&reading_data, query);

    (
        StatusCode::OK,
        Json(ApiResponse {
            data,
            meta: ResponseMeta::now(),
        }),
    )
        .into_response()
}
