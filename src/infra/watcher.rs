use super::scanner::MetadataLocation;
use crate::config::SiteConfig;
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::infra::stores::{SharedReadingDataStore, SharedSiteStore, UpdateNotifier};
use crate::models::LibraryItemFormat;
use anyhow::Result;
use log::{debug, info, warn};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Payload sent through the rebuild channel: raw file paths that triggered the rebuild.
struct WatcherRebuildEvent {
    paths: Vec<PathBuf>,
}

pub struct FileWatcher {
    config: SiteConfig,
    site_store: Option<SharedSiteStore>,
    reading_data_store: Option<SharedReadingDataStore>,
    update_notifier: Option<UpdateNotifier>,
    library_repo: Option<LibraryRepository>,
}

impl std::ops::Deref for FileWatcher {
    type Target = SiteConfig;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl FileWatcher {
    pub fn new(
        config: SiteConfig,
        site_store: Option<SharedSiteStore>,
        reading_data_store: Option<SharedReadingDataStore>,
        update_notifier: Option<UpdateNotifier>,
        library_repo: Option<LibraryRepository>,
    ) -> Self {
        Self {
            config,
            site_store,
            reading_data_store,
            update_notifier,
            library_repo,
        }
    }

    pub async fn run(self) -> Result<()> {
        let (file_tx, mut file_rx) = mpsc::unbounded_channel();
        let (rebuild_tx, mut rebuild_rx) = mpsc::unbounded_channel::<WatcherRebuildEvent>();

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
            && let Some(parent) = stats_path.parent()
        {
            watcher.watch(parent, RecursiveMode::NonRecursive)?;
            info!(
                "File watcher started for statistics database: {:?}",
                stats_path
            );
        }

        // Clone dependencies for the rebuild task
        let config_clone = self.config.clone();
        let site_store_clone = self.site_store.clone();
        let reading_data_store_clone = self.reading_data_store.clone();
        let update_notifier_clone = self.update_notifier.clone();
        let library_repo_clone = self.library_repo.clone();

        // Spawn delayed rebuild task.
        // NOTE: Statistics loading uses non-Send types (e.g. mlua::Lua, Rc-based translations),
        // so this rebuild loop must not be spawned onto the multithreaded executor.
        let rebuild_task = tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                let rebuild_delay = Duration::from_secs(1);

                while let Some(initial_event) = rebuild_rx.recv().await {
                    // Accumulate paths across the debounce window.
                    let mut accumulated_paths: HashSet<PathBuf> =
                        initial_event.paths.into_iter().collect();

                    sleep(rebuild_delay).await;

                    // Drain any additional events that came in during the delay
                    while let Ok(event) = rebuild_rx.try_recv() {
                        accumulated_paths.extend(event.paths);
                    }

                    let result = if let Some(ref repo) = library_repo_clone {
                        crate::runtime::rebuild::targeted_rebuild(
                            accumulated_paths,
                            &config_clone,
                            repo,
                            site_store_clone.as_ref(),
                            reading_data_store_clone.as_ref(),
                            update_notifier_clone.as_ref(),
                        )
                        .await
                    } else {
                        crate::runtime::rebuild::full_rebuild(
                            &config_clone,
                            site_store_clone.as_ref(),
                            reading_data_store_clone.as_ref(),
                            update_notifier_clone.as_ref(),
                        )
                        .await
                    };

                    if let Err(e) = result {
                        warn!("Rebuild failed: {}", e);
                    }
                }
            })
        });

        // Main file event processing loop
        while let Some(event) = file_rx.recv().await {
            if self.is_relevant_event(&event) {
                self.log_file_event(&event);

                let rebuild_event = WatcherRebuildEvent { paths: event.paths };

                let _ = rebuild_tx.send(rebuild_event);
            }
        }

        rebuild_task.abort();
        Ok(())
    }

    /// Check whether a file-system event contains changes we care about.
    fn is_relevant_event(&self, event: &Event) -> bool {
        match &event.kind {
            EventKind::Create(_) | EventKind::Remove(_) => {
                self.paths_contain_relevant_files(&event.paths)
            }
            EventKind::Modify(modify_kind) => {
                use notify::event::ModifyKind;
                match modify_kind {
                    ModifyKind::Data(_) | ModifyKind::Name(_) | ModifyKind::Any => {
                        self.paths_contain_relevant_files(&event.paths)
                    }
                    _ => {
                        debug!("Ignoring modify event: {:?}", modify_kind);
                        false
                    }
                }
            }
            _ => {
                debug!("Ignoring event kind: {:?}", event.kind);
                false
            }
        }
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

            if let Some(format) = LibraryItemFormat::from_path(path) {
                info!("{:?} file {}: {:?}", format, action, path);
            }

            if let Some(filename) = filename
                && LibraryItemFormat::is_metadata_file(filename)
            {
                info!("Metadata file {}: {:?}", action, path);
            }

            if let Some(filename) = filename
                && filename.ends_with(".sdr")
            {
                info!("KoReader metadata directory {}: {:?}", action, path);
            }

            if let Some(ref stats_path) = self.statistics_db_path
                && path == stats_path
            {
                info!("Statistics database {}: {:?}", action, path);
            }
        }
    }
}
