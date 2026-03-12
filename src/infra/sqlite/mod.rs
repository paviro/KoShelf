//! SQLite infrastructure boundaries for cache and statistics repositories.

pub mod library_db;
pub mod migrations;

pub use library_db::{
    LIBRARY_DB_REQUIRED_INDEXES, LIBRARY_DB_REQUIRED_TABLES, LIBRARY_DB_SCHEMA_VERSION,
};
pub use migrations::ensure_library_db_schema;
