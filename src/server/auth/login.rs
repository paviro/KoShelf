use axum::Json;
use axum::extract::{ConnectInfo, Extension, Path, State};
use axum::http::{HeaderMap, StatusCode, header::HOST, header::RETRY_AFTER, header::SET_COOKIE};
use axum::response::{IntoResponse, Response};
use governor::clock::{Clock, DefaultClock};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};

use crate::server::ServerState;
use crate::server::api::responses::common::ApiResponse;
use crate::server::api::responses::error::{ApiErrorCode, ApiErrorResponse};

use super::password::{
    get_stored_auth, hash_password, set_password_hash_and_revoke_sessions,
    validate_password_length, verify_password,
};
use super::session::{SessionInfo, create_token, delete_session, list_sessions};
use super::{CurrentSessionId, SESSION_COOKIE_NAME};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize)]
struct AuthOk {
    ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionInfoResponse {
    pub id: String,
    pub user_agent: Option<String>,
    pub browser: String,
    pub os: String,
    pub last_seen_ip: Option<String>,
    pub created_at: String,
    pub last_seen_at: String,
    pub is_current: bool,
}

pub async fn login_submit(
    State(state): State<ServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Response {
    let Some(auth_state) = state.auth_state.as_ref() else {
        return auth_not_configured_response();
    };

    let client_context = auth_state.client_addr_resolver.resolve(&headers, addr.ip());
    let secure_cookie = should_use_secure_cookie(
        client_context.is_https,
        addr.ip(),
        headers.get(HOST).and_then(|value| value.to_str().ok()),
    );

    if let Err(not_until) = auth_state
        .login_limiter
        .check_key(&client_context.client_ip)
    {
        let retry_after = not_until
            .wait_time_from(DefaultClock::default().now())
            .as_secs()
            .max(1);

        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(RETRY_AFTER, retry_after.to_string())],
            Json(ApiErrorResponse::from_code(ApiErrorCode::RateLimited)),
        )
            .into_response();
    }

    let auth_row = match get_stored_auth(&auth_state.pool).await {
        Ok(value) => value,
        Err(_) => return internal_error_response(),
    };

    let Some((stored_hash, _stored_key)) = auth_row else {
        return api_error_response(StatusCode::UNAUTHORIZED, ApiErrorCode::InvalidCredentials);
    };

    if !verify_password(&payload.password, &stored_hash) {
        return api_error_response(StatusCode::UNAUTHORIZED, ApiErrorCode::InvalidCredentials);
    }

    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok());
    let token = match create_token(
        auth_state.token_key.as_ref(),
        &auth_state.pool,
        user_agent,
        client_context.client_ip,
    )
    .await
    {
        Ok(token) => token,
        Err(_) => return internal_error_response(),
    };

    (
        StatusCode::OK,
        [(SET_COOKIE, build_session_cookie(&token, secure_cookie))],
        Json(ApiResponse::new(AuthOk { ok: true })),
    )
        .into_response()
}

pub async fn logout(
    State(state): State<ServerState>,
    Extension(current_session): Extension<CurrentSessionId>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    let Some(auth_state) = state.auth_state.as_ref() else {
        return auth_not_configured_response();
    };

    let _ = delete_session(&auth_state.pool, &current_session.0).await;

    let client_context = auth_state.client_addr_resolver.resolve(&headers, addr.ip());
    let secure_cookie = should_use_secure_cookie(
        client_context.is_https,
        addr.ip(),
        headers.get(HOST).and_then(|value| value.to_str().ok()),
    );

    (
        StatusCode::OK,
        [(SET_COOKIE, build_cleared_cookie(secure_cookie))],
        Json(ApiResponse::new(AuthOk { ok: true })),
    )
        .into_response()
}

pub async fn change_password(
    State(state): State<ServerState>,
    Extension(current_session): Extension<CurrentSessionId>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> Response {
    let Some(auth_state) = state.auth_state.as_ref() else {
        return auth_not_configured_response();
    };

    let client_context = auth_state.client_addr_resolver.resolve(&headers, addr.ip());
    if let Err(not_until) = auth_state
        .login_limiter
        .check_key(&client_context.client_ip)
    {
        let retry_after = not_until
            .wait_time_from(DefaultClock::default().now())
            .as_secs()
            .max(1);

        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(RETRY_AFTER, retry_after.to_string())],
            Json(ApiErrorResponse::from_code(ApiErrorCode::RateLimited)),
        )
            .into_response();
    }

    if let Err(error) = validate_password_length(&payload.new_password) {
        return api_error_message_response(
            StatusCode::BAD_REQUEST,
            ApiErrorCode::InvalidQuery,
            error.to_string(),
        );
    }

    let auth_row = match get_stored_auth(&auth_state.pool).await {
        Ok(value) => value,
        Err(_) => return internal_error_response(),
    };

    let Some((stored_hash, _stored_key)) = auth_row else {
        return api_error_response(StatusCode::BAD_REQUEST, ApiErrorCode::InvalidCredentials);
    };

    if !verify_password(&payload.current_password, &stored_hash) {
        return api_error_response(StatusCode::BAD_REQUEST, ApiErrorCode::InvalidCredentials);
    }

    let new_hash = match hash_password(&payload.new_password) {
        Ok(hash) => hash,
        Err(error) => {
            return api_error_message_response(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::InvalidQuery,
                error.to_string(),
            );
        }
    };

    if set_password_hash_and_revoke_sessions(&auth_state.pool, &new_hash, Some(&current_session.0))
        .await
        .is_err()
    {
        return internal_error_response();
    }

    (StatusCode::OK, Json(ApiResponse::new(AuthOk { ok: true }))).into_response()
}

