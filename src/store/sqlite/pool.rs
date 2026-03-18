use std::path::Path;
use std::str::FromStr;

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

pub const TABLE_LIBRARY_ITEMS: &str = "library_items";
pub const TABLE_LIBRARY_ANNOTATIONS: &str = "library_annotations";
pub const TABLE_LIBRARY_ITEM_FINGERPRINTS: &str = "library_item_fingerprints";
pub const TABLE_LIBRARY_COLLISION_DIAGNOSTICS: &str = "library_collision_diagnostics";

pub const LIBRARY_DB_REQUIRED_TABLES: &[&str] = &[
    TABLE_LIBRARY_ITEMS,
    TABLE_LIBRARY_ANNOTATIONS,
    TABLE_LIBRARY_ITEM_FINGERPRINTS,
    TABLE_LIBRARY_COLLISION_DIAGNOSTICS,
];

pub const LIBRARY_DB_REQUIRED_INDEXES: &[&str] = &[
    "idx_library_items_scope_status",
    "idx_library_items_scope_progress",
    "idx_library_items_scope_rating",
    "idx_library_items_scope_annotations",
    "idx_library_items_scope_last_open_at",
    "idx_library_items_partial_md5_checksum",
    "idx_library_annotations_item_kind",
    "idx_library_item_fingerprints_book_path",
    "idx_library_item_fingerprints_metadata_path",
    "idx_library_collision_diagnostics_winner_item_id",
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

/// Open a SQLite connection pool for the KoShelf app DB at the given file path.
pub async fn open_koshelf_pool(path: &Path) -> Result<SqlitePool> {
    let url = format!("sqlite:{}?mode=rwc", path.display());
    let options = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("Invalid KoShelf DB path: {}", path.display()))?
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .with_context(|| format!("Failed to open KoShelf DB at {}", path.display()))
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
