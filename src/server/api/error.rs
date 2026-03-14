use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::server::api::responses::error::{ApiErrorCode, ApiErrorResponse};

// ── Error plumbing ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct ApiResponseError {
    status: StatusCode,
    code: ApiErrorCode,
    message: Option<String>,
}

impl ApiResponseError {
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

impl From<(ApiErrorCode, String)> for ApiResponseError {
    fn from((code, msg): (ApiErrorCode, String)) -> Self {
        Self::bad_request_with_message(code, msg)
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
}
