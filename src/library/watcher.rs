use super::scanner::MetadataLocation;
use crate::config::SiteConfig;
use crate::domain::library::{LibraryBuildMode, LibraryBuildPipeline};
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::library::scan_library;
use crate::models::LibraryItemFormat;
use crate::runtime::{
    DomainUpdateNotifier, RevisionDomain, RuntimeObservability, SharedReadingDataStore,
    SharedSnapshotStore,
};
use crate::snapshot_builder::SnapshotBuilder;
use anyhow::Result;
use log::{debug, info, warn};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

pub struct FileWatcher {
    config: SiteConfig,
    snapshot_store: Option<SharedSnapshotStore>,
    reading_data_store: Option<SharedReadingDataStore>,
    update_notifier: Option<DomainUpdateNotifier>,
    library_repo: Option<LibraryRepository>,
    observability: RuntimeObservability,
}

impl std::ops::Deref for FileWatcher {
    type Target = SiteConfig;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

fn decrement_pending_rebuilds(counter: &AtomicU64) -> u64 {
    let mut current = counter.load(Ordering::Relaxed);

    loop {
        if current == 0 {
            return 0;
        }

        match counter.compare_exchange_weak(
            current,
            current - 1,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return current - 1,
            Err(observed) => current = observed,
        }
    }
}

impl FileWatcher {
    pub fn new(
        config: SiteConfig,
        snapshot_store: Option<SharedSnapshotStore>,
        reading_data_store: Option<SharedReadingDataStore>,
        update_notifier: Option<DomainUpdateNotifier>,
        library_repo: Option<LibraryRepository>,
        observability: RuntimeObservability,
    ) -> Self {
        Self {
            config,
            snapshot_store,
            reading_data_store,
            update_notifier,
            library_repo,
            observability,
        }
    }

