//! HTTP server module — API, embedded frontend, and media serving.

pub mod api;
mod frontend;

use crate::store::memory::{SharedReadingDataStore, SharedSiteStore, UpdateNotifier};
use crate::store::sqlite::repo::LibraryRepository;
use anyhow::Result;
use log::info;
use std::path::PathBuf;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

#[derive(Clone)]
pub struct ServerState {
    pub site_store: SharedSiteStore,
    pub reading_data_store: SharedReadingDataStore,
    pub update_notifier: UpdateNotifier,
    pub library_repo: LibraryRepository,
}

/// Axum-based HTTP server serving the API, embedded React frontend, and media assets.
pub struct WebServer {
    media_cache_dir: PathBuf,
    port: u16,
    site_store: SharedSiteStore,
    reading_data_store: SharedReadingDataStore,
    update_notifier: UpdateNotifier,
    library_repo: LibraryRepository,
}

impl WebServer {
    pub fn new(
        media_cache_dir: PathBuf,
        port: u16,
        site_store: SharedSiteStore,
        reading_data_store: SharedReadingDataStore,
        update_notifier: UpdateNotifier,
        library_repo: LibraryRepository,
    ) -> Self {
        Self {
            media_cache_dir,
            port,
            site_store,
            reading_data_store,
            update_notifier,
            library_repo,
        }
    }

    /// Start listening and serving requests. Blocks until the server shuts down.
    pub async fn run(self) -> Result<()> {
        let state = ServerState {
            site_store: self.site_store.clone(),
            reading_data_store: self.reading_data_store.clone(),
            update_notifier: self.update_notifier.clone(),
            library_repo: self.library_repo,
        };
        let covers_cache_dir = self.media_cache_dir.join("covers");
        let recap_cache_dir = self.media_cache_dir.join("recap");

        let mut app = api::routes()
            .with_state(state)
            .merge(frontend::routes())
            // Runtime-generated media cache directories are mounted under public /assets URLs.
            .nest_service("/assets/covers", ServeDir::new(covers_cache_dir))
            .nest_service("/assets/recap", ServeDir::new(recap_cache_dir));

        app = app.layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::X_CONTENT_TYPE_OPTIONS,
                    axum::http::HeaderValue::from_static("nosniff"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::X_FRAME_OPTIONS,
                    axum::http::HeaderValue::from_static("DENY"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::REFERRER_POLICY,
                    axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
                )),
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
