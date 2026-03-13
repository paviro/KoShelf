//! Runtime data storage, ingest pipeline, and media asset management.

pub mod export;
pub mod ingest;
pub mod media;
pub mod observability;
pub mod reading_data;
pub mod rebuild;
pub mod recap;
pub mod store;
pub mod updates;

pub use crate::models::ReadingData;
pub use ingest::{IngestResult, ingest};
pub use media::MediaDirs;
pub use observability::{
    LibraryDbRebuildReason, RuntimeObservability, RuntimeObservabilitySnapshot,
    RuntimeReconcileCounters, SqliteRouteClass,
};
pub use reading_data::{ReadingDataStore, SharedReadingDataStore};
pub use store::{SharedSiteStore, SiteStore};
pub use updates::{Update, UpdateNotifier};
