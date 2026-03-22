use crate::server::ServerState;
use crate::server::api::error::{ApiResponseError, ApiResult};
use crate::server::api::params::{
    DetailQuery, ScopeQuery, parse_include, parse_item_sort, parse_scope, parse_sort_order,
};
use crate::server::api::responses::common::ApiResponse;
use crate::server::api::responses::error::ApiErrorCode;
use crate::shelf::library::{self, LibraryDetailQuery, LibraryListQuery};
use crate::source::FileFingerprint;
use crate::source::koreader::{lua_writer, mutations};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::warn;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

pub(crate) async fn items(
    State(state): State<ServerState>,
    Query(query): Query<ScopeQuery>,
) -> ApiResult<impl IntoResponse> {
    let scope = parse_scope(query.scope.as_deref())?;
    let sort = parse_item_sort(query.sort.as_deref())?;
    let order = parse_sort_order(query.order.as_deref())?;

    let list_query = LibraryListQuery { scope, sort, order };

    let payload = library::list(&state.library_repo, list_query)
        .await
        .map_err(|_| ApiResponseError::internal_server_error())?;

    Ok(Json(ApiResponse::new(payload)))
}

pub(crate) async fn item_detail(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Query(detail_query): Query<DetailQuery>,
) -> ApiResult<impl IntoResponse> {
    let includes = parse_include(detail_query.include.as_deref())?;

    let query = LibraryDetailQuery::new(id, includes);
    let reading_data = state.reading_data_store.get();

    let payload = library::detail(&state.library_repo, &query, reading_data.as_deref())
        .await
        .map_err(|_| ApiResponseError::internal_server_error())?
        .ok_or_else(ApiResponseError::not_found)?;

    Ok(Json(ApiResponse::new(payload)))
}

// ── Write handlers (requires enable_writeback) ───────────────────────────

#[derive(Deserialize)]
pub struct UpdateItemRequest {
    pub review_note: Option<String>,
    pub rating: Option<u32>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateAnnotationRequest {
    pub note: Option<String>,
    pub color: Option<String>,
    pub drawer: Option<String>,
}

/// Resolve the metadata sidecar path and its last-ingested fingerprint.
async fn resolve_metadata(state: &ServerState, item_id: &str) -> ApiResult<(PathBuf, i64, i64)> {
    let (path, size, modified) = state
        .library_repo
        .find_metadata_fingerprint_by_item_id(item_id)
        .await
        .map_err(|e| {
            warn!("Failed to resolve metadata for {}: {}", item_id, e);
            ApiResponseError::internal_server_error()
        })?
        .ok_or_else(ApiResponseError::not_found)?;

    Ok((PathBuf::from(path), size, modified))
}

/// Acquire a write lock, restoring the `.old` backup if the primary metadata
/// file went missing (e.g. a previous write panicked after backup-rotate).
async fn acquire_write_lock<'a>(
    lock: &'a Arc<Mutex<()>>,
    metadata_path: &std::path::Path,
) -> MutexGuard<'a, ()> {
    let guard = lock.lock().await;

    // Safety net: if a prior write panicked between backup-rotate and the
    // final write, the primary file may be missing while the .old exists.
    if !metadata_path.exists() {
        let old_path = lua_writer::backup_path(metadata_path);
        if old_path.exists() {
            let _ = std::fs::rename(&old_path, metadata_path);
            warn!("Restored backup for {:?}", metadata_path);
        }
    }

    guard
}

/// Verify the on-disk metadata file matches the last-ingested fingerprint.
///
/// Must be called while holding the `WriteCoordinator` lock to prevent TOCTOU.
/// Returns 409 Conflict if the file was modified externally (e.g. by KoReader).
fn verify_fingerprint(
    metadata_path: &std::path::Path,
    db_size: i64,
    db_modified: i64,
) -> ApiResult<()> {
    let disk = FileFingerprint::capture(metadata_path).map_err(|e| {
        warn!("Failed to stat metadata file {:?}: {}", metadata_path, e);
        ApiResponseError::internal_server_error()
    })?;

    if disk.size_bytes as i64 != db_size || disk.modified_unix_ms as i64 != db_modified {
        return Err(ApiResponseError::conflict(
            "metadata file was modified externally; re-fetch and retry",
        ));
    }

    Ok(())
}

