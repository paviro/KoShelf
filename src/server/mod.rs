//! Web server module.

use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::infra::stores::{SharedReadingDataStore, SharedSiteStore, UpdateNotifier};

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
