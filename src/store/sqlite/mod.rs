//! SQLite infrastructure for cache and statistics repositories.

pub mod migrations;
pub mod pool;
pub mod repo;

pub use migrations::{run_koshelf_migrations, run_library_migrations};
#[cfg(test)]
pub use pool::open_library_pool_in_memory;
pub use pool::{LIBRARY_DB_REQUIRED_INDEXES, LIBRARY_DB_REQUIRED_TABLES};
pub use pool::{open_koshelf_pool, open_library_pool};
