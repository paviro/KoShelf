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
use crate::contracts::common::{MonthKey, Scope, WeekKey, YearKey};
use crate::contracts::error::{ApiErrorCode, ApiErrorResponse};
use crate::runtime::{ContractSnapshot, SnapshotUpdate};

#[derive(Debug, Deserialize)]
pub struct ScopeQuery {
    scope: Option<String>,
}

fn api_error(status: StatusCode, code: ApiErrorCode) -> Response {
    (status, Json(ApiErrorResponse::from_code(code))).into_response()
}

fn internal_error() -> Response {
    api_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        ApiErrorCode::InternalServerError,
    )
}

fn not_found() -> Response {
    api_error(StatusCode::NOT_FOUND, ApiErrorCode::NotFound)
}

fn parse_scope(query: ScopeQuery) -> Result<Scope, Response> {
    Scope::parse(query.scope.as_deref())
        .map_err(|_| api_error(StatusCode::BAD_REQUEST, ApiErrorCode::InvalidScope))
}

fn validate_week_key(value: &str) -> Result<WeekKey, Response> {
    WeekKey::parse(value)
        .map_err(|_| api_error(StatusCode::BAD_REQUEST, ApiErrorCode::InvalidWeekKey))
}

fn validate_month_key(value: &str) -> Result<MonthKey, Response> {
    MonthKey::parse(value)
        .map_err(|_| api_error(StatusCode::BAD_REQUEST, ApiErrorCode::InvalidMonthKey))
}

fn validate_year_key(value: &str) -> Result<YearKey, Response> {
    YearKey::parse(value).map_err(|_| api_error(StatusCode::BAD_REQUEST, ApiErrorCode::InvalidYear))
}

fn runtime_snapshot(state: &ServerState) -> Result<Arc<ContractSnapshot>, Response> {
    state.snapshot_store.get().ok_or_else(internal_error)
}

fn snapshot_update_event(update: &SnapshotUpdate) -> Event {
    let payload = match serde_json::to_string(update) {
        Ok(payload) => payload,
        Err(_) => "{}".to_string(),
    };

    Event::default()
        .event("snapshot_updated")
        .data(payload)
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
        Err(response) => return response,
    };

    match snapshot.site.as_ref() {
        Some(site) => (StatusCode::OK, Json(site.clone())).into_response(),
        None => not_found(),
    }
}

pub async fn locales(State(state): State<ServerState>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    match snapshot.locales.as_ref() {
        Some(locales) => (StatusCode::OK, Json(locales.clone())).into_response(),
        None => not_found(),
    }
}

pub async fn books(State(state): State<ServerState>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    match snapshot.books.as_ref() {
        Some(books) => (StatusCode::OK, Json(books.clone())).into_response(),
        None => not_found(),
    }
}

pub async fn book_detail(State(state): State<ServerState>, Path(id): Path<String>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    match snapshot.book_details.get(&id) {
        Some(book) => (StatusCode::OK, Json(book.clone())).into_response(),
        None => not_found(),
    }
}

pub async fn comics(State(state): State<ServerState>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    match snapshot.comics.as_ref() {
        Some(comics) => (StatusCode::OK, Json(comics.clone())).into_response(),
        None => not_found(),
    }
}

pub async fn comic_detail(State(state): State<ServerState>, Path(id): Path<String>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    match snapshot.comic_details.get(&id) {
        Some(comic) => (StatusCode::OK, Json(comic.clone())).into_response(),
        None => not_found(),
    }
}

pub async fn statistics_index(
    State(state): State<ServerState>,
    Query(query): Query<ScopeQuery>,
) -> Response {
    let scope = match parse_scope(query) {
        Ok(scope) => scope,
        Err(response) => return response,
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    match snapshot.statistics_index.as_ref() {
        Some(index) => (StatusCode::OK, Json(index.scoped(scope))).into_response(),
        None => not_found(),
    }
}

pub async fn statistics_week(
    State(state): State<ServerState>,
    Path(week_key): Path<String>,
    Query(query): Query<ScopeQuery>,
) -> Response {
    let scope = match parse_scope(query) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    let week_key = match validate_week_key(&week_key) {
        Ok(week_key) => week_key,
        Err(response) => return response,
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    match snapshot.statistics_weeks.get(week_key.as_str()) {
        Some(week) => (StatusCode::OK, Json(week.scoped(scope))).into_response(),
        None => not_found(),
    }
}

pub async fn statistics_year(
    State(state): State<ServerState>,
    Path(year): Path<String>,
    Query(query): Query<ScopeQuery>,
) -> Response {
    let scope = match parse_scope(query) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    let year = match validate_year_key(&year) {
        Ok(year) => year,
        Err(response) => return response,
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    match snapshot.statistics_years.get(year.as_str()) {
        Some(year_stats) => (StatusCode::OK, Json(year_stats.scoped(scope))).into_response(),
        None => not_found(),
    }
}

pub async fn calendar_months(State(state): State<ServerState>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    match snapshot.calendar_months.as_ref() {
        Some(months) => (StatusCode::OK, Json(months.clone())).into_response(),
        None => not_found(),
    }
}

pub async fn calendar_month(
    State(state): State<ServerState>,
    Path(month_key): Path<String>,
) -> Response {
    let month_key = match validate_month_key(&month_key) {
        Ok(month_key) => month_key,
        Err(response) => return response,
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    match snapshot.calendar_by_month.get(month_key.as_str()) {
        Some(month) => (StatusCode::OK, Json(month.clone())).into_response(),
        None => not_found(),
    }
}

pub async fn recap_index(
    State(state): State<ServerState>,
    Query(query): Query<ScopeQuery>,
) -> Response {
    let scope = match parse_scope(query) {
        Ok(scope) => scope,
        Err(response) => return response,
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    match snapshot.recap_index.as_ref() {
        Some(index) => (StatusCode::OK, Json(index.scoped(scope))).into_response(),
        None => not_found(),
    }
}

pub async fn recap_year(
    State(state): State<ServerState>,
    Path(year): Path<String>,
    Query(query): Query<ScopeQuery>,
) -> Response {
    let scope = match parse_scope(query) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    let year = match validate_year_key(&year) {
        Ok(year) => year,
        Err(response) => return response,
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    match snapshot.recap_years.get(year.as_str()) {
        Some(year_recap) => (StatusCode::OK, Json(year_recap.scoped(scope))).into_response(),
        None => not_found(),
    }
}
