use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiErrorCode {
    InvalidQuery,
    InvalidWeekKey,
    InvalidMonthKey,
    InvalidYear,
    InvalidCredentials,
    Unauthorized,
    RateLimited,
    NotFound,
    InternalServerError,
}

impl ApiErrorCode {
    pub fn default_message(self) -> &'static str {
        match self {
            Self::InvalidQuery => "invalid query parameter",
            Self::InvalidWeekKey => "week_key must be a valid Monday date in YYYY-MM-DD format",
            Self::InvalidMonthKey => "month_key must be in YYYY-MM format",
            Self::InvalidYear => "year must be a valid YYYY value",
            Self::InvalidCredentials => "invalid credentials",
            Self::Unauthorized => "unauthorized",
            Self::RateLimited => "too many requests",
            Self::NotFound => "requested resource was not found",
            Self::InternalServerError => "internal server error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiError,
}

impl ApiErrorResponse {
    pub fn new(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self {
            error: ApiError {
                code,
                message: message.into(),
            },
        }
    }

    pub fn from_code(code: ApiErrorCode) -> Self {
        Self::new(code, code.default_message())
    }
}
