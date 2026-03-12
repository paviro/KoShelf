//! Domain update notifier used by runtime server mode.

use super::observability::RuntimeObservability;
use super::revisions::{
    DomainRevision, DomainRevisionState, DomainRevisionTracker, RevisionDomain,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::watch;

/// Payload broadcast to SSE consumers on every domain revision bump.
#[derive(Debug, Clone, Serialize)]
pub struct DomainUpdate {
    pub revision_epoch: String,
    pub revision: DomainRevision,
    pub domains: Vec<RevisionDomain>,
    pub generated_at: String,
}

#[derive(Debug)]
struct DomainUpdateNotifierInner {
    revision_tracker: DomainRevisionTracker,
    observability: RuntimeObservability,
    tx: watch::Sender<DomainUpdate>,
}

/// Publishes domain-scoped revision updates for SSE consumers.
#[derive(Debug, Clone)]
pub struct DomainUpdateNotifier {
    inner: Arc<DomainUpdateNotifierInner>,
}

impl DomainUpdateNotifier {
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
        let revision_epoch = revision_epoch.into();
        let initial_update = DomainUpdate {
            revision_epoch: revision_epoch.clone(),
            revision: DomainRevision::default(),
            domains: vec![
                RevisionDomain::Library,
                RevisionDomain::Metadata,
                RevisionDomain::Stats,
                RevisionDomain::Assets,
            ],
            generated_at: initial_generated_at.into(),
        };
        let (tx, _rx) = watch::channel(initial_update);

        Self {
            inner: Arc::new(DomainUpdateNotifierInner {
                revision_tracker: DomainRevisionTracker::new(revision_epoch),
                observability,
                tx,
            }),
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<DomainUpdate> {
        self.inner.tx.subscribe()
    }

    pub fn publish(&self, generated_at: impl Into<String>) -> DomainUpdate {
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
    ) -> DomainUpdate
    where
        I: IntoIterator<Item = RevisionDomain>,
    {
        let domains: Vec<_> = domains.into_iter().collect();
        self.inner
            .revision_tracker
            .bump_domains(domains.iter().copied());
        for &domain in &domains {
            self.inner.observability.record_invalidation_event(domain);
        }

        let state = self.inner.revision_tracker.snapshot_state();
        let update = DomainUpdate {
            revision_epoch: state.revision_epoch,
            revision: state.revision,
            domains,
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
    use super::DomainUpdateNotifier;
    use crate::runtime::{DomainRevision, RevisionDomain};

    #[test]
    fn publish_tracks_domain_revisions_and_affected_domains() {
        let notifier =
            DomainUpdateNotifier::new("serve_2026-03-11T11:00:00Z", "2026-03-11T11:00:00Z");

        assert_eq!(
            notifier.domain_revisions().revision,
            DomainRevision::default()
        );

        let first = notifier.publish_for_domains(
            "2026-03-11T11:01:00Z",
            [RevisionDomain::Library, RevisionDomain::Stats],
        );
        assert_eq!(first.revision_epoch, "serve_2026-03-11T11:00:00Z");
        assert_eq!(first.revision.library, 1);
        assert_eq!(first.revision.stats, 1);
        assert_eq!(first.revision.metadata, 0);
        assert_eq!(first.revision.assets, 0);
        assert_eq!(
            first.domains,
            vec![RevisionDomain::Library, RevisionDomain::Stats]
        );

        let second =
            notifier.publish_for_domains("2026-03-11T11:02:00Z", [RevisionDomain::Metadata]);
        assert_eq!(second.revision.metadata, 1);
        assert_eq!(second.domains, vec![RevisionDomain::Metadata]);

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
        let notifier = DomainUpdateNotifier::new("serve_epoch", "2026-03-11T11:00:00Z");

        let update = notifier.publish("2026-03-11T11:05:00Z");

        assert_eq!(update.revision.library, 1);
        assert_eq!(update.revision.metadata, 1);
        assert_eq!(update.revision.stats, 1);
        assert_eq!(update.revision.assets, 1);
        assert_eq!(
            update.domains,
            vec![
                RevisionDomain::Library,
                RevisionDomain::Metadata,
                RevisionDomain::Stats,
                RevisionDomain::Assets,
            ]
        );

        let telemetry = notifier.observability().snapshot();
        assert_eq!(telemetry.invalidation_events.library, 1);
        assert_eq!(telemetry.invalidation_events.metadata, 1);
        assert_eq!(telemetry.invalidation_events.stats, 1);
        assert_eq!(telemetry.invalidation_events.assets, 1);
    }

    #[test]
    fn initial_state_includes_all_domains() {
        let notifier = DomainUpdateNotifier::new("serve_epoch", "2026-03-11T11:00:00Z");

        let receiver = notifier.subscribe();
        let initial = receiver.borrow().clone();

        assert_eq!(initial.revision_epoch, "serve_epoch");
        assert_eq!(initial.revision, DomainRevision::default());
        assert_eq!(
            initial.domains,
            vec![
                RevisionDomain::Library,
                RevisionDomain::Metadata,
                RevisionDomain::Stats,
                RevisionDomain::Assets,
            ]
        );
        assert_eq!(initial.generated_at, "2026-03-11T11:00:00Z");
    }
}
