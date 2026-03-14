use std::ops::Deref;
use std::sync::Arc;

use axum::{
    Json,
    extract::FromRequestParts,
    http::StatusCode,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::contracts::common::{ContentTypeFilter, MonthKey, YearKey};
use crate::contracts::error::{ApiErrorCode, ApiErrorResponse};
use crate::server::ServerState;
use crate::shelf::library::queries::{IncludeSet, ItemSort, SortOrder};
use crate::shelf::statistics::queries::{
    self as rq, CompletionsGroupBy, CompletionsIncludeSet, CompletionsSelector, DateRange,
    MetricsGroupBy, PeriodGroupBy, PeriodSource, ReadingScope,
};
use crate::store::memory::ReadingData;

// ── Query params ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ScopeQuery {
    pub scope: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DetailQuery {
    pub include: Option<String>,
}

// ── Reading endpoint query params ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ReadingSummaryParams {
    pub scope: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub tz: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReadingMetricsParams {
    pub scope: Option<String>,
    pub metric: Option<String>,
    pub group_by: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub tz: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReadingAvailablePeriodsParams {
    pub scope: Option<String>,
    pub source: Option<String>,
    pub group_by: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub tz: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReadingCalendarParams {
    pub month: Option<String>,
    pub scope: Option<String>,
    pub tz: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReadingCompletionsParams {
    pub scope: Option<String>,
    pub year: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub group_by: Option<String>,
    pub include: Option<String>,
    pub tz: Option<String>,
}

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

// ── ReadingData extractor ─────────────────────────────────────────────────

pub(crate) struct ReadingDataGuard(pub(crate) Arc<ReadingData>);

impl Deref for ReadingDataGuard {
    type Target = ReadingData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<ServerState> for ReadingDataGuard {
    type Rejection = ApiResponseError;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        state
            .reading_data_store
            .get()
            .map(ReadingDataGuard)
            .ok_or_else(ApiResponseError::internal_server_error)
    }
}

// ── Parsing helpers ────────────────────────────────────────────────────────

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

pub(crate) fn parse_include(value: Option<&str>) -> ApiResult<IncludeSet> {
    Ok(IncludeSet::parse(value)?)
}

// ── Reading endpoint parsing ──────────────────────────────────────────────

/// Parse an optional date-range pair.  Both must be present or both absent.
pub(crate) fn parse_optional_date_range(
    from: Option<&str>,
    to: Option<&str>,
) -> ApiResult<Option<DateRange>> {
    match (from, to) {
        (None, None) => Ok(None),
        (Some(f), Some(t)) => Ok(Some(DateRange::from_str(f, t)?)),
        (Some(_), None) => Err(ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            "'from' and 'to' must be provided together",
        )),
        (None, Some(_)) => Err(ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            "'from' and 'to' must be provided together",
        )),
    }
}

fn parse_reading_tz(value: Option<&str>) -> ApiResult<Option<chrono_tz::Tz>> {
    Ok(rq::parse_timezone(value)?)
}

fn parse_reading_scope(value: Option<&str>) -> ApiResult<ReadingScope> {
    parse_scope(value)
}

fn require_param<'a>(value: Option<&'a str>, name: &str) -> ApiResult<&'a str> {
    value.filter(|v| !v.is_empty()).ok_or_else(|| {
        ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            format!("'{name}' is required"),
        )
    })
}

pub(crate) fn parse_reading_summary_query(
    params: &ReadingSummaryParams,
) -> ApiResult<rq::ReadingSummaryQuery> {
    let scope = parse_reading_scope(params.scope.as_deref())?;
    let range = parse_optional_date_range(params.from.as_deref(), params.to.as_deref())?;
    let tz = parse_reading_tz(params.tz.as_deref())?;
    Ok(rq::ReadingSummaryQuery { scope, range, tz })
}

pub(crate) fn parse_reading_metrics_query(
    params: &ReadingMetricsParams,
) -> ApiResult<rq::ReadingMetricsQuery> {
    let scope = parse_reading_scope(params.scope.as_deref())?;
    let metric_str = require_param(params.metric.as_deref(), "metric")?;
    let metrics = rq::parse_metrics(metric_str)?;
    let group_by_str = require_param(params.group_by.as_deref(), "group_by")?;
    let group_by = MetricsGroupBy::parse(group_by_str)?;
    let range = parse_optional_date_range(params.from.as_deref(), params.to.as_deref())?;
    let tz = parse_reading_tz(params.tz.as_deref())?;
    Ok(rq::ReadingMetricsQuery {
        scope,
        metrics,
        group_by,
        range,
        tz,
    })
}

