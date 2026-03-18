use std::convert::TryFrom;
use std::net::IpAddr;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use rusty_paseto::core::{Key, Local, PasetoSymmetricKey, V4};
use rusty_paseto::prelude::{
    CustomClaim, ExpirationClaim, PasetoBuilder, PasetoParser, SubjectClaim,
};
use serde::Serialize;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

const SESSION_TTL_SECONDS: i64 = 30 * 24 * 60 * 60;
const LAST_SEEN_UPDATE_WINDOW_SECONDS: i64 = 60;

#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub user_agent: Option<String>,
    pub browser: String,
    pub os: String,
    pub last_seen_ip: Option<String>,
    pub created_at: String,
    pub last_seen_at: String,
}

pub fn paseto_key_from_bytes(raw_key: &[u8]) -> Result<PasetoSymmetricKey<V4, Local>> {
    if raw_key.len() != 32 {
        anyhow::bail!(
            "Invalid token key length: expected 32 bytes, got {}",
            raw_key.len()
        );
    }

    let key = Key::<32>::from(raw_key);
    Ok(PasetoSymmetricKey::<V4, Local>::from(key))
}

pub async fn create_token(
    key: &PasetoSymmetricKey<V4, Local>,
    pool: &SqlitePool,
    user_agent: Option<&str>,
    ip: IpAddr,
) -> Result<String> {
    let session_id = Uuid::now_v7().to_string();
    let expires_at = Utc::now() + Duration::seconds(SESSION_TTL_SECONDS);
    let expires_at_unix = expires_at.timestamp();
    let expiration = expires_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    let exp_claim = ExpirationClaim::try_from(expiration.as_str())
        .map_err(|error| anyhow::anyhow!("Failed to build token expiration claim: {error}"))?;
    let session_claim = CustomClaim::try_from(("session_id", session_id.clone()))
        .map_err(|error| anyhow::anyhow!("Failed to build session_id token claim: {error}"))?;

    let token = {
        let mut builder = PasetoBuilder::<V4, Local>::default();
        builder
            .set_claim(exp_claim)
            .set_claim(SubjectClaim::from("user"))
            .set_claim(session_claim)
            .build(key)
            .map_err(|error| anyhow::anyhow!("Failed to build session token: {error}"))?
    };

    sqlx::query(
        "INSERT INTO sessions (id, user_agent, last_seen_ip, last_seen_at, expires_at_unix)
         VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?4)",
    )
    .bind(&session_id)
    .bind(user_agent)
    .bind(ip.to_string())
    .bind(expires_at_unix)
    .execute(pool)
    .await
    .context("Failed to insert session row")?;

    Ok(token)
}

pub async fn validate_token(
    key: &PasetoSymmetricKey<V4, Local>,
    pool: &SqlitePool,
    token_str: &str,
    ip: Option<IpAddr>,
) -> Result<Option<String>> {
    let claims = {
        let mut parser = PasetoParser::<V4, Local>::default();
        match parser.parse(token_str, key) {
            Ok(claims) => claims,
            Err(_) => return Ok(None),
        }
    };

    if claims.get("sub").and_then(|v| v.as_str()) != Some("user") {
        return Ok(None);
    }

    if claims.get("exp").and_then(|v| v.as_str()).is_none() {
        return Ok(None);
    }

    let Some(session_id) = claims
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned)
    else {
        return Ok(None);
    };

    let now_unix = Utc::now().timestamp();

    let row = sqlx::query(
        "SELECT last_seen_ip, last_seen_at FROM sessions WHERE id = ?1 AND expires_at_unix > ?2",
    )
    .bind(&session_id)
    .bind(now_unix)
    .fetch_optional(pool)
    .await
    .context("Failed to query session row")?;

    let Some(row) = row else {
        return Ok(None);
    };

    if let Some(client_ip) = ip {
        let stored_ip: Option<String> = row.try_get("last_seen_ip").unwrap_or(None);
        let last_seen_at: String = row
            .try_get("last_seen_at")
            .context("Failed to decode session last_seen_at")?;

        if should_update_last_seen(stored_ip.as_deref(), client_ip, &last_seen_at) {
            sqlx::query(
                "UPDATE sessions
                 SET last_seen_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                     last_seen_ip = ?1
                 WHERE id = ?2",
            )
            .bind(client_ip.to_string())
            .bind(&session_id)
            .execute(pool)
            .await
            .context("Failed to update session activity")?;
        }
    }

    Ok(Some(session_id))
}

