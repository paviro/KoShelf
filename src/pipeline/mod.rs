//! Pipeline orchestration: ingest, rebuild, export, media assets, and file watching.

pub mod export;
pub mod ingest;
pub mod media;
pub mod rebuild;
pub mod recap;
pub mod share;
pub mod watcher;

pub use ingest::{IngestStats, ingest_paths};
pub use media::MediaDirs;
pub use watcher::FileWatcher;
