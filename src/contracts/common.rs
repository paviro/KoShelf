use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::error::ApiErrorCode;

// ── Response envelope ─────────────────────────────────────────────────────

/// Generic response envelope for all API endpoints.
///
/// Wraps a `data` payload with response metadata.
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    pub meta: ResponseMeta,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            meta: ResponseMeta::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseMeta {
    pub version: String,
    pub generated_at: String,
}

impl ResponseMeta {
    pub fn now() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContentTypeFilter {
    #[default]
    All,
    Books,
    Comics,
}

impl ContentTypeFilter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Books => "books",
            Self::Comics => "comics",
        }
    }

    /// SQL bind value for the `content_type` WHERE clause.
    /// Returns `None` for `All` (matches everything via `?1 IS NULL`).
    pub fn sql_value(self) -> Option<&'static str> {
        match self {
            Self::All => None,
            Self::Books => Some("book"),
            Self::Comics => Some("comic"),
        }
    }
}

impl fmt::Display for ContentTypeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WeekKey(String);

impl WeekKey {
    pub fn parse(value: &str) -> Result<Self, ApiErrorCode> {
        let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|_| ApiErrorCode::InvalidWeekKey)?;

        if date.weekday() != Weekday::Mon {
            return Err(ApiErrorCode::InvalidWeekKey);
        }

        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WeekKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for WeekKey {
    type Err = ApiErrorCode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MonthKey(String);

impl MonthKey {
    pub fn parse(value: &str) -> Result<Self, ApiErrorCode> {
        if value.len() != 7 {
            return Err(ApiErrorCode::InvalidMonthKey);
        }

        let parsed = NaiveDate::parse_from_str(&format!("{}-01", value), "%Y-%m-%d")
            .map_err(|_| ApiErrorCode::InvalidMonthKey)?;

        if parsed.format("%Y-%m").to_string() != value {
            return Err(ApiErrorCode::InvalidMonthKey);
        }

        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MonthKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for MonthKey {
    type Err = ApiErrorCode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct YearKey(String);

impl YearKey {
    pub fn parse(value: &str) -> Result<Self, ApiErrorCode> {
        if value.len() != 4 || !value.chars().all(|c| c.is_ascii_digit()) {
            return Err(ApiErrorCode::InvalidYear);
        }

        let year = value
            .parse::<i32>()
            .map_err(|_| ApiErrorCode::InvalidYear)?;

        if year <= 0 {
            return Err(ApiErrorCode::InvalidYear);
        }

        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for YearKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for YearKey {
    type Err = ApiErrorCode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::{ContentTypeFilter, MonthKey, WeekKey, YearKey};

    #[test]
    fn content_type_filter_serializes_as_lowercase_values() {
        assert_eq!(
            serde_json::to_string(&ContentTypeFilter::All)
                .expect("content type filter should serialize"),
            "\"all\""
        );
        assert_eq!(
            serde_json::to_string(&ContentTypeFilter::Books)
                .expect("content type filter should serialize"),
            "\"books\""
        );
        assert_eq!(
            serde_json::to_string(&ContentTypeFilter::Comics)
                .expect("content type filter should serialize"),
            "\"comics\""
        );
    }

    #[test]
    fn week_key_accepts_monday_and_rejects_other_days() {
        assert!(WeekKey::parse("2026-03-02").is_ok()); // Monday
        assert!(WeekKey::parse("2026-03-03").is_err()); // Tuesday
        assert!(WeekKey::parse("2026-13-01").is_err());
    }

    #[test]
    fn month_key_requires_valid_yyyy_mm() {
        assert!(MonthKey::parse("2026-03").is_ok());
        assert!(MonthKey::parse("2026-3").is_err());
        assert!(MonthKey::parse("2026-13").is_err());
    }

    #[test]
    fn year_key_requires_four_digit_positive_year() {
        assert!(YearKey::parse("2026").is_ok());
        assert!(YearKey::parse("026").is_err());
        assert!(YearKey::parse("abcd").is_err());
        assert!(YearKey::parse("0000").is_err());
    }
}
