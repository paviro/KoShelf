//! Web server and version checking.

use std::path::PathBuf;

use self::version::SharedVersionNotifier;
use crate::runtime::SharedSnapshotStore;

pub mod api;
pub mod version;
pub mod web;

pub use version::create_version_notifier;
pub use web::WebServer;

#[derive(Clone)]
pub struct ServerState {
    pub site_dir: PathBuf,
    pub version_notifier: SharedVersionNotifier,
    pub snapshot_store: SharedSnapshotStore,
}
