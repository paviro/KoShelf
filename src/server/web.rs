use super::ServerState;
use super::api;
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::runtime::{SharedReadingDataStore, SharedSnapshotStore, SnapshotUpdateNotifier};
use anyhow::Result;
use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::get,
};
use include_dir::{Dir, include_dir};
use log::info;
use std::path::PathBuf;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

static FRONTEND_DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");

pub struct WebServer {
    media_cache_dir: PathBuf,
    port: u16,
    snapshot_store: SharedSnapshotStore,
    reading_data_store: SharedReadingDataStore,
    update_notifier: SnapshotUpdateNotifier,
    library_repo: LibraryRepository,
}

impl WebServer {
    pub fn new(
        media_cache_dir: PathBuf,
        port: u16,
        snapshot_store: SharedSnapshotStore,
        reading_data_store: SharedReadingDataStore,
        update_notifier: SnapshotUpdateNotifier,
        library_repo: LibraryRepository,
    ) -> Self {
        Self {
            media_cache_dir,
            port,
            snapshot_store,
            reading_data_store,
            update_notifier,
            library_repo,
        }
    }

    pub async fn run(self) -> Result<()> {
        let state = ServerState {
            snapshot_store: self.snapshot_store.clone(),
            reading_data_store: self.reading_data_store.clone(),
            update_notifier: self.update_notifier.clone(),
            library_repo: self.library_repo,
        };
        let covers_cache_dir = self.media_cache_dir.join("covers");
        let recap_cache_dir = self.media_cache_dir.join("recap");

        let mut app = Router::new()
            // API endpoints
            .route("/api/site", get(api::site))
            .route("/api/items", get(api::items))
            .route("/api/items/{id}", get(api::item_detail))
            .route("/api/activity/weeks", get(api::activity_weeks))
            .route("/api/activity/weeks/{week_key}", get(api::activity_week))
            .route(
                "/api/activity/years/{year}/daily",
                get(api::activity_year_daily),
            )
            .route(
                "/api/activity/years/{year}/summary",
                get(api::activity_year_summary),
            )
            .route("/api/activity/months", get(api::activity_months))
            .route("/api/activity/months/{month_key}", get(api::activity_month))
            .route("/api/completions/years", get(api::completion_years))
            .route("/api/completions/years/{year}", get(api::completion_year))
            .route("/api/reading/summary", get(api::reading_summary))
            .route("/api/reading/metrics", get(api::reading_metrics))
            .route("/api/events/stream", get(api::events_stream))
            // Embedded React shell mounted at /.
            .route("/", get(react_shell_index_handler))
            .route("/index.html", get(react_shell_index_handler))
            .route("/manifest.json", get(react_shell_manifest_handler))
            .route("/assets/icons/{*path}", get(react_shell_icon_handler))
            .route("/assets/js/{*path}", get(react_shell_js_handler))
            .route("/assets/css/{*path}", get(react_shell_css_handler))
            .with_state(state)
            // Runtime-generated media cache directories are mounted under public /assets URLs.
            .nest_service("/assets/covers", ServeDir::new(covers_cache_dir))
            .nest_service("/assets/recap", ServeDir::new(recap_cache_dir));

        app = app.layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive()),
        );

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;

        info!(
            "Web server running on http://localhost:{}, binding to: 0.0.0.0",
            self.port
        );

        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn react_shell_index_handler() -> Response {
    serve_embedded_frontend_file("index.html")
}

async fn react_shell_manifest_handler() -> Response {
    serve_embedded_frontend_file("manifest.json")
}

async fn react_shell_icon_handler(Path(path): Path<String>) -> Response {
    let full_path = format!("assets/icons/{}", path);
    serve_embedded_frontend_file(&full_path)
}

async fn react_shell_js_handler(Path(path): Path<String>) -> Response {
    let full_path = format!("assets/js/{}", path);
    serve_embedded_frontend_file(&full_path)
}

async fn react_shell_css_handler(Path(path): Path<String>) -> Response {
    let full_path = format!("assets/css/{}", path);
    serve_embedded_frontend_file(&full_path)
}

fn serve_embedded_frontend_file(path: &str) -> Response {
    let Some(file) = FRONTEND_DIST.get_file(path) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    (
        StatusCode::OK,
        [(CONTENT_TYPE, guess_content_type(path))],
        file.contents(),
    )
        .into_response()
}

fn guess_content_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".map") {
        "application/json; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}
