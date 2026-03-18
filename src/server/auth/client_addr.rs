use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::extract::ConnectInfo;
use axum::http::HeaderMap;
use axum::http::request::Parts;
use ipnet::IpNet;
use log::{debug, warn};

use crate::server::ServerState;

#[derive(Debug, Clone, Copy)]
pub struct ClientContext {
    pub client_ip: IpAddr,
    pub peer_ip: IpAddr,
    pub is_https: bool,
}

impl axum::extract::FromRequestParts<ServerState> for ClientContext {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let peer_ip = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0.ip())
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

        let context = match state.auth_state.as_ref() {
            Some(auth_state) => {
                auth_state
                    .client_addr_resolver
                    .resolve(&parts.headers, peer_ip)
            }
            None => ClientContext {
                client_ip: peer_ip,
                peer_ip,
                is_https: false,
            },
        };

        Ok(context)
    }
}

#[derive(Debug, Clone)]
pub struct ClientAddrResolver {
    trusted_proxies: Vec<IpNet>,
}

impl ClientAddrResolver {
    pub fn new(trusted_proxies: Vec<IpNet>) -> Self {
        Self { trusted_proxies }
    }

    pub fn resolve(&self, headers: &HeaderMap, connect_info_ip: IpAddr) -> ClientContext {
        if !self.is_trusted_proxy(connect_info_ip) {
            return ClientContext {
                client_ip: connect_info_ip,
                peer_ip: connect_info_ip,
                is_https: false,
            };
        }

        let forwarded_ips = match self.resolve_forwarded_ips(headers) {
            Ok(ips) => ips,
            Err(error) => {
                warn!("Ignoring forwarded headers due to parse error: {}", error);
                return ClientContext {
                    client_ip: connect_info_ip,
                    peer_ip: connect_info_ip,
                    is_https: false,
                };
            }
        };

        let is_https = match self.resolve_https(headers) {
            Ok(value) => value,
            Err(error) => {
                debug!("Failed to parse forwarded proto header: {}", error);
                false
            }
        };

        let mut chain = forwarded_ips;
        chain.push(connect_info_ip);

        let client_ip = chain
            .iter()
            .rev()
            .copied()
            .find(|ip| !self.is_trusted_proxy(*ip))
            .or_else(|| chain.first().copied())
            .unwrap_or(connect_info_ip);

        ClientContext {
            client_ip,
            peer_ip: connect_info_ip,
            is_https,
        }
    }

    fn is_trusted_proxy(&self, ip: IpAddr) -> bool {
        self.trusted_proxies.iter().any(|entry| entry.contains(&ip))
    }

    fn resolve_forwarded_ips(&self, headers: &HeaderMap) -> Result<Vec<IpAddr>, String> {
        if let Some(raw_forwarded) = header_value(headers, "forwarded")? {
            return parse_forwarded_for_chain(raw_forwarded);
        }

        if let Some(raw_xff) = header_value(headers, "x-forwarded-for")? {
            return parse_x_forwarded_for_chain(raw_xff);
        }

        Ok(Vec::new())
    }

    fn resolve_https(&self, headers: &HeaderMap) -> Result<bool, String> {
        if let Some(raw_forwarded) = header_value(headers, "forwarded")?
            && let Some(proto) = parse_forwarded_proto(raw_forwarded)?
        {
            return Ok(proto.eq_ignore_ascii_case("https"));
        }

        if let Some(raw_proto) = header_value(headers, "x-forwarded-proto")?
            && let Some(proto) = parse_x_forwarded_proto(raw_proto)?
        {
            return Ok(proto.eq_ignore_ascii_case("https"));
        }

        Ok(false)
    }
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Result<Option<&'a str>, String> {
    let Some(value) = headers.get(name) else {
        return Ok(None);
    };

    value
        .to_str()
        .map(Some)
        .map_err(|_| format!("invalid {name} header encoding"))
}