pub(crate) async fn update_item(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateItemRequest>,
) -> ApiResult<impl IntoResponse> {
    let coordinator = state
        .write_coordinator
        .as_ref()
        .ok_or_else(ApiResponseError::not_found)?;

    if let Some(rating) = body.rating {
        if rating > 5 {
            return Err(ApiResponseError::bad_request_with_message(
                ApiErrorCode::InvalidQuery,
                "rating must be between 0 and 5 (0 clears the rating)",
            ));
        }
    }

    if let Some(ref status) = body.status {
        if !matches!(status.as_str(), "reading" | "complete" | "abandoned") {
            return Err(ApiResponseError::bad_request_with_message(
                ApiErrorCode::InvalidQuery,
                "status must be one of: reading, complete, abandoned",
            ));
        }
    }

    let (metadata_path, db_size, db_modified) = resolve_metadata(&state, &id).await?;
    let lock = coordinator.lock_for(&metadata_path);
    let _guard = acquire_write_lock(&lock, &metadata_path).await;

    verify_fingerprint(&metadata_path, db_size, db_modified)?;

    mutations::write_item_metadata(
        &metadata_path,
        body.review_note.as_deref(),
        body.rating,
        body.status.as_deref(),
    )
    .map_err(|e| {
        warn!("Failed to write metadata for item {}: {}", id, e);
        ApiResponseError::internal_server_error()
    })?;

    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn update_annotation(
    State(state): State<ServerState>,
    Path((id, annotation_id)): Path<(String, String)>,
    Json(body): Json<UpdateAnnotationRequest>,
) -> ApiResult<impl IntoResponse> {
    let coordinator = state
        .write_coordinator
        .as_ref()
        .ok_or_else(ApiResponseError::not_found)?;

    if let Some(ref color) = body.color {
        if !matches!(
            color.as_str(),
            "red" | "orange" | "yellow" | "green" | "olive" | "cyan" | "blue" | "purple" | "gray"
        ) {
            return Err(ApiResponseError::bad_request_with_message(
                ApiErrorCode::InvalidQuery,
                "color must be one of: red, orange, yellow, green, olive, cyan, blue, purple, gray",
            ));
        }
    }

    if let Some(ref drawer) = body.drawer {
        if !matches!(
            drawer.as_str(),
            "lighten" | "underscore" | "strikeout" | "invert"
        ) {
            return Err(ApiResponseError::bad_request_with_message(
                ApiErrorCode::InvalidQuery,
                "drawer must be one of: lighten, underscore, strikeout, invert",
            ));
        }
    }

    let lua_index = state
        .library_repo
        .find_lua_index_by_annotation_id(&id, &annotation_id)
        .await
        .map_err(|e| {
            warn!("Failed to resolve annotation {}: {}", annotation_id, e);
            ApiResponseError::internal_server_error()
        })?
        .ok_or_else(ApiResponseError::not_found)?;

    let (metadata_path, db_size, db_modified) = resolve_metadata(&state, &id).await?;
    let lock = coordinator.lock_for(&metadata_path);
    let _guard = acquire_write_lock(&lock, &metadata_path).await;

    verify_fingerprint(&metadata_path, db_size, db_modified)?;

    mutations::write_annotation_metadata(
        &metadata_path,
        lua_index,
        body.note.as_deref(),
        body.color.as_deref(),
        body.drawer.as_deref(),
    )
    .map_err(|e| {
        warn!(
            "Failed to write annotation {} for item {}: {}",
            annotation_id, id, e
        );
        ApiResponseError::internal_server_error()
    })?;

    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn delete_annotation(
    State(state): State<ServerState>,
    Path((id, annotation_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let coordinator = state
        .write_coordinator
        .as_ref()
        .ok_or_else(ApiResponseError::not_found)?;

    let (lua_index, annotation_kind) = state
        .library_repo
        .find_annotation_index_and_kind(&id, &annotation_id)
        .await
        .map_err(|e| {
            warn!("Failed to resolve annotation {}: {}", annotation_id, e);
            ApiResponseError::internal_server_error()
        })?
        .ok_or_else(ApiResponseError::not_found)?;

    let is_highlight = annotation_kind == "highlight";

    let (metadata_path, db_size, db_modified) = resolve_metadata(&state, &id).await?;
    let lock = coordinator.lock_for(&metadata_path);
    let _guard = acquire_write_lock(&lock, &metadata_path).await;

    verify_fingerprint(&metadata_path, db_size, db_modified)?;

    mutations::delete_annotation(&metadata_path, lua_index, is_highlight).map_err(|e| {
        warn!(
            "Failed to delete annotation {} for item {}: {}",
            annotation_id, id, e
        );
        ApiResponseError::internal_server_error()
    })?;

    // Remove from DB and shift lua_index values for subsequent annotations.
    // The write lock is held through this to keep the Lua file and DB in sync.
    state
        .library_repo
        .delete_annotation_and_shift(&id, &annotation_id, lua_index)
        .await
        .map_err(|e| {
            warn!(
                "Failed to update DB after deleting annotation {}: {}",
                annotation_id, e
            );
            ApiResponseError::internal_server_error()
        })?;

    Ok(StatusCode::NO_CONTENT)
}
