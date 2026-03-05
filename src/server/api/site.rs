use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::contracts::common::ApiMeta;
use crate::contracts::site::{SiteCapabilities, SiteResponse};
use crate::server::ServerState;

use super::shared::{fallback_meta, runtime_snapshot};

fn empty_site_response(meta: ApiMeta) -> SiteResponse {
    SiteResponse {
        meta,
        title: String::new(),
        language: "en_US".to_string(),
        capabilities: SiteCapabilities {
            has_books: false,
            has_comics: false,
            has_activity: false,
            has_completions: false,
        },
    }
}

pub async fn site(State(state): State<ServerState>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let payload = snapshot
        .site
        .as_ref()
        .cloned()
        .unwrap_or_else(|| empty_site_response(fallback_meta(&snapshot)));

    (StatusCode::OK, Json(payload)).into_response()
}
