//! Ingest pipeline: library item processing and statistics loading.
//!
//! `library` handles parsing book files, deduplicating via the DB, persisting
//! items, and generating covers — all one item at a time with no bulk vectors.
//!
//! `statistics` loads the KOReader statistics database and tags entries using
//! DB queries rather than in-memory item collections.

pub mod library;
pub mod statistics;

pub use library::{IngestStats, UpdateResult, ingest_paths, update_library};
pub use statistics::load_reading_data;
