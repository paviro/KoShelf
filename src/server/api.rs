use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
};
use chrono::{Duration as ChronoDuration, NaiveDate};
use futures::stream;
use serde::Deserialize;
use std::{collections::BTreeMap, convert::Infallible, sync::Arc, time::Duration};

use super::ServerState;
use crate::contracts::calendar::{
    ActivityMonthResponse, ActivityMonthsResponse, CalendarMonthlyStats,
};
use crate::contracts::common::{ApiMeta, ContentTypeFilter, MonthKey, WeekKey, YearKey};
use crate::contracts::error::{ApiErrorCode, ApiErrorResponse};
use crate::contracts::library::{
    LibraryContentType, LibraryDetailItem, LibraryDetailResponse, LibraryDetailStatistics,
    LibraryListResponse, LibraryStatus,
};
use crate::contracts::recap::{
    CompletionYearResponse, CompletionYearsResponse, RecapSummaryResponse,
};
use crate::contracts::site::{SiteCapabilities, SiteResponse};
use crate::contracts::statistics::{
    ActivityHeatmapConfig, ActivityOverview, ActivityStreaks, ActivityWeekResponse,
    ActivityWeeksResponse, ActivityYearDailyResponse, ActivityYearSummaryResponse, YearlySummary,
};
use crate::models::{StreakInfo, WeeklyStats};
use crate::runtime::{ContractSnapshot, SnapshotUpdate};

#[derive(Debug, Deserialize)]
pub struct ContentTypeQuery {
    content_type: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct ApiResponseError {
    status: StatusCode,
    code: ApiErrorCode,
}

impl ApiResponseError {
    const fn bad_request(code: ApiErrorCode) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
        }
    }

    const fn internal_server_error() -> Self {
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

type ApiResult<T> = Result<T, ApiResponseError>;

fn api_error(status: StatusCode, code: ApiErrorCode) -> Response {
    (status, Json(ApiErrorResponse::from_code(code))).into_response()
}

fn parse_content_type(query: ContentTypeQuery) -> ApiResult<ContentTypeFilter> {
    ContentTypeFilter::parse(query.content_type.as_deref()).map_err(ApiResponseError::bad_request)
}

fn validate_week_key(value: &str) -> ApiResult<WeekKey> {
    WeekKey::parse(value).map_err(ApiResponseError::bad_request)
}

fn validate_month_key(value: &str) -> ApiResult<MonthKey> {
    MonthKey::parse(value).map_err(ApiResponseError::bad_request)
}

fn validate_year_key(value: &str) -> ApiResult<YearKey> {
    YearKey::parse(value).map_err(ApiResponseError::bad_request)
}

fn runtime_snapshot(state: &ServerState) -> ApiResult<Arc<ContractSnapshot>> {
    state
        .snapshot_store
        .get()
        .ok_or_else(ApiResponseError::internal_server_error)
}

fn fallback_meta(snapshot: &ContractSnapshot) -> ApiMeta {
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

fn parse_validated_year(year: &YearKey) -> ApiResult<i32> {
    year.as_str()
        .parse::<i32>()
        .map_err(|_| ApiResponseError::internal_server_error())
}

fn resolve_week_bounds(week_key: &str) -> (String, String) {
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

fn empty_site_response(meta: ApiMeta) -> SiteResponse {
    SiteResponse {
        meta,
        title: String::new(),
        language: "en_US".to_string(),
        capabilities: SiteCapabilities {
            has_books: false,
            has_comics: false,
            has_activity: false,
            has_completions: false,
        },
    }
}

fn empty_library_list_response(meta: ApiMeta) -> LibraryListResponse {
    LibraryListResponse { meta, items: vec![] }
}

fn empty_library_detail_response(meta: ApiMeta, id: String) -> LibraryDetailResponse {
    LibraryDetailResponse {
        meta,
        item: LibraryDetailItem {
            id,
            title: String::new(),
            authors: vec![],
            series: None,
            status: LibraryStatus::Unknown,
            progress_percentage: None,
            rating: None,
            cover_url: String::new(),
            content_type: LibraryContentType::Book,
            language: None,
            publisher: None,
            description: None,
            review_note: None,
            pages: None,
            search_base_path: String::new(),
            subjects: vec![],
            identifiers: vec![],
        },
        highlights: vec![],
        bookmarks: vec![],
        statistics: LibraryDetailStatistics {
            item_stats: None,
            session_stats: None,
            completions: None,
        },
    }
}

fn empty_activity_overview() -> ActivityOverview {
    ActivityOverview {
        total_read_time: 0,
        total_page_reads: 0,
        longest_read_time_in_day: 0,
        most_pages_in_day: 0,
        average_session_duration: None,
        longest_session_duration: None,
        total_completions: 0,
        items_completed: 0,
        most_completions: 0,
    }
}

fn empty_activity_streaks() -> ActivityStreaks {
    ActivityStreaks {
        longest: StreakInfo::new(0, None, None),
        current: StreakInfo::new(0, None, None),
    }
}

fn empty_activity_weeks_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
) -> ActivityWeeksResponse {
    ActivityWeeksResponse {
        meta,
        content_type,
        available_years: vec![],
        available_weeks: vec![],
        overview: empty_activity_overview(),
        streaks: empty_activity_streaks(),
        heatmap_config: ActivityHeatmapConfig {
            max_scale_seconds: None,
        },
    }
}

fn empty_activity_week_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    week_key: &str,
) -> ActivityWeekResponse {
    let (start_date, end_date) = resolve_week_bounds(week_key);

    ActivityWeekResponse {
        meta,
        content_type,
        week_key: week_key.to_string(),
        stats: WeeklyStats {
            start_date,
            end_date,
            read_time: 0,
            pages_read: 0,
            avg_pages_per_day: 0.0,
            avg_read_time_per_day: 0.0,
            longest_session_duration: None,
            average_session_duration: None,
        },
        daily_activity: vec![],
    }
}

fn empty_activity_months_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
) -> ActivityMonthsResponse {
    ActivityMonthsResponse {
        meta,
        content_type,
        months: vec![],
    }
}

