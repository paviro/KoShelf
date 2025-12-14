//! Library scanning and file watching.

pub mod scanner;
pub mod watcher;

pub use scanner::{MetadataLocation, scan_library};
pub use watcher::FileWatcher;
