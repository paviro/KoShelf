//! Runtime orchestration: ingest pipeline, rebuild, export, and media assets.

pub mod export;
pub mod ingest;
pub mod media;
pub mod rebuild;
pub mod recap;

pub use ingest::{IngestResult, ingest};
pub use media::MediaDirs;

// Re-exported from canonical home in `infra::stores`.
pub use crate::infra::stores::{
    ReadingData, ReadingDataStore, SharedReadingDataStore, SharedSiteStore, SiteStore, Update,
    UpdateNotifier,
};
