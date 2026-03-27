use anyhow::{Context, Result};
use log::warn;
use sqlx::SqlitePool;
use sqlx::migrate::MigrateError;

use crate::store::sqlite::pool::LIBRARY_DB_REQUIRED_TABLES;

/// Run all pending library DB migrations using sqlx's embedded migration system.
///
/// If a previously applied migration has been modified (schema change), the
/// database is reset and migrations are re-applied from scratch.  This is safe
/// because the library DB is a rebuild-on-change cache.
pub async fn run_library_migrations(pool: &SqlitePool) -> Result<()> {
    let migrator = sqlx::migrate!("src/store/sqlite/migrations/library");

    match migrator.run(pool).await {
        Ok(()) => Ok(()),
        Err(MigrateError::VersionMismatch(version)) => {
            warn!("Library DB migration {version} was modified — resetting database");
            reset_library_db(pool).await?;
            migrator
                .run(pool)
                .await
                .context("Failed to run library DB migrations after reset")
        }
        Err(MigrateError::VersionMissing(version)) => {
            warn!("Library DB migration {version} was removed — resetting database");
            reset_library_db(pool).await?;
            migrator
                .run(pool)
                .await
                .context("Failed to run library DB migrations after reset")
        }
        Err(error) => Err(error).context("Failed to run library DB migrations"),
    }
}

/// Run all pending KoShelf application DB migrations.
pub async fn run_koshelf_migrations(pool: &SqlitePool) -> Result<()> {
    let migrator = sqlx::migrate!("src/store/sqlite/migrations/koshelf");
    migrator
        .run(pool)
        .await
        .context("Failed to run KoShelf DB migrations")
}

/// Drop all library tables and the sqlx migration tracking table.
async fn reset_library_db(pool: &SqlitePool) -> Result<()> {
    for table in LIBRARY_DB_REQUIRED_TABLES.iter().rev() {
        sqlx::query(&format!("DROP TABLE IF EXISTS {table}"))
            .execute(pool)
            .await
            .with_context(|| format!("Failed to drop table `{table}` during library DB reset"))?;
    }
    sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations")
        .execute(pool)
        .await
        .context("Failed to drop `_sqlx_migrations` table during library DB reset")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{run_koshelf_migrations, run_library_migrations};
    use crate::store::sqlite::pool::{
        LIBRARY_DB_REQUIRED_INDEXES, LIBRARY_DB_REQUIRED_TABLES, open_library_pool_in_memory,
    };

    #[tokio::test]
    async fn creates_library_schema_tables_and_indexes() {
        let pool = open_library_pool_in_memory()
            .await
            .expect("in-memory pool should open");

        run_library_migrations(&pool)
            .await
            .expect("migrations should succeed");

        for table in LIBRARY_DB_REQUIRED_TABLES {
            assert!(
                sqlite_object_exists(&pool, "table", table).await,
                "expected table `{table}` to exist"
            );
        }

        for index in LIBRARY_DB_REQUIRED_INDEXES {
            assert!(
                sqlite_object_exists(&pool, "index", index).await,
                "expected index `{index}` to exist"
            );
        }
    }

    #[tokio::test]
    async fn migrations_are_idempotent() {
        let pool = open_library_pool_in_memory()
            .await
            .expect("in-memory pool should open");

        run_library_migrations(&pool)
            .await
            .expect("first migration run should succeed");

        run_library_migrations(&pool)
            .await
            .expect("second migration run should succeed");
    }

    #[tokio::test]
    async fn version_mismatch_resets_and_reapplies_schema() {
        let pool = open_library_pool_in_memory()
            .await
            .expect("in-memory pool should open");

        run_library_migrations(&pool)
            .await
            .expect("first migration run should succeed");

        sqlx::query(
            "INSERT INTO library_items (
                id, file_path, format, content_type, title, status,
                cover_url, search_base_path, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )
        .bind("book-1")
        .bind("/books/book-1.epub")
        .bind("epub")
        .bind("book")
        .bind("Book 1")
        .bind("reading")
        .bind("/assets/covers/book-1.webp")
        .bind("/books")
        .bind("2026-01-01T00:00:00Z")
        .bind("2026-01-01T00:00:00Z")
        .execute(&pool)
        .await
        .expect("seed row should insert");

        sqlx::query("UPDATE _sqlx_migrations SET checksum = X'00' WHERE version = 1")
            .execute(&pool)
            .await
            .expect("should be able to tamper migration checksum");

        run_library_migrations(&pool)
            .await
            .expect("version mismatch should trigger reset and re-apply");

        let item_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM library_items")
            .fetch_one(&pool)
            .await
            .expect("count query should succeed");

        assert_eq!(item_count, 0, "reset should clear cached library rows");
    }

    #[tokio::test]
    async fn version_missing_resets_and_reapplies_schema() {
        let pool = open_library_pool_in_memory()
            .await
            .expect("in-memory pool should open");

        run_library_migrations(&pool)
            .await
            .expect("first migration run should succeed");

        sqlx::query(
            "INSERT INTO library_items (
                id, file_path, format, content_type, title, status,
                cover_url, search_base_path, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )
        .bind("book-1")
        .bind("/books/book-1.epub")
        .bind("epub")
        .bind("book")
        .bind("Book 1")
        .bind("reading")
        .bind("/assets/covers/book-1.webp")
        .bind("/books")
        .bind("2026-01-01T00:00:00Z")
        .bind("2026-01-01T00:00:00Z")
        .execute(&pool)
        .await
        .expect("seed row should insert");

        // Simulate a removed migration by inserting a phantom migration record
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
             VALUES (999, 'phantom', CURRENT_TIMESTAMP, TRUE, X'AA', 0)",
        )
        .execute(&pool)
        .await
        .expect("should be able to insert phantom migration");

        run_library_migrations(&pool)
            .await
            .expect("version missing should trigger reset and re-apply");

        let item_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM library_items")
            .fetch_one(&pool)
            .await
            .expect("count query should succeed");

        assert_eq!(item_count, 0, "reset should clear cached library rows");
    }

    async fn sqlite_object_exists(pool: &sqlx::SqlitePool, object_type: &str, name: &str) -> bool {
        sqlx::query("SELECT 1 FROM sqlite_master WHERE type = ?1 AND name = ?2")
            .bind(object_type)
            .bind(name)
            .fetch_optional(pool)
            .await
            .expect("schema query should succeed")
            .is_some()
    }

    #[tokio::test]
    async fn creates_koshelf_auth_schema() {
        let pool = open_library_pool_in_memory()
            .await
            .expect("in-memory pool should open");

        run_koshelf_migrations(&pool)
            .await
            .expect("migrations should succeed");

        assert!(
            sqlite_object_exists(&pool, "table", "auth").await,
            "expected `auth` table to exist"
        );
        assert!(
            sqlite_object_exists(&pool, "table", "sessions").await,
            "expected `sessions` table to exist"
        );
        assert!(
            sqlite_object_exists(&pool, "index", "idx_sessions_expires_at_unix").await,
            "expected `idx_sessions_expires_at_unix` index to exist"
        );
    }
}
