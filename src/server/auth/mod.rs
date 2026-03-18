use std::net::IpAddr;
use std::sync::Arc;

use rusty_paseto::core::{Local, PasetoSymmetricKey, V4};
use sqlx::SqlitePool;

pub mod client_addr;
pub mod login;
pub mod middleware;
pub mod password;
pub mod rate_limit;
pub mod session;

pub const SESSION_COOKIE_NAME: &str = "koshelf_session";

pub type LoginRateLimiter = governor::DefaultKeyedRateLimiter<IpAddr>;

#[derive(Clone)]
pub struct AuthState {
    pub pool: SqlitePool,
    pub token_key: Arc<PasetoSymmetricKey<V4, Local>>,
    pub login_limiter: Arc<LoginRateLimiter>,
    pub client_addr_resolver: Arc<client_addr::ClientAddrResolver>,
}

#[derive(Debug, Clone)]
pub struct CurrentSessionId(pub String);
