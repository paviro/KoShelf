use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::CACHE_CONTROL},
    response::{IntoResponse, Response},
};

use crate::server::ServerState;
use crate::server::api::responses::common::ApiResponse;
use crate::server::api::responses::site::SiteData;
use crate::server::auth::SESSION_COOKIE_NAME;
use crate::server::auth::client_addr::ClientContext;
use crate::server::auth::middleware::cookie_value;
use crate::server::auth::session::validate_token;

pub(crate) async fn site(
    State(state): State<ServerState>,
    client_context: ClientContext,
    headers: HeaderMap,
) -> Response {
    let mut data = state
        .site_store
        .get()
        .map(|arc| (*arc).clone())
        .unwrap_or_else(SiteData::default);

    if let Some(auth_state) = state.auth_state.as_ref() {
        let is_authenticated = match cookie_value(&headers, SESSION_COOKIE_NAME) {
            Some(token) => validate_token(
                auth_state.token_key.as_ref(),
                &auth_state.pool,
                &token,
                client_context.client_ip,
            )
            .await
            .unwrap_or(None)
            .is_some(),
            None => false,
        };
        if let Some(auth) = data.auth.as_mut() {
            auth.authenticated = is_authenticated;
        }
    }

    (
        StatusCode::OK,
        [(CACHE_CONTROL, "private, no-store")],
        Json(ApiResponse::new(data)),
    )
        .into_response()
}
