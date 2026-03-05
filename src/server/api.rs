use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
};
use futures::stream;
use serde::Deserialize;
use std::{convert::Infallible, sync::Arc, time::Duration};

use super::ServerState;
use crate::contracts::common::{ContentTypeFilter, MonthKey, WeekKey, YearKey};
use crate::contracts::error::{ApiErrorCode, ApiErrorResponse};
use crate::contracts::library::{LibraryContentType, LibraryListResponse};
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

fn not_found() -> Response {
    api_error(StatusCode::NOT_FOUND, ApiErrorCode::NotFound)
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

    match snapshot.site.as_ref() {
        Some(site) => (StatusCode::OK, Json(site.clone())).into_response(),
        None => not_found(),
    }
}

pub async fn locales(State(state): State<ServerState>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    match snapshot.locales.as_ref() {
        Some(locales) => (StatusCode::OK, Json(locales.clone())).into_response(),
        None => not_found(),
    }
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

    match snapshot.items.as_ref() {
        Some(items) => {
            (StatusCode::OK, Json(filter_library_items(items, content_type))).into_response()
        }
        None => not_found(),
    }
}

pub async fn item_detail(State(state): State<ServerState>, Path(id): Path<String>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    match snapshot.item_details.get(&id) {
        Some(item) => (StatusCode::OK, Json(item.clone())).into_response(),
        None => not_found(),
    }
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

    match snapshot.activity_weeks.get(content_type.as_str()) {
        Some(weeks) => (StatusCode::OK, Json(weeks.clone())).into_response(),
        None => not_found(),
    }
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

    match snapshot
        .activity_weeks_by_key
        .get(content_type.as_str())
        .and_then(|weeks| weeks.get(week_key.as_str()))
    {
        Some(week) => (StatusCode::OK, Json(week.clone())).into_response(),
        None => not_found(),
    }
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

    match snapshot
        .activity_year_daily
        .get(content_type.as_str())
        .and_then(|years| years.get(year.as_str()))
    {
        Some(payload) => (StatusCode::OK, Json(payload.clone())).into_response(),
        None => not_found(),
    }
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

    match snapshot
        .activity_year_summary
        .get(content_type.as_str())
        .and_then(|years| years.get(year.as_str()))
    {
        Some(payload) => (StatusCode::OK, Json(payload.clone())).into_response(),
        None => not_found(),
    }
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

    match snapshot.activity_months.get(content_type.as_str()) {
        Some(months) => (StatusCode::OK, Json(months.clone())).into_response(),
        None => not_found(),
    }
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

    match snapshot
        .activity_months_by_key
        .get(content_type.as_str())
        .and_then(|months| months.get(month_key.as_str()))
    {
        Some(month) => (StatusCode::OK, Json(month.clone())).into_response(),
        None => not_found(),
    }
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

    match snapshot.completion_years.get(content_type.as_str()) {
        Some(years) => (StatusCode::OK, Json(years.clone())).into_response(),
        None => not_found(),
    }
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

    match snapshot
        .completion_years_by_key
        .get(content_type.as_str())
        .and_then(|years| years.get(year.as_str()))
    {
        Some(year_recap) => (StatusCode::OK, Json(year_recap.clone())).into_response(),
        None => not_found(),
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
}
