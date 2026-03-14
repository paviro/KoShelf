//! Web server module.

use crate::store::memory::{SharedReadingDataStore, SharedSiteStore, UpdateNotifier};
use crate::store::sqlite::repo::LibraryRepository;

pub mod api;
pub mod web;

pub use web::WebServer;

#[derive(Clone)]
pub struct ServerState {
    pub site_store: SharedSiteStore,
    pub reading_data_store: SharedReadingDataStore,
    pub update_notifier: UpdateNotifier,
    pub library_repo: LibraryRepository,
}
