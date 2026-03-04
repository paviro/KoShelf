use super::ServerState;
use super::api;
use crate::runtime::SharedSnapshotStore;
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
}

impl WebServer {
    pub fn new(
        media_cache_dir: PathBuf,
        port: u16,
        snapshot_store: SharedSnapshotStore,
    ) -> Self {
        Self {
            media_cache_dir,
            port,
            snapshot_store,
        }
    }

    pub async fn run(self) -> Result<()> {
        let state = ServerState {
            snapshot_store: self.snapshot_store.clone(),
        };
        let covers_cache_dir = self.media_cache_dir.join("covers");
        let recap_cache_dir = self.media_cache_dir.join("recap");

        let mut app = Router::new()
            // API endpoints
            .route("/api/site", get(api::site))
            .route("/api/locales", get(api::locales))
            .route("/api/books", get(api::books))
            .route("/api/books/{id}", get(api::book_detail))
            .route("/api/comics", get(api::comics))
            .route("/api/comics/{id}", get(api::comic_detail))
            .route("/api/statistics", get(api::statistics_index))
            .route(
                "/api/statistics/weeks/{week_key}",
                get(api::statistics_week),
            )
            .route("/api/statistics/years/{year}", get(api::statistics_year))
            .route("/api/calendar/months", get(api::calendar_months))
            .route("/api/calendar/months/{month_key}", get(api::calendar_month))
            .route("/api/recap", get(api::recap_index))
            .route("/api/recap/years/{year}", get(api::recap_year))
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
