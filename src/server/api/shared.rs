use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::contracts::common::{ContentTypeFilter, MonthKey, WeekKey, YearKey};
use crate::contracts::error::{ApiErrorCode, ApiErrorResponse};
use crate::domain::library::queries::{ItemSort, SortOrder};
use crate::runtime::ContractSnapshot;
use crate::server::ServerState;

// ── Legacy query param (used by activity/completion handlers) ──────────────

#[derive(Debug, Deserialize)]
pub struct ContentTypeQuery {
    content_type: Option<String>,
}

// ── New query params (new contract) ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ScopeQuery {
    pub scope: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

// ── Error plumbing ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct ApiResponseError {
    status: StatusCode,
    code: ApiErrorCode,
    message: Option<String>,
}

impl ApiResponseError {
    pub(crate) fn bad_request(code: ApiErrorCode) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: None,
        }
    }

    pub(crate) fn bad_request_with_message(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: Some(message.into()),
        }
    }

    pub(crate) fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: ApiErrorCode::NotFound,
            message: None,
        }
    }

    pub(crate) fn internal_server_error() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: ApiErrorCode::InternalServerError,
            message: None,
        }
    }
}

impl IntoResponse for ApiResponseError {
    fn into_response(self) -> Response {
        let body = match self.message {
            Some(msg) => ApiErrorResponse::new(self.code, msg),
            None => ApiErrorResponse::from_code(self.code),
        };
        (self.status, Json(body)).into_response()
    }
}

pub(crate) type ApiResult<T> = Result<T, ApiResponseError>;

// ── Parsing helpers ────────────────────────────────────────────────────────

pub(crate) fn parse_content_type(query: ContentTypeQuery) -> ApiResult<ContentTypeFilter> {
    ContentTypeFilter::parse(query.content_type.as_deref()).map_err(ApiResponseError::bad_request)
}

pub(crate) fn parse_scope(value: Option<&str>) -> ApiResult<ContentTypeFilter> {
    match value {
        None => Ok(ContentTypeFilter::All),
        Some("all") => Ok(ContentTypeFilter::All),
        Some("books") => Ok(ContentTypeFilter::Books),
        Some("comics") => Ok(ContentTypeFilter::Comics),
        Some(_) => Err(ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            "scope must be one of: all, books, comics",
        )),
    }
}

pub(crate) fn parse_item_sort(value: Option<&str>) -> ApiResult<ItemSort> {
    match value {
        None => Ok(ItemSort::default()),
        Some(v) => ItemSort::parse(v).map_err(|_| {
            ApiResponseError::bad_request_with_message(
                ApiErrorCode::InvalidQuery,
                "sort must be one of: title, author, status, progress, rating, annotations, last_open_at",
            )
        }),
    }
}

pub(crate) fn parse_sort_order(value: Option<&str>) -> ApiResult<Option<SortOrder>> {
    match value {
        None => Ok(None),
        Some(v) => SortOrder::parse(v).map(Some).map_err(|_| {
            ApiResponseError::bad_request_with_message(
                ApiErrorCode::InvalidQuery,
                "order must be one of: asc, desc",
            )
        }),
    }
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

pub(crate) fn parse_validated_year(year: &YearKey) -> ApiResult<i32> {
    year.as_str()
        .parse::<i32>()
        .map_err(|_| ApiResponseError::internal_server_error())
}

pub(crate) fn request_meta() -> crate::contracts::common::ApiMeta {
    crate::contracts::common::ApiMeta {
        version: env!("CARGO_PKG_VERSION").to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
    }
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

    #[tokio::test]
    async fn not_found_maps_to_404_status_and_code() {
        let response = ApiResponseError::not_found().into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let payload = decode_error_response(response).await;
        assert_eq!(payload.error.code, ApiErrorCode::NotFound);
    }

    #[tokio::test]
    async fn bad_request_with_custom_message_uses_provided_message() {
        let response = ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            "scope must be one of: all, books, comics",
        )
        .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let payload = decode_error_response(response).await;
        assert_eq!(payload.error.code, ApiErrorCode::InvalidQuery);
        assert_eq!(
            payload.error.message,
            "scope must be one of: all, books, comics"
        );
    }

    #[test]
    fn parse_scope_accepts_valid_values() {
        assert_eq!(parse_scope(None).unwrap(), ContentTypeFilter::All);
        assert_eq!(parse_scope(Some("all")).unwrap(), ContentTypeFilter::All);
        assert_eq!(
            parse_scope(Some("books")).unwrap(),
            ContentTypeFilter::Books
        );
        assert_eq!(
            parse_scope(Some("comics")).unwrap(),
            ContentTypeFilter::Comics
        );
    }

    #[test]
    fn parse_scope_rejects_invalid_values() {
        assert!(parse_scope(Some("invalid")).is_err());
    }

    #[test]
    fn parse_item_sort_accepts_valid_values() {
        assert_eq!(parse_item_sort(None).unwrap(), ItemSort::Title);
        assert_eq!(parse_item_sort(Some("title")).unwrap(), ItemSort::Title);
        assert_eq!(parse_item_sort(Some("rating")).unwrap(), ItemSort::Rating);
    }

    #[test]
    fn parse_item_sort_rejects_invalid_values() {
        assert!(parse_item_sort(Some("invalid")).is_err());
    }

    #[test]
    fn parse_sort_order_accepts_valid_values() {
        assert_eq!(parse_sort_order(None).unwrap(), None);
        assert_eq!(parse_sort_order(Some("asc")).unwrap(), Some(SortOrder::Asc));
        assert_eq!(
            parse_sort_order(Some("desc")).unwrap(),
            Some(SortOrder::Desc)
        );
    }

    #[test]
    fn parse_sort_order_rejects_invalid_values() {
        assert!(parse_sort_order(Some("invalid")).is_err());
    }
}
