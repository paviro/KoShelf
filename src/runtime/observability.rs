//! Runtime observability counters.

use serde::Serialize;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SqliteRouteClass {
    List,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryDbRebuildReason {
    Corruption,
    SchemaMismatch,
    ParserRevision,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct RuntimeReconcileCounters {
    pub parsed: u64,
    pub unchanged: u64,
    pub added: u64,
    pub updated: u64,
    pub removed: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct RuntimeLatencySnapshot {
    pub samples: u64,
    pub total_ms: u64,
    pub average_ms: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct RuntimeDbRebuildCounters {
    pub corruption: u64,
    pub schema_mismatch: u64,
    pub parser_revision: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct RuntimeObservabilitySnapshot {
    pub startup_library_build_duration_ms: u64,
    pub reconcile: RuntimeReconcileCounters,
    pub watcher_queue_depth_current: u64,
    pub watcher_queue_depth_peak: u64,
    pub watcher_update_latency: RuntimeLatencySnapshot,
    pub sqlite_list_query_latency: RuntimeLatencySnapshot,
    pub sqlite_detail_query_latency: RuntimeLatencySnapshot,
    pub db_rebuilds: RuntimeDbRebuildCounters,
    pub invalidation_events: u64,
}

#[derive(Debug, Default)]
struct RuntimeObservabilityInner {
    startup_library_build_duration_ms: AtomicU64,
    reconcile_parsed: AtomicU64,
    reconcile_unchanged: AtomicU64,
    reconcile_added: AtomicU64,
    reconcile_updated: AtomicU64,
    reconcile_removed: AtomicU64,
    watcher_queue_depth_current: AtomicU64,
    watcher_queue_depth_peak: AtomicU64,
    watcher_update_latency_total_ms: AtomicU64,
    watcher_update_latency_samples: AtomicU64,
    sqlite_list_query_latency_total_ms: AtomicU64,
    sqlite_list_query_latency_samples: AtomicU64,
    sqlite_detail_query_latency_total_ms: AtomicU64,
    sqlite_detail_query_latency_samples: AtomicU64,
    db_rebuilds_corruption: AtomicU64,
    db_rebuilds_schema_mismatch: AtomicU64,
    db_rebuilds_parser_revision: AtomicU64,
    invalidation_events: AtomicU64,
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeObservability {
    inner: Arc<RuntimeObservabilityInner>,
}

impl RuntimeObservability {
    pub fn record_startup_library_build_duration(&self, duration: Duration) {
        self.inner
            .startup_library_build_duration_ms
            .store(duration_to_ms(duration), Ordering::Relaxed);
    }

    pub fn record_reconcile_batch(&self, counts: RuntimeReconcileCounters) {
        self.inner
            .reconcile_parsed
            .fetch_add(counts.parsed, Ordering::Relaxed);
        self.inner
            .reconcile_unchanged
            .fetch_add(counts.unchanged, Ordering::Relaxed);
        self.inner
            .reconcile_added
            .fetch_add(counts.added, Ordering::Relaxed);
        self.inner
            .reconcile_updated
            .fetch_add(counts.updated, Ordering::Relaxed);
        self.inner
            .reconcile_removed
            .fetch_add(counts.removed, Ordering::Relaxed);
    }

    pub fn set_watcher_queue_depth(&self, depth: u64) {
        self.inner
            .watcher_queue_depth_current
            .store(depth, Ordering::Relaxed);
        self.inner
            .watcher_queue_depth_peak
            .fetch_max(depth, Ordering::Relaxed);
    }

    pub fn record_watcher_update_latency(&self, duration: Duration) {
        self.inner
            .watcher_update_latency_total_ms
            .fetch_add(duration_to_ms(duration), Ordering::Relaxed);
        self.inner
            .watcher_update_latency_samples
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_sqlite_query_latency(&self, route: SqliteRouteClass, duration: Duration) {
        let duration_ms = duration_to_ms(duration);

        match route {
            SqliteRouteClass::List => {
                self.inner
                    .sqlite_list_query_latency_total_ms
                    .fetch_add(duration_ms, Ordering::Relaxed);
                self.inner
                    .sqlite_list_query_latency_samples
                    .fetch_add(1, Ordering::Relaxed);
            }
            SqliteRouteClass::Detail => {
                self.inner
                    .sqlite_detail_query_latency_total_ms
                    .fetch_add(duration_ms, Ordering::Relaxed);
                self.inner
                    .sqlite_detail_query_latency_samples
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn record_db_rebuild(&self, reason: LibraryDbRebuildReason) {
        match reason {
            LibraryDbRebuildReason::Corruption => {
                self.inner
                    .db_rebuilds_corruption
                    .fetch_add(1, Ordering::Relaxed);
            }
            LibraryDbRebuildReason::SchemaMismatch => {
                self.inner
                    .db_rebuilds_schema_mismatch
                    .fetch_add(1, Ordering::Relaxed);
            }
            LibraryDbRebuildReason::ParserRevision => {
                self.inner
                    .db_rebuilds_parser_revision
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn record_invalidation_event(&self) {
        self.inner
            .invalidation_events
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> RuntimeObservabilitySnapshot {
        let watcher_update_total = self
            .inner
            .watcher_update_latency_total_ms
            .load(Ordering::Relaxed);
        let watcher_update_samples = self
            .inner
            .watcher_update_latency_samples
            .load(Ordering::Relaxed);

        let sqlite_list_total = self
            .inner
            .sqlite_list_query_latency_total_ms
            .load(Ordering::Relaxed);
        let sqlite_list_samples = self
            .inner
            .sqlite_list_query_latency_samples
            .load(Ordering::Relaxed);

        let sqlite_detail_total = self
            .inner
            .sqlite_detail_query_latency_total_ms
            .load(Ordering::Relaxed);
        let sqlite_detail_samples = self
            .inner
            .sqlite_detail_query_latency_samples
            .load(Ordering::Relaxed);

        RuntimeObservabilitySnapshot {
            startup_library_build_duration_ms: self
                .inner
                .startup_library_build_duration_ms
                .load(Ordering::Relaxed),
            reconcile: RuntimeReconcileCounters {
                parsed: self.inner.reconcile_parsed.load(Ordering::Relaxed),
                unchanged: self.inner.reconcile_unchanged.load(Ordering::Relaxed),
                added: self.inner.reconcile_added.load(Ordering::Relaxed),
                updated: self.inner.reconcile_updated.load(Ordering::Relaxed),
                removed: self.inner.reconcile_removed.load(Ordering::Relaxed),
            },
            watcher_queue_depth_current: self
                .inner
                .watcher_queue_depth_current
                .load(Ordering::Relaxed),
            watcher_queue_depth_peak: self.inner.watcher_queue_depth_peak.load(Ordering::Relaxed),
            watcher_update_latency: latency_snapshot(watcher_update_total, watcher_update_samples),
            sqlite_list_query_latency: latency_snapshot(sqlite_list_total, sqlite_list_samples),
            sqlite_detail_query_latency: latency_snapshot(
                sqlite_detail_total,
                sqlite_detail_samples,
            ),
            db_rebuilds: RuntimeDbRebuildCounters {
                corruption: self.inner.db_rebuilds_corruption.load(Ordering::Relaxed),
                schema_mismatch: self
                    .inner
                    .db_rebuilds_schema_mismatch
                    .load(Ordering::Relaxed),
                parser_revision: self
                    .inner
                    .db_rebuilds_parser_revision
                    .load(Ordering::Relaxed),
            },
            invalidation_events: self.inner.invalidation_events.load(Ordering::Relaxed),
        }
    }
}

fn duration_to_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn latency_snapshot(total_ms: u64, samples: u64) -> RuntimeLatencySnapshot {
    let average_ms = if samples == 0 { 0 } else { total_ms / samples };

    RuntimeLatencySnapshot {
        samples,
        total_ms,
        average_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LibraryDbRebuildReason, RuntimeObservability, RuntimeReconcileCounters, SqliteRouteClass,
    };
    use std::time::Duration;

    #[test]
    fn records_startup_library_build_duration_and_reconcile_counts() {
        let observability = RuntimeObservability::default();

        observability.record_startup_library_build_duration(Duration::from_millis(1234));
        observability.record_reconcile_batch(RuntimeReconcileCounters {
            parsed: 10,
            unchanged: 7,
            added: 2,
            updated: 1,
            removed: 0,
        });
        observability.record_reconcile_batch(RuntimeReconcileCounters {
            parsed: 3,
            unchanged: 1,
            added: 0,
            updated: 1,
            removed: 1,
        });

        let snapshot = observability.snapshot();
        assert_eq!(snapshot.startup_library_build_duration_ms, 1234);
        assert_eq!(snapshot.reconcile.parsed, 13);
        assert_eq!(snapshot.reconcile.unchanged, 8);
        assert_eq!(snapshot.reconcile.added, 2);
        assert_eq!(snapshot.reconcile.updated, 2);
        assert_eq!(snapshot.reconcile.removed, 1);
    }

    #[test]
    fn tracks_watcher_depth_and_latency_snapshots() {
        let observability = RuntimeObservability::default();

        observability.set_watcher_queue_depth(3);
        observability.set_watcher_queue_depth(1);
        observability.record_watcher_update_latency(Duration::from_millis(250));
        observability.record_watcher_update_latency(Duration::from_millis(150));

        let snapshot = observability.snapshot();
        assert_eq!(snapshot.watcher_queue_depth_current, 1);
        assert_eq!(snapshot.watcher_queue_depth_peak, 3);
        assert_eq!(snapshot.watcher_update_latency.samples, 2);
        assert_eq!(snapshot.watcher_update_latency.total_ms, 400);
        assert_eq!(snapshot.watcher_update_latency.average_ms, 200);
    }

    #[test]
    fn tracks_query_latency_rebuild_reasons_and_invalidation_counts() {
        let observability = RuntimeObservability::default();

        observability
            .record_sqlite_query_latency(SqliteRouteClass::List, Duration::from_millis(40));
        observability
            .record_sqlite_query_latency(SqliteRouteClass::List, Duration::from_millis(20));
        observability
            .record_sqlite_query_latency(SqliteRouteClass::Detail, Duration::from_millis(90));

        observability.record_db_rebuild(LibraryDbRebuildReason::Corruption);
        observability.record_db_rebuild(LibraryDbRebuildReason::SchemaMismatch);
        observability.record_db_rebuild(LibraryDbRebuildReason::Corruption);

        observability.record_invalidation_event();
        observability.record_invalidation_event();

        let snapshot = observability.snapshot();
        assert_eq!(snapshot.sqlite_list_query_latency.samples, 2);
        assert_eq!(snapshot.sqlite_list_query_latency.total_ms, 60);
        assert_eq!(snapshot.sqlite_detail_query_latency.samples, 1);
        assert_eq!(snapshot.sqlite_detail_query_latency.total_ms, 90);
        assert_eq!(snapshot.db_rebuilds.corruption, 2);
        assert_eq!(snapshot.db_rebuilds.schema_mismatch, 1);
        assert_eq!(snapshot.db_rebuilds.parser_revision, 0);
        assert_eq!(snapshot.invalidation_events, 2);
    }
}
