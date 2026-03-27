use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiErrorCode {
    InvalidQuery,
    InvalidMonthKey,
    InvalidYear,
    InvalidCredentials,
    PasswordTooShort,
    Unauthorized,
    RateLimited,
    Conflict,
    NotFound,
    InternalServerError,
}

impl ApiErrorCode {
    pub fn default_message(self) -> &'static str {
        match self {
            Self::InvalidQuery => "invalid query parameter",
            Self::InvalidMonthKey => "month_key must be in YYYY-MM format",
            Self::InvalidYear => "year must be a valid YYYY value",
            Self::InvalidCredentials => "invalid credentials",
            Self::PasswordTooShort => "password is too short",
            Self::Unauthorized => "unauthorized",
            Self::RateLimited => "too many requests",
            Self::Conflict => "resource was modified externally; re-fetch and retry",
            Self::NotFound => "requested resource was not found",
            Self::InternalServerError => "internal server error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiError,
}

impl ApiErrorResponse {
    pub fn new(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self::new_with_details(code, message, None)
    }

    pub fn new_with_details(
        code: ApiErrorCode,
        message: impl Into<String>,
        details: Option<Value>,
    ) -> Self {
        Self {
            error: ApiError {
                code,
                message: message.into(),
                details,
            },
        }
    }

    pub fn from_code(code: ApiErrorCode) -> Self {
        Self::new(code, code.default_message())
    }

    pub fn from_code_with_details(code: ApiErrorCode, details: Option<Value>) -> Self {
        Self::new_with_details(code, code.default_message(), details)
    }
}
