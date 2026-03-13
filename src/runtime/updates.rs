//! Update notifier used by runtime server mode.

use serde::Serialize;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::watch;

/// Payload broadcast to SSE consumers on every data change.
#[derive(Debug, Clone, Serialize)]
pub struct Update {
    pub revision_epoch: String,
    pub revision: u64,
    pub generated_at: String,
}

#[derive(Debug)]
struct UpdateNotifierInner {
    revision_epoch: String,
    revision: AtomicU64,
    tx: watch::Sender<Update>,
}

/// Publishes revision updates for SSE consumers.
#[derive(Debug, Clone)]
pub struct UpdateNotifier {
    inner: Arc<UpdateNotifierInner>,
}

impl UpdateNotifier {
    pub fn new(revision_epoch: impl Into<String>, initial_generated_at: impl Into<String>) -> Self {
        let revision_epoch = revision_epoch.into();
        let initial_update = Update {
            revision_epoch: revision_epoch.clone(),
            revision: 0,
            generated_at: initial_generated_at.into(),
        };
        let (tx, _rx) = watch::channel(initial_update);

        Self {
            inner: Arc::new(UpdateNotifierInner {
                revision_epoch,
                revision: AtomicU64::new(0),
                tx,
            }),
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<Update> {
        self.inner.tx.subscribe()
    }

    pub fn publish(&self, generated_at: impl Into<String>) -> Update {
        let revision = self.inner.revision.fetch_add(1, Ordering::Relaxed) + 1;

        let update = Update {
            revision_epoch: self.inner.revision_epoch.clone(),
            revision,
            generated_at: generated_at.into(),
        };

        // Ignore send failures when no subscribers are connected.
        let _ = self.inner.tx.send(update.clone());
        update
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateNotifier;

    #[test]
    fn publish_increments_revision() {
        let notifier = UpdateNotifier::new("serve_2026-03-11T11:00:00Z", "2026-03-11T11:00:00Z");

        let first = notifier.publish("2026-03-11T11:01:00Z");
        assert_eq!(first.revision_epoch, "serve_2026-03-11T11:00:00Z");
        assert_eq!(first.revision, 1);

        let second = notifier.publish("2026-03-11T11:02:00Z");
        assert_eq!(second.revision, 2);
    }

    #[test]
    fn initial_state_has_zero_revision() {
        let notifier = UpdateNotifier::new("serve_epoch", "2026-03-11T11:00:00Z");

        let receiver = notifier.subscribe();
        let initial = receiver.borrow().clone();

        assert_eq!(initial.revision_epoch, "serve_epoch");
        assert_eq!(initial.revision, 0);
        assert_eq!(initial.generated_at, "2026-03-11T11:00:00Z");
    }
}