fn empty_calendar_monthly_stats() -> CalendarMonthlyStats {
    CalendarMonthlyStats {
        items_read: 0,
        pages_read: 0,
        time_read: 0,
        days_read_pct: 0,
    }
}

fn empty_activity_month_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
) -> ActivityMonthResponse {
    ActivityMonthResponse {
        meta,
        content_type,
        events: vec![],
        items: BTreeMap::new(),
        stats: empty_calendar_monthly_stats(),
    }
}

fn empty_activity_year_daily_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    year: i32,
) -> ActivityYearDailyResponse {
    ActivityYearDailyResponse {
        meta,
        content_type,
        year,
        daily_activity: vec![],
        config: Some(ActivityHeatmapConfig {
            max_scale_seconds: None,
        }),
    }
}

fn empty_activity_year_summary_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    year: i32,
) -> ActivityYearSummaryResponse {
    ActivityYearSummaryResponse {
        meta,
        content_type,
        year,
        summary: YearlySummary { completed_count: 0 },
        monthly_aggregates: vec![],
        config: Some(ActivityHeatmapConfig {
            max_scale_seconds: None,
        }),
    }
}

fn empty_recap_summary_response() -> RecapSummaryResponse {
    RecapSummaryResponse {
        total_items: 0,
        total_time_seconds: 0,
        total_time_days: 0,
        total_time_hours: 0,
        longest_session_hours: 0,
        longest_session_minutes: 0,
        average_session_hours: 0,
        average_session_minutes: 0,
        active_days: 0,
        active_days_percentage: 0.0,
        longest_streak: 0,
        best_month_name: None,
    }
}

fn empty_completion_years_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
) -> CompletionYearsResponse {
    CompletionYearsResponse {
        meta,
        content_type,
        available_years: vec![],
        latest_year: None,
    }
}

fn empty_completion_year_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    year: i32,
) -> CompletionYearResponse {
    CompletionYearResponse {
        meta,
        content_type,
        year,
        summary: empty_recap_summary_response(),
        months: vec![],
        items: vec![],
        share_assets: None,
    }
}

fn snapshot_update_event(update: &SnapshotUpdate) -> Event {
    let payload = match serde_json::to_string(update) {
        Ok(payload) => payload,
        Err(_) => "{}".to_string(),
    };

    Event::default().event("snapshot_updated").data(payload)
}

pub async fn events_stream(
    State(state): State<ServerState>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.update_notifier.subscribe();
    let events = stream::unfold(
        (receiver, true),
        |(mut receiver, include_current)| async move {
            if include_current {
                let current = receiver.borrow().clone();
                return Some((Ok(snapshot_update_event(&current)), (receiver, false)));
            }

            match receiver.changed().await {
                Ok(()) => {
                    let update = receiver.borrow().clone();
                    Some((Ok(snapshot_update_event(&update)), (receiver, false)))
                }
                Err(_) => None,
            }
        },
    );

    Sse::new(events).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    )
}

pub async fn site(State(state): State<ServerState>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let payload = snapshot
        .site
        .as_ref()
        .cloned()
        .unwrap_or_else(|| empty_site_response(fallback_meta(&snapshot)));

    (StatusCode::OK, Json(payload)).into_response()
}

fn item_matches_content_type(
    content_type: ContentTypeFilter,
    item_content_type: LibraryContentType,
) -> bool {
    match content_type {
        ContentTypeFilter::All => true,
        ContentTypeFilter::Books => item_content_type == LibraryContentType::Book,
        ContentTypeFilter::Comics => item_content_type == LibraryContentType::Comic,
    }
}

fn filter_library_items(
    response: &LibraryListResponse,
    content_type: ContentTypeFilter,
) -> LibraryListResponse {
    if content_type == ContentTypeFilter::All {
        return response.clone();
    }

    LibraryListResponse {
        meta: response.meta.clone(),
        items: response
            .items
            .iter()
            .filter(|item| item_matches_content_type(content_type, item.content_type))
            .cloned()
            .collect(),
    }
}

