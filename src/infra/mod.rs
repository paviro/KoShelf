//! Infrastructure boundaries for storage and assets.

pub mod assets;
pub mod lifecycle;
pub mod sqlite;
pub mod stores;
pub mod watcher;

pub use watcher::FileWatcher;
