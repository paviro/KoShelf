//! Snapshot update notifier used by runtime server mode.

use super::observability::RuntimeObservability;
use super::revisions::{DomainRevisionState, DomainRevisionTracker, RevisionDomain};
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
    revision_tracker: DomainRevisionTracker,
    observability: RuntimeObservability,
    tx: watch::Sender<SnapshotUpdate>,
}

/// Publishes monotonically increasing snapshot-update revisions for SSE consumers.
#[derive(Debug, Clone)]
pub struct SnapshotUpdateNotifier {
    inner: Arc<SnapshotUpdateNotifierInner>,
}

impl SnapshotUpdateNotifier {
    pub fn new(revision_epoch: impl Into<String>, initial_generated_at: impl Into<String>) -> Self {
        Self::with_observability(
            revision_epoch,
            initial_generated_at,
            RuntimeObservability::default(),
        )
    }

    pub fn with_observability(
        revision_epoch: impl Into<String>,
        initial_generated_at: impl Into<String>,
        observability: RuntimeObservability,
    ) -> Self {
        let initial_update = SnapshotUpdate {
            revision: 0,
            generated_at: initial_generated_at.into(),
        };
        let (tx, _rx) = watch::channel(initial_update);

        Self {
            inner: Arc::new(SnapshotUpdateNotifierInner {
                next_revision: AtomicU64::new(1),
                revision_tracker: DomainRevisionTracker::new(revision_epoch),
                observability,
                tx,
            }),
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<SnapshotUpdate> {
        self.inner.tx.subscribe()
    }

    pub fn publish(&self, generated_at: impl Into<String>) -> SnapshotUpdate {
        self.publish_for_domains(
            generated_at,
            [
                RevisionDomain::Library,
                RevisionDomain::Metadata,
                RevisionDomain::Stats,
                RevisionDomain::Assets,
            ],
        )
    }

    pub fn publish_for_domains<I>(
        &self,
        generated_at: impl Into<String>,
        domains: I,
    ) -> SnapshotUpdate
    where
        I: IntoIterator<Item = RevisionDomain>,
    {
        let domains: Vec<_> = domains.into_iter().collect();
        self.inner
            .revision_tracker
            .bump_domains(domains.iter().copied());
        for domain in domains {
            self.inner.observability.record_invalidation_event(domain);
        }

        let update = SnapshotUpdate {
            revision: self.inner.next_revision.fetch_add(1, Ordering::Relaxed),
            generated_at: generated_at.into(),
        };

        // Ignore send failures when no subscribers are connected.
        let _ = self.inner.tx.send(update.clone());
        update
    }

    pub fn domain_revisions(&self) -> DomainRevisionState {
        self.inner.revision_tracker.snapshot_state()
    }

    pub fn observability(&self) -> RuntimeObservability {
        self.inner.observability.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::SnapshotUpdateNotifier;
    use crate::runtime::{DomainRevision, RevisionDomain};

    #[test]
    fn publish_tracks_monotonic_legacy_revision_and_domain_revisions() {
        let notifier =
            SnapshotUpdateNotifier::new("serve_2026-03-11T11:00:00Z", "2026-03-11T11:00:00Z");

        assert_eq!(
            notifier.domain_revisions().revision,
            DomainRevision::default()
        );

        let first = notifier.publish_for_domains(
            "2026-03-11T11:01:00Z",
            [RevisionDomain::Library, RevisionDomain::Stats],
        );
        assert_eq!(first.revision, 1);

        let second =
            notifier.publish_for_domains("2026-03-11T11:02:00Z", [RevisionDomain::Metadata]);
        assert_eq!(second.revision, 2);

        let revisions = notifier.domain_revisions();
        assert_eq!(revisions.revision_epoch, "serve_2026-03-11T11:00:00Z");
        assert_eq!(revisions.revision.library, 1);
        assert_eq!(revisions.revision.metadata, 1);
        assert_eq!(revisions.revision.stats, 1);
        assert_eq!(revisions.revision.assets, 0);

        let telemetry = notifier.observability().snapshot();
        assert_eq!(telemetry.invalidation_events.library, 1);
        assert_eq!(telemetry.invalidation_events.metadata, 1);
        assert_eq!(telemetry.invalidation_events.stats, 1);
        assert_eq!(telemetry.invalidation_events.assets, 0);
    }

    #[test]
    fn publish_defaults_to_bumping_all_domains() {
        let notifier = SnapshotUpdateNotifier::new("serve_epoch", "2026-03-11T11:00:00Z");

        notifier.publish("2026-03-11T11:05:00Z");
        let revisions = notifier.domain_revisions();

        assert_eq!(revisions.revision.library, 1);
        assert_eq!(revisions.revision.metadata, 1);
        assert_eq!(revisions.revision.stats, 1);
        assert_eq!(revisions.revision.assets, 1);

        let telemetry = notifier.observability().snapshot();
        assert_eq!(telemetry.invalidation_events.library, 1);
        assert_eq!(telemetry.invalidation_events.metadata, 1);
        assert_eq!(telemetry.invalidation_events.stats, 1);
        assert_eq!(telemetry.invalidation_events.assets, 1);
    }
}
