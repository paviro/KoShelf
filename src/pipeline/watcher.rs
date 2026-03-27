use crate::app::config::SiteConfig;
use crate::pipeline::rebuild::rebuild;
use crate::server::RecentWrites;
use crate::shelf::models::LibraryItemFormat;
use crate::source::scanner::MetadataLocation;
use crate::store::memory::{SharedReadingDataStore, SharedSiteStore, UpdateNotifier};
use crate::store::sqlite::repo::LibraryRepository;
use anyhow::Result;
use log::{debug, info, warn};
use notify::event::ModifyKind;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::timeout;

/// Events for paths written by the app within this window are suppressed.
/// Must exceed the poll interval (1 s) to cover the gap between `mark_written()`
/// and the arrival of the corresponding filesystem event.
const SELF_WRITE_SUPPRESSION_WINDOW: Duration = Duration::from_secs(3);

/// Watches library paths and statistics DB for changes, triggering debounced rebuilds.
pub struct FileWatcher {
    config: SiteConfig,
    site_store: Option<SharedSiteStore>,
    reading_data_store: Option<SharedReadingDataStore>,
    update_notifier: Option<UpdateNotifier>,
    library_repo: Option<LibraryRepository>,
    recent_writes: Option<RecentWrites>,
}

impl Deref for FileWatcher {
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
        recent_writes: Option<RecentWrites>,
    ) -> Self {
        Self {
            config,
            site_store,
            reading_data_store,
            update_notifier,
            library_repo,
            recent_writes,
        }
    }

    /// Start watching and processing file changes. Blocks until an error occurs.
    pub async fn run(self) -> Result<()> {
        let (file_tx, mut file_rx) = mpsc::unbounded_channel();
        let (rebuild_tx, mut rebuild_rx) = mpsc::unbounded_channel::<Vec<PathBuf>>();

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

        let mut watched: Vec<String> = Vec::new();

        for library_path in &self.library_paths {
            watcher.watch(library_path, RecursiveMode::Recursive)?;
            watched.push(format!("{}", library_path.display()));
        }

        match &self.metadata_location {
            MetadataLocation::DocSettings(path) => {
                watcher.watch(path, RecursiveMode::Recursive)?;
                watched.push(format!("{}", path.display()));
            }
            MetadataLocation::HashDocSettings(path) => {
                watcher.watch(path, RecursiveMode::Recursive)?;
                watched.push(format!("{}", path.display()));
            }
            MetadataLocation::InBookFolder => {
                // Already watching books_path recursively
            }
        }

        if let Some(ref stats_path) = self.statistics_db_path
            && stats_path.exists()
            && let Some(parent) = stats_path.parent()
        {
            watcher.watch(parent, RecursiveMode::NonRecursive)?;
            watched.push(format!("{}", stats_path.display()));
        }

        info!(
            "File watcher started for {} paths: {}",
            watched.len(),
            collapse_paths(&watched)
        );

        let config_clone = self.config.clone();
        let site_store_clone = self.site_store.clone();
        let reading_data_store_clone = self.reading_data_store.clone();
        let update_notifier_clone = self.update_notifier.clone();
        let library_repo_clone = self.library_repo.clone();

        // NOTE: Statistics loading uses non-Send types (e.g. mlua::Lua, Rc-based translations),
        // so this rebuild loop must not be spawned onto the multithreaded executor.
        let rebuild_task = tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                let settle = Duration::from_secs(2);
                let max_wait = Duration::from_secs(30);

                while let Some(initial_paths) = rebuild_rx.recv().await {
                    // Accumulate paths, waiting for events to settle before rebuilding.
                    let mut accumulated_paths: HashSet<PathBuf> =
                        initial_paths.into_iter().collect();
                    let deadline = tokio::time::Instant::now() + max_wait;

                    loop {
                        let remaining = deadline - tokio::time::Instant::now();
                        let wait = settle.min(remaining);

                        if wait.is_zero() {
                            break;
                        }

                        match timeout(wait, rebuild_rx.recv()).await {
                            Ok(Some(paths)) => accumulated_paths.extend(paths),
                            Ok(None) => return, // channel closed
                            Err(_) => break,    // settled — no events for the window
                        }
                    }

                    log_accumulated_paths(
                        &accumulated_paths,
                        config_clone.statistics_db_path.as_deref(),
                    );

                    let result = if let Some(ref repo) = library_repo_clone {
                        rebuild(
                            accumulated_paths,
                            &config_clone,
                            repo,
                            site_store_clone.as_ref(),
                            reading_data_store_clone.as_ref(),
                            update_notifier_clone.as_ref(),
                        )
                        .await
                    } else {
                        warn!(
                            "No library repository available for rebuild — this should not happen"
                        );
                        continue;
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
                // Filter out paths recently written by our own write handlers.
                let paths = self.filter_recent_writes(event.paths);
                if paths.is_empty() {
                    continue;
                }

                let _ = rebuild_tx.send(paths);
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
            EventKind::Modify(modify_kind) => match modify_kind {
                ModifyKind::Data(_) | ModifyKind::Name(_) | ModifyKind::Any => {
                    self.paths_contain_relevant_files(&event.paths)
                }
                _ => {
                    debug!("Ignoring modify event: {:?}", modify_kind);
                    false
                }
            },
            _ => {
                debug!("Ignoring event kind: {:?}", event.kind);
                false
            }
        }
    }

    fn paths_contain_relevant_files(&self, paths: &[PathBuf]) -> bool {
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

    /// Remove paths that were recently written by a write handler.
    fn filter_recent_writes(&self, paths: Vec<PathBuf>) -> Vec<PathBuf> {
        let Some(ref recent) = self.recent_writes else {
            return paths;
        };

        let now = Instant::now();

        paths
            .into_iter()
            .filter(|p| {
                if let Some(entry) = recent.get(p) {
                    let elapsed = now.duration_since(*entry.value());
                    if elapsed < SELF_WRITE_SUPPRESSION_WINDOW {
                        debug!(
                            "Suppressing self-triggered event for {:?} (written {:.1}s ago)",
                            p,
                            elapsed.as_secs_f64()
                        );
                        return false;
                    }
                    // Expired — clean up, but only if the timestamp hasn't
                    // been refreshed by a concurrent `mark_written()`.
                    drop(entry);
                    recent.remove_if(p, |_, ts| {
                        now.duration_since(*ts) >= SELF_WRITE_SUPPRESSION_WINDOW
                    });
                }
                true
            })
            .collect()
    }
}

/// Format a list of paths by factoring out the longest common directory prefix.
/// e.g. ["/a/b/Books", "/a/b/Comics", "/a/b/Stats/db"] → "/a/b/{Books, Comics, Stats/db}"
fn collapse_paths(paths: &[String]) -> String {
    if paths.len() <= 1 {
        return paths.join(", ");
    }

    let first = Path::new(&paths[0]);
    let prefix: PathBuf = first
        .ancestors()
        .find(|ancestor| {
            ancestor.as_os_str() != first.as_os_str()
                && paths.iter().all(|p| Path::new(p).starts_with(ancestor))
        })
        .unwrap_or(Path::new(""))
        .to_path_buf();

    if prefix == Path::new("") || prefix == Path::new("/") {
        return paths.join(", ");
    }

    let suffixes: Vec<&str> = paths
        .iter()
        .map(|p| p.strip_prefix(prefix.to_str().unwrap_or("")).unwrap_or(p))
        .map(|s| s.strip_prefix('/').unwrap_or(s))
        .collect();

    format!("{}/{{{}}}", prefix.display(), suffixes.join(", "))
}

/// Log accumulated unique paths once before a rebuild.
fn log_accumulated_paths(paths: &HashSet<PathBuf>, statistics_db_path: Option<&std::path::Path>) {
    for path in paths {
        let filename = path.file_name().and_then(|s| s.to_str());

        if let Some(format) = LibraryItemFormat::from_path(path) {
            info!("{:?} file changed: {:?}", format, path);
        } else if let Some(filename) = filename
            && LibraryItemFormat::is_metadata_file(filename)
        {
            info!("Metadata file changed: {:?}", path);
        } else if let Some(filename) = filename
            && filename.ends_with(".sdr")
        {
            info!("KoReader metadata directory changed: {:?}", path);
        } else if let Some(stats_path) = statistics_db_path
            && path == stats_path
        {
            info!("Statistics database changed: {:?}", path);
        }
    }
}
