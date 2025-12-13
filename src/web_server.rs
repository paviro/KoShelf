use crate::version_notifier::SharedVersionNotifier;
use anyhow::Result;
use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use log::info;
use std::path::PathBuf;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

pub struct WebServer {
    site_dir: PathBuf,
    port: u16,
    version_notifier: SharedVersionNotifier,
}

impl WebServer {
    pub fn new(site_dir: PathBuf, port: u16, version_notifier: SharedVersionNotifier) -> Self {
        Self {
            site_dir,
            port,
            version_notifier,
        }
    }

    pub async fn run(self) -> Result<()> {
        // Create a 404 page service
        let not_found_service = ServeFile::new("templates/404.html");

        let app = Router::new()
            // Long-poll endpoint for version changes
            .route("/api/events/version", get(version_poll_handler))
            .with_state(self.version_notifier.clone())
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

        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Long-poll handler that waits for version changes.
/// Returns the new version when available, or times out after 60 seconds.
async fn version_poll_handler(State(notifier): State<SharedVersionNotifier>) -> impl IntoResponse {
    let mut receiver = notifier.subscribe();

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
