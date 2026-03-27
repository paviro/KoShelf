use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::Json;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderMap, Method, StatusCode, header::COOKIE};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::server::ServerState;
use crate::server::api::responses::error::{ApiErrorCode, ApiErrorResponse};

use crate::server::auth::session::validate_token;
use crate::server::auth::{CurrentSessionId, SESSION_COOKIE_NAME};

pub async fn auth_middleware(
    State(state): State<ServerState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(auth_state) = state.auth_state.as_ref() else {
        return next.run(request).await;
    };

    let path = request.uri().path().to_string();
    let method = request.method().clone();

    if !is_protected_path(&method, &path) {
        return next.run(request).await;
    }

    let Some(token) = cookie_value(request.headers(), SESSION_COOKIE_NAME) else {
        return unauthorized_response();
    };

    let peer_ip = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|connect_info| connect_info.0.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

    let client_context = auth_state
        .client_addr_resolver
        .resolve(request.headers(), peer_ip);

    match validate_token(
        auth_state.token_key.as_ref(),
        &auth_state.pool,
        &token,
        client_context.client_ip,
    )
    .await
    {
        Ok(Some(session_id)) => {
            request
                .extensions_mut()
                .insert(CurrentSessionId(session_id));
            next.run(request).await
        }
        Ok(None) | Err(_) => unauthorized_response(),
    }
}

fn is_protected_path(method: &Method, path: &str) -> bool {
    if path.starts_with("/api/") {
        if method == Method::GET && path == "/api/site" {
            return false;
        }
        if method == Method::POST && path == "/api/auth/login" {
            return false;
        }
        return true;
    }

    path == "/assets" || path.starts_with("/assets/")
}

pub(crate) fn cookie_value(headers: &HeaderMap, key: &str) -> Option<String> {
    let raw = headers.get(COOKIE)?.to_str().ok()?;

    for part in raw.split(';') {
        let trimmed = part.trim();
        let Some((name, value)) = trimmed.split_once('=') else {
            continue;
        };

        if name.trim() == key {
            return Some(value.trim().to_string());
        }
    }

    None
}

fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(ApiErrorResponse::from_code(ApiErrorCode::Unauthorized)),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::is_protected_path;
    use axum::http::Method;

    #[test]
    fn public_site_and_login_routes_are_unprotected() {
        assert!(!is_protected_path(&Method::GET, "/api/site"));
        assert!(!is_protected_path(&Method::POST, "/api/auth/login"));
    }

    #[test]
    fn api_and_assets_routes_are_protected() {
        assert!(is_protected_path(&Method::GET, "/api/items"));
        assert!(is_protected_path(&Method::GET, "/assets/covers/abc.webp"));
    }
}
