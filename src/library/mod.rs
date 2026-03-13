//! Library scanning and file watching.

pub mod scanner;
pub mod watcher;

pub use scanner::{MetadataLocation, ScannedItem, scan_library, scan_specific_files};
pub use watcher::FileWatcher;
