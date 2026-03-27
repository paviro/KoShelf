//! REST API layer — handlers, error types, extractors, query params, and response types.

pub mod error;
pub(crate) mod extractors;
pub(crate) mod handlers;
pub(crate) mod params;
pub mod responses;

use crate::server::ServerState;
use axum::{Router, routing::get};

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/api/site", get(handlers::site))
        .route("/api/items", get(handlers::items))
        .route("/api/items/{id}", get(handlers::item_detail))
        .route(
            "/api/items/{id}/page-activity",
            get(handlers::item_page_activity),
        )
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

/// Canonical list of all API route paths (test-only).
///
/// Used by `pipeline::export::tests` to verify every data endpoint has a
/// static-export implementation. Keep in sync with `routes()` above.
#[cfg(test)]
pub fn route_paths() -> &'static [&'static str] {
    &[
        "/api/site",
        "/api/items",
        "/api/items/{id}",
        "/api/items/{id}/page-activity",
        "/api/reading/summary",
        "/api/reading/metrics",
        "/api/reading/available-periods",
        "/api/reading/calendar",
        "/api/reading/completions",
        "/api/events/stream",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Counts `.route(` calls in the non-test source of this file and
    /// verifies the count matches `route_paths()`. If someone adds a
    /// `.route()` call without updating `route_paths()`, this fails.
    #[test]
    fn route_paths_matches_router() {
        let source = include_str!("mod.rs");
        // Only inspect source above the test module.
        let non_test = source.split("#[cfg(test)]").next().unwrap_or(source);
        let router_count = non_test.matches(".route(").count();

        assert_eq!(
            route_paths().len(),
            router_count,
            "route_paths() has {} entries but routes() has {} .route() calls. \
             Update route_paths() when adding or removing routes.",
            route_paths().len(),
            router_count,
        );
    }
}
