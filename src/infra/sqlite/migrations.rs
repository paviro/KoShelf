use anyhow::{Result, bail};
use rusqlite::Connection;

use crate::infra::sqlite::library_db::{LIBRARY_DB_SCHEMA_V1_SQL, LIBRARY_DB_SCHEMA_VERSION};

pub fn ensure_library_db_schema(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;

    match current_schema_version(conn)? {
        0 => install_schema_v1(conn),
        LIBRARY_DB_SCHEMA_VERSION => Ok(()),
        current => bail!(
            "Unsupported library DB schema version {current}; expected {LIBRARY_DB_SCHEMA_VERSION}"
        ),
    }
}

fn install_schema_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(LIBRARY_DB_SCHEMA_V1_SQL)?;
    conn.pragma_update(None, "user_version", LIBRARY_DB_SCHEMA_VERSION)?;
    Ok(())
}

fn current_schema_version(conn: &Connection) -> Result<i32> {
    Ok(conn.query_row("PRAGMA user_version", [], |row| row.get(0))?)
}

#[cfg(test)]
mod tests {
    use super::{current_schema_version, ensure_library_db_schema};
    use crate::infra::sqlite::library_db::{
        LIBRARY_DB_REQUIRED_INDEXES, LIBRARY_DB_REQUIRED_TABLES, LIBRARY_DB_SCHEMA_VERSION,
    };
    use rusqlite::{Connection, params};

    #[test]
    fn creates_library_schema_tables_indexes_and_version() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");

        ensure_library_db_schema(&conn).expect("schema creation should succeed");

        assert_eq!(
            current_schema_version(&conn).expect("schema version should be readable"),
            LIBRARY_DB_SCHEMA_VERSION
        );

        for table in LIBRARY_DB_REQUIRED_TABLES {
            assert!(
                sqlite_object_exists(&conn, "table", table),
                "expected table `{table}` to exist"
            );
        }

        for index in LIBRARY_DB_REQUIRED_INDEXES {
            assert!(
                sqlite_object_exists(&conn, "index", index),
                "expected index `{index}` to exist"
            );
        }
    }

    #[test]
    fn schema_creation_is_idempotent_at_current_version() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");

        ensure_library_db_schema(&conn).expect("first schema creation should succeed");
        ensure_library_db_schema(&conn).expect("second schema creation should succeed");

        assert_eq!(
            current_schema_version(&conn).expect("schema version should be readable"),
            LIBRARY_DB_SCHEMA_VERSION
        );
    }

    #[test]
    fn rejects_unknown_schema_version() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        conn.pragma_update(None, "user_version", 99)
            .expect("user_version should be writable");

        let err =
            ensure_library_db_schema(&conn).expect_err("unknown schema versions should fail fast");

        assert!(
            err.to_string()
                .contains("Unsupported library DB schema version 99"),
            "unexpected error: {err:#}"
        );
    }

    fn sqlite_object_exists(conn: &Connection, object_type: &str, name: &str) -> bool {
        conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE type = ?1 AND name = ?2",
            params![object_type, name],
            |_row| Ok(()),
        )
        .is_ok()
    }
}