pub async fn list_sessions(pool: &SqlitePool) -> Result<Vec<SessionInfo>> {
    let rows = sqlx::query(
        "SELECT id, user_agent, last_seen_ip, last_seen_at
         FROM sessions
         WHERE expires_at_unix > ?1
         ORDER BY last_seen_at DESC",
    )
    .bind(Utc::now().timestamp())
    .fetch_all(pool)
    .await
    .context("Failed to list sessions")?;

    let mut sessions = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.try_get("id").context("Failed to decode session id")?;
        let user_agent: Option<String> = row
            .try_get("user_agent")
            .context("Failed to decode user agent")?;
        let last_seen_ip: Option<String> = row
            .try_get("last_seen_ip")
            .context("Failed to decode last seen IP")?;
        let last_seen_at: String = row
            .try_get("last_seen_at")
            .context("Failed to decode last seen timestamp")?;
        let created_at = created_at_from_uuid_v7(&id).unwrap_or_else(|| last_seen_at.clone());
        let (browser, os) = parse_user_agent(user_agent.as_deref());

        sessions.push(SessionInfo {
            id,
            user_agent,
            browser,
            os,
            last_seen_ip,
            created_at,
            last_seen_at,
        });
    }

    Ok(sessions)
}

pub async fn delete_session(pool: &SqlitePool, session_id: &str) -> Result<bool> {
    let result = sqlx::query("DELETE FROM sessions WHERE id = ?1")
        .bind(session_id)
        .execute(pool)
        .await
        .context("Failed to delete session")?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_all_sessions(pool: &SqlitePool) -> Result<()> {
    sqlx::query("DELETE FROM sessions")
        .execute(pool)
        .await
        .context("Failed to delete all sessions")?;
    Ok(())
}

pub async fn delete_other_sessions(pool: &SqlitePool, keep_session_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE id != ?1")
        .bind(keep_session_id)
        .execute(pool)
        .await
        .context("Failed to delete other sessions")?;
    Ok(())
}

pub async fn cleanup_expired(pool: &SqlitePool) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE expires_at_unix <= ?1")
        .bind(Utc::now().timestamp())
        .execute(pool)
        .await
        .context("Failed to clean up expired sessions")?;
    Ok(())
}

fn should_update_last_seen(
    stored_ip: Option<&str>,
    current_ip: IpAddr,
    last_seen_at: &str,
) -> bool {
    let current_ip_text = current_ip.to_string();
    if stored_ip != Some(current_ip_text.as_str()) {
        return true;
    }

    let Ok(parsed) = DateTime::parse_from_rfc3339(last_seen_at) else {
        return true;
    };

    let age = Utc::now().signed_duration_since(parsed.with_timezone(&Utc));
    age.num_seconds() >= LAST_SEEN_UPDATE_WINDOW_SECONDS
}

fn created_at_from_uuid_v7(session_id: &str) -> Option<String> {
    let uuid = Uuid::parse_str(session_id).ok()?;
    let timestamp = uuid.get_timestamp()?;
    let (seconds, nanos) = timestamp.to_unix();
    let dt = DateTime::<Utc>::from_timestamp(seconds as i64, nanos)?;
    Some(dt.to_rfc3339_opts(SecondsFormat::Secs, true))
}

fn parse_user_agent(user_agent: Option<&str>) -> (String, String) {
    let ua = user_agent.unwrap_or_default();

    let browser = if let Some(version) = extract_version(ua, "Firefox/") {
        format!("Firefox {version}")
    } else if let Some(version) = extract_version(ua, "Edg/") {
        format!("Edge {version}")
    } else if let Some(version) = extract_version(ua, "Chrome/") {
        format!("Chrome {version}")
    } else if let Some(version) = extract_version(ua, "Version/")
        && ua.contains("Safari/")
    {
        format!("Safari {version}")
    } else {
        "Unknown browser".to_string()
    };

    let os = if ua.contains("Windows") {
        "Windows".to_string()
    } else if ua.contains("Macintosh") || ua.contains("Mac OS X") {
        "macOS".to_string()
    } else if ua.contains("Android") {
        "Android".to_string()
    } else if ua.contains("iPhone") || ua.contains("iPad") {
        "iOS".to_string()
    } else if ua.contains("Linux") {
        "Linux".to_string()
    } else {
        "Unknown OS".to_string()
    };

    (browser, os)
}

fn extract_version(ua: &str, marker: &str) -> Option<String> {
    let start = ua.find(marker)? + marker.len();
    let suffix = &ua[start..];
    let version: String = suffix
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    if version.is_empty() {
        None
    } else {
        version
            .split('.')
            .next()
            .map(str::to_string)
            .or(Some(version))
    }
}
