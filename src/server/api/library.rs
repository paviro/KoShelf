use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::domain::library::{LibraryDetailQuery, LibraryListQuery, LibraryService};
use crate::server::ServerState;

use super::shared::{ContentTypeQuery, parse_content_type, runtime_snapshot};

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

    let payload = LibraryService::list(
        &snapshot,
        LibraryListQuery {
            scope: content_type,
        },
    );

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn item_detail(State(state): State<ServerState>, Path(id): Path<String>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let payload = LibraryService::detail(&snapshot, &LibraryDetailQuery::new(id));

    (StatusCode::OK, Json(payload)).into_response()
}
