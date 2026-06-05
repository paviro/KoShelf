use crate::source::sqlite_snapshot::copy_sqlite_snapshot;
use anyhow::{Context, Result};
use log::{debug, info, warn};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{AssertSqlSafe, Row, SqlitePool};
use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;
use tempfile::TempDir;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KoboFileHints {
    pub content_id: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub is_encrypted: bool,
    pub has_content_keys: bool,
}

impl KoboFileHints {
    pub fn suggests_encryption(&self) -> bool {
        self.is_encrypted || self.has_content_keys
    }
}

#[derive(Clone, Debug)]
pub struct KoboContentEntry {
    pub hints: KoboFileHints,
}

pub struct KoboDbParser;

impl KoboDbParser {
    pub async fn parse<P: AsRef<Path>>(path: P) -> Result<Vec<KoboContentEntry>> {
        info!("Opening Kobo database: {:?}", path.as_ref());

        let temp_dir = TempDir::new().with_context(|| "Failed to create temporary directory")?;
        let temp_db_path = temp_dir.path().join("KoboReader.sqlite");

        debug!(
            "Copying Kobo database to temporary file: {:?}",
            temp_db_path
        );
        copy_sqlite_snapshot(path.as_ref(), &temp_db_path)?;

        let url = format!("sqlite:{}?mode=ro", temp_db_path.display());
        let options = SqliteConnectOptions::from_str(&url)
            .with_context(|| format!("Failed to parse Kobo DB URL for {:?}", temp_db_path))?;

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .with_context(|| {
                format!("Failed to open temporary Kobo database: {:?}", temp_db_path)
            })?;

        let entries = Self::parse_entries(&pool).await?;
        pool.close().await;

        info!("Found {} Kobo book entries", entries.len());
        Ok(entries)
    }

    async fn parse_entries(pool: &SqlitePool) -> Result<Vec<KoboContentEntry>> {
        if !table_exists(pool, "content").await? {
            warn!("Kobo database has no content table");
            return Ok(Vec::new());
        }

        let content_columns = table_columns(pool, "content").await?;
        if !content_columns.contains("ContentID") {
            warn!("Kobo content table has no ContentID column");
            return Ok(Vec::new());
        }

        let selected_columns = [
            "ContentID",
            "BookID",
            "Title",
            "Attribution",
            "IsEncrypted",
            "MimeType",
        ]
        .into_iter()
        .filter(|column| content_columns.contains(*column))
        .collect::<Vec<_>>();

        let sql = format!("SELECT {} FROM content", selected_columns.join(", "));
        let rows = sqlx::query(AssertSqlSafe(sql))
            .fetch_all(pool)
            .await
            .context("Failed to query Kobo content rows")?;

        let content_key_ids = load_content_key_ids(pool).await?;
        let mut entries = Vec::new();

        for row in rows {
            let content_id = match string_column(&row, "ContentID") {
                Some(value) if !value.trim().is_empty() => value,
                _ => continue,
            };

            if is_chapter_row(&row, &content_id) {
                continue;
            }

            let has_content_keys = content_key_ids.contains(&content_id);
            entries.push(KoboContentEntry {
                hints: KoboFileHints {
                    content_id,
                    title: string_column(&row, "Title"),
                    author: string_column(&row, "Attribution"),
                    is_encrypted: int_column(&row, "IsEncrypted").unwrap_or(0) != 0,
                    has_content_keys,
                },
            });
        }

        Ok(entries)
    }
}

async fn table_exists(pool: &SqlitePool, table: &str) -> Result<bool> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1")
            .bind(table)
            .fetch_optional(pool)
            .await
            .with_context(|| format!("Failed to inspect table {}", table))?;
    Ok(row.is_some())
}

async fn table_columns(pool: &SqlitePool, table: &str) -> Result<HashSet<String>> {
    let sql = format!("PRAGMA table_info({})", table);
    let rows = sqlx::query(AssertSqlSafe(sql))
        .fetch_all(pool)
        .await
        .with_context(|| format!("Failed to inspect columns for table {}", table))?;

    Ok(rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect())
}

async fn load_content_key_ids(pool: &SqlitePool) -> Result<HashSet<String>> {
    if !table_exists(pool, "content_keys").await? {
        return Ok(HashSet::new());
    }

    let columns = table_columns(pool, "content_keys").await?;
    let mut key_ids = HashSet::new();

    for column in ["ContentID", "VolumeID"] {
        if !columns.contains(column) {
            continue;
        }

        let sql = format!(
            "SELECT \"{}\" AS content_key_id FROM content_keys WHERE \"{}\" IS NOT NULL",
            column, column
        );
        let rows = sqlx::query(AssertSqlSafe(sql))
            .fetch_all(pool)
            .await
            .with_context(|| format!("Failed to query Kobo content_keys.{}", column))?;

        for row in rows {
            if let Ok(value) = row.try_get::<String, _>("content_key_id")
                && !value.trim().is_empty()
            {
                key_ids.insert(value);
            }
        }
    }

    Ok(key_ids)
}

fn is_chapter_row(row: &sqlx::sqlite::SqliteRow, content_id: &str) -> bool {
    let Some(book_id) = string_column(row, "BookID") else {
        return false;
    };
    let book_id = book_id.trim();
    !book_id.is_empty() && book_id != content_id
}

