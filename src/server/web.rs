use super::ServerState;
use super::api;
use super::version::SharedVersionNotifier;
use crate::runtime::SharedSnapshotStore;
use anyhow::Result;
use axum::{
    Router,
    extract::{Path, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::get,
};
use include_dir::{Dir, include_dir};
use log::info;
use std::path::PathBuf;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_status::SetStatus;

static FRONTEND_DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");

pub struct WebServer {
    site_dir: PathBuf,
    port: u16,
    version_notifier: SharedVersionNotifier,
    snapshot_store: SharedSnapshotStore,
    serve_react_shell: bool,
}

impl WebServer {
    pub fn new(
        site_dir: PathBuf,
        port: u16,
        version_notifier: SharedVersionNotifier,
        snapshot_store: SharedSnapshotStore,
        serve_react_shell: bool,
    ) -> Self {
        Self {
            site_dir,
            port,
            version_notifier,
            snapshot_store,
            serve_react_shell,
        }
    }

    pub async fn run(self) -> Result<()> {
        let serve_react_shell = self.serve_react_shell;

        let state = ServerState {
            site_dir: self.site_dir.clone(),
            version_notifier: self.version_notifier.clone(),
            snapshot_store: self.snapshot_store.clone(),
        };

        // Serve the generated static 404 page.
        let not_found_service = SetStatus::new(
            ServeFile::new(self.site_dir.join("404.html")),
            StatusCode::NOT_FOUND,
        );

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
            // Long-poll endpoint for version changes
            .route("/api/events/version", get(version_poll_handler))
            .with_state(state);

        if serve_react_shell {
            app = app
                .route("/", get(react_shell_index_handler))
                .route("/index.html", get(react_shell_index_handler))
                .route("/react-assets/{*path}", get(react_shell_asset_handler));
        }

        app = app
            .fallback_service(ServeDir::new(&self.site_dir).not_found_service(not_found_service))
            .layer(
                ServiceBuilder::new()
                    .layer(CompressionLayer::new())
                    .layer(CorsLayer::permissive()),
            );

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;

        info!(
            "Web server running on http://localhost:{}, binding to: 0.0.0.0",
            self.port
        );
        if serve_react_shell {
            info!("Embedded React shell enabled at /");
        } else {
            info!("Embedded React shell disabled; serving legacy root route");
        }

        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Long-poll handler that waits for version changes.
/// Returns the new version when available, or times out after 60 seconds.
async fn version_poll_handler(State(state): State<ServerState>) -> impl IntoResponse {
    let mut receiver = state.version_notifier.subscribe();

    // Wait up to 60 seconds for a version change
    match tokio::time::timeout(Duration::from_secs(60), receiver.recv()).await {
        Ok(Ok(version)) => {
            // New version available
            (StatusCode::OK, version)
        }
        Ok(Err(_)) => {
            // Channel closed (shouldn't happen in normal operation)
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Channel closed".to_string(),
            )
        }
        Err(_) => {
            // Timeout - client should reconnect
            (StatusCode::NO_CONTENT, String::new())
        }
    }
}

async fn react_shell_index_handler() -> Response {
    serve_embedded_frontend_file("index.html")
}

async fn react_shell_asset_handler(Path(path): Path<String>) -> Response {
    let full_path = format!("react-assets/{}", path);
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
