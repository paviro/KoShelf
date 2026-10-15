//! Explicit proxy authentication, request prefixes and browser write protection.
//! Identity trust is deliberately independent of forwarded-address trust.
use crate::server::ServerState;
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Clone, Default)]
pub struct RequestContext {
    pub proxy_user: Option<String>,
    pub prefix: String,
}

fn single_header<'a>(headers: &'a HeaderMap, name: &str) -> Result<Option<&'a str>, ()> {
    let mut values = headers.get_all(name).iter();
    let value = values
        .next()
        .map(|v| v.to_str().map_err(|_| ()))
        .transpose()?;
    if values.next().is_some() {
        return Err(());
    }
    Ok(value)
}

pub fn valid_prefix(value: &str) -> bool {
    value.is_empty()
        || (value.starts_with('/')
            && !value.ends_with('/')
            && value.split('/').skip(1).all(|part| {
                !part.is_empty()
                    && part != "."
                    && part != ".."
                    && part
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
            }))
}

fn resolve_context(headers: &HeaderMap, trusted: bool) -> Result<RequestContext, ()> {
    let user = single_header(headers, "x-koshelf-user")?;
    let prefix = single_header(headers, "x-forwarded-prefix")?.unwrap_or("");
    if (user.is_some() || !prefix.is_empty()) && !trusted {
        return Err(());
    }
    if !valid_prefix(prefix) {
        return Err(());
    }
    if let Some(user) = user
        && (user.is_empty()
            || user.len() > 256
            || !user
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"@._-".contains(&b)))
    {
        return Err(());
    }
    Ok(RequestContext {
        proxy_user: user.map(str::to_owned),
        prefix: prefix.to_owned(),
    })
}

fn valid_write(headers: &HeaderMap) -> bool {
    // A custom header forces a CORS preflight, which KoShelf never approves, so
    // cross-origin pages cannot send it. Fetch Metadata adds a check where
    // browsers supply it. Origin is not compared with Host because reverse
    // proxies commonly rewrite Host and terminate TLS.
    single_header(headers, "x-koshelf-request") == Ok(Some("1"))
        && matches!(
            single_header(headers, "sec-fetch-site"),
            Ok(None | Some("same-origin"))
        )
}

pub async fn request_context(
    State(state): State<ServerState>,
    mut request: Request,
    next: Next,
) -> Response {
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|v| v.0.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    let trusted = state.auth_proxy_peers.iter().any(|net| net.contains(&peer));
    let Ok(context) = resolve_context(request.headers(), trusted) else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let proxy = context.proxy_user.is_some();
    // Never turn proxy authentication into a portable password session.
    if proxy && request.uri().path().starts_with("/api/auth/") {
        return StatusCode::FORBIDDEN.into_response();
    }
    if !matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    ) && !valid_write(request.headers())
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    request.extensions_mut().insert(context);
    let mut response = next.run(request).await;
    if proxy {
        response.headers_mut().remove("set-cookie");
        response
            .headers_mut()
            .insert("x-frame-options", "SAMEORIGIN".parse().unwrap());
        response.headers_mut().insert(
            "content-security-policy",
            "frame-ancestors 'self'".parse().unwrap(),
        );
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_requires_explicit_peer_and_rejects_duplicates() {
        let mut h = HeaderMap::new();
        h.insert("x-koshelf-user", "alice".parse().unwrap());
        assert!(resolve_context(&h, false).is_err());
        assert_eq!(
            resolve_context(&h, true).unwrap().proxy_user.as_deref(),
            Some("alice")
        );
        h.append("x-koshelf-user", "bob".parse().unwrap());
        assert!(resolve_context(&h, true).is_err());
    }
    #[test]
    fn prefixes_cannot_escape_origin_or_inject_html() {
        for bad in [
            "//evil",
            "/a/../b",
            "/a%2fb",
            "/a/",
            "/a?b",
            "/a\\b",
            "/<script>",
        ] {
            assert!(!valid_prefix(bad), "{bad}");
        }
        assert!(valid_prefix("/api/hassio_ingress/abc-123_456"));
    }
    #[test]
    fn writes_require_custom_header_and_same_origin_fetch() {
        let mut h = HeaderMap::new();
        assert!(!valid_write(&h));
        h.insert("x-koshelf-request", "1".parse().unwrap());
        assert!(valid_write(&h)); // non-browser clients
        h.insert("sec-fetch-site", "same-origin".parse().unwrap());
        assert!(valid_write(&h));
        h.insert("sec-fetch-site", "cross-site".parse().unwrap());
        assert!(!valid_write(&h));
        h.insert("sec-fetch-site", "same-site".parse().unwrap());
        assert!(!valid_write(&h));
    }
}
