//! SQLite infrastructure boundaries for cache and statistics repositories.

pub mod library_db;
pub mod library_repo;
pub mod migrations;

pub use library_db::open_library_pool;
#[cfg(test)]
pub use library_db::open_library_pool_in_memory;
pub use library_db::{LIBRARY_DB_REQUIRED_INDEXES, LIBRARY_DB_REQUIRED_TABLES};
pub use migrations::run_library_migrations;