pub async fn items(
    State(state): State<ServerState>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let base_payload = snapshot
        .items
        .as_ref()
        .cloned()
        .unwrap_or_else(|| empty_library_list_response(fallback_meta(&snapshot)));
    let payload = filter_library_items(&base_payload, content_type);

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn item_detail(State(state): State<ServerState>, Path(id): Path<String>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let payload = snapshot
        .item_details
        .get(&id)
        .cloned()
        .unwrap_or_else(|| empty_library_detail_response(fallback_meta(&snapshot), id.clone()));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn activity_weeks(
    State(state): State<ServerState>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let payload = snapshot
        .activity_weeks
        .get(content_type.as_str())
        .cloned()
        .unwrap_or_else(|| empty_activity_weeks_response(fallback_meta(&snapshot), content_type));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn activity_week(
    State(state): State<ServerState>,
    Path(week_key): Path<String>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };
    let week_key = match validate_week_key(&week_key) {
        Ok(week_key) => week_key,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let meta = snapshot
        .activity_weeks
        .get(content_type.as_str())
        .map(|weeks| weeks.meta.clone())
        .unwrap_or_else(|| fallback_meta(&snapshot));
    let payload = snapshot
        .activity_weeks_by_key
        .get(content_type.as_str())
        .and_then(|weeks| weeks.get(week_key.as_str()))
        .cloned()
        .unwrap_or_else(|| empty_activity_week_response(meta, content_type, week_key.as_str()));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn activity_year_daily(
    State(state): State<ServerState>,
    Path(year): Path<String>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };
    let year = match validate_year_key(&year) {
        Ok(year) => year,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let year_value = match parse_validated_year(&year) {
        Ok(value) => value,
        Err(error) => return error.into_response(),
    };
    let meta = snapshot
        .activity_weeks
        .get(content_type.as_str())
        .map(|weeks| weeks.meta.clone())
        .unwrap_or_else(|| fallback_meta(&snapshot));
    let payload = snapshot
        .activity_year_daily
        .get(content_type.as_str())
        .and_then(|years| years.get(year.as_str()))
        .cloned()
        .unwrap_or_else(|| empty_activity_year_daily_response(meta, content_type, year_value));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn activity_year_summary(
    State(state): State<ServerState>,
    Path(year): Path<String>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };
    let year = match validate_year_key(&year) {
        Ok(year) => year,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let year_value = match parse_validated_year(&year) {
        Ok(value) => value,
        Err(error) => return error.into_response(),
    };
    let meta = snapshot
        .activity_weeks
        .get(content_type.as_str())
        .map(|weeks| weeks.meta.clone())
        .unwrap_or_else(|| fallback_meta(&snapshot));
    let payload = snapshot
        .activity_year_summary
        .get(content_type.as_str())
        .and_then(|years| years.get(year.as_str()))
        .cloned()
        .unwrap_or_else(|| empty_activity_year_summary_response(meta, content_type, year_value));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn activity_months(
    State(state): State<ServerState>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let payload = snapshot
        .activity_months
        .get(content_type.as_str())
        .cloned()
        .unwrap_or_else(|| empty_activity_months_response(fallback_meta(&snapshot), content_type));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn activity_month(
    State(state): State<ServerState>,
    Path(month_key): Path<String>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };
    let month_key = match validate_month_key(&month_key) {
        Ok(month_key) => month_key,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let meta = snapshot
        .activity_months
        .get(content_type.as_str())
        .map(|months| months.meta.clone())
        .unwrap_or_else(|| fallback_meta(&snapshot));
    let payload = snapshot
        .activity_months_by_key
        .get(content_type.as_str())
        .and_then(|months| months.get(month_key.as_str()))
        .cloned()
        .unwrap_or_else(|| empty_activity_month_response(meta, content_type));

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn completion_years(
    State(state): State<ServerState>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let payload = snapshot
        .completion_years
        .get(content_type.as_str())
        .cloned()
        .unwrap_or_else(|| {
            empty_completion_years_response(fallback_meta(&snapshot), content_type)
        });

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn completion_year(
    State(state): State<ServerState>,
    Path(year): Path<String>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };
    let year = match validate_year_key(&year) {
        Ok(year) => year,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let year_value = match parse_validated_year(&year) {
        Ok(value) => value,
        Err(error) => return error.into_response(),
    };
    let meta = snapshot
        .completion_years
        .get(content_type.as_str())
        .map(|years| years.meta.clone())
        .unwrap_or_else(|| fallback_meta(&snapshot));
    let payload = snapshot
        .completion_years_by_key
        .get(content_type.as_str())
        .and_then(|years| years.get(year.as_str()))
        .cloned()
        .unwrap_or_else(|| empty_completion_year_response(meta, content_type, year_value));

    (StatusCode::OK, Json(payload)).into_response()
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
