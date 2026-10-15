use crate::server::ServerState;
use crate::server::api::error::{ApiResponseError, ApiResult};
use crate::server::api::params::{
    DetailQuery, PageActivityParams, ScopeQuery, parse_include, parse_item_sort,
    parse_page_activity_completion, parse_scope, parse_sort_order,
};
use crate::server::api::responses::common::ApiResponse;
use crate::server::api::responses::error::ApiErrorCode;
use crate::shelf::library::{self, LibraryDetailQuery, LibraryListQuery};
use crate::source::koreader::mutations;
use crate::source::metadata::CapturedMetadata;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::{Local, TimeZone, Utc};
use log::warn;
use serde::{Deserialize, Deserializer};
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
    let _guard = state.library_repo.metadata_guard().await;
    let includes = parse_include(detail_query.include.as_deref())?;

    let query = LibraryDetailQuery::new(id, includes);
    let reading_data = state.reading_data_store.get();

    let mut payload = library::detail(&state.library_repo, &query, reading_data.as_deref())
        .await
        .map_err(|_| ApiResponseError::internal_server_error())?
        .ok_or_else(ApiResponseError::not_found)?;
    payload.item.custom_edit =
        super::custom::CustomEdit::new(&payload.item, state.write_coordinator.is_some());
    Ok(Json(ApiResponse::new(payload)))
}

pub(crate) async fn item_page_activity(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Query(params): Query<PageActivityParams>,
) -> ApiResult<impl IntoResponse> {
    let completion_filter = parse_page_activity_completion(params.completion.as_deref())?;
    let reading_data = state.reading_data_store.get();

    let result = library::page_activity(
        &state.library_repo,
        &id,
        reading_data.as_deref(),
        completion_filter,
    )
    .await
    .map_err(|e| {
        warn!("Failed to build page-activity for item {}: {}", id, e);
        ApiResponseError::internal_server_error()
    })?
    .ok_or_else(ApiResponseError::not_found)?;

    Ok(Json(ApiResponse::new(result.response)))
}

// ── Write handlers (requires enable_writeback) ───────────────────────────

/// Three-state patch field: absent (don't change), null (clear), or value (set).
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) enum Patch<T> {
    #[default]
    Absent,
    Null,
    Value(T),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Patch<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize_patch(deserializer)
    }
}

