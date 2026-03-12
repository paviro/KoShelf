use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::contracts::common::{ApiResponse, ResponseMeta};
use crate::domain::reading::ReadingService;
use crate::server::ServerState;

use super::shared::{ApiResponseError, ReadingSummaryParams, parse_reading_summary_query};

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
