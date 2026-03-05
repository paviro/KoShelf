//! Shared in-memory snapshot store.

use super::snapshot::ContractSnapshot;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct SnapshotStore {
    inner: Arc<RwLock<Option<Arc<ContractSnapshot>>>>,
}

impl SnapshotStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace(&self, snapshot: ContractSnapshot) {
        let mut guard = self
            .inner
            .write()
            .expect("snapshot store lock poisoned while writing");
        *guard = Some(Arc::new(snapshot));
    }

    pub fn clear(&self) {
        let mut guard = self
            .inner
            .write()
            .expect("snapshot store lock poisoned while clearing");
        *guard = None;
    }

    pub fn get(&self) -> Option<Arc<ContractSnapshot>> {
        let guard = self
            .inner
            .read()
            .expect("snapshot store lock poisoned while reading");
        guard.clone()
    }
}

pub type SharedSnapshotStore = Arc<SnapshotStore>;
