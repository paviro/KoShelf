//! Web server module.

use crate::runtime::SharedSnapshotStore;

pub mod api;
pub mod web;

pub use web::WebServer;

#[derive(Clone)]
pub struct ServerState {
    pub snapshot_store: SharedSnapshotStore,
}
