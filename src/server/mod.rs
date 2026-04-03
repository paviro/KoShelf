//! HTTP server module — API, embedded frontend, and media serving.

pub mod api;
pub mod auth;
mod frontend;

use crate::store::memory::{SharedReadingDataStore, SharedSiteStore, UpdateNotifier};
use crate::store::sqlite::repo::LibraryRepository;
use anyhow::Result;
use axum::Router;
use axum::routing::{delete, get, patch, post, put};
use dashmap::DashMap;
use log::info;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

/// Per-file mutex coordinator for serializing metadata writes.
///
/// One entry per distinct metadata file path is kept in the map for the
/// lifetime of the server. This is bounded by the number of books in the
/// library (typically hundreds to low thousands), so unbounded growth is
/// not a practical concern.
#[derive(Clone, Default)]
pub struct WriteCoordinator {
    locks: Arc<DashMap<PathBuf, Arc<Mutex<()>>>>,
    recent_writes: Arc<DashMap<PathBuf, Instant>>,
}

/// Shared map of recently-written metadata paths to the time they were written.
///
/// Write handlers insert paths after modifying a metadata file; the file
/// watcher checks timestamps to suppress self-triggered events within a
/// suppression window.
pub type RecentWrites = Arc<DashMap<PathBuf, Instant>>;

impl WriteCoordinator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Acquire a lock for the given metadata file path.
    pub fn lock_for(&self, path: &Path) -> Arc<Mutex<()>> {
        self.locks
            .entry(path.to_path_buf())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    /// Mark a metadata path as recently written by a write handler.
    pub fn mark_written(&self, path: &Path) {
        self.recent_writes
            .insert(path.to_path_buf(), Instant::now());
    }

    /// Remove a path from the recent-writes map so the file watcher will
    /// pick up the change and trigger re-ingestion.
    pub fn unmark_written(&self, path: &Path) {
        self.recent_writes.remove(path);
    }

    /// Get a shared handle to the recent-writes map for the file watcher.
    pub fn recent_writes(&self) -> RecentWrites {
        self.recent_writes.clone()
    }
}

#[derive(Clone)]
pub struct ServerState {
    pub site_store: SharedSiteStore,
    pub reading_data_store: SharedReadingDataStore,
    pub update_notifier: UpdateNotifier,
    pub library_repo: LibraryRepository,
    pub auth_state: Option<auth::AuthState>,
    pub write_coordinator: Option<WriteCoordinator>,
    pub timezone: Option<chrono_tz::Tz>,
}

/// Axum-based HTTP server serving the API, embedded React frontend, and media assets.
pub struct WebServer {
    media_cache_dir: PathBuf,
    port: u16,
    site_store: SharedSiteStore,
    reading_data_store: SharedReadingDataStore,
    update_notifier: UpdateNotifier,
    library_repo: LibraryRepository,
    auth_state: Option<auth::AuthState>,
    write_coordinator: Option<WriteCoordinator>,
    timezone: Option<chrono_tz::Tz>,
}

pub struct WebServerOptions {
    pub media_cache_dir: PathBuf,
    pub port: u16,
    pub site_store: SharedSiteStore,
    pub reading_data_store: SharedReadingDataStore,
    pub update_notifier: UpdateNotifier,
    pub library_repo: LibraryRepository,
    pub auth_state: Option<auth::AuthState>,
    pub write_coordinator: Option<WriteCoordinator>,
    pub timezone: Option<chrono_tz::Tz>,
}

impl WebServer {
    pub fn new(options: WebServerOptions) -> Self {
        let WebServerOptions {
            media_cache_dir,
            port,
            site_store,
            reading_data_store,
            update_notifier,
            library_repo,
            auth_state,
            write_coordinator,
            timezone,
        } = options;

        Self {
            media_cache_dir,
            port,
            site_store,
            reading_data_store,
            update_notifier,
            library_repo,
            auth_state,
            write_coordinator,
            timezone,
        }
    }

    /// Start listening and serving requests. Blocks until the server shuts down.
    pub async fn run(self) -> Result<()> {
        let state = ServerState {
            site_store: self.site_store.clone(),
            reading_data_store: self.reading_data_store.clone(),
            update_notifier: self.update_notifier.clone(),
            library_repo: self.library_repo,
            auth_state: self.auth_state,
            write_coordinator: self.write_coordinator,
            timezone: self.timezone,
        };
        let covers_cache_dir = self.media_cache_dir.join("covers");
        let files_cache_dir = self.media_cache_dir.join("files");
        let recap_cache_dir = self.media_cache_dir.join("recap");

        let mut app = api::routes()
            .with_state(state.clone())
            .merge(frontend::routes())
            // Runtime-generated media cache directories are mounted under public /assets URLs.
            .nest_service("/assets/covers", ServeDir::new(covers_cache_dir))
            .nest_service("/assets/files", ServeDir::new(files_cache_dir))
            .nest_service("/assets/recap", ServeDir::new(recap_cache_dir));

        if state.auth_state.is_some() {
            let auth_routes = Router::new()
                .route("/api/auth/login", post(auth::login::login_submit))
                .route("/api/auth/logout", post(auth::login::logout))
                .route("/api/auth/password", put(auth::login::change_password))
                .route("/api/auth/sessions", get(auth::login::list_sessions))
                .route(
                    "/api/auth/sessions/{session_id}",
                    delete(auth::login::revoke_session),
                )
                .with_state(state.clone());
            app = app.merge(auth_routes);
        }

        if state.write_coordinator.is_some() {
            let write_routes = Router::new()
                .route("/api/items/{id}", patch(api::handlers::update_item))
                .route(
                    "/api/items/{id}/annotations/{annotation_id}",
                    patch(api::handlers::update_annotation),
                )
                .route(
                    "/api/items/{id}/annotations/{annotation_id}",
                    delete(api::handlers::delete_annotation),
                )
                .with_state(state.clone());
            app = app.merge(write_routes);
        }

        app = app.layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::auth_middleware,
        ));

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
            "Listening on http://0.0.0.0:{} (auth: {}, writeback: {})",
            self.port,
            if state.auth_state.is_some() {
                "on"
            } else {
                "off"
            },
            if state.write_coordinator.is_some() {
                "on"
            } else {
                "off"
            },
        );

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;

        Ok(())
    }
}