pub async fn list_sessions_handler(
    State(state): State<ServerState>,
    Extension(current_session): Extension<CurrentSessionId>,
) -> Response {
    let Some(auth_state) = state.auth_state.as_ref() else {
        return auth_not_configured_response();
    };

    let sessions = match list_sessions(&auth_state.pool).await {
        Ok(sessions) => sessions,
        Err(_) => return internal_error_response(),
    };

    let payload: Vec<SessionInfoResponse> = sessions
        .into_iter()
        .map(|session| to_session_response(session, &current_session.0))
        .collect();

    (StatusCode::OK, Json(ApiResponse::new(payload))).into_response()
}

pub async fn revoke_session(
    State(state): State<ServerState>,
    Extension(current_session): Extension<CurrentSessionId>,
    Path(session_id): Path<String>,
) -> Response {
    let Some(auth_state) = state.auth_state.as_ref() else {
        return auth_not_configured_response();
    };

    if session_id == current_session.0 {
        return api_error_message_response(
            StatusCode::BAD_REQUEST,
            ApiErrorCode::InvalidQuery,
            "Cannot revoke the current session. Use logout instead.",
        );
    }

    match delete_session(&auth_state.pool, &session_id).await {
        Ok(true) => (StatusCode::OK, Json(ApiResponse::new(AuthOk { ok: true }))).into_response(),
        Ok(false) => api_error_response(StatusCode::NOT_FOUND, ApiErrorCode::NotFound),
        Err(_) => internal_error_response(),
    }
}

fn auth_not_configured_response() -> Response {
    api_error_response(StatusCode::NOT_FOUND, ApiErrorCode::NotFound)
}

fn internal_error_response() -> Response {
    api_error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        ApiErrorCode::InternalServerError,
    )
}

fn api_error_response(status: StatusCode, code: ApiErrorCode) -> Response {
    (status, Json(ApiErrorResponse::from_code(code))).into_response()
}

fn api_error_message_response(
    status: StatusCode,
    code: ApiErrorCode,
    message: impl Into<String>,
) -> Response {
    (status, Json(ApiErrorResponse::new(code, message.into()))).into_response()
}

fn to_session_response(session: SessionInfo, current_session_id: &str) -> SessionInfoResponse {
    SessionInfoResponse {
        is_current: session.id == current_session_id,
        id: session.id,
        user_agent: session.user_agent,
        browser: session.browser,
        os: session.os,
        last_seen_ip: session.last_seen_ip,
        created_at: session.created_at,
        last_seen_at: session.last_seen_at,
    }
}

fn should_use_secure_cookie(
    request_is_https: bool,
    peer_ip: IpAddr,
    host_header: Option<&str>,
) -> bool {
    if request_is_https {
        return true;
    }

    if peer_ip.is_loopback() && host_header.is_some_and(is_localhost_host_header) {
        return false;
    }

    true
}

fn is_localhost_host_header(raw_host: &str) -> bool {
    let host = raw_host.trim();

    if let Some(stripped) = host.strip_prefix('[')
        && let Some((ipv6_host, _)) = stripped.split_once(']')
    {
        return ipv6_host == "::1";
    }

    let host_without_port = host.split(':').next().unwrap_or(host);
    host_without_port.eq_ignore_ascii_case("localhost")
        || host_without_port == "127.0.0.1"
        || host_without_port == "::1"
}

fn build_session_cookie(token: &str, secure: bool) -> String {
    let mut value = format!(
        "{SESSION_COOKIE_NAME}={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age=2592000"
    );
    if secure {
        value.push_str("; Secure");
    }
    value
}

fn build_cleared_cookie(secure: bool) -> String {
    let mut value = format!("{SESSION_COOKIE_NAME}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0");
    if secure {
        value.push_str("; Secure");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::should_use_secure_cookie;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn secure_cookie_is_forced_for_https_requests() {
        let should_secure = should_use_secure_cookie(
            true,
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            Some("localhost:3000"),
        );
        assert!(should_secure);
    }

    #[test]
    fn secure_cookie_is_disabled_for_local_http_dev() {
        let should_secure = should_use_secure_cookie(
            false,
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            Some("localhost:3000"),
        );
        assert!(!should_secure);
    }

    #[test]
    fn secure_cookie_is_forced_for_non_local_hosts() {
        let should_secure = should_use_secure_cookie(
            false,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
            Some("books.example.com"),
        );
        assert!(should_secure);
    }
}
