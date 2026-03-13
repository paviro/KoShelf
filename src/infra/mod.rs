//! Infrastructure boundaries for storage, source adapters, and assets.

pub mod assets;
pub mod lifecycle;
pub mod scanner;
pub mod sources;
pub mod sqlite;
pub mod stores;
pub mod watcher;

pub use scanner::MetadataLocation;
pub use watcher::FileWatcher;
