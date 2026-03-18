use anyhow::{Context, Result};
use argon2::password_hash::{PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rusty_paseto::core::Key;
use sqlx::{Row, SqlitePool};

const MIN_PASSWORD_CHARS: usize = 8;
const AUTO_PASSWORD_CHARS: usize = 16;
const TOKEN_KEY_BYTES: usize = 32;
const ALPHANUMERIC: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

pub fn is_argon2_hash(value: &str) -> bool {
    value.starts_with("$argon2id$")
}

pub fn validate_password_length(raw: &str) -> Result<()> {
    if raw.chars().count() < MIN_PASSWORD_CHARS {
        anyhow::bail!("Password must be at least {MIN_PASSWORD_CHARS} characters long");
    }
    Ok(())
}

pub fn hash_password(raw: &str) -> Result<String> {
    validate_password_length(raw)?;

    let salt_bytes = Key::<16>::try_new_random()
        .map_err(|error| anyhow::anyhow!("Failed to generate password salt: {error}"))?;
    let salt = SaltString::encode_b64(salt_bytes.as_slice())
        .map_err(|error| anyhow::anyhow!("Failed to encode password salt: {error}"))?;

    let hash = Argon2::default()
        .hash_password(raw.as_bytes(), &salt)
        .map_err(|error| anyhow::anyhow!("Failed to hash password: {error}"))?
        .to_string();

    Ok(hash)
}

pub fn verify_password(candidate: &str, hash_str: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash_str) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(candidate.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn generate_random_password() -> Result<String> {
    let raw_bytes = Key::<AUTO_PASSWORD_CHARS>::try_new_random()
        .map_err(|error| anyhow::anyhow!("Failed to generate random password bytes: {error}"))?;

    let mut raw = String::with_capacity(AUTO_PASSWORD_CHARS);
    for byte in raw_bytes.as_slice() {
        let idx = usize::from(*byte) % ALPHANUMERIC.len();
        raw.push(ALPHANUMERIC[idx] as char);
    }

    Ok(format!(
        "{}-{}-{}-{}",
        &raw[0..4],
        &raw[4..8],
        &raw[8..12],
        &raw[12..16]
    ))
}

pub fn generate_token_key() -> Result<Vec<u8>> {
    let key = Key::<TOKEN_KEY_BYTES>::try_new_random()
        .map_err(|error| anyhow::anyhow!("Failed to generate token key bytes: {error}"))?
        .as_slice()
        .to_vec();
    Ok(key)
}

pub async fn get_stored_auth(pool: &SqlitePool) -> Result<Option<(String, Vec<u8>)>> {
    let row = sqlx::query("SELECT password_hash, token_key FROM auth WHERE id = 1")
        .fetch_optional(pool)
        .await
        .context("Failed to read stored auth config")?;

    let Some(row) = row else {
        return Ok(None);
    };

    let password_hash: String = row
        .try_get("password_hash")
        .context("Failed to decode password hash from auth table")?;
    let token_key: Vec<u8> = row
        .try_get("token_key")
        .context("Failed to decode token key from auth table")?;

    Ok(Some((password_hash, token_key)))
}

pub async fn set_stored_auth(pool: &SqlitePool, hash: &str, token_key: &[u8]) -> Result<()> {
    sqlx::query(
        "INSERT INTO auth (id, password_hash, token_key, updated_at)
         VALUES (1, ?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
         ON CONFLICT(id) DO UPDATE SET
           password_hash = excluded.password_hash,
           token_key = excluded.token_key,
           updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
    )
    .bind(hash)
    .bind(token_key)
    .execute(pool)
    .await
    .context("Failed to upsert auth config")?;

    Ok(())
}

pub async fn set_password_hash_and_revoke_sessions(
    pool: &SqlitePool,
    hash: &str,
    keep_session_id: Option<&str>,
) -> Result<()> {
    let mut tx = pool
        .begin()
        .await
        .context("Failed to start auth update transaction")?;

    let result = sqlx::query(
        "UPDATE auth
         SET password_hash = ?1,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE id = 1",
    )
    .bind(hash)
    .execute(&mut *tx)
    .await
    .context("Failed to update auth password hash")?;

    if result.rows_affected() == 0 {
        anyhow::bail!("Authentication is not initialized yet");
    }

    match keep_session_id {
        Some(session_id) => {
            sqlx::query("DELETE FROM sessions WHERE id != ?1")
                .bind(session_id)
                .execute(&mut *tx)
                .await
                .context("Failed to revoke non-current sessions")?;
        }
        None => {
            sqlx::query("DELETE FROM sessions")
                .execute(&mut *tx)
                .await
                .context("Failed to revoke sessions")?;
        }
    }

    tx.commit()
        .await
        .context("Failed to commit auth update transaction")
}

#[cfg(test)]
mod tests {
    use super::{
        generate_random_password, generate_token_key, get_stored_auth, hash_password,
        set_password_hash_and_revoke_sessions, set_stored_auth,
    };
    use crate::store::sqlite::{open_library_pool_in_memory, run_koshelf_migrations};

    #[test]
    fn generated_password_has_expected_format() {
        let password = generate_random_password().expect("password generation should succeed");
        let parts: Vec<&str> = password.split('-').collect();

        assert_eq!(parts.len(), 4);
        assert!(parts.iter().all(|part| part.len() == 4));
    }

    #[test]
    fn generated_token_key_has_expected_length() {
        let key = generate_token_key().expect("token key generation should succeed");
        assert_eq!(key.len(), 32);
    }

    #[tokio::test]
    async fn password_update_keeps_requested_session() {
        let pool = open_library_pool_in_memory()
            .await
            .expect("in-memory pool should open");
        run_koshelf_migrations(&pool)
            .await
            .expect("migrations should succeed");

        let initial_hash = hash_password("initial-pass").expect("hashing should succeed");
        set_stored_auth(
            &pool,
            &initial_hash,
            &generate_token_key().expect("token key generation should succeed"),
        )
        .await
        .expect("auth should initialize");

        let now = chrono::Utc::now().timestamp() + 3600;
        sqlx::query(
            "INSERT INTO sessions (id, user_agent, last_seen_ip, last_seen_at, expires_at_unix)
             VALUES (?1, NULL, NULL, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?2)",
        )
        .bind("keep")
        .bind(now)
        .execute(&pool)
        .await
        .expect("first session insert should succeed");

        sqlx::query(
            "INSERT INTO sessions (id, user_agent, last_seen_ip, last_seen_at, expires_at_unix)
             VALUES (?1, NULL, NULL, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?2)",
        )
        .bind("drop")
        .bind(now)
        .execute(&pool)
        .await
        .expect("second session insert should succeed");

        let next_hash = hash_password("next-password").expect("hashing should succeed");
        set_password_hash_and_revoke_sessions(&pool, &next_hash, Some("keep"))
            .await
            .expect("password update should succeed");

        let stored = get_stored_auth(&pool)
            .await
            .expect("auth lookup should succeed")
            .expect("auth row should exist");
        assert_eq!(stored.0, next_hash);

        let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
            .fetch_one(&pool)
            .await
            .expect("count query should succeed");
        assert_eq!(remaining, 1);

        let kept_exists: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE id = 'keep'")
                .fetch_one(&pool)
                .await
                .expect("keep lookup should succeed");
        assert_eq!(kept_exists, 1);
    }

    #[tokio::test]
    async fn password_update_can_revoke_all_sessions() {
        let pool = open_library_pool_in_memory()
            .await
            .expect("in-memory pool should open");
        run_koshelf_migrations(&pool)
            .await
            .expect("migrations should succeed");

        let initial_hash = hash_password("initial-pass").expect("hashing should succeed");
        set_stored_auth(
            &pool,
            &initial_hash,
            &generate_token_key().expect("token key generation should succeed"),
        )
        .await
        .expect("auth should initialize");

        let now = chrono::Utc::now().timestamp() + 3600;
        sqlx::query(
            "INSERT INTO sessions (id, user_agent, last_seen_ip, last_seen_at, expires_at_unix)
             VALUES (?1, NULL, NULL, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?2)",
        )
        .bind("session-a")
        .bind(now)
        .execute(&pool)
        .await
        .expect("session insert should succeed");

        let next_hash = hash_password("another-pass").expect("hashing should succeed");
        set_password_hash_and_revoke_sessions(&pool, &next_hash, None)
            .await
            .expect("password update should succeed");

        let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
            .fetch_one(&pool)
            .await
            .expect("count query should succeed");
        assert_eq!(remaining, 0);
    }
}
