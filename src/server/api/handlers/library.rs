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
use chrono::{Local, Utc};
use log::warn;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::sync::OwnedMutexGuard;

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

// ── Write context ─────────────────────────────────────────────────────────
//
// Encapsulates the lock → verify → … → refresh → publish contract shared by
// all write handlers.  Each handler validates input, then calls
// `WriteContext::prepare` to acquire the per-file lock and verify the
// fingerprint, performs its mutation + DB sync, and finally calls `finish`.

/// Holds the per-file write lock and metadata path for the duration of a
/// write operation.  The lock is released when this struct is dropped.
struct WriteContext {
    metadata_path: PathBuf,
    item_id: String,
    _guard: OwnedMutexGuard<()>,
}

impl WriteContext {
    /// Resolve metadata, acquire per-file lock, verify fingerprint.
    async fn prepare(state: &ServerState, item_id: &str) -> ApiResult<Self> {
        let coordinator = state
            .write_coordinator
            .as_ref()
            .ok_or_else(ApiResponseError::not_found)?;

        let (metadata_path, db_size, db_modified) = {
            let (path, size, modified) = state
                .library_repo
                .find_metadata_fingerprint_by_item_id(item_id)
                .await
                .map_err(|e| {
                    warn!("Failed to resolve metadata for {}: {}", item_id, e);
                    ApiResponseError::internal_server_error()
                })?
                .ok_or_else(ApiResponseError::not_found)?;
            (PathBuf::from(path), size, modified)
        };

        let lock = coordinator.lock_for(&metadata_path);
        let guard = lock.lock_owned().await;

        // Safety net: if a prior write panicked between backup-rotate and the
        // final write, the primary file may be missing while the .old exists.
        if !metadata_path.exists() {
            let old_path = lua_writer::backup_path(&metadata_path);
            if old_path.exists() {
                let _ = std::fs::rename(&old_path, &metadata_path);
                warn!("Restored backup for {:?}", metadata_path);
            }
        }

        let disk = FileFingerprint::capture(&metadata_path).map_err(|e| {
            warn!("Failed to stat metadata file {:?}: {}", metadata_path, e);
            ApiResponseError::internal_server_error()
        })?;

        if disk.size_bytes as i64 != db_size || disk.modified_unix_ms as i64 != db_modified {
            return Err(ApiResponseError::conflict(
                "metadata file was modified externally; re-fetch and retry",
            ));
        }

        Ok(Self {
            metadata_path,
            item_id: item_id.to_string(),
            _guard: guard,
        })
    }

    /// Refresh the DB fingerprint and publish an SSE notification.
    ///
    /// Must be called while the write lock is still held (i.e. before this
    /// struct is dropped). Errors are logged but not propagated — the Lua
    /// write already succeeded; the worst case is the next write gets a 409
    /// that self-heals on re-ingestion.
    async fn finish(&self, state: &ServerState) {
        match FileFingerprint::capture(&self.metadata_path) {
            Ok(fp) => {
                if let Err(e) = state
                    .library_repo
                    .update_metadata_fingerprint(
                        &self.item_id,
                        fp.size_bytes as i64,
                        fp.modified_unix_ms as i64,
                    )
                    .await
                {
                    warn!(
                        "Failed to update fingerprint after write for {}: {}",
                        self.item_id, e
                    );
                }
            }
            Err(e) => {
                warn!(
                    "Failed to capture fingerprint after write for {}: {}",
                    self.item_id, e
                );
            }
        }

        state.update_notifier.publish(Utc::now().to_rfc3339());
    }
}

// ── Write handlers ────────────────────────────────────────────────────────

pub(crate) async fn update_item(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateItemRequest>,
) -> ApiResult<impl IntoResponse> {
    if let Some(rating) = body.rating
        && rating > 5
    {
        return Err(ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            "rating must be between 0 and 5 (0 clears the rating)",
        ));
    }

    if let Some(ref status) = body.status
        && !matches!(status.as_str(), "reading" | "complete" | "abandoned")
    {
        return Err(ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            "status must be one of: reading, complete, abandoned",
        ));
    }

    let ctx = WriteContext::prepare(&state, &id).await?;

    mutations::write_item_metadata(
        &ctx.metadata_path,
        body.review_note.as_deref(),
        body.rating,
        body.status.as_deref(),
    )
    .map_err(|e| {
        warn!("Failed to write metadata for item {}: {}", id, e);
        ApiResponseError::internal_server_error()
    })?;

    if let Err(e) = state
        .library_repo
        .update_item_writeback_fields(
            &id,
            body.review_note.as_deref(),
            body.rating,
            body.status.as_deref(),
        )
        .await
    {
        warn!("Failed to update DB after item write {}: {}", id, e);
    }

    ctx.finish(&state).await;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn update_annotation(
    State(state): State<ServerState>,
    Path((id, annotation_id)): Path<(String, String)>,
    Json(body): Json<UpdateAnnotationRequest>,
) -> ApiResult<impl IntoResponse> {
    if let Some(ref color) = body.color
        && !matches!(
            color.as_str(),
            "red" | "orange" | "yellow" | "green" | "olive" | "cyan" | "blue" | "purple" | "gray"
        )
    {
        return Err(ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            "color must be one of: red, orange, yellow, green, olive, cyan, blue, purple, gray",
        ));
    }

    if let Some(ref drawer) = body.drawer
        && !matches!(
            drawer.as_str(),
            "lighten" | "underscore" | "strikeout" | "invert"
        )
    {
        return Err(ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            "drawer must be one of: lighten, underscore, strikeout, invert",
        ));
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

    // Compute datetime_updated once so the Lua file and DB receive the same
    // timestamp (previously this was computed in two separate Local::now() calls).
    let datetime_updated = if body.color.is_some() || body.drawer.is_some() {
        Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
    } else {
        None
    };

    let ctx = WriteContext::prepare(&state, &id).await?;

    mutations::write_annotation_metadata(
        &ctx.metadata_path,
        lua_index,
        body.note.as_deref(),
        body.color.as_deref(),
        body.drawer.as_deref(),
        datetime_updated.as_deref(),
    )
    .map_err(|e| {
        warn!(
            "Failed to write annotation {} for item {}: {}",
            annotation_id, id, e
        );
        ApiResponseError::internal_server_error()
    })?;

    if let Err(e) = state
        .library_repo
        .update_annotation_writeback_fields(
            &annotation_id,
            body.note.as_deref(),
            body.color.as_deref(),
            body.drawer.as_deref(),
            datetime_updated.as_deref(),
        )
        .await
    {
        warn!(
            "Failed to update DB after annotation write {}: {}",
            annotation_id, e
        );
    }

    ctx.finish(&state).await;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn delete_annotation(
    State(state): State<ServerState>,
    Path((id, annotation_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
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

    let ctx = WriteContext::prepare(&state, &id).await?;

    mutations::delete_annotation(&ctx.metadata_path, lua_index, is_highlight).map_err(|e| {
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

    if let Err(e) = state
        .library_repo
        .decrement_annotation_count(&id, is_highlight)
        .await
    {
        warn!("Failed to decrement annotation count for {}: {}", id, e);
    }

    ctx.finish(&state).await;
    Ok(StatusCode::NO_CONTENT)
}
