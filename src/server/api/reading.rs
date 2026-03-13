use axum::{Json, extract::Query, response::IntoResponse};

use crate::contracts::common::ApiResponse;
use crate::domain::reading;

use super::shared::{
    ApiResult, ReadingAvailablePeriodsParams, ReadingCalendarParams, ReadingCompletionsParams,
    ReadingDataGuard, ReadingMetricsParams, ReadingSummaryParams,
    parse_reading_available_periods_query, parse_reading_calendar_query,
    parse_reading_completions_query, parse_reading_metrics_query, parse_reading_summary_query,
};

pub(crate) async fn reading_summary(
    reading_data: ReadingDataGuard,
    Query(params): Query<ReadingSummaryParams>,
) -> ApiResult<impl IntoResponse> {
    let query = parse_reading_summary_query(&params)?;
    let data = reading::summary(&reading_data, query);
    Ok(Json(ApiResponse::new(data)))
}

pub(crate) async fn reading_metrics(
    reading_data: ReadingDataGuard,
    Query(params): Query<ReadingMetricsParams>,
) -> ApiResult<impl IntoResponse> {
    let query = parse_reading_metrics_query(&params)?;
    let data = reading::metrics(&reading_data, query);
    Ok(Json(ApiResponse::new(data)))
}

pub(crate) async fn reading_available_periods(
    reading_data: ReadingDataGuard,
    Query(params): Query<ReadingAvailablePeriodsParams>,
) -> ApiResult<impl IntoResponse> {
    let query = parse_reading_available_periods_query(&params)?;
    let data = reading::available_periods(&reading_data, query);
    Ok(Json(ApiResponse::new(data)))
}

pub(crate) async fn reading_calendar(
    reading_data: ReadingDataGuard,
    Query(params): Query<ReadingCalendarParams>,
) -> ApiResult<impl IntoResponse> {
    let query = parse_reading_calendar_query(&params)?;
    let data = reading::calendar(&reading_data, query);
    Ok(Json(ApiResponse::new(data)))
}

pub(crate) async fn reading_completions(
    reading_data: ReadingDataGuard,
    Query(params): Query<ReadingCompletionsParams>,
) -> ApiResult<impl IntoResponse> {
    let query = parse_reading_completions_query(&params)?;
    let data = reading::completions(&reading_data, query);
    Ok(Json(ApiResponse::new(data)))
}
