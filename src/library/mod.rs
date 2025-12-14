//! Library scanning and file watching.

pub mod scanner;
pub mod watcher;

pub use scanner::{scan_library, MetadataLocation};
pub use watcher::FileWatcher;
