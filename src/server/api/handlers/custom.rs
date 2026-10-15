//! Edits of KOReader custom files (`custom_metadata.lua`, `cover.*`).
//!
//! Each request carries the revision the client saw (`X-Edit-Revision`): the
//! digest of the file it edits, or `none`. Writes recheck it under the folder
//! lock, so concurrent changes on disk are rejected instead of overwritten.
use super::library::Patch;
use crate::{
    pipeline::{custom_covers, ingest::refresh_capture},
    server::{
        ServerState,
        api::{
            error::{ApiResponseError, ApiResult},
            responses::{error::ApiErrorCode, library::LibraryDetailItem},
        },
    },
    shelf::{models::LibraryItemFormat, time_config::TimeConfig},
    source::{
        custom::{self, Resolution},
        koreader::{
            calculate_partial_md5,
            custom_metadata::{self, Properties},
        },
        metadata::CapturedMetadata,
    },
};
use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capability {
    pub available: bool,
    pub reasons: Vec<String>,
    pub revision: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomEdit {
    pub base: Properties,
    pub overrides: Properties,
    pub metadata: Capability,
    pub cover: Capability,
    pub has_custom_cover: bool,
    pub issues: Vec<String>,
}

impl CustomEdit {
    /// Built from the state recorded at ingestion; writes recheck the files.
    pub fn new(item: &LibraryDetailItem, writeback: bool) -> Option<Self> {
        let saved = item.custom_metadata.as_ref()?;
        let mut reasons = Vec::new();
        if !writeback {
            reasons.push("writeback_disabled".to_string());
        }
        if item.is_archived {
            reasons.push("archived".to_string());
        }
        if saved.error.is_some() {
            reasons.push("inaccessible_custom_files".to_string());
        }
        let capability = |digest: Option<&str>| Capability {
            available: reasons.is_empty(),
            reasons: reasons.clone(),
            revision: revision(digest),
        };
        Some(Self {
            base: saved.base.clone(),
            overrides: saved.overrides.clone(),
            metadata: capability(saved.metadata_digest.as_deref()),
            cover: capability(saved.cover_digest.as_deref()),
            has_custom_cover: saved.cover_digest.is_some(),
            issues: saved.error.iter().cloned().collect(),
        })
    }
}

fn revision(digest: Option<&str>) -> String {
    digest.unwrap_or("none").to_string()
}

fn internal(e: anyhow::Error) -> ApiResponseError {
    log::warn!("Custom edit: {e:#}");
    ApiResponseError::internal_server_error()
}

fn bad(message: &str) -> ApiResponseError {
    ApiResponseError::bad_request_with_message(ApiErrorCode::InvalidQuery, message)
}

/// The book's current custom files, resolved under the metadata gate.
struct Target {
    reading: Option<CapturedMetadata>,
    custom: Resolution,
    saved: custom_metadata::Metadata,
}

async fn locate(state: &ServerState, id: &str) -> ApiResult<Target> {
    let repo = &state.library_repo;
    let item = repo
        .get_item(id)
        .await
        .map_err(internal)?
        .ok_or_else(ApiResponseError::not_found)?;
    if item.is_archived {
        return Err(ApiResponseError::conflict(
            "archived items cannot be edited",
        ));
    }
    let saved = item
        .custom_metadata
        .ok_or_else(|| ApiResponseError::conflict("book is not ingested yet"))?;
    let book = PathBuf::from(
        repo.find_book_path_by_id(id)
            .await
            .map_err(internal)?
            .ok_or_else(ApiResponseError::not_found)?,
    );
    let format =
        LibraryItemFormat::from_path(std::path::Path::new(&format!("book.{}", item.format)))
            .ok_or_else(ApiResponseError::not_found)?;
    let checksum = calculate_partial_md5(&book).map_err(internal)?;
    let indices = repo.metadata_indices().map_err(internal)?;
    let reading = indices.resolve(&book, format, &checksum, Some(id));
    if reading.blocked {
        return Err(ApiResponseError::conflict(
            "reading metadata is ambiguous; resolve it before editing",
        ));
    }
    let preference = repo
        .runtime_config
        .read()
        .expect("runtime configuration lock")
        .as_ref()
        .and_then(|c| c.document_metadata_folder);
    let custom = custom::resolve(
        &book,
        &checksum,
        reading.selected.as_ref().map(|s| s.primary_path.as_path()),
        &indices,
        preference,
    );
    if !custom.issues.is_empty() {
        return Err(ApiResponseError::conflict(custom.issues.join("; ")));
    }
    Ok(Target {
        reading: reading.selected,
        custom,
        saved,
    })
}

fn check_revision(headers: &HeaderMap, current: Option<&str>) -> ApiResult<()> {
    let expected = headers
        .get("X-Edit-Revision")
        .and_then(|h| h.to_str().ok())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| bad("X-Edit-Revision is required"))?;
    if expected != revision(current) {
        return Err(ApiResponseError::conflict(
            "custom files changed; refresh before editing",
        ));
    }
    Ok(())
}

