use anyhow::{Context, Result};
use log::warn;
use sqlx::SqlitePool;

use super::pool::LIBRARY_DB_REQUIRED_TABLES;

/// Run all pending library DB migrations using sqlx's embedded migration system.
///
/// If a previously applied migration has been modified (schema change), the
/// database is reset and migrations are re-applied from scratch.  This is safe
/// because the library DB is a rebuild-on-change cache.
pub async fn run_library_migrations(pool: &SqlitePool) -> Result<()> {
    let migrator = sqlx::migrate!("src/infra/sqlite/migrations/library");

    match migrator.run(pool).await {
        Ok(()) => Ok(()),
        Err(_) => {
            warn!("Library DB schema changed — resetting database");
            reset_library_db(pool).await?;
            migrator
                .run(pool)
                .await
                .context("Failed to run library DB migrations after reset")
        }
    }
}

/// Drop all library tables and the sqlx migration tracking table.
async fn reset_library_db(pool: &SqlitePool) -> Result<()> {
    for table in LIBRARY_DB_REQUIRED_TABLES {
        sqlx::query(&format!("DROP TABLE IF EXISTS {table}"))
            .execute(pool)
            .await
            .ok();
    }
    sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations")
        .execute(pool)
        .await
        .ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run_library_migrations;
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

    async fn sqlite_object_exists(pool: &sqlx::SqlitePool, object_type: &str, name: &str) -> bool {
        sqlx::query("SELECT 1 FROM sqlite_master WHERE type = ?1 AND name = ?2")
            .bind(object_type)
            .bind(name)
            .fetch_optional(pool)
            .await
            .expect("schema query should succeed")
            .is_some()
    }
}
