//! Web server module.

use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::runtime::{SharedReadingDataStore, SharedSnapshotStore, SnapshotUpdateNotifier};

pub mod api;
pub mod web;

pub use web::WebServer;

#[derive(Clone)]
pub struct ServerState {
    pub snapshot_store: SharedSnapshotStore,
    pub reading_data_store: SharedReadingDataStore,
    pub update_notifier: SnapshotUpdateNotifier,
    pub library_repo: LibraryRepository,
}
