use std::path::Path;
use std::str::FromStr;

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

pub const TABLE_LIBRARY_ITEMS: &str = "library_items";
pub const TABLE_LIBRARY_ANNOTATIONS: &str = "library_annotations";
pub const TABLE_LIBRARY_ITEM_FINGERPRINTS: &str = "library_item_fingerprints";
pub const TABLE_LIBRARY_COLLISION_DIAGNOSTICS: &str = "library_collision_diagnostics";

pub const INDEX_LIBRARY_ITEMS_SCOPE_TITLE: &str = "idx_library_items_scope_title";
pub const INDEX_LIBRARY_ITEMS_SCOPE_AUTHOR: &str = "idx_library_items_scope_author";
pub const INDEX_LIBRARY_ITEMS_SCOPE_STATUS: &str = "idx_library_items_scope_status";
pub const INDEX_LIBRARY_ITEMS_SCOPE_PROGRESS: &str = "idx_library_items_scope_progress";
pub const INDEX_LIBRARY_ITEMS_SCOPE_RATING: &str = "idx_library_items_scope_rating";
pub const INDEX_LIBRARY_ITEMS_SCOPE_ANNOTATIONS: &str = "idx_library_items_scope_annotations";
pub const INDEX_LIBRARY_ITEMS_SCOPE_LAST_OPEN_AT: &str = "idx_library_items_scope_last_open_at";
pub const INDEX_LIBRARY_ITEMS_PARTIAL_MD5: &str = "idx_library_items_partial_md5_checksum";
pub const INDEX_LIBRARY_ANNOTATIONS_ITEM_KIND: &str = "idx_library_annotations_item_kind";
pub const INDEX_LIBRARY_ITEM_FINGERPRINTS_BOOK_PATH: &str =
    "idx_library_item_fingerprints_book_path";
pub const INDEX_LIBRARY_ITEM_FINGERPRINTS_METADATA_PATH: &str =
    "idx_library_item_fingerprints_metadata_path";
pub const INDEX_LIBRARY_COLLISION_DIAGNOSTICS_WINNER_ITEM_ID: &str =
    "idx_library_collision_diagnostics_winner_item_id";

pub const LIBRARY_DB_REQUIRED_TABLES: &[&str] = &[
    TABLE_LIBRARY_ITEMS,
    TABLE_LIBRARY_ANNOTATIONS,
    TABLE_LIBRARY_ITEM_FINGERPRINTS,
    TABLE_LIBRARY_COLLISION_DIAGNOSTICS,
];

pub const LIBRARY_DB_REQUIRED_INDEXES: &[&str] = &[
    INDEX_LIBRARY_ITEMS_SCOPE_TITLE,
    INDEX_LIBRARY_ITEMS_SCOPE_AUTHOR,
    INDEX_LIBRARY_ITEMS_SCOPE_STATUS,
    INDEX_LIBRARY_ITEMS_SCOPE_PROGRESS,
    INDEX_LIBRARY_ITEMS_SCOPE_RATING,
    INDEX_LIBRARY_ITEMS_SCOPE_ANNOTATIONS,
    INDEX_LIBRARY_ITEMS_SCOPE_LAST_OPEN_AT,
    INDEX_LIBRARY_ITEMS_PARTIAL_MD5,
    INDEX_LIBRARY_ANNOTATIONS_ITEM_KIND,
    INDEX_LIBRARY_ITEM_FINGERPRINTS_BOOK_PATH,
    INDEX_LIBRARY_ITEM_FINGERPRINTS_METADATA_PATH,
    INDEX_LIBRARY_COLLISION_DIAGNOSTICS_WINNER_ITEM_ID,
];

/// Open a SQLite connection pool for the library cache at the given file path.
pub async fn open_library_pool(path: &Path) -> Result<SqlitePool> {
    let url = format!("sqlite:{}?mode=rwc", path.display());
    let options = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("Invalid library DB path: {}", path.display()))?
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .with_context(|| format!("Failed to open library DB at {}", path.display()))
}

/// Open an in-memory SQLite pool for ephemeral / test usage.
///
/// Uses a single connection so the in-memory database is shared across queries.
pub async fn open_library_pool_in_memory() -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .context("Failed to parse in-memory SQLite URL")?
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .context("Failed to open in-memory library DB")
}