pub(crate) fn parse_reading_available_periods_query(
    params: &ReadingAvailablePeriodsParams,
) -> ApiResult<rq::ReadingAvailablePeriodsQuery> {
    let scope = parse_reading_scope(params.scope.as_deref())?;
    let source_str = require_param(params.source.as_deref(), "source")?;
    let source = PeriodSource::parse(source_str)?;
    let group_by_str = require_param(params.group_by.as_deref(), "group_by")?;
    let group_by = PeriodGroupBy::parse(group_by_str)?;
    rq::validate_period_source_group(source, group_by)?;
    let range = parse_optional_date_range(params.from.as_deref(), params.to.as_deref())?;
    let tz = parse_reading_tz(params.tz.as_deref())?;
    Ok(rq::ReadingAvailablePeriodsQuery {
        scope,
        source,
        group_by,
        range,
        tz,
    })
}

pub(crate) fn parse_reading_calendar_query(
    params: &ReadingCalendarParams,
) -> ApiResult<rq::ReadingCalendarQuery> {
    let month_str = require_param(params.month.as_deref(), "month")?;
    let month_key = MonthKey::parse(month_str).map_err(|_| {
        ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            "'month' must be in YYYY-MM format",
        )
    })?;
    let scope = parse_reading_scope(params.scope.as_deref())?;
    let tz = parse_reading_tz(params.tz.as_deref())?;
    Ok(rq::ReadingCalendarQuery {
        month: month_key.as_str().to_string(),
        scope,
        tz,
    })
}

