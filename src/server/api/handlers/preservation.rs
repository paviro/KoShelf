use crate::{
    server::{
        ServerState,
        api::error::{ApiResponseError, ApiResult},
    },
    store::bundles::State as Lifecycle,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

fn internal(e: anyhow::Error) -> ApiResponseError {
    log::warn!("Library preservation: {e:#}");
    ApiResponseError::internal_server_error()
}

pub(crate) async fn remove_item(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    if !state.library_management {
        return Err(ApiResponseError::not_found());
    }
    let repo = &state.library_repo;
    let lifecycle = repo.lifecycle_gate.lock().await;
    repo.get_item(&id)
        .await
        .map_err(internal)?
        .ok_or_else(ApiResponseError::not_found)?;
    let store = repo
        .bundles
        .as_ref()
        .ok_or_else(ApiResponseError::not_found)?;
    let config = repo
        .preservation_config
        .as_ref()
        .ok_or_else(ApiResponseError::not_found)?;
    if !crate::pipeline::ingest::preservation::confirm_absent(config, repo, &id)
        .await
        .map_err(internal)?
    {
        return Err(ApiResponseError::conflict(
            "ebook is available or its metadata identity is conflicting",
        ));
    }
    let metadata = repo.metadata_guard().await;
    store
        .transition(&id, Lifecycle::Retired)
        .map_err(internal)?;
    repo.delete_item(&id).await.map_err(internal)?;
    let media =
        crate::pipeline::media::resolve_media_dirs(&config.output_dir, config.is_internal_server);
    crate::pipeline::media::remove_item_files_by_id(&id, &media.files_dir).map_err(internal)?;
    let legacy = media.covers_dir.join(format!("{id}.webp"));
    if let Err(e) = std::fs::remove_file(legacy)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        log::warn!("Legacy cover cleanup: {e}");
    }
    if let Err(e) = store.expire() {
        log::warn!("Retired file cleanup pending: {e:#}");
    }
    drop(metadata);
    drop(lifecycle);
    crate::pipeline::rebuild::rebuild(
        config.statistics_db_paths.iter().cloned().collect(),
        config,
        repo,
        Some(&state.site_store),
        Some(&state.reading_data_store),
        Some(&state.update_notifier),
    )
    .await
    .map_err(internal)?;
    state
        .update_notifier
        .publish(chrono::Utc::now().to_rfc3339());
    Ok(StatusCode::NO_CONTENT)
}
