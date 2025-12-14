use crate::config::SiteConfig;
use super::scanner::MetadataLocation;
use crate::models::LibraryItemFormat;
use crate::site_generator::SiteGenerator;
use crate::server::version::SharedVersionNotifier;
use anyhow::Result;
use log::{debug, info, warn};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

pub struct FileWatcher {
    config: SiteConfig,
    version_notifier: Option<SharedVersionNotifier>,
}

impl std::ops::Deref for FileWatcher {
    type Target = SiteConfig;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl FileWatcher {
    pub fn new(config: SiteConfig, version_notifier: Option<SharedVersionNotifier>) -> Self {
        Self {
            config,
            version_notifier,
        }
    }

    pub async fn run(self) -> Result<()> {
        let (file_tx, mut file_rx) = mpsc::unbounded_channel();
        let (rebuild_tx, mut rebuild_rx) = mpsc::unbounded_channel::<()>();

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

        // Clone the config and version notifier for the rebuild task
        let config_clone = self.config.clone();
        let version_notifier_clone = self.version_notifier.clone();

        // Spawn delayed rebuild task
        // NOTE: Site generation uses non-Send types (e.g. mlua::Lua, Rc-based translations),
        // so this rebuild loop must not be spawned onto the multithreaded executor.
        let rebuild_task = tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                let rebuild_delay = Duration::from_secs(10); // Wait 10 seconds after last event

                while (rebuild_rx.recv().await).is_some() {
                    // Wait for the delay period to debounce multiple events
                    sleep(rebuild_delay).await;

                    // Drain any additional events that came in during the delay
                    while rebuild_rx.try_recv().is_ok() {}

                    info!("Starting delayed site rebuild after quiet period");

                    // Create new site generator and regenerate everything
                    let site_generator = SiteGenerator::new(config_clone.clone());

                    match site_generator.generate().await {
                        Ok(_) => {
                            info!("Delayed site rebuild completed successfully");

                            // Notify long-polling clients that a new version is available
                            if let Some(ref notifier) = version_notifier_clone {
                                let version = chrono::Local::now().to_rfc3339();
                                notifier.notify(version);
                            }
                        }
                        Err(e) => warn!("Failed to rebuild site: {}", e),
                    }
                }
            })
        });

        // Main file event processing loop
        while let Some(event) = file_rx.recv().await {
            if self.should_process_event(&event) {
                // Log what triggered the rebuild
                self.log_file_event(&event);

                // Just queue a rebuild - no need to track individual file changes anymore
                let _ = rebuild_tx.send(());
            }
        }

        rebuild_task.abort();
        Ok(())
    }

    fn should_process_event(&self, event: &Event) -> bool {
        match &event.kind {
            // Only process actual file changes, not access events
            EventKind::Create(_) | EventKind::Remove(_) => self.event_affects_relevant_files(event),
            EventKind::Modify(modify_kind) => {
                // Only process content modifications, not metadata or access
                use notify::event::ModifyKind;
                match modify_kind {
                    ModifyKind::Data(_) => self.event_affects_relevant_files(event),
                    ModifyKind::Name(_) => true, // Renames are important
                    ModifyKind::Any => {
                        // For Any modifications, we need to be more careful
                        // Log these for debugging but still process them
                        debug!("Processing Modify(Any) event: {:?}", event);
                        self.event_affects_relevant_files(event)
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

    fn event_affects_relevant_files(&self, event: &Event) -> bool {
        event.paths.iter().any(|path| {
            let filename = path.file_name().and_then(|s| s.to_str());

            // Check for library items using LibraryItemFormat (handles .epub, .fb2, .fb2.zip, .cbz, .cbr)
            if LibraryItemFormat::from_path(path).is_some() {
                return true;
            }

            // Check for metadata files
            if let Some(filename) = filename
                && LibraryItemFormat::is_metadata_file(filename) {
                    return true;
                }

            // Check for .sdr directories (KoReader metadata folders)
            if let Some(filename) = filename
                && filename.ends_with(".sdr") {
                    return true;
                }

            // Check for statistics database files
            if let Some(ref stats_path) = self.statistics_db_path
                && path == stats_path {
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
                && LibraryItemFormat::is_metadata_file(filename) {
                    info!("Metadata file {}: {:?}", action, path);
                }

            // Check .sdr directories
            if let Some(filename) = filename
                && filename.ends_with(".sdr") {
                    info!("KoReader metadata directory {}: {:?}", action, path);
                }

            // Check statistics database
            if let Some(ref stats_path) = self.statistics_db_path
                && path == stats_path {
                    info!("Statistics database {}: {:?}", action, path);
                }
        }
    }
}
