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
use chrono::{Local, TimeZone, Utc};
use log::warn;
use serde::{Deserialize, Deserializer};
use std::path::PathBuf;
use tokio::sync::OwnedMutexGuard;

const VALID_COLORS: &[&str] = &[
    "red", "orange", "yellow", "green", "olive", "cyan", "blue", "purple", "gray",
];
const VALID_DRAWERS: &[&str] = &["lighten", "underscore", "strikeout", "invert"];

/// Format the current time in the configured timezone (falls back to system local).
/// Matches KoReader's `os.date()` which uses the device's local time.
fn now_in_tz(tz: Option<&chrono_tz::Tz>, fmt: &str) -> String {
    let utc = Utc::now();
    match tz {
        Some(tz) => tz
            .from_utc_datetime(&utc.naive_utc())
            .format(fmt)
            .to_string(),
        None => utc.with_timezone(&Local).format(fmt).to_string(),
    }
}

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
        .map_err(|e| {
            warn!("Failed to list library items: {}", e);
            ApiResponseError::internal_server_error()
        })?;

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

/// Three-state patch field: absent (don't change), null (clear), or value (set).
#[derive(Debug, Clone, Default, PartialEq)]
enum Patch<T> {
    #[default]
    Absent,
    Null,
    Value(T),
}

impl Patch<String> {
    /// Convert to `Option<Option<String>>` for DB/Lua layers, treating empty
    /// strings the same as null (clear the field).
    fn into_nullable(self) -> Option<Option<String>> {
        match self {
            Patch::Absent => None,
            Patch::Null => Some(None),
            Patch::Value(v) if v.trim().is_empty() => Some(None),
            Patch::Value(v) => Some(Some(v)),
        }
    }
}

fn deserialize_patch<'de, T, D>(deserializer: D) -> Result<Patch<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    // This is only called when the field is present in JSON.
    // If null → None → Patch::Null, if value → Some(v) → Patch::Value(v).
    Option::<T>::deserialize(deserializer).map(|opt| match opt {
        Some(v) => Patch::Value(v),
        None => Patch::Null,
    })
}

#[derive(Deserialize)]
pub(crate) struct UpdateItemRequest {
    #[serde(default, deserialize_with = "deserialize_patch")]
    review_note: Patch<String>,
    #[serde(default, deserialize_with = "deserialize_patch")]
    rating: Patch<u32>,
    #[serde(default, deserialize_with = "deserialize_patch")]
    status: Patch<String>,
}

/// `note` uses `Patch` (three-state: absent / null / value) because notes can
/// be cleared.  `color` and `drawer` use plain `Option` (absent / value)
/// because KoReader annotations always have a color and drawer once created.
#[derive(Deserialize)]
pub(crate) struct UpdateAnnotationRequest {
    #[serde(default, deserialize_with = "deserialize_patch")]
    note: Patch<String>,
    color: Option<String>,
    drawer: Option<String>,
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

        // FileFingerprint uses u64; SQLite stores i64. Cast is lossless for realistic values.
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

    /// Mark the path as recently written, refresh the DB fingerprint, and
    /// publish an SSE notification.
    ///
    /// Must be called while the write lock is still held (i.e. before this
    /// struct is dropped). Errors are logged but not propagated — the Lua
    /// write already succeeded; the worst case is the next write gets a 409
    /// that self-heals on re-ingestion.
    async fn finish(&self, state: &ServerState) {
        // Tell the file watcher to ignore the filesystem event we just caused.
        if let Some(ref coordinator) = state.write_coordinator {
            coordinator.mark_written(&self.metadata_path);
        }
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
    if let Patch::Value(rating) = body.rating
        && rating > 5
    {
        return Err(ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            "rating must be between 0 and 5 (0 clears the rating)",
        ));
    }

    if let Patch::Value(ref status) = body.status
        && !matches!(status.as_str(), "reading" | "complete" | "abandoned")
    {
        return Err(ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            "status must be one of: reading, complete, abandoned",
        ));
    }

    let review_note = body.review_note.into_nullable();
    let rating: Option<u32> = match body.rating {
        Patch::Absent => None,
        Patch::Null => Some(0),
        Patch::Value(v) => Some(v),
    };
    let status: Option<String> = match body.status {
        Patch::Absent => None,
        Patch::Null => None, // can't clear status
        Patch::Value(v) => Some(v),
    };

    let modified_date = now_in_tz(state.timezone.as_ref(), "%Y-%m-%d");
    let ctx = WriteContext::prepare(&state, &id).await?;

    mutations::write_item_metadata(
        &ctx.metadata_path,
        review_note.as_ref().map(|o| o.as_deref()),
        rating,
        status.as_deref(),
        &modified_date,
    )
    .map_err(|e| {
        warn!("Failed to write metadata for item {}: {}", id, e);
        ApiResponseError::internal_server_error()
    })?;

    if let Err(e) = state
        .library_repo
        .update_item_writeback_fields(
            &id,
            review_note.as_ref().map(|o| o.as_deref()),
            rating,
            status.as_deref(),
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
        && !VALID_COLORS.contains(&color.as_str())
    {
        return Err(ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            format!("color must be one of: {}", VALID_COLORS.join(", ")),
        ));
    }

    if let Some(ref drawer) = body.drawer
        && !VALID_DRAWERS.contains(&drawer.as_str())
    {
        return Err(ApiResponseError::bad_request_with_message(
            ApiErrorCode::InvalidQuery,
            format!("drawer must be one of: {}", VALID_DRAWERS.join(", ")),
        ));
    }

    let (lua_index, had_note) = state
        .library_repo
        .find_annotation_write_info(&id, &annotation_id)
        .await
        .map_err(|e| {
            warn!("Failed to resolve annotation {}: {}", annotation_id, e);
            ApiResponseError::internal_server_error()
        })?
        .ok_or_else(ApiResponseError::not_found)?;

    let note = body.note.into_nullable();

    // KOReader sets datetime_updated on color/drawer changes and on note type
    // transitions (highlight↔note), but NOT on plain note text edits.
    let note_type_transition = matches!(
        (had_note, &note),
        (Some(false), Some(Some(_))) | (Some(true), Some(None))
    );

    let datetime_updated = if body.color.is_some() || body.drawer.is_some() || note_type_transition
    {
        Some(now_in_tz(state.timezone.as_ref(), "%Y-%m-%d %H:%M:%S"))
    } else {
        None
    };

    let ctx = WriteContext::prepare(&state, &id).await?;

    mutations::write_annotation_metadata(
        &ctx.metadata_path,
        lua_index,
        note.as_ref().map(|o| o.as_deref()),
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
            note.as_ref().map(|o| o.as_deref()),
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
    let (lua_index, had_note) = state
        .library_repo
        .find_annotation_write_info(&id, &annotation_id)
        .await
        .map_err(|e| {
            warn!("Failed to resolve annotation {}: {}", annotation_id, e);
            ApiResponseError::internal_server_error()
        })?
        .ok_or_else(ApiResponseError::not_found)?;

    let is_highlight = had_note.is_some(); // Some(_) = has drawer = highlight

    let ctx = WriteContext::prepare(&state, &id).await?;

    mutations::delete_annotation(&ctx.metadata_path, lua_index).map_err(|e| {
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
