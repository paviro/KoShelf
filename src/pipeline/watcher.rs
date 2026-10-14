use crate::app::config::SiteConfig;
use crate::pipeline::rebuild::rebuild;
use crate::shelf::models::LibraryItemFormat;
use crate::source::sqlite_snapshot::is_sqlite_db_or_companion;
use crate::store::memory::{SharedReadingDataStore, SharedSiteStore, UpdateNotifier};
use crate::store::sqlite::repo::LibraryRepository;
use anyhow::Result;
use log::{debug, info, warn};
use notify::event::ModifyKind;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

/// Watches library paths and statistics DB for changes, triggering debounced rebuilds.
pub struct FileWatcher {
    config: SiteConfig,
    site_store: Option<SharedSiteStore>,
    reading_data_store: Option<SharedReadingDataStore>,
    update_notifier: Option<UpdateNotifier>,
    library_repo: Option<LibraryRepository>,
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
    ) -> Self {
        Self {
            config,
            site_store,
            reading_data_store,
            update_notifier,
            library_repo,
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

        let mut metadata_watches = HashMap::new();
        refresh_metadata_watches(
            &mut watcher,
            &self.metadata_location.central_paths(),
            &self.library_paths,
            &mut metadata_watches,
        );
        watched.extend(
            self.metadata_location
                .central_paths()
                .iter()
                .map(|p| p.display().to_string()),
        );

        let mut watched_db_parents: HashSet<PathBuf> = HashSet::new();
        for stats_path in &self.statistics_db_paths {
            if stats_path.exists()
                && let Some(parent) = stats_path.parent()
            {
                if watched_db_parents.insert(parent.to_path_buf()) {
                    watcher.watch(parent, RecursiveMode::NonRecursive)?;
                }
                watched.push(format!("{}", stats_path.display()));
            }
        }

        if let Some(ref kobo_db_path) = self.kobo_db_path
            && kobo_db_path.exists()
            && let Some(parent) = kobo_db_path.parent()
        {
            if watched_db_parents.insert(parent.to_path_buf()) {
                watcher.watch(parent, RecursiveMode::NonRecursive)?;
            }
            watched.push(format!("{}", kobo_db_path.display()));
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
                        &config_clone.statistics_db_paths,
                        config_clone.kobo_db_path.as_deref(),
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

        // Retry unavailable roots even when permissions/mount changes produce no event.
        let mut retry_watches = tokio::time::interval(Duration::from_secs(5));
        loop {
            let event = tokio::select! {
                event = file_rx.recv() => match event {
                    Some(event) => event,
                    None => break,
                },
                _ = retry_watches.tick() => {
                    let restored = refresh_metadata_watches(
                        &mut watcher, &self.metadata_location.central_paths(),
                        &self.library_paths, &mut metadata_watches,
                    );
                    if !restored.is_empty() {
                        let _ = rebuild_tx.send(restored);
                    }
                    continue;
                }
            };
            if matches!(
                event.kind,
                EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(_))
            ) {
                let invalidated: Vec<_> = metadata_watches
                    .keys()
                    .filter(|watched| event.paths.iter().any(|path| watched.starts_with(path)))
                    .cloned()
                    .collect();
                for path in invalidated {
                    let _ = watcher.unwatch(&path);
                    metadata_watches.remove(&path);
                }
            }
            refresh_metadata_watches(
                &mut watcher,
                &self.metadata_location.central_paths(),
                &self.library_paths,
                &mut metadata_watches,
            );
            if self.is_relevant_event(&event) {
                // Our own writes also require reevaluation of competing originals.
                let paths = event.paths;
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
                ModifyKind::Data(_)
                | ModifyKind::Name(_)
                | ModifyKind::Metadata(_)
                | ModifyKind::Any => self.paths_contain_relevant_files(&event.paths),
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
            if self
                .metadata_location
                .central_paths()
                .iter()
                .any(|root| path.starts_with(root) || root.starts_with(path))
            {
                return true;
            }
            let filename = path.file_name().and_then(|s| s.to_str());

            if LibraryItemFormat::from_path(path).is_some() {
                return true;
            }
            if LibraryItemFormat::is_metadata_path(path) {
                return true;
            }
            if let Some(filename) = filename
                && filename.ends_with(".sdr")
            {
                return true;
            }
            if self
                .statistics_db_paths
                .iter()
                .any(|stats_path| is_sqlite_db_or_companion(path, stats_path))
            {
                return true;
            }
            if let Some(ref kobo_db_path) = self.kobo_db_path
                && is_sqlite_db_or_companion(path, kobo_db_path)
            {
                return true;
            }
            if self.kobo_db_path.is_some() && path.extension().is_none() {
                return true;
            }
            false
        })
    }
}

