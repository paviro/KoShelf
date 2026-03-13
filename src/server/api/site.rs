use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::contracts::common::ApiResponse;
use crate::contracts::site::{SiteCapabilities, SiteData};
use crate::server::ServerState;

fn empty_site_data() -> SiteData {
    SiteData {
        title: String::new(),
        language: "en_US".to_string(),
        capabilities: SiteCapabilities {
            has_books: false,
            has_comics: false,
            has_reading_data: false,
        },
    }
}

pub(crate) async fn site(State(state): State<ServerState>) -> Response {
    let data = state
        .site_store
        .get()
        .map(|arc| (*arc).clone())
        .unwrap_or_else(empty_site_data);

    (StatusCode::OK, Json(ApiResponse::new(data))).into_response()
}
