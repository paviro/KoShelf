use std::net::{IpAddr, Ipv4Addr};

use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::server::ServerState;
use crate::server::api::responses::common::ApiResponse;
use crate::server::api::responses::site::SiteData;
use crate::server::auth::SESSION_COOKIE_NAME;
use crate::server::auth::middleware::cookie_value;
use crate::server::auth::session::validate_token;

pub(crate) async fn site(State(state): State<ServerState>, request: Request) -> Response {
    let mut data = state
        .site_store
        .get()
        .map(|arc| (*arc).clone())
        .unwrap_or_else(SiteData::default);

    if let Some(auth_state) = state.auth_state.as_ref() {
        let is_authenticated = match cookie_value(&request, SESSION_COOKIE_NAME) {
            Some(token) => validate_token(
                auth_state.token_key.as_ref(),
                &auth_state.pool,
                &token,
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            )
            .await
            .unwrap_or(None)
            .is_some(),
            None => false,
        };
        data.authenticated = Some(is_authenticated);
    }

    (StatusCode::OK, Json(ApiResponse::new(data))).into_response()
}
