//! Web server module.

use crate::runtime::{SharedSnapshotStore, SnapshotUpdateNotifier};

pub mod api;
pub mod web;

pub use web::WebServer;

#[derive(Clone)]
pub struct ServerState {
    pub snapshot_store: SharedSnapshotStore,
    pub update_notifier: SnapshotUpdateNotifier,
}
