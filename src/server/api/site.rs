use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::contracts::common::ApiMeta;
use crate::contracts::site::{SiteCapabilities, SiteResponse};
use crate::server::ServerState;

fn empty_site_response() -> SiteResponse {
    SiteResponse {
        meta: ApiMeta {
            version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: String::new(),
        },
        title: String::new(),
        language: "en_US".to_string(),
        capabilities: SiteCapabilities {
            has_books: false,
            has_comics: false,
            has_reading_data: false,
        },
    }
}

pub async fn site(State(state): State<ServerState>) -> Response {
    let payload = state
        .site_store
        .get()
        .map(|arc| (*arc).clone())
        .unwrap_or_else(empty_site_response);

    (StatusCode::OK, Json(payload)).into_response()
}
