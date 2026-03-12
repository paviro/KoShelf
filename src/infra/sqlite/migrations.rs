use anyhow::{Context, Result};
use sqlx::SqlitePool;

/// Run all pending library DB migrations using sqlx's embedded migration system.
///
/// Migration files live in `src/infra/sqlite/migrations/library/` and are
/// embedded at compile time.
pub async fn run_library_migrations(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!("src/infra/sqlite/migrations/library")
        .run(pool)
        .await
        .context("Failed to run library DB migrations")
}

#[cfg(test)]
mod tests {
    use super::run_library_migrations;
    use crate::infra::sqlite::library_db::{
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