    pub async fn run(self) -> Result<()> {
        let (file_tx, mut file_rx) = mpsc::unbounded_channel();
        let (rebuild_tx, mut rebuild_rx) = mpsc::unbounded_channel::<Vec<RevisionDomain>>();
        let pending_rebuilds = Arc::new(AtomicU64::new(0));
        self.observability.set_watcher_queue_depth(0);

        // Set up file watcher
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| match result {
                Ok(event) => {
                    if let Err(e) = file_tx.send(event) {
                        warn!("Failed to send file event: {}", e);
                    }
                }
                Err(e) => warn!("File watcher error: {}", e),
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )?;

        // Watch all library paths if provided
        for library_path in &self.library_paths {
            watcher.watch(library_path, RecursiveMode::Recursive)?;
            info!(
                "File watcher started for library directory: {:?}",
                library_path
            );
        }

        // Watch external metadata locations if configured
        match &self.metadata_location {
            MetadataLocation::DocSettings(path) => {
                watcher.watch(path, RecursiveMode::Recursive)?;
                info!("File watcher started for docsettings directory: {:?}", path);
            }
            MetadataLocation::HashDocSettings(path) => {
                watcher.watch(path, RecursiveMode::Recursive)?;
                info!(
                    "File watcher started for hashdocsettings directory: {:?}",
                    path
                );
            }
            MetadataLocation::InBookFolder => {
                // Already watching books_path recursively
            }
        }

        // Also watch the statistics database if provided
        if let Some(ref stats_path) = self.statistics_db_path
            && stats_path.exists()
        {
            // Watch the parent directory of the statistics database
            if let Some(parent) = stats_path.parent() {
                watcher.watch(parent, RecursiveMode::NonRecursive)?;
                info!(
                    "File watcher started for statistics database: {:?}",
                    stats_path
                );
            }
        }

        // Clone the config for the rebuild task
        let config_clone = self.config.clone();
        let snapshot_store_clone = self.snapshot_store.clone();
        let reading_data_store_clone = self.reading_data_store.clone();
        let update_notifier_clone = self.update_notifier.clone();
        let library_repo_clone = self.library_repo.clone();
        let observability_clone = self.observability.clone();
        let pending_rebuilds_clone = pending_rebuilds.clone();

        // Spawn delayed rebuild task.
        // NOTE: Snapshot building uses non-Send types (e.g. mlua::Lua, Rc-based translations),
        // so this rebuild loop must not be spawned onto the multithreaded executor.
        let rebuild_task = tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                let rebuild_delay = Duration::from_secs(10); // Wait 10 seconds after last event

                while let Some(initial_domains) = rebuild_rx.recv().await {
                    let current_depth = decrement_pending_rebuilds(pending_rebuilds_clone.as_ref());
                    observability_clone.set_watcher_queue_depth(current_depth);
                    let queued_at = Instant::now();

                    // Accumulate affected domains across the debounce window.
                    let mut affected_domains: HashSet<RevisionDomain> =
                        initial_domains.into_iter().collect();

                    // Wait for the delay period to debounce multiple events
                    sleep(rebuild_delay).await;

                    // Drain any additional events that came in during the delay
                    while let Ok(domains) = rebuild_rx.try_recv() {
                        let current_depth =
                            decrement_pending_rebuilds(pending_rebuilds_clone.as_ref());
                        observability_clone.set_watcher_queue_depth(current_depth);
                        affected_domains.extend(domains);
                    }

                    info!("Starting delayed snapshot refresh after quiet period");

                    // Create a fresh snapshot builder and recompute all payloads.
                    let snapshot_builder = SnapshotBuilder::new(config_clone.clone());

                    match snapshot_builder.refresh_snapshot().await {
                        Ok(result) => {
                            let generated_at = result
                                .snapshot
                                .generated_at()
                                .map(str::to_owned)
                                .unwrap_or_else(|| config_clone.time_config.now_rfc3339());

                            if !config_clone.is_internal_server
                                && let Err(error) = result
                                    .snapshot
                                    .write_to_data_dir(&config_clone.output_dir.join("data"))
                            {
                                warn!("Failed to write static contract data: {}", error);
                                continue;
                            }

                            if let Some(ref snapshot_store) = snapshot_store_clone {
                                snapshot_store.replace(result.snapshot);
                            }

                            if let Some(ref reading_data_store) = reading_data_store_clone
                                && let Some(rd) = result.reading_data
                            {
                                reading_data_store.replace(rd);
                            }

                            if let Some(ref update_notifier) = update_notifier_clone {
                                let update = update_notifier.publish_for_domains(
                                    generated_at,
                                    affected_domains.iter().copied(),
                                );
                                info!(
                                    "Published data_changed event for domains {:?}, revision {:?}",
                                    update.domains, update.revision,
                                );
                            }

                            info!("Delayed snapshot refresh completed successfully");
                        }
                        Err(e) => warn!("Failed to refresh snapshot: {}", e),
                    }

                    // Run incremental library DB update if a repository is available.
                    if let Some(ref repo) = library_repo_clone {
                        let db_start = Instant::now();
                        let scanned = if !config_clone.library_paths.is_empty() {
                            match scan_library(
                                &config_clone.library_paths,
                                &config_clone.metadata_location,
                            )
                            .await
                            {
                                Ok((items, _md5s)) => items,
                                Err(e) => {
                                    warn!("Watcher library scan failed: {}", e);
                                    continue;
                                }
                            }
                        } else {
                            Vec::new()
                        };

                        let pipeline = LibraryBuildPipeline::new(
                            repo,
                            config_clone.include_unread,
                            config_clone.use_stable_page_metadata,
                            &config_clone.time_config,
                        );

                        match pipeline
                            .build(LibraryBuildMode::Incremental, scanned)
                            .await
                        {
                            Ok(result) => {
                                info!(
                                    "Watcher library DB update completed in {} ms: {} scanned, {} upserted, {} removed, {} collisions",
                                    db_start.elapsed().as_millis(),
                                    result.scanned_files,
                                    result.upserted_items,
                                    result.removed_items,
                                    result.collision_count,
                                );
                            }
                            Err(e) => warn!("Watcher library DB update failed: {}", e),
                        }
                    }

                    observability_clone.record_watcher_update_latency(queued_at.elapsed());
                }
            })
        });

        // Main file event processing loop
        while let Some(event) = file_rx.recv().await {
            let domains = self.classify_event_domains(&event);
            if !domains.is_empty() {
                self.log_file_event(&event);

                if rebuild_tx.send(domains).is_ok() {
                    let queued_depth = pending_rebuilds
                        .fetch_add(1, Ordering::Relaxed)
                        .saturating_add(1);
                    self.observability.set_watcher_queue_depth(queued_depth);
                }
            }
        }

        rebuild_task.abort();
        Ok(())
    }

    /// Classify a file-system event into the revision domains it affects.
    ///
    /// Returns an empty vec for events that should be ignored.
    fn classify_event_domains(&self, event: &Event) -> Vec<RevisionDomain> {
        let dominated_domains = match &event.kind {
            EventKind::Create(_) | EventKind::Remove(_) => {
                self.classify_paths_to_domains(&event.paths)
            }
            EventKind::Modify(modify_kind) => {
                use notify::event::ModifyKind;
                match modify_kind {
                    ModifyKind::Data(_) => self.classify_paths_to_domains(&event.paths),
                    ModifyKind::Name(_) => {
                        // Renames can affect library identity, metadata, and covers.
                        if self.paths_contain_relevant_files(&event.paths) {
                            vec![
                                RevisionDomain::Library,
                                RevisionDomain::Metadata,
                                RevisionDomain::Assets,
                            ]
                        } else {
                            vec![]
                        }
                    }
                    ModifyKind::Any => {
                        debug!("Processing Modify(Any) event: {:?}", event);
                        self.classify_paths_to_domains(&event.paths)
                    }
                    _ => {
                        debug!("Ignoring modify event: {:?}", modify_kind);
                        vec![]
                    }
                }
            }
            _ => {
                debug!("Ignoring event kind: {:?}", event.kind);
                vec![]
            }
        };

        // Deduplicate while preserving deterministic order.
        let mut seen = HashSet::new();
        dominated_domains
            .into_iter()
            .filter(|d| seen.insert(*d))
            .collect()
    }

    fn classify_paths_to_domains(&self, paths: &[std::path::PathBuf]) -> Vec<RevisionDomain> {
        let mut domains = Vec::new();
        for path in paths {
            let filename = path.file_name().and_then(|s| s.to_str());

            // Library items → Library + Assets (cover may change).
            if LibraryItemFormat::from_path(path).is_some() {
                domains.push(RevisionDomain::Library);
                domains.push(RevisionDomain::Assets);
            }

            // Metadata sidecar files → Metadata.
            if let Some(filename) = filename
                && LibraryItemFormat::is_metadata_file(filename)
            {
                domains.push(RevisionDomain::Metadata);
            }

            // KoReader .sdr directories → Metadata.
            if let Some(filename) = filename
                && filename.ends_with(".sdr")
            {
                domains.push(RevisionDomain::Metadata);
            }

            // Statistics database → Stats.
            if let Some(ref stats_path) = self.statistics_db_path
                && path == stats_path
            {
                domains.push(RevisionDomain::Stats);
            }
        }
        domains
    }

    fn paths_contain_relevant_files(&self, paths: &[std::path::PathBuf]) -> bool {
        paths.iter().any(|path| {
            let filename = path.file_name().and_then(|s| s.to_str());

            if LibraryItemFormat::from_path(path).is_some() {
                return true;
            }
            if let Some(filename) = filename
                && LibraryItemFormat::is_metadata_file(filename)
            {
                return true;
            }
            if let Some(filename) = filename
                && filename.ends_with(".sdr")
            {
                return true;
            }
            if let Some(ref stats_path) = self.statistics_db_path
                && path == stats_path
            {
                return true;
            }
            false
        })
    }

    fn log_file_event(&self, event: &Event) {
        for path in &event.paths {
            let filename = path.file_name().and_then(|s| s.to_str());
            let action = match &event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => "modified",
                EventKind::Remove(_) => "removed",
                _ => continue,
            };

            // Check for library items using LibraryItemFormat
            if let Some(format) = LibraryItemFormat::from_path(path) {
                info!("{:?} file {}: {:?}", format, action, path);
            }

            // Check metadata files
            if let Some(filename) = filename
                && LibraryItemFormat::is_metadata_file(filename)
            {
                info!("Metadata file {}: {:?}", action, path);
            }

            // Check .sdr directories
            if let Some(filename) = filename
                && filename.ends_with(".sdr")
            {
                info!("KoReader metadata directory {}: {:?}", action, path);
            }

            // Check statistics database
            if let Some(ref stats_path) = self.statistics_db_path
                && path == stats_path
            {
                info!("Statistics database {}: {:?}", action, path);
            }
        }
    }
}