/// Keep a nonrecursive watch on the nearest available ancestor so missing
/// directory chains can be discovered without recursively watching unrelated files.
fn refresh_metadata_watches(
    watcher: &mut impl Watcher,
    roots: &[&Path],
    library_paths: &[PathBuf],
    registered: &mut HashMap<PathBuf, RecursiveMode>,
) -> Vec<PathBuf> {
    let mut restored = Vec::new();
    let missing: Vec<_> = registered
        .keys()
        .filter(|path| !path.is_dir())
        .cloned()
        .collect();
    for path in missing {
        let _ = watcher.unwatch(&path);
        registered.remove(&path);
    }
    for root in roots {
        let absolute = match std::path::absolute(root) {
            Ok(path) => path,
            Err(error) => {
                warn!("Cannot resolve metadata watch {:?}: {}", root, error);
                continue;
            }
        };
        let ancestor = absolute
            .parent()
            .and_then(|parent| parent.ancestors().find(|p| p.is_dir()));
        let targets = ancestor
            .into_iter()
            .map(|p| (p, RecursiveMode::NonRecursive))
            .chain(
                absolute
                    .is_dir()
                    .then_some((absolute.as_path(), RecursiveMode::Recursive)),
            );
        for (path, mode) in targets {
            if library_paths.iter().any(|library| {
                std::path::absolute(library).is_ok_and(|library| path.starts_with(library))
            }) || registered
                .get(path)
                .is_some_and(|existing| *existing == RecursiveMode::Recursive || *existing == mode)
            {
                continue;
            }
            match watcher.watch(path, mode) {
                Ok(()) => {
                    registered.insert(path.to_path_buf(), mode);
                    if mode == RecursiveMode::Recursive {
                        restored.push(root.to_path_buf());
                    }
                }
                Err(error) => warn!("Metadata watch unavailable for {:?}: {}", path, error),
            }
        }
    }
    restored
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
fn log_accumulated_paths(
    paths: &HashSet<PathBuf>,
    statistics_db_paths: &[PathBuf],
    kobo_db_path: Option<&std::path::Path>,
) {
    for path in paths {
        let filename = path.file_name().and_then(|s| s.to_str());

        if let Some(format) = LibraryItemFormat::from_path(path) {
            info!("{:?} file changed: {:?}", format, path);
        } else if LibraryItemFormat::is_metadata_path(path) {
            info!("Metadata file changed: {:?}", path);
        } else if let Some(filename) = filename
            && filename.ends_with(".sdr")
        {
            info!("KoReader metadata directory changed: {:?}", path);
        } else if statistics_db_paths
            .iter()
            .any(|stats_path| is_sqlite_db_or_companion(path, stats_path))
        {
            info!("Statistics database changed: {:?}", path);
        } else if let Some(kobo_db_path) = kobo_db_path
            && is_sqlite_db_or_companion(path, kobo_db_path)
        {
            info!("Kobo database changed: {:?}", path);
        } else if kobo_db_path.is_some() && path.extension().is_none() {
            info!("Extensionless Kobo candidate changed: {:?}", path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, RemoveKind, RenameMode};

    #[test]
    fn missing_central_directory_is_watched_when_it_returns() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("mount/settings/docsettings");
        let mut watcher =
            notify::PollWatcher::new(|_: notify::Result<Event>| {}, Config::default()).unwrap();
        let mut registered = HashMap::new();
        assert!(refresh_metadata_watches(&mut watcher, &[&root], &[], &mut registered).is_empty());
        assert_eq!(
            registered.get(dir.path()),
            Some(&RecursiveMode::NonRecursive)
        );
        std::fs::create_dir_all(root.parent().unwrap()).unwrap();
        refresh_metadata_watches(&mut watcher, &[&root], &[], &mut registered);
        assert_eq!(
            registered.get(root.parent().unwrap()),
            Some(&RecursiveMode::NonRecursive)
        );
        std::fs::create_dir(&root).unwrap();
        assert_eq!(
            refresh_metadata_watches(&mut watcher, &[&root], &[], &mut registered),
            vec![root.clone()]
        );
        assert_eq!(registered.get(&root), Some(&RecursiveMode::Recursive));
        std::fs::remove_dir(&root).unwrap();
        refresh_metadata_watches(&mut watcher, &[&root], &[], &mut registered);
        assert!(!registered.contains_key(&root));
        std::fs::create_dir(&root).unwrap();
        assert_eq!(
            refresh_metadata_watches(&mut watcher, &[&root], &[], &mut registered),
            vec![root]
        );
    }

    #[test]
    fn backup_and_replaced_root_events_trigger_reconciliation() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = crate::pipeline::rebuild::tests::test_config(dir.path(), None);
        let root = dir.path().join("docsettings");
        config.metadata_location =
            crate::source::scanner::MetadataLocation::DocSettings(root.clone());
        let watcher = FileWatcher::new(config, None, None, None, None);
        for kind in [
            EventKind::Create(CreateKind::File),
            EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
            EventKind::Modify(ModifyKind::Name(RenameMode::Any)),
            EventKind::Remove(RemoveKind::File),
        ] {
            for path in [
                dir.path().join("Book.sdr/metadata.epub.lua.old"),
                dir.path().join("Book.fb2.sdr/metadata.zip.lua.old"),
            ] {
                assert!(watcher.is_relevant_event(&Event::new(kind).add_path(path)));
            }
        }
        for kind in [
            EventKind::Create(CreateKind::Folder),
            EventKind::Remove(RemoveKind::Folder),
            EventKind::Modify(ModifyKind::Name(RenameMode::Any)),
        ] {
            assert!(watcher.is_relevant_event(&Event::new(kind).add_path(root.clone())));
        }
    }
}
