use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::api::responses::common::ApiResponse;
use crate::api::responses::site::{SiteCapabilities, SiteData};
use crate::api::server::ServerState;

fn empty_site_data() -> SiteData {
    SiteData {
        title: String::new(),
        language: "en_US".to_string(),
        capabilities: SiteCapabilities::default(),
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
