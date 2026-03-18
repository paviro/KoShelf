//! HTTP server module — API, embedded frontend, and media serving.

pub mod api;
pub mod auth;
mod frontend;

use crate::store::memory::{SharedReadingDataStore, SharedSiteStore, UpdateNotifier};
use crate::store::sqlite::repo::LibraryRepository;
use anyhow::Result;
use axum::Router;
use axum::routing::{delete, get, post, put};
use log::info;
use sqlx::SqlitePool;
use std::net::SocketAddr;
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
    pub koshelf_pool: SqlitePool,
    pub auth_state: Option<auth::AuthState>,
}

/// Axum-based HTTP server serving the API, embedded React frontend, and media assets.
pub struct WebServer {
    media_cache_dir: PathBuf,
    port: u16,
    site_store: SharedSiteStore,
    reading_data_store: SharedReadingDataStore,
    update_notifier: UpdateNotifier,
    library_repo: LibraryRepository,
    koshelf_pool: SqlitePool,
    auth_state: Option<auth::AuthState>,
}

pub struct WebServerOptions {
    pub media_cache_dir: PathBuf,
    pub port: u16,
    pub site_store: SharedSiteStore,
    pub reading_data_store: SharedReadingDataStore,
    pub update_notifier: UpdateNotifier,
    pub library_repo: LibraryRepository,
    pub koshelf_pool: SqlitePool,
    pub auth_state: Option<auth::AuthState>,
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
            koshelf_pool,
            auth_state,
        } = options;

        Self {
            media_cache_dir,
            port,
            site_store,
            reading_data_store,
            update_notifier,
            library_repo,
            koshelf_pool,
            auth_state,
        }
    }

    /// Start listening and serving requests. Blocks until the server shuts down.
    pub async fn run(self) -> Result<()> {
        let state = ServerState {
            site_store: self.site_store.clone(),
            reading_data_store: self.reading_data_store.clone(),
            update_notifier: self.update_notifier.clone(),
            library_repo: self.library_repo,
            koshelf_pool: self.koshelf_pool,
            auth_state: self.auth_state,
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
                .route(
                    "/api/auth/sessions",
                    get(auth::login::list_sessions_handler),
                )
                .route(
                    "/api/auth/sessions/{session_id}",
                    delete(auth::login::revoke_session),
                )
                .with_state(state.clone());
            app = app.merge(auth_routes);
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

        if state.auth_state.is_some() {
            info!("Authentication enabled");
        }

        info!(
            "Web server running on http://localhost:{}, binding to: 0.0.0.0",
            self.port
        );

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;

        Ok(())
    }
}