fn string_column(row: &sqlx::sqlite::SqliteRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
        .filter(|value| !value.trim().is_empty())
}

fn int_column(row: &sqlx::sqlite::SqliteRow, column: &str) -> Option<i64> {
    row.try_get::<Option<i64>, _>(column).ok().flatten()
}

#[cfg(test)]
mod tests {
    use super::KoboDbParser;
    use sqlx::Executor;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    async fn synthetic_kobo_db() -> tempfile::NamedTempFile {
        let file = tempfile::NamedTempFile::new().expect("temp file");
        let url = format!("sqlite:{}?mode=rwc", file.path().display());
        let options = SqliteConnectOptions::from_str(&url).expect("sqlite options");
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("sqlite pool");

        sqlx::query(
            "CREATE TABLE content (
                ContentID TEXT PRIMARY KEY,
                BookID TEXT,
                Title TEXT,
                Attribution TEXT,
                IsEncrypted INTEGER,
                MimeType TEXT
            )",
        )
        .execute(&pool)
        .await
        .expect("content table");
        sqlx::query("CREATE TABLE content_keys (ContentID TEXT, VolumeID TEXT)")
            .execute(&pool)
            .await
            .expect("content_keys table");

        sqlx::query(
            "INSERT INTO content
             (ContentID, BookID, Title, Attribution, IsEncrypted, MimeType)
             VALUES
             ('plain-kepub', NULL, 'Plain', 'Author A', 0, 'application/x-kobo-epub+zip'),
             ('encrypted-kepub', '', 'Encrypted', 'Author B', 1, 'application/x-kobo-epub+zip'),
             ('keyed-kepub', NULL, 'Keyed', 'Author C', 0, 'application/x-kobo-epub+zip'),
             ('book-parent!chapter-1', 'book-parent', 'Chapter 1', 'Author D', 0, 'text/html'),
             ('file:///mnt/onboard/.kobo/kepub/file-row', NULL, 'File Row', 'Author E', 0, 'application/x-kobo-epub+zip')",
        )
        .execute(&pool)
        .await
        .expect("content rows");
        sqlx::query("INSERT INTO content_keys (ContentID, VolumeID) VALUES ('keyed-kepub', NULL)")
            .execute(&pool)
            .await
            .expect("content key row");

        pool.close().await;
        file
    }

    #[tokio::test]
    async fn parses_kobo_book_rows_and_diagnostics() {
        let db = synthetic_kobo_db().await;
        let entries = KoboDbParser::parse(db.path()).await.expect("parse kobo db");

        assert_eq!(entries.len(), 4);

        let plain = entries
            .iter()
            .find(|entry| entry.hints.content_id == "plain-kepub")
            .expect("plain row");
        assert_eq!(plain.hints.title.as_deref(), Some("Plain"));
        assert_eq!(plain.hints.author.as_deref(), Some("Author A"));
        assert!(!plain.hints.is_encrypted);
        assert!(!plain.hints.has_content_keys);

        let encrypted = entries
            .iter()
            .find(|entry| entry.hints.content_id == "encrypted-kepub")
            .expect("encrypted row");
        assert!(encrypted.hints.is_encrypted);

        let keyed = entries
            .iter()
            .find(|entry| entry.hints.content_id == "keyed-kepub")
            .expect("keyed row");
        assert!(keyed.hints.has_content_keys);

        assert!(
            entries
                .iter()
                .all(|entry| entry.hints.content_id != "book-parent!chapter-1"),
            "chapter rows should be skipped"
        );

        assert!(
            entries
                .iter()
                .any(|entry| entry.hints.content_id == "file:///mnt/onboard/.kobo/kepub/file-row"),
            "file:// content IDs should be retained for matching"
        );
    }

    #[tokio::test]
    async fn parse_reads_rows_from_wal_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("KoboReader.sqlite");
        let url = format!("sqlite:{}?mode=rwc", db_path.display());
        let options = SqliteConnectOptions::from_str(&url).expect("sqlite options");
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("sqlite pool");

        pool.execute("PRAGMA journal_mode=WAL")
            .await
            .expect("wal mode");
        pool.execute("PRAGMA wal_autocheckpoint=0")
            .await
            .expect("disable autocheckpoint");
        pool.execute(
            "CREATE TABLE content (
                ContentID TEXT PRIMARY KEY,
                BookID TEXT,
                Title TEXT,
                Attribution TEXT,
                IsEncrypted INTEGER,
                MimeType TEXT
            )",
        )
        .await
        .expect("content table");
        pool.execute(
            "INSERT INTO content
             (ContentID, BookID, Title, Attribution, IsEncrypted, MimeType)
             VALUES ('wal-kepub', NULL, 'Wal Kobo', 'Author', 0, 'application/x-kobo-epub+zip')",
        )
        .await
        .expect("content row");

        assert!(
            db_path.with_file_name("KoboReader.sqlite-wal").exists(),
            "test setup should leave rows in WAL"
        );

        let entries = KoboDbParser::parse(&db_path).await.expect("parse kobo db");

        let entry = entries
            .iter()
            .find(|entry| entry.hints.content_id == "wal-kepub")
            .expect("wal row");
        assert_eq!(entry.hints.title.as_deref(), Some("Wal Kobo"));

        pool.close().await;
    }
}
