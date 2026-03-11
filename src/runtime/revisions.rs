//! Domain-scoped revision tracking within a runtime epoch.

use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionDomain {
    Library,
    Metadata,
    Stats,
    Assets,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainRevision {
    pub library: u64,
    pub metadata: u64,
    pub stats: u64,
    pub assets: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainRevisionState {
    pub revision_epoch: String,
    pub revision: DomainRevision,
}

#[derive(Debug)]
struct DomainRevisionTrackerInner {
    revision_epoch: String,
    library: AtomicU64,
    metadata: AtomicU64,
    stats: AtomicU64,
    assets: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct DomainRevisionTracker {
    inner: Arc<DomainRevisionTrackerInner>,
}

impl DomainRevisionTracker {
    pub fn new(revision_epoch: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(DomainRevisionTrackerInner {
                revision_epoch: revision_epoch.into(),
                library: AtomicU64::new(0),
                metadata: AtomicU64::new(0),
                stats: AtomicU64::new(0),
                assets: AtomicU64::new(0),
            }),
        }
    }

    pub fn revision_epoch(&self) -> &str {
        &self.inner.revision_epoch
    }

    pub fn snapshot(&self) -> DomainRevision {
        DomainRevision {
            library: self.inner.library.load(Ordering::Relaxed),
            metadata: self.inner.metadata.load(Ordering::Relaxed),
            stats: self.inner.stats.load(Ordering::Relaxed),
            assets: self.inner.assets.load(Ordering::Relaxed),
        }
    }

    pub fn snapshot_state(&self) -> DomainRevisionState {
        DomainRevisionState {
            revision_epoch: self.inner.revision_epoch.clone(),
            revision: self.snapshot(),
        }
    }

    pub fn bump(&self, domain: RevisionDomain) -> DomainRevisionState {
        self.bump_counter(domain);
        self.snapshot_state()
    }

    pub fn bump_domains<I>(&self, domains: I) -> DomainRevisionState
    where
        I: IntoIterator<Item = RevisionDomain>,
    {
        for domain in domains {
            self.bump_counter(domain);
        }
        self.snapshot_state()
    }

    pub fn bump_all(&self) -> DomainRevisionState {
        self.bump_domains([
            RevisionDomain::Library,
            RevisionDomain::Metadata,
            RevisionDomain::Stats,
            RevisionDomain::Assets,
        ])
    }

    fn bump_counter(&self, domain: RevisionDomain) {
        match domain {
            RevisionDomain::Library => {
                self.inner.library.fetch_add(1, Ordering::Relaxed);
            }
            RevisionDomain::Metadata => {
                self.inner.metadata.fetch_add(1, Ordering::Relaxed);
            }
            RevisionDomain::Stats => {
                self.inner.stats.fetch_add(1, Ordering::Relaxed);
            }
            RevisionDomain::Assets => {
                self.inner.assets.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DomainRevision, DomainRevisionTracker, RevisionDomain};

    #[test]
    fn tracker_starts_with_zeroed_domain_revisions() {
        let tracker = DomainRevisionTracker::new("serve_2026-03-11T11:00:00Z");

        assert_eq!(tracker.revision_epoch(), "serve_2026-03-11T11:00:00Z");
        assert_eq!(tracker.snapshot(), DomainRevision::default());
    }

    #[test]
    fn tracker_bumps_only_selected_domain() {
        let tracker = DomainRevisionTracker::new("serve_epoch");

        let state = tracker.bump(RevisionDomain::Metadata);

        assert_eq!(state.revision_epoch, "serve_epoch");
        assert_eq!(state.revision.library, 0);
        assert_eq!(state.revision.metadata, 1);
        assert_eq!(state.revision.stats, 0);
        assert_eq!(state.revision.assets, 0);
    }

    #[test]
    fn tracker_bump_all_increments_every_domain() {
        let tracker = DomainRevisionTracker::new("serve_epoch");

        let state = tracker.bump_all();

        assert_eq!(state.revision.library, 1);
        assert_eq!(state.revision.metadata, 1);
        assert_eq!(state.revision.stats, 1);
        assert_eq!(state.revision.assets, 1);
    }
}