pub(crate) fn parse_reading_completions_query(
    params: &ReadingCompletionsParams,
) -> ApiResult<rq::ReadingCompletionsQuery> {
    let scope = parse_reading_scope(params.scope.as_deref())?;
    let tz = parse_reading_tz(params.tz.as_deref())?;

    // year and from/to are mutually exclusive
    let selector = match (
        params.year.as_deref(),
        params.from.as_deref(),
        params.to.as_deref(),
    ) {
        (Some(y), None, None) => {
            let year_key = YearKey::parse(y).map_err(|_| {
                ApiResponseError::bad_request_with_message(
                    ApiErrorCode::InvalidQuery,
                    "'year' must be a valid four-digit year",
                )
            })?;
            let year_value: i32 = year_key.as_str().parse().map_err(|_| {
                ApiResponseError::bad_request_with_message(
                    ApiErrorCode::InvalidQuery,
                    "'year' must be a valid four-digit year",
                )
            })?;
            CompletionsSelector::Year(year_value)
        }
        (None, Some(f), Some(t)) => {
            let range = DateRange::from_str(f, t)?;
            CompletionsSelector::Range(range)
        }
        (None, None, None) => CompletionsSelector::Default,
        _ => {
            return Err(ApiResponseError::bad_request_with_message(
                ApiErrorCode::InvalidQuery,
                "'year' and 'from/to' are mutually exclusive; provide one or the other",
            ));
        }
    };

    let group_by = CompletionsGroupBy::parse(params.group_by.as_deref())?;
    let includes = CompletionsIncludeSet::parse(params.include.as_deref())?;

    Ok(rq::ReadingCompletionsQuery {
        scope,
        selector,
        group_by,
        includes,
        tz,
    })
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

    // ── Date range parsing ────────────────────────────────────────────

    #[test]
    fn parse_optional_date_range_both_none_returns_none() {
        assert!(parse_optional_date_range(None, None).unwrap().is_none());
    }

    #[test]
    fn parse_optional_date_range_both_present_returns_range() {
        let range = parse_optional_date_range(Some("2026-01-01"), Some("2026-01-31"))
            .unwrap()
            .unwrap();
        assert_eq!(
            range.from,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        );
    }

    #[test]
    fn parse_optional_date_range_only_from_is_error() {
        assert!(parse_optional_date_range(Some("2026-01-01"), None).is_err());
    }

    #[test]
    fn parse_optional_date_range_only_to_is_error() {
        assert!(parse_optional_date_range(None, Some("2026-01-31")).is_err());
    }

    // ── Summary query parsing ─────────────────────────────────────────

    #[test]
    fn parse_reading_summary_defaults() {
        let params = ReadingSummaryParams {
            scope: None,
            from: None,
            to: None,
            tz: None,
        };
        let query = parse_reading_summary_query(&params).unwrap();
        assert_eq!(query.scope, ContentTypeFilter::All);
        assert!(query.range.is_none());
        assert!(query.tz.is_none());
    }

    #[test]
    fn parse_reading_summary_with_range_and_tz() {
        let params = ReadingSummaryParams {
            scope: Some("books".to_string()),
            from: Some("2026-01-01".to_string()),
            to: Some("2026-03-31".to_string()),
            tz: Some("UTC".to_string()),
        };
        let query = parse_reading_summary_query(&params).unwrap();
        assert_eq!(query.scope, ContentTypeFilter::Books);
        assert!(query.range.is_some());
        assert!(query.tz.is_some());
    }

    // ── Metrics query parsing ─────────────────────────────────────────

    #[test]
    fn parse_reading_metrics_requires_metric_and_group_by() {
        let params = ReadingMetricsParams {
            scope: None,
            metric: None,
            group_by: None,
            from: None,
            to: None,
            tz: None,
        };
        assert!(parse_reading_metrics_query(&params).is_err());
    }

    #[test]
    fn parse_reading_metrics_valid_single() {
        let params = ReadingMetricsParams {
            scope: Some("books".to_string()),
            metric: Some("reading_time_sec".to_string()),
            group_by: Some("day".to_string()),
            from: Some("2026-01-01".to_string()),
            to: Some("2026-01-31".to_string()),
            tz: None,
        };
        let query = parse_reading_metrics_query(&params).unwrap();
        assert_eq!(query.metrics.len(), 1);
        assert_eq!(query.metrics[0], rq::ReadingMetric::ReadingTimeSec);
        assert_eq!(query.group_by, MetricsGroupBy::Day);
    }

    #[test]
    fn parse_reading_metrics_valid_multiple() {
        let params = ReadingMetricsParams {
            scope: None,
            metric: Some("reading_time_sec,pages_read".to_string()),
            group_by: Some("day".to_string()),
            from: None,
            to: None,
            tz: None,
        };
        let query = parse_reading_metrics_query(&params).unwrap();
        assert_eq!(query.metrics.len(), 2);
        assert_eq!(query.metrics[0], rq::ReadingMetric::ReadingTimeSec);
        assert_eq!(query.metrics[1], rq::ReadingMetric::PagesRead);
    }

    // ── Available-periods query parsing ───────────────────────────────

    #[test]
    fn parse_reading_available_periods_requires_source_and_group_by() {
        let params = ReadingAvailablePeriodsParams {
            scope: None,
            source: None,
            group_by: None,
            from: None,
            to: None,
            tz: None,
        };
        assert!(parse_reading_available_periods_query(&params).is_err());
    }

    #[test]
    fn parse_reading_available_periods_rejects_completions_week() {
        let params = ReadingAvailablePeriodsParams {
            scope: None,
            source: Some("completions".to_string()),
            group_by: Some("week".to_string()),
            from: None,
            to: None,
            tz: None,
        };
        assert!(parse_reading_available_periods_query(&params).is_err());
    }

    #[test]
    fn parse_reading_available_periods_valid() {
        let params = ReadingAvailablePeriodsParams {
            scope: None,
            source: Some("reading_data".to_string()),
            group_by: Some("week".to_string()),
            from: None,
            to: None,
            tz: None,
        };
        let query = parse_reading_available_periods_query(&params).unwrap();
        assert_eq!(query.source, PeriodSource::ReadingData);
        assert_eq!(query.group_by, PeriodGroupBy::Week);
    }

    // ── Calendar query parsing ────────────────────────────────────────

    #[test]
    fn parse_reading_calendar_requires_month() {
        let params = ReadingCalendarParams {
            month: None,
            scope: None,
            tz: None,
        };
        assert!(parse_reading_calendar_query(&params).is_err());
    }

    #[test]
    fn parse_reading_calendar_valid() {
        let params = ReadingCalendarParams {
            month: Some("2026-03".to_string()),
            scope: Some("comics".to_string()),
            tz: None,
        };
        let query = parse_reading_calendar_query(&params).unwrap();
        assert_eq!(query.month, "2026-03");
        assert_eq!(query.scope, ContentTypeFilter::Comics);
    }

    // ── Completions query parsing ─────────────────────────────────────

    #[test]
    fn parse_reading_completions_default_selector() {
        let params = ReadingCompletionsParams {
            scope: None,
            year: None,
            from: None,
            to: None,
            group_by: None,
            include: None,
            tz: None,
        };
        let query = parse_reading_completions_query(&params).unwrap();
        assert_eq!(query.selector, CompletionsSelector::Default);
        assert_eq!(query.group_by, CompletionsGroupBy::Month);
    }

    #[test]
    fn parse_reading_completions_year_selector() {
        let params = ReadingCompletionsParams {
            scope: None,
            year: Some("2025".to_string()),
            from: None,
            to: None,
            group_by: None,
            include: Some("summary,share_assets".to_string()),
            tz: None,
        };
        let query = parse_reading_completions_query(&params).unwrap();
        assert_eq!(query.selector, CompletionsSelector::Year(2025));
        assert!(query.includes.has_summary());
        assert!(query.includes.has_share_assets());
    }

    #[test]
    fn parse_reading_completions_range_selector() {
        let params = ReadingCompletionsParams {
            scope: None,
            year: None,
            from: Some("2025-01-01".to_string()),
            to: Some("2025-12-31".to_string()),
            group_by: Some("none".to_string()),
            include: None,
            tz: None,
        };
        let query = parse_reading_completions_query(&params).unwrap();
        assert!(matches!(query.selector, CompletionsSelector::Range(_)));
        assert_eq!(query.group_by, CompletionsGroupBy::None);
    }

    #[test]
    fn parse_reading_completions_rejects_year_with_from_to() {
        let params = ReadingCompletionsParams {
            scope: None,
            year: Some("2025".to_string()),
            from: Some("2025-01-01".to_string()),
            to: Some("2025-12-31".to_string()),
            group_by: None,
            include: None,
            tz: None,
        };
        assert!(parse_reading_completions_query(&params).is_err());
    }
}
