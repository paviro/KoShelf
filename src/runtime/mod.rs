//! Runtime data storage for contract snapshots.

pub mod snapshot;
pub mod store;

pub use snapshot::ContractSnapshot;
pub use store::{SharedSnapshotStore, SnapshotStore, create_snapshot_store};