fn parse_forwarded_for_chain(raw: &str) -> Result<Vec<IpAddr>, String> {
    let mut out = Vec::new();

    for entry in raw.split(',') {
        for part in entry.split(';') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            let Some((key, value)) = trimmed.split_once('=') else {
                return Err("malformed Forwarded parameter".to_string());
            };

            if !key.trim().eq_ignore_ascii_case("for") {
                continue;
            }

            let value = value.trim().trim_matches('"');
            if value.eq_ignore_ascii_case("unknown") {
                continue;
            }

            let Some(ip) = parse_ip_token(value) else {
                return Err(format!("unable to parse Forwarded for={value}"));
            };
            out.push(ip);
        }
    }

    Ok(out)
}

fn parse_forwarded_proto(raw: &str) -> Result<Option<&str>, String> {
    for entry in raw.split(',') {
        for part in entry.split(';') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            let Some((key, value)) = trimmed.split_once('=') else {
                return Err("malformed Forwarded parameter".to_string());
            };

            if key.trim().eq_ignore_ascii_case("proto") {
                let proto = value.trim().trim_matches('"');
                return Ok(Some(proto));
            }
        }
    }

    Ok(None)
}

fn parse_x_forwarded_for_chain(raw: &str) -> Result<Vec<IpAddr>, String> {
    let mut out = Vec::new();

    for token in raw.split(',') {
        let value = token.trim();
        if value.is_empty() || value.eq_ignore_ascii_case("unknown") {
            continue;
        }

        let Some(ip) = parse_ip_token(value) else {
            return Err(format!("unable to parse X-Forwarded-For entry: {value}"));
        };
        out.push(ip);
    }

    Ok(out)
}

fn parse_x_forwarded_proto(raw: &str) -> Result<Option<&str>, String> {
    let Some(first) = raw.split(',').next() else {
        return Ok(None);
    };

    let proto = first.trim();
    if proto.is_empty() {
        return Ok(None);
    }

    if proto.eq_ignore_ascii_case("http") || proto.eq_ignore_ascii_case("https") {
        Ok(Some(proto))
    } else {
        Err(format!("invalid X-Forwarded-Proto value: {proto}"))
    }
}

fn parse_ip_token(raw: &str) -> Option<IpAddr> {
    let token = raw.trim().trim_matches('"');

    if let Some(stripped) = token.strip_prefix('[')
        && let Some((host, _)) = stripped.split_once(']')
        && let Ok(ip) = host.parse::<IpAddr>()
    {
        return Some(ip);
    }

    if let Ok(ip) = token.parse::<IpAddr>() {
        return Some(ip);
    }

    if let Ok(sock) = token.parse::<SocketAddr>() {
        return Some(sock.ip());
    }

    if let Some((host, _port)) = token.rsplit_once(':')
        && !host.contains(':')
        && let Ok(ip) = host.parse::<IpAddr>()
    {
        return Some(ip);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::ClientAddrResolver;
    use axum::http::HeaderMap;
    use ipnet::IpNet;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn untrusted_proxy_ignores_forwarded_headers() {
        let resolver = ClientAddrResolver::new(vec![]);
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "203.0.113.5".parse().expect("header value"),
        );
        headers.insert("x-forwarded-proto", "https".parse().expect("header value"));

        let context = resolver.resolve(&headers, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)));
        assert_eq!(context.client_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)));
        assert!(!context.is_https);
    }

    #[test]
    fn trusted_proxy_uses_forwarded_chain_and_proto() {
        let resolver = ClientAddrResolver::new(vec![
            "10.0.0.0/8"
                .parse::<IpNet>()
                .expect("trusted network should parse"),
        ]);

        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "198.51.100.42, 10.2.3.4"
                .parse()
                .expect("header value should parse"),
        );
        headers.insert(
            "x-forwarded-proto",
            "https".parse().expect("header value should parse"),
        );

        let context = resolver.resolve(&headers, IpAddr::V4(Ipv4Addr::new(10, 1, 1, 1)));
        assert_eq!(
            context.client_ip,
            IpAddr::V4(Ipv4Addr::new(198, 51, 100, 42))
        );
        assert!(context.is_https);
    }
}
