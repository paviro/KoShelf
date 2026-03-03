use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

use super::ServerState;
use crate::contracts::common::{MonthKey, Scope, WeekKey, YearKey};
use crate::contracts::error::{ApiErrorCode, ApiErrorResponse};

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

async fn read_data_json(state: &ServerState, relative_path: PathBuf) -> Result<Value, Response> {
    let file_path = state.site_dir.join("data").join(relative_path);

    let content = tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                api_error(StatusCode::NOT_FOUND, ApiErrorCode::NotFound)
            } else {
                internal_error()
            }
        })?;

    serde_json::from_str::<Value>(&content).map_err(|_| internal_error())
}

fn project_scope(value: Value, scope: Scope) -> Value {
    let mut root = match value {
        Value::Object(map) => map,
        other => return other,
    };

    let Some(Value::Object(scopes)) = root.remove("scopes") else {
        return Value::Object(root);
    };

    let selected = scopes
        .get(scope.as_str())
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

    if let Value::Object(selected_scope) = selected {
        for (key, value) in selected_scope {
            root.insert(key, value);
        }
    }

    Value::Object(root)
}

async fn serve_data_json(state: &ServerState, relative_path: PathBuf) -> Response {
    match read_data_json(state, relative_path).await {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(response) => response,
    }
}

async fn serve_data_json_scoped(
    state: &ServerState,
    relative_path: PathBuf,
    scope: Scope,
) -> Response {
    match read_data_json(state, relative_path).await {
        Ok(value) => (StatusCode::OK, Json(project_scope(value, scope))).into_response(),
        Err(response) => response,
    }
}

pub async fn site(State(state): State<ServerState>) -> Response {
    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(site) = snapshot.site.as_ref()
    {
        return (StatusCode::OK, Json(site.clone())).into_response();
    }

    serve_data_json(&state, PathBuf::from("site.json")).await
}

pub async fn locales(State(state): State<ServerState>) -> Response {
    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(locales) = snapshot.locales.as_ref()
    {
        return (StatusCode::OK, Json(locales.clone())).into_response();
    }

    serve_data_json(&state, PathBuf::from("locales.json")).await
}

pub async fn books(State(state): State<ServerState>) -> Response {
    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(books) = snapshot.books.as_ref()
    {
        return (StatusCode::OK, Json(books.clone())).into_response();
    }

    serve_data_json(&state, PathBuf::from("books.json")).await
}

pub async fn book_detail(State(state): State<ServerState>, Path(id): Path<String>) -> Response {
    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(book) = snapshot.book_details.get(&id)
    {
        return (StatusCode::OK, Json(book.clone())).into_response();
    }

    serve_data_json(&state, PathBuf::from("books").join(format!("{}.json", id))).await
}

pub async fn comics(State(state): State<ServerState>) -> Response {
    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(comics) = snapshot.comics.as_ref()
    {
        return (StatusCode::OK, Json(comics.clone())).into_response();
    }

    serve_data_json(&state, PathBuf::from("comics.json")).await
}

pub async fn comic_detail(State(state): State<ServerState>, Path(id): Path<String>) -> Response {
    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(comic) = snapshot.comic_details.get(&id)
    {
        return (StatusCode::OK, Json(comic.clone())).into_response();
    }

    serve_data_json(&state, PathBuf::from("comics").join(format!("{}.json", id))).await
}

pub async fn statistics_index(
    State(state): State<ServerState>,
    Query(query): Query<ScopeQuery>,
) -> Response {
    let scope = match parse_scope(query) {
        Ok(scope) => scope,
        Err(response) => return response,
    };

    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(index) = snapshot.statistics_index.as_ref()
    {
        return (StatusCode::OK, Json(index.scoped(scope))).into_response();
    }

    serve_data_json_scoped(
        &state,
        PathBuf::from("statistics").join("index.json"),
        scope,
    )
    .await
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

    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(week) = snapshot.statistics_weeks.get(week_key.as_str())
    {
        return (StatusCode::OK, Json(week.scoped(scope))).into_response();
    }

    serve_data_json_scoped(
        &state,
        PathBuf::from("statistics")
            .join("weeks")
            .join(format!("{}.json", week_key.as_str())),
        scope,
    )
    .await
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

    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(year_stats) = snapshot.statistics_years.get(year.as_str())
    {
        return (StatusCode::OK, Json(year_stats.scoped(scope))).into_response();
    }

    serve_data_json_scoped(
        &state,
        PathBuf::from("statistics")
            .join("years")
            .join(format!("{}.json", year.as_str())),
        scope,
    )
    .await
}

pub async fn calendar_months(State(state): State<ServerState>) -> Response {
    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(months) = snapshot.calendar_months.as_ref()
    {
        return (StatusCode::OK, Json(months.clone())).into_response();
    }

    serve_data_json(&state, PathBuf::from("calendar").join("months.json")).await
}

pub async fn calendar_month(
    State(state): State<ServerState>,
    Path(month_key): Path<String>,
) -> Response {
    let month_key = match validate_month_key(&month_key) {
        Ok(month_key) => month_key,
        Err(response) => return response,
    };

    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(month) = snapshot.calendar_by_month.get(month_key.as_str())
    {
        return (StatusCode::OK, Json(month.clone())).into_response();
    }

    serve_data_json(
        &state,
        PathBuf::from("calendar")
            .join("months")
            .join(format!("{}.json", month_key.as_str())),
    )
    .await
}

pub async fn recap_index(
    State(state): State<ServerState>,
    Query(query): Query<ScopeQuery>,
) -> Response {
    let scope = match parse_scope(query) {
        Ok(scope) => scope,
        Err(response) => return response,
    };

    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(index) = snapshot.recap_index.as_ref()
    {
        return (StatusCode::OK, Json(index.scoped(scope))).into_response();
    }

    serve_data_json_scoped(&state, PathBuf::from("recap").join("index.json"), scope).await
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

    if let Some(snapshot) = state.snapshot_store.get()
        && let Some(year_recap) = snapshot.recap_years.get(year.as_str())
    {
        return (StatusCode::OK, Json(year_recap.scoped(scope))).into_response();
    }

    serve_data_json_scoped(
        &state,
        PathBuf::from("recap")
            .join("years")
            .join(format!("{}.json", year.as_str())),
        scope,
    )
    .await
}
