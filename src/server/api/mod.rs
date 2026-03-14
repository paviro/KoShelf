//! REST API layer — handlers, error types, extractors, query params, and response types.

pub mod error;
pub(crate) mod extractors;
pub(crate) mod handlers;
pub(crate) mod params;
pub mod responses;

use super::ServerState;
use axum::{Router, routing::get};

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/api/site", get(handlers::site))
        .route("/api/items", get(handlers::items))
        .route("/api/items/{id}", get(handlers::item_detail))
        .route("/api/reading/summary", get(handlers::reading_summary))
        .route("/api/reading/metrics", get(handlers::reading_metrics))
        .route(
            "/api/reading/available-periods",
            get(handlers::reading_available_periods),
        )
        .route("/api/reading/calendar", get(handlers::reading_calendar))
        .route(
            "/api/reading/completions",
            get(handlers::reading_completions),
        )
        .route("/api/events/stream", get(handlers::events_stream))
}