/// Refresh the library record after a write, like reading-sidecar writeback.
async fn finish(state: &ServerState, id: &str, reading: Option<&CapturedMetadata>) -> Response {
    let result = refresh_capture(
        &state.library_repo,
        id,
        reading,
        &TimeConfig::new(state.timezone, 0),
    )
    .await;
    // No event suppression: failed cache updates must be recoverable by ingestion.
    state.update_notifier.publish(Utc::now().to_rfc3339());
    match result {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            log::warn!("Custom files saved; capture refresh pending for {id}: {e:#}");
            Json(serde_json::json!({"backup_pending": true})).into_response()
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MetadataPatch {
    title: Patch<String>,
    authors: Patch<String>,
    series: Patch<String>,
    series_index: Patch<String>,
    language: Patch<String>,
    keywords: Patch<String>,
    description: Patch<String>,
}

pub(crate) async fn metadata(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Result<Json<MetadataPatch>, axum::extract::rejection::JsonRejection>,
) -> ApiResult<Response> {
    let Json(body) = body.map_err(|e| bad(&e.body_text()))?;
    let patch: BTreeMap<_, _> = [
        ("title", body.title),
        ("authors", body.authors),
        ("series", body.series),
        ("series_index", body.series_index),
        ("language", body.language),
        ("keywords", body.keywords),
        ("description", body.description),
    ]
    .into_iter()
    .filter_map(|(k, v)| v.into_nullable().map(|v| (k.to_string(), v)))
    .collect();
    if let Some(Some(index)) = patch.get("series_index")
        && !index.trim().is_empty()
        && custom_metadata::series_index(index).is_none()
    {
        return Err(bad("series_index must be a number"));
    }
    let coordinator = state
        .write_coordinator
        .clone()
        .ok_or_else(ApiResponseError::not_found)?;
    let _guard = state.library_repo.metadata_guard().await;
    let target = locate(&state, &id).await?;
    let folder = target.custom.metadata_folder().to_path_buf();
    let _lock = coordinator.lock_for(&folder).lock_owned().await;
    check_revision(&headers, target.custom.metadata_digest())?;
    let source = target.custom.metadata.as_ref();
    let path = source.map_or_else(|| folder.join("custom_metadata.lua"), |c| c.path.clone());
    custom_metadata::write(
        &path,
        source,
        &target.saved.base,
        target.saved.base_pages,
        &patch,
    )
    .map_err(|e| ApiResponseError::conflict(format!("{e:#}")))?;
    // Removing the last override can leave an empty `.sdr` folder behind.
    let _ = fs::remove_dir(&folder);
    Ok(finish(&state, &id, target.reading.as_ref()).await)
}

pub(crate) async fn upload(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    bytes: Bytes,
) -> ApiResult<Response> {
    let (ext, cover) = tokio::task::spawn_blocking(move || custom_covers::device_bytes(&bytes))
        .await
        .map_err(|e| internal(e.into()))?
        .map_err(|e| bad(&format!("{e:#}")))?;
    write_cover(state, id, headers, Some((ext, cover))).await
}

pub(crate) async fn reset(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    write_cover(state, id, headers, None).await
}

async fn write_cover(
    state: ServerState,
    id: String,
    headers: HeaderMap,
    upload: Option<(String, Vec<u8>)>,
) -> ApiResult<Response> {
    let coordinator = state
        .write_coordinator
        .clone()
        .ok_or_else(ApiResponseError::not_found)?;
    let _guard = state.library_repo.metadata_guard().await;
    let target = locate(&state, &id).await?;
    let folder = target.custom.cover_folder().to_path_buf();
    let _lock = coordinator.lock_for(&folder).lock_owned().await;
    check_revision(&headers, target.custom.cover_digest())?;
    custom_covers::replace(
        &folder,
        upload.as_ref().map(|(ext, b)| (ext.as_str(), b.as_slice())),
    )
    .map_err(|e| ApiResponseError::conflict(format!("{e:#}")))?;
    let _ = fs::remove_dir(&folder);
    Ok(finish(&state, &id, target.reading.as_ref()).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        pipeline::{ingest::ingest_items, media, rebuild::tests::test_config},
        server::WriteCoordinator,
        source::scanner::CollectedItem,
        store::{
            bundles::{BundleStore, Retention, State as BundleState},
            memory::{ReadingDataStore, SiteStore, UpdateNotifier},
            sqlite::repo::tests::test_repo,
        },
    };
    use std::sync::Arc;

    async fn fixture() -> (tempfile::TempDir, ServerState, String) {
        let temp = tempfile::tempdir().unwrap();
        let books = temp.path().join("books");
        fs::create_dir_all(&books).unwrap();
        let book = books.join("Book.fb2");
        fs::write(&book, r#"<?xml version="1.0"?><FictionBook><description><title-info><book-title>Original</book-title><author><first-name>A</first-name><last-name>B</last-name></author><lang>en</lang></title-info></description><body><section><p>Text</p></section></body></FictionBook>"#).unwrap();
        let mut config = test_config(&temp.path().join("output"), None);
        config.library_paths = vec![books];
        config.include_unread = true;
        config.writeback_enabled = true;
        let repo = test_repo().await;
        let dirs = media::resolve_media_dirs(&config.output_dir);
        media::create_media_directories(&dirs).unwrap();
        ingest_items(
            &[CollectedItem {
                path: book.clone(),
                format: LibraryItemFormat::Fb2,
                kobo_hints: None,
            }],
            &config,
            &repo,
            &dirs,
        )
        .await
        .unwrap();
        let id = repo
            .find_fingerprint_by_book_path(&book.to_string_lossy())
            .await
            .unwrap()
            .unwrap()
            .item_id;
        let state = ServerState {
            library_repo: repo,
            site_store: Arc::new(SiteStore::new()),
            reading_data_store: Arc::new(ReadingDataStore::new()),
            update_notifier: UpdateNotifier::new("test", "now"),
            auth_state: None,
            auth_proxy_peers: Vec::new(),
            write_coordinator: Some(WriteCoordinator::new()),
            library_management: false,
            timezone: None,
        };
        (temp, state, id)
    }

    async fn edit(state: &ServerState, id: &str) -> CustomEdit {
        let item = state.library_repo.get_item(id).await.unwrap().unwrap();
        CustomEdit::new(&item, true).unwrap()
    }

    fn headers(revision: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("X-Edit-Revision", revision.parse().unwrap());
        headers
    }

    fn png() -> Bytes {
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(2, 2)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        bytes.into_inner().into()
    }

    #[tokio::test]
    async fn never_opened_book_save_reset_and_stale_revision() {
        let (temp, state, id) = fixture().await;
        let revision = edit(&state, &id).await.metadata.revision;
        assert_eq!(revision, "none");
        let patch = serde_json::from_value(serde_json::json!({"title":"Edited", "authors":"One\nTwo", "series_index":"2.5", "description":"<b>plain</b>\nnext"})).unwrap();
        let response = metadata(
            State(state.clone()),
            Path(id.clone()),
            headers(&revision),
            Ok(Json(patch)),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let item = state.library_repo.get_item(&id).await.unwrap().unwrap();
        assert_eq!(item.title, "Edited");
        assert_eq!(&item.authors.0, &["One", "Two"]);
        let source = temp.path().join("books/Book.sdr/custom_metadata.lua");
        let bytes = fs::read(&source).unwrap();
        let lua = mlua::Lua::new();
        let root: mlua::Table = lua.load(&bytes).eval().unwrap();
        // Like KOReader: the snapshot omits missing values, and series
        // indices are numbers.
        let base: mlua::Table = root.get("doc_props").unwrap();
        for field in ["title", "authors", "language"] {
            assert!(base.contains_key(field).unwrap());
        }
        for field in ["series", "series_index", "keywords", "description"] {
            assert!(!base.contains_key(field).unwrap());
        }
        assert_eq!(base.get::<String>("title").unwrap(), "Original");
        let custom: mlua::Table = root.get("custom_props").unwrap();
        assert!(matches!(
            custom.get::<mlua::Value>("series_index").unwrap(),
            mlua::Value::Number(n) if n == 2.5
        ));
        let invalid = metadata(
            State(state.clone()),
            Path(id.clone()),
            headers(&edit(&state, &id).await.metadata.revision),
            Ok(Json(
                serde_json::from_value(serde_json::json!({"series_index":"volume one"})).unwrap(),
            )),
        )
        .await
        .unwrap_err()
        .into_response();
        assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
        assert_eq!(fs::read(&source).unwrap(), bytes);
        let stale = metadata(
            State(state.clone()),
            Path(id.clone()),
            headers(&revision),
            Ok(Json(MetadataPatch::default())),
        )
        .await
        .unwrap_err()
        .into_response();
        assert_eq!(stale.status(), StatusCode::CONFLICT);
        assert_eq!(fs::read(&source).unwrap(), bytes);
        let patch = serde_json::from_value(
            serde_json::json!({"title":null,"authors":null,"series_index":null,"description":" "}),
        )
        .unwrap();
        let revision = edit(&state, &id).await.metadata.revision;
        metadata(
            State(state.clone()),
            Path(id.clone()),
            headers(&revision),
            Ok(Json(patch)),
        )
        .await
        .unwrap();
        assert!(!source.parent().unwrap().exists());
        let item = state.library_repo.get_item(&id).await.unwrap().unwrap();
        assert_eq!(item.title, "Original");
    }

    #[tokio::test]
    async fn invalid_metadata_keeps_last_overrides_and_blocks_edits() {
        let (temp, state, id) = fixture().await;
        let patch = serde_json::from_value(serde_json::json!({"title":"Saved"})).unwrap();
        metadata(
            State(state.clone()),
            Path(id.clone()),
            headers("none"),
            Ok(Json(patch)),
        )
        .await
        .unwrap();
        fs::write(
            temp.path().join("books/Book.sdr/custom_metadata.lua"),
            b"not valid lua",
        )
        .unwrap();
        {
            let _guard = state.library_repo.metadata_guard().await;
            refresh_capture(&state.library_repo, &id, None, &TimeConfig::new(None, 0))
                .await
                .unwrap();
        }
        let item = state.library_repo.get_item(&id).await.unwrap().unwrap();
        assert_eq!(item.title, "Saved");
        assert!(!edit(&state, &id).await.metadata.available);
    }

    #[tokio::test]
    async fn cover_upload_reset_and_invalid_upload() {
        let (temp, state, id) = fixture().await;
        let cover = edit(&state, &id).await.cover;
        assert!(cover.available);
        let bytes = png();
        let response = upload(
            State(state.clone()),
            Path(id.clone()),
            headers(&cover.revision),
            bytes.clone(),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let source = temp.path().join("books/Book.sdr/cover.png");
        assert_eq!(fs::read(&source).unwrap(), bytes);
        let served = media::resolve_media_dirs(&temp.path().join("output"))
            .covers_dir
            .join(format!("{id}.webp"));
        assert!(served.is_file());
        let item = state.library_repo.get_item(&id).await.unwrap().unwrap();
        assert!(item.cover_url.contains("?v="));
        let edited = edit(&state, &id).await;
        assert!(edited.has_custom_cover);
        let invalid = upload(
            State(state.clone()),
            Path(id.clone()),
            headers(&edited.cover.revision),
            Bytes::from_static(b"<svg/>"),
        )
        .await
        .unwrap_err()
        .into_response();
        assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
        assert_eq!(fs::read(&source).unwrap(), bytes);
        reset(
            State(state.clone()),
            Path(id.clone()),
            headers(&edited.cover.revision),
        )
        .await
        .unwrap();
        assert!(!source.parent().unwrap().exists());
        // The ebook has no cover, so the served cover goes away too.
        assert!(!served.exists());
        let item = state.library_repo.get_item(&id).await.unwrap().unwrap();
        assert!(!item.cover_url.contains("?v="));
    }

    #[tokio::test]
    async fn archived_books_keep_custom_metadata_and_are_read_only() {
        let (temp, mut state, id) = fixture().await;
        let store =
            Arc::new(BundleStore::open(&temp.path().join("durable"), Retention::Forever).unwrap());
        state.library_repo.bundles = Some(store.clone());
        let patch = serde_json::from_value(serde_json::json!({"title":"Captured"})).unwrap();
        metadata(
            State(state.clone()),
            Path(id.clone()),
            headers("none"),
            Ok(Json(patch)),
        )
        .await
        .unwrap();
        let archived = store.transition(&id, BundleState::Archived).unwrap();
        let rebuilt = test_repo().await;
        rebuilt
            .project_bundle(&archived, &TimeConfig::new(None, 0))
            .await
            .unwrap();
        let item = rebuilt.get_item(&id).await.unwrap().unwrap();
        assert!(item.is_archived);
        assert_eq!(item.title, "Captured");
        let edit = CustomEdit::new(&item, true).unwrap();
        assert_eq!(edit.base["title"], "Original");
        assert_eq!(edit.metadata.reasons, ["archived"]);
    }

    #[test]
    fn patch_rejects_incompatible_types_and_preserves_omissions() {
        assert!(
            serde_json::from_value::<MetadataPatch>(serde_json::json!({"authors":["A"]})).is_err()
        );
        assert!(
            serde_json::from_value::<MetadataPatch>(serde_json::json!({"series_index":2})).is_err()
        );
        assert!(
            serde_json::from_value::<MetadataPatch>(serde_json::json!({"unknown":"x"})).is_err()
        );
        let patch: MetadataPatch =
            serde_json::from_value(serde_json::json!({"title":null})).unwrap();
        assert!(matches!(patch.title, Patch::Null));
        assert!(matches!(patch.authors, Patch::Absent));
    }
}