impl Patch<String> {
    /// Convert to `Option<Option<String>>` for DB/Lua layers, treating empty
    /// strings the same as null (clear the field).
    pub(super) fn into_nullable(self) -> Option<Option<String>> {
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
// `WriteContext::prepare` to acquire metadata/file locks and verify the
// revision, performs its mutation, and finally calls `finish` to refresh the cache.

/// Holds the metadata and per-file write locks alongside the captured source.
/// Both locks are released when this context is dropped.
struct WriteContext {
    captured: CapturedMetadata,
    item_id: String,
    _guard: OwnedMutexGuard<()>,
    _file_guard: OwnedMutexGuard<()>,
}

impl WriteContext {
    async fn prepare(state: &ServerState, item_id: &str, headers: &HeaderMap) -> ApiResult<Self> {
        if state.write_coordinator.is_none() {
            return Err(ApiResponseError::not_found());
        }
        let expected = headers
            .get("X-Metadata-Revision")
            .and_then(|h| h.to_str().ok())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                ApiResponseError::bad_request_with_message(
                    ApiErrorCode::InvalidQuery,
                    "X-Metadata-Revision is required",
                )
            })?;
        let guard = state.library_repo.metadata_guard().await;
        let repo = &state.library_repo;
        // Archive transitions happen under the metadata gate, so this check holds
        // for the whole write.
        if repo
            .get_item(item_id)
            .await
            .map_err(internal_metadata_error)?
            .is_some_and(|i| i.is_archived)
        {
            return Err(ApiResponseError::conflict(
                "archived items cannot be edited",
            ));
        }
        let saved = repo
            .metadata_provenance(item_id)
            .await
            .map_err(internal_metadata_error)?;
        let path = repo
            .find_book_path_by_id(item_id)
            .await
            .map_err(internal_metadata_error)?
            .ok_or_else(ApiResponseError::not_found)?;
        let detail = repo
            .get_item(item_id)
            .await
            .map_err(internal_metadata_error)?
            .ok_or_else(ApiResponseError::not_found)?;
        let format = crate::shelf::models::LibraryItemFormat::from_path(std::path::Path::new(
            &format!("book.{}", detail.format),
        ))
        .ok_or_else(ApiResponseError::not_found)?;
        let checksum = crate::source::koreader::calculate_partial_md5(std::path::Path::new(&path))
            .map_err(internal_metadata_error)?;
        let indices = repo.metadata_indices().map_err(internal_metadata_error)?;
        let resolved = indices.resolve(
            std::path::Path::new(&path),
            format,
            &checksum,
            Some(item_id),
        );
        for issue in &resolved.issues {
            warn!("{}: {}", item_id, issue);
        }
        let current = resolved.selected;
        let Some(captured) = current else {
            repo.set_metadata_writable(item_id, false)
                .await
                .map_err(internal_metadata_error)?;
            state.update_notifier.publish(Utc::now().to_rfc3339());
            return Err(ApiResponseError::conflict(
                "no current valid metadata source; editing is blocked",
            ));
        };
        if saved
            .as_ref()
            .is_none_or(|s| s.revision != captured.revision)
        {
            crate::pipeline::ingest::refresh_capture(
                repo,
                item_id,
                Some(&captured),
                &crate::shelf::time_config::TimeConfig::new(state.timezone, 0),
            )
            .await
            .map_err(internal_metadata_error)?;
            state.update_notifier.publish(Utc::now().to_rfc3339());
            return Err(ApiResponseError::conflict(
                "metadata source changed; re-fetch before editing",
            ));
        }
        if expected != captured.revision {
            return Err(ApiResponseError::conflict(
                "metadata revision is stale; re-fetch before editing",
            ));
        }
        let file_guard = state
            .write_coordinator
            .as_ref()
            .expect("writeback enabled")
            .lock_for(&captured.primary_path)
            .lock_owned()
            .await;
        captured
            .verify()
            .map_err(|_| ApiResponseError::conflict("metadata changed; re-fetch before editing"))?;
        Ok(Self {
            captured,
            item_id: item_id.to_string(),
            _guard: guard,
            _file_guard: file_guard,
        })
    }

    async fn finish(&self, state: &ServerState) -> ApiResult<axum::response::Response> {
        let result = async {
            let captured =
                CapturedMetadata::capture(&self.captured.primary_path, self.captured.kind)?;
            crate::pipeline::ingest::refresh_capture(
                &state.library_repo,
                &self.item_id,
                Some(&captured),
                &crate::shelf::time_config::TimeConfig::new(state.timezone, 0),
            )
            .await
        }
        .await;
        // No event suppression: failed cache updates must be recoverable by ingestion.
        state.update_notifier.publish(Utc::now().to_rfc3339());
        match result {
            Ok(()) => Ok(StatusCode::NO_CONTENT.into_response()),
            Err(e) => {
                warn!(
                    "Source saved; capture refresh pending for {}: {e:#}",
                    self.item_id
                );
                Ok(Json(serde_json::json!({"backup_pending": true})).into_response())
            }
        }
    }
}

fn internal_metadata_error(error: anyhow::Error) -> ApiResponseError {
    if error.is::<crate::source::metadata::MetadataChanged>() {
        return ApiResponseError::conflict(
            "metadata changed before write; re-fetch before editing",
        );
    }
    warn!("Metadata operation failed: {}", error);
    ApiResponseError::internal_server_error()
}

pub(crate) async fn update_item(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    headers: HeaderMap,
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
    let ctx = WriteContext::prepare(&state, &id, &headers).await?;

    mutations::write_item_metadata(
        &ctx.captured,
        review_note.as_ref().map(|o| o.as_deref()),
        rating,
        status.as_deref(),
        &modified_date,
    )
    .map_err(|e| {
        warn!("Failed to write metadata for item {}: {}", id, e);
        internal_metadata_error(e)
    })?;

    ctx.finish(&state).await
}

