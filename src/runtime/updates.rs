//! Snapshot update notifier used by runtime server mode.

use serde::Serialize;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::watch;

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotUpdate {
    pub revision: u64,
    pub generated_at: String,
}

#[derive(Debug)]
struct SnapshotUpdateNotifierInner {
    next_revision: AtomicU64,
    tx: watch::Sender<SnapshotUpdate>,
}

/// Publishes monotonically increasing snapshot-update revisions for SSE consumers.
#[derive(Debug, Clone)]
pub struct SnapshotUpdateNotifier {
    inner: Arc<SnapshotUpdateNotifierInner>,
}

impl SnapshotUpdateNotifier {
    pub fn new(initial_generated_at: impl Into<String>) -> Self {
        let initial_update = SnapshotUpdate {
            revision: 0,
            generated_at: initial_generated_at.into(),
        };
        let (tx, _rx) = watch::channel(initial_update);

        Self {
            inner: Arc::new(SnapshotUpdateNotifierInner {
                next_revision: AtomicU64::new(1),
                tx,
            }),
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<SnapshotUpdate> {
        self.inner.tx.subscribe()
    }

    pub fn publish(&self, generated_at: impl Into<String>) -> SnapshotUpdate {
        let update = SnapshotUpdate {
            revision: self.inner.next_revision.fetch_add(1, Ordering::Relaxed),
            generated_at: generated_at.into(),
        };

        // Ignore send failures when no subscribers are connected.
        let _ = self.inner.tx.send(update.clone());
        update
    }
}
