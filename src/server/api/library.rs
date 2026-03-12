use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::domain::library::{LibraryDetailQuery, LibraryListQuery, LibraryService};
use crate::server::ServerState;

use super::shared::{
    ApiResponseError, ScopeQuery, parse_item_sort, parse_scope, parse_sort_order, request_meta,
};

pub async fn items(State(state): State<ServerState>, Query(query): Query<ScopeQuery>) -> Response {
    let scope = match parse_scope(query.scope.as_deref()) {
        Ok(s) => s,
        Err(e) => return e.into_response(),
    };
    let sort = match parse_item_sort(query.sort.as_deref()) {
        Ok(s) => s,
        Err(e) => return e.into_response(),
    };
    let order = match parse_sort_order(query.order.as_deref()) {
        Ok(o) => o,
        Err(e) => return e.into_response(),
    };

    let list_query = LibraryListQuery { scope, sort, order };

    match LibraryService::list(&state.library_repo, list_query, request_meta()).await {
        Ok(payload) => (StatusCode::OK, Json(payload)).into_response(),
        Err(_) => ApiResponseError::internal_server_error().into_response(),
    }
}

pub async fn item_detail(State(state): State<ServerState>, Path(id): Path<String>) -> Response {
    let query = LibraryDetailQuery::new(id);

    match LibraryService::detail(&state.library_repo, &query, request_meta()).await {
        Ok(Some(payload)) => (StatusCode::OK, Json(payload)).into_response(),
        Ok(None) => ApiResponseError::not_found().into_response(),
        Err(_) => ApiResponseError::internal_server_error().into_response(),
    }
}