pub(crate) async fn update_annotation(
    State(state): State<ServerState>,
    Path((id, annotation_id)): Path<(String, String)>,
    headers: HeaderMap,
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

    let ctx = WriteContext::prepare(&state, &id, &headers).await?;

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

    mutations::write_annotation_metadata(
        &ctx.captured,
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
        internal_metadata_error(e)
    })?;

    ctx.finish(&state).await
}

pub(crate) async fn delete_annotation(
    State(state): State<ServerState>,
    Path((id, annotation_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let ctx = WriteContext::prepare(&state, &id, &headers).await?;

    let (lua_index, _had_note) = state
        .library_repo
        .find_annotation_write_info(&id, &annotation_id)
        .await
        .map_err(|e| {
            warn!("Failed to resolve annotation {}: {}", annotation_id, e);
            ApiResponseError::internal_server_error()
        })?
        .ok_or_else(ApiResponseError::not_found)?;

    mutations::delete_annotation(&ctx.captured, lua_index).map_err(|e| {
        warn!(
            "Failed to delete annotation {} for item {}: {}",
            annotation_id, id, e
        );
        internal_metadata_error(e)
    })?;

    ctx.finish(&state).await
}

#[cfg(test)]
mod metadata_tests {
    use super::*;
    use crate::source::metadata::SourceKind;
    use crate::source::scanner::MetadataLocation;
    use crate::store::sqlite::repo::tests::{sample_fingerprint, sample_item, test_repo};
    use std::{path::PathBuf, sync::Arc};
    const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    async fn fixture() -> (tempfile::TempDir, ServerState, PathBuf, HeaderMap) {
        let dir = tempfile::tempdir().unwrap();
        let book = dir.path().join("Book.fb2");
        std::fs::write(&book, r#"<?xml version="1.0"?><FictionBook><description><title-info><book-title>Book</book-title><author><first-name>A</first-name><last-name>B</last-name></author></title-info></description><body><section><p>Text</p></section></body></FictionBook>"#).unwrap();
        let path = dir.path().join("Book.sdr/metadata.fb2.lua");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            format!("return {{partial_md5_checksum='{ID}', summary={{rating=2}}}}"),
        )
        .unwrap();
        let captured = CapturedMetadata::capture(&path, SourceKind::Adjacent).unwrap();
        let repo = test_repo().await;
        repo.configure_metadata(&MetadataLocation::InBookFolder);
        *repo.runtime_config.write().unwrap() = Some(crate::pipeline::rebuild::tests::test_config(
            &dir.path().join("output"),
            None,
        ));
        let mut row = sample_item(ID);
        row.file_path = book.to_string_lossy().into_owned();
        row.format = "fb2".into();
        let mut fp = sample_fingerprint(ID);
        fp.book_path = row.file_path.clone();
        fp.metadata_path = Some(path.to_string_lossy().into_owned());
        repo.persist_captured_item(&row, &[], &fp, Some(&captured), "checksum", true)
            .await
            .unwrap();
        let state = ServerState {
            site_store: Arc::new(crate::store::memory::SiteStore::new()),
            reading_data_store: Arc::new(crate::store::memory::ReadingDataStore::new()),
            update_notifier: crate::store::memory::UpdateNotifier::new("test", "now"),
            library_repo: repo,
            auth_state: None,
            auth_proxy_peers: Vec::new(),
            write_coordinator: Some(crate::server::WriteCoordinator::new()),
            library_management: true,
            timezone: None,
        };
        let mut headers = HeaderMap::new();
        headers.insert("X-Metadata-Revision", captured.revision.parse().unwrap());
        (dir, state, path, headers)
    }

    #[tokio::test]
    async fn reading_edits_refresh_external_cover_changes() {
        use crate::store::bundles::{BundleStore, Retention};
        for durable in [false, true] {
            let (dir, mut state, path, _) = fixture().await;
            if durable {
                state.library_repo.bundles = Some(Arc::new(
                    BundleStore::open(&dir.path().join("data"), Retention::Forever).unwrap(),
                ));
            }
            let source = path.parent().unwrap().join("cover.png");
            let served = crate::pipeline::media::resolve_media_dirs(&dir.path().join("output"))
                .covers_dir
                .join(format!("{ID}.webp"));
            let mut previous_url = String::new();
            let mut previous_cover = None;
            for color in [Some([255, 0, 0]), Some([0, 0, 255]), None] {
                if let Some(color) = color {
                    image::RgbImage::from_pixel(2, 2, image::Rgb(color))
                        .save(&source)
                        .unwrap();
                } else {
                    std::fs::remove_file(&source).unwrap();
                }
                // Save a reading edit before the watcher processes the cover change.
                let captured = CapturedMetadata::capture(&path, SourceKind::Adjacent).unwrap();
                let mut headers = HeaderMap::new();
                headers.insert("X-Metadata-Revision", captured.revision.parse().unwrap());
                let ctx = WriteContext::prepare(&state, ID, &headers).await.unwrap();
                mutations::write_item_metadata(&ctx.captured, None, Some(5), None, "2026-10-15")
                    .unwrap();
                assert_eq!(
                    ctx.finish(&state).await.unwrap().status(),
                    StatusCode::NO_CONTENT
                );
                drop(ctx);
                let item = state.library_repo.get_item(ID).await.unwrap().unwrap();
                assert_eq!(item.rating, Some(5));
                assert_ne!(item.cover_url, previous_url);
                previous_url = item.cover_url;
                let cover = std::fs::read(&served).ok();
                assert_eq!(cover.is_some(), color.is_some());
                assert_ne!(cover, previous_cover);
                if let Some(store) = &state.library_repo.bundles
                    && cover.is_some()
                {
                    let bundle = store.get(ID).unwrap().unwrap();
                    assert_eq!(bundle.cover().map(|p| std::fs::read(p).unwrap()), cover);
                }
                previous_cover = cover;
            }
        }
    }

    #[tokio::test]
    async fn annotation_edits_refresh_cache_and_stale_deletes_leave_files_untouched() {
        let (_dir, state, path, _) = fixture().await;
        std::fs::write(&path, format!("return {{partial_md5_checksum='{ID}',summary={{rating=2}},stats={{highlights=1,notes=0}},annotations={{{{text='quote',drawer='lighten',color='yellow',pageno=1}}}}}}" )).unwrap();
        let captured = CapturedMetadata::capture(&path, SourceKind::Adjacent).unwrap();
        {
            let _guard = state.library_repo.metadata_guard().await;
            crate::pipeline::ingest::refresh_capture(
                &state.library_repo,
                ID,
                Some(&captured),
                &crate::shelf::time_config::TimeConfig::new(None, 0),
            )
            .await
            .unwrap();
        }
        let annotation = state
            .library_repo
            .get_annotations(ID, None)
            .await
            .unwrap()
            .remove(0);
        let mut headers = HeaderMap::new();
        headers.insert("X-Metadata-Revision", captured.revision.parse().unwrap());
        let response = update_annotation(
            State(state.clone()),
            Path((ID.into(), annotation.id.clone())),
            headers.clone(),
            Json(UpdateAnnotationRequest {
                note: Patch::Value("new note".into()),
                color: None,
                drawer: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(response.into_response().status(), StatusCode::NO_CONTENT);
        let parsed = crate::source::koreader::LuaParser::new()
            .parse(&path)
            .unwrap();
        assert_eq!(parsed.annotations[0].note.as_deref(), Some("new note"));
        let updated = state
            .library_repo
            .get_annotations(ID, None)
            .await
            .unwrap()
            .remove(0);
        assert_eq!(updated.note.as_deref(), Some("new note"));
        assert_eq!(updated.color.as_deref(), Some("yellow"));
        let before = std::fs::read(&path).unwrap();
        let error = delete_annotation(
            State(state.clone()),
            Path((ID.into(), updated.id.clone())),
            headers,
        )
        .await
        .err()
        .unwrap();
        assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Metadata-Revision",
            state
                .library_repo
                .get_item(ID)
                .await
                .unwrap()
                .unwrap()
                .metadata_revision
                .unwrap()
                .parse()
                .unwrap(),
        );
        let response =
            delete_annotation(State(state.clone()), Path((ID.into(), updated.id)), headers)
                .await
                .unwrap();
        assert_eq!(response.into_response().status(), StatusCode::NO_CONTENT);
        assert!(
            state
                .library_repo
                .get_annotations(ID, None)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            crate::source::koreader::LuaParser::new()
                .parse(&path)
                .unwrap()
                .annotations
                .is_empty()
        );
    }

    #[tokio::test]
    async fn refresh_rejects_changed_bytes_or_a_newer_competing_source() {
        for competing_source in [false, true] {
            let (dir, state, path, _) = fixture().await;
            let captured = CapturedMetadata::capture(&path, SourceKind::Adjacent).unwrap();
            let destination = if competing_source {
                let doc = dir.path().join("doc");
                state
                    .library_repo
                    .configure_metadata(&MetadataLocation::DocSettings(doc.clone()));
                let destination = doc.join("Book.sdr/metadata.fb2.lua");
                std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
                destination
            } else {
                path.clone()
            };
            std::fs::write(
                &destination,
                format!("return {{partial_md5_checksum='{ID}',summary={{rating=5}}}}"),
            )
            .unwrap();
            std::fs::File::options()
                .write(true)
                .open(&destination)
                .unwrap()
                .set_times(
                    std::fs::FileTimes::new().set_modified(
                        std::time::SystemTime::now() + std::time::Duration::from_secs(5),
                    ),
                )
                .unwrap();
            let _guard = state.library_repo.metadata_guard().await;
            let error = crate::pipeline::ingest::refresh_capture(
                &state.library_repo,
                ID,
                Some(&captured),
                &crate::shelf::time_config::TimeConfig::new(None, 0),
            )
            .await
            .unwrap_err();
            assert!(error.is::<crate::source::metadata::MetadataChanged>());
            assert_eq!(
                state
                    .library_repo
                    .metadata_provenance(ID)
                    .await
                    .unwrap()
                    .unwrap()
                    .revision,
                captured.revision
            );
        }
    }

    #[tokio::test]
    async fn write_requires_revision_and_rejects_stale_clients_after_cache_refresh() {
        let (_dir, state, path, headers) = fixture().await;
        assert!(
            WriteContext::prepare(&state, ID, &HeaderMap::new())
                .await
                .is_err()
        );
        let ctx = WriteContext::prepare(&state, ID, &headers).await.unwrap();
        mutations::write_item_metadata(&ctx.captured, None, Some(4), None, "2026-10-14").unwrap();
        ctx.finish(&state).await.unwrap();
        drop(ctx);
        assert!(WriteContext::prepare(&state, ID, &headers).await.is_err());
        assert_eq!(
            crate::source::koreader::LuaParser::new()
                .parse(&path)
                .unwrap()
                .summary
                .unwrap()
                .rating,
            Some(4)
        );
        let current = state.library_repo.get_item(ID).await.unwrap().unwrap();
        assert_eq!(current.rating, Some(4));
        assert_ne!(
            current.metadata_revision.as_deref().unwrap(),
            headers["X-Metadata-Revision"].to_str().unwrap()
        );
    }

    #[tokio::test]
    async fn backup_is_readable_but_restored_only_on_explicit_edit() {
        let (_dir, state, path, headers) = fixture().await;
        let backup = crate::source::koreader::lua_writer::backup_path(&path);
        std::fs::rename(&path, &backup).unwrap();
        assert!(WriteContext::prepare(&state, ID, &headers).await.is_err());
        assert!(!path.exists());
        assert!(backup.exists());
        let current = state.library_repo.get_item(ID).await.unwrap().unwrap();
        assert_eq!(current.metadata_source.as_deref(), Some("backup"));
        let mut current_headers = HeaderMap::new();
        current_headers.insert(
            "X-Metadata-Revision",
            current.metadata_revision.unwrap().parse().unwrap(),
        );
        let before = std::fs::read(&backup).unwrap();
        let ctx = WriteContext::prepare(&state, ID, &current_headers)
            .await
            .unwrap();
        assert!(!path.exists());
        mutations::write_item_metadata(&ctx.captured, None, Some(5), None, "2026-10-14").unwrap();
        ctx.finish(&state).await.unwrap();
        drop(ctx);
        assert_eq!(std::fs::read(&backup).unwrap(), before);
        let restored = state.library_repo.get_item(ID).await.unwrap().unwrap();
        assert_eq!(restored.rating, Some(5));
        assert_eq!(restored.metadata_source.as_deref(), Some("primary"));
        assert_eq!(
            WriteContext::prepare(&state, ID, &current_headers)
                .await
                .err()
                .unwrap()
                .into_response()
                .status(),
            StatusCode::CONFLICT
        );
        assert!(
            state
                .library_repo
                .get_item(ID)
                .await
                .unwrap()
                .unwrap()
                .has_metadata
        );
        assert!(
            state
                .library_repo
                .metadata_provenance(ID)
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn source_switch_refreshes_cache_and_conflicts_before_any_write() {
        let (dir, state, path, headers) = fixture().await;
        // Keep the leftover adjacent source in an older whole second so a
        // subsequent write remains selected under KOReader's tie rules.
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(
                std::fs::FileTimes::new()
                    .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(100)),
            )
            .unwrap();
        let doc = dir.path().join("doc");
        let moved = doc.join("Book.sdr/metadata.fb2.lua");
        std::fs::create_dir_all(moved.parent().unwrap()).unwrap();
        std::fs::copy(&path, &moved).unwrap();
        std::fs::File::options()
            .write(true)
            .open(&moved)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(std::time::SystemTime::now()))
            .unwrap();
        state
            .library_repo
            .configure_metadata(&MetadataLocation::DocSettings(doc));
        let before = std::fs::read(&moved).unwrap();
        assert!(WriteContext::prepare(&state, ID, &headers).await.is_err());
        assert_eq!(std::fs::read(&moved).unwrap(), before);
        let selected_path: String = sqlx::query_scalar(
            "SELECT metadata_path FROM library_item_fingerprints WHERE item_id = ?",
        )
        .bind(ID)
        .fetch_one(state.library_repo.pool())
        .await
        .unwrap();
        assert_eq!(std::path::Path::new(&selected_path), moved);
        let mut current = HeaderMap::new();
        current.insert(
            "X-Metadata-Revision",
            state
                .library_repo
                .get_item(ID)
                .await
                .unwrap()
                .unwrap()
                .metadata_revision
                .unwrap()
                .parse()
                .unwrap(),
        );
        let ctx = WriteContext::prepare(&state, ID, &current).await.unwrap();
        mutations::write_item_metadata(&ctx.captured, None, Some(5), None, "2026-10-14").unwrap();
        ctx.finish(&state).await.unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
    #[tokio::test]
    async fn older_original_refreshes_cache_then_allows_an_explicit_edit() {
        let (_dir, state, path, headers) = fixture().await;
        std::fs::write(
            &path,
            format!("return {{partial_md5_checksum='{ID}', summary={{rating=1}}}}"),
        )
        .unwrap();
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(
                std::fs::FileTimes::new()
                    .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1)),
            )
            .unwrap();
        let error = WriteContext::prepare(&state, ID, &headers)
            .await
            .err()
            .unwrap();
        assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
        let current = state.library_repo.get_item(ID).await.unwrap().unwrap();
        assert_eq!(current.rating, Some(1));
        assert!(current.has_metadata);
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Metadata-Revision",
            current.metadata_revision.unwrap().parse().unwrap(),
        );
        assert!(WriteContext::prepare(&state, ID, &headers).await.is_ok());
    }
    #[tokio::test]
    async fn successful_source_write_reports_pending_backup_and_retries_without_reverting() {
        let (dir, mut state, path, headers) = fixture().await;
        let store = Arc::new(
            crate::store::bundles::BundleStore::open(
                &dir.path().join("data"),
                crate::store::bundles::Retention::Forever,
            )
            .unwrap(),
        );
        state.library_repo.bundles = Some(store.clone());
        let initial = CapturedMetadata::capture(&path, SourceKind::Adjacent).unwrap();
        refresh_capture_for_test(&state, &initial).await;
        // A deterministic I/O failure before publication, without changing permissions.
        store
            .fail_publication
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let response = update_item(
            State(state.clone()),
            Path(ID.into()),
            headers,
            Json(UpdateItemRequest {
                rating: Patch::Value(5),
                status: Patch::Absent,
                review_note: Patch::Absent,
            }),
        )
        .await
        .unwrap()
        .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["backup_pending"],
            true
        );
        assert_eq!(
            CapturedMetadata::capture(&path, SourceKind::Adjacent)
                .unwrap()
                .parse()
                .unwrap()
                .summary
                .unwrap()
                .rating,
            Some(5)
        );
        assert_eq!(
            state
                .library_repo
                .get_item(ID)
                .await
                .unwrap()
                .unwrap()
                .rating,
            Some(5)
        );
        assert_eq!(
            store.get(ID).unwrap().unwrap().item().unwrap().rating(),
            Some(2)
        );
        store
            .fail_publication
            .store(false, std::sync::atomic::Ordering::Relaxed);
        refresh_capture_for_test(
            &state,
            &CapturedMetadata::capture(&path, SourceKind::Adjacent).unwrap(),
        )
        .await;
        assert_eq!(
            store.get(ID).unwrap().unwrap().item().unwrap().rating(),
            Some(5)
        );
    }
    async fn refresh_capture_for_test(state: &ServerState, capture: &CapturedMetadata) {
        let _guard = state.library_repo.metadata_guard().await;
        crate::pipeline::ingest::refresh_capture(
            &state.library_repo,
            ID,
            Some(capture),
            &crate::shelf::time_config::TimeConfig::new(None, 0),
        )
        .await
        .unwrap();
    }
    #[tokio::test]
    async fn archive_removal_enforces_permissions_absence_and_blocks_edits() {
        use crate::server::api::handlers::preservation::remove_item;
        use crate::store::bundles::{BundleStore, Retention, State as Lifecycle};
        let (dir, mut state, path, headers) = fixture().await;
        let output = std::fs::canonicalize(dir.path()).unwrap().join("output");
        let mut config = crate::pipeline::rebuild::tests::test_config(&output, None);
        config.library_paths = vec![dir.path().to_path_buf()];
        config.is_internal_server = true;
        let media = crate::pipeline::media::resolve_media_dirs(&output);
        crate::pipeline::media::create_media_directories(&media).unwrap();
        let store =
            Arc::new(BundleStore::open(&dir.path().join("data"), Retention::Immediate).unwrap());
        state.library_repo.bundles = Some(store.clone());
        state.library_repo.preservation_config = Some(Arc::new(config.clone()));
        refresh_capture_for_test(
            &state,
            &CapturedMetadata::capture(&path, SourceKind::Adjacent).unwrap(),
        )
        .await;
        let error = remove_item(State(state.clone()), Path(ID.into()))
            .await
            .err()
            .unwrap();
        assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
        let book = dir.path().join("Book.fb2");
        let ebook_bytes = std::fs::read(&book).unwrap();
        std::fs::remove_file(&book).unwrap();
        let archived = store.transition(ID, Lifecycle::Archived).unwrap();
        state
            .library_repo
            .project_bundle(&archived, &config.time_config)
            .await
            .unwrap();
        assert_eq!(
            WriteContext::prepare(&state, ID, &headers)
                .await
                .err()
                .unwrap()
                .into_response()
                .status(),
            StatusCode::CONFLICT
        );
        let mut disabled = state.clone();
        disabled.library_management = false;
        assert_eq!(
            remove_item(State(disabled), Path(ID.into()))
                .await
                .err()
                .unwrap()
                .into_response()
                .status(),
            StatusCode::NOT_FOUND
        );
        // A returned ebook must block removal even before reconciliation marks it available.
        std::fs::write(&book, ebook_bytes).unwrap();
        assert_eq!(
            remove_item(State(state.clone()), Path(ID.into()))
                .await
                .err()
                .unwrap()
                .into_response()
                .status(),
            StatusCode::CONFLICT
        );
        std::fs::remove_file(&book).unwrap();
        assert_eq!(
            remove_item(State(state.clone()), Path(ID.into()))
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::NO_CONTENT
        );
        assert!(state.library_repo.get_item(ID).await.unwrap().is_none());
        assert!(store.get(ID).unwrap().is_none());
        assert_eq!(
            std::fs::read_dir(store.root.join("retired"))
                .unwrap()
                .count(),
            0
        );
        assert!(path.exists());
    }
}
