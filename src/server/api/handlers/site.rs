use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::server::ServerState;
use crate::server::api::responses::common::ApiResponse;
use crate::server::api::responses::site::SiteData;

pub(crate) async fn site(State(state): State<ServerState>) -> Response {
    let data = state
        .site_store
        .get()
        .map(|arc| (*arc).clone())
        .unwrap_or_else(SiteData::default);

    (StatusCode::OK, Json(ApiResponse::new(data))).into_response()
}
