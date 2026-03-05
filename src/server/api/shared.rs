use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{Duration as ChronoDuration, NaiveDate};
use serde::Deserialize;
use std::sync::Arc;

use crate::contracts::common::{ApiMeta, ContentTypeFilter, MonthKey, WeekKey, YearKey};
use crate::contracts::error::{ApiErrorCode, ApiErrorResponse};
use crate::runtime::ContractSnapshot;
use crate::server::ServerState;

#[derive(Debug, Deserialize)]
pub struct ContentTypeQuery {
    content_type: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ApiResponseError {
    status: StatusCode,
    code: ApiErrorCode,
}

impl ApiResponseError {
    pub(crate) const fn bad_request(code: ApiErrorCode) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
        }
    }

    pub(crate) const fn internal_server_error() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: ApiErrorCode::InternalServerError,
        }
    }
}

impl IntoResponse for ApiResponseError {
    fn into_response(self) -> Response {
        api_error(self.status, self.code)
    }
}

pub(crate) type ApiResult<T> = Result<T, ApiResponseError>;

fn api_error(status: StatusCode, code: ApiErrorCode) -> Response {
    (status, Json(ApiErrorResponse::from_code(code))).into_response()
}

pub(crate) fn parse_content_type(query: ContentTypeQuery) -> ApiResult<ContentTypeFilter> {
    ContentTypeFilter::parse(query.content_type.as_deref()).map_err(ApiResponseError::bad_request)
}

pub(crate) fn validate_week_key(value: &str) -> ApiResult<WeekKey> {
    WeekKey::parse(value).map_err(ApiResponseError::bad_request)
}

pub(crate) fn validate_month_key(value: &str) -> ApiResult<MonthKey> {
    MonthKey::parse(value).map_err(ApiResponseError::bad_request)
}

pub(crate) fn validate_year_key(value: &str) -> ApiResult<YearKey> {
    YearKey::parse(value).map_err(ApiResponseError::bad_request)
}

pub(crate) fn runtime_snapshot(state: &ServerState) -> ApiResult<Arc<ContractSnapshot>> {
    state
        .snapshot_store
        .get()
        .ok_or_else(ApiResponseError::internal_server_error)
}

pub(crate) fn fallback_meta(snapshot: &ContractSnapshot) -> ApiMeta {
    snapshot
        .site
        .as_ref()
        .map(|site| site.meta.clone())
        .or_else(|| snapshot.items.as_ref().map(|items| items.meta.clone()))
        .or_else(|| {
            snapshot
                .activity_weeks
                .values()
                .next()
                .map(|weeks| weeks.meta.clone())
        })
        .or_else(|| {
            snapshot
                .activity_months
                .values()
                .next()
                .map(|months| months.meta.clone())
        })
        .or_else(|| {
            snapshot
                .completion_years
                .values()
                .next()
                .map(|years| years.meta.clone())
        })
        .unwrap_or(ApiMeta {
            version: "unknown".to_string(),
            generated_at: "".to_string(),
        })
}

pub(crate) fn parse_validated_year(year: &YearKey) -> ApiResult<i32> {
    year.as_str()
        .parse::<i32>()
        .map_err(|_| ApiResponseError::internal_server_error())
}

pub(crate) fn resolve_week_bounds(week_key: &str) -> (String, String) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn decode_error_response(response: Response) -> ApiErrorResponse {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error response body should be readable");
        serde_json::from_slice::<ApiErrorResponse>(&bytes)
            .expect("error response body should contain valid JSON")
    }

    #[tokio::test]
    async fn bad_request_error_maps_to_bad_request_status_and_code() {
        let response =
            ApiResponseError::bad_request(ApiErrorCode::InvalidContentType).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let payload = decode_error_response(response).await;
        assert_eq!(payload.error.code, ApiErrorCode::InvalidContentType);
        assert_eq!(
            payload.error.message,
            ApiErrorCode::InvalidContentType.default_message()
        );
    }

    #[tokio::test]
    async fn internal_error_maps_to_internal_server_error_status_and_code() {
        let response = ApiResponseError::internal_server_error().into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let payload = decode_error_response(response).await;
        assert_eq!(payload.error.code, ApiErrorCode::InternalServerError);
        assert_eq!(
            payload.error.message,
            ApiErrorCode::InternalServerError.default_message()
        );
    }
}
