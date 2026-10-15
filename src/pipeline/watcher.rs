use crate::app::config::SiteConfig;
use crate::pipeline::rebuild::rebuild;
use crate::shelf::models::LibraryItemFormat;
use crate::source::custom::is_custom_path;
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

/// Maintenance is independent of ebook event processing. Cancelling `poll`
/// leaves the running worker owned here and does not permit overlapping jobs.
struct RetiredCleanup {
    store: Option<std::sync::Arc<crate::store::bundles::BundleStore>>,
    timer: tokio::time::Interval,
    task: Option<tokio::task::JoinHandle<Result<()>>>,
}

impl RetiredCleanup {
    fn new(store: Option<std::sync::Arc<crate::store::bundles::BundleStore>>) -> Self {
        let cadence = Duration::from_secs(3600);
        let mut timer = tokio::time::interval_at(tokio::time::Instant::now() + cadence, cadence);
        timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        Self {
            store: store.filter(|s| s.retention != crate::store::bundles::Retention::Forever),
            timer,
            task: None,
        }
    }

    async fn poll(&mut self) {
        tokio::select! {
            _ = self.timer.tick(), if self.store.is_some() => {
                if self.task.is_none() {
                    let store = self.store.as_ref().expect("enabled cleanup").clone();
                    self.task = Some(tokio::task::spawn_blocking(move || store.expire()));
                }
            }
            result = async { self.task.as_mut().expect("running cleanup").await }, if self.task.is_some() => {
                report_cleanup(result);
                self.task = None;
            }
            else => std::future::pending::<()>().await,
        }
    }
}

fn report_cleanup(result: Result<Result<()>, tokio::task::JoinError>) {
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => warn!("Retired file cleanup pending: {e:#}"),
        Err(e) => warn!("Retired file cleanup worker failed: {e}"),
    }
}

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

        // Unavailable roots are not fatal: the retry loop watches them once they return.
        let mut library_watches = LibraryWatches::default();
        refresh_library_watches(&mut watcher, &self.library_paths, &mut library_watches);
        watched.extend(self.library_paths.iter().map(|p| p.display().to_string()));

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
        let mut cleanup = RetiredCleanup::new(
            self.library_repo
                .as_ref()
                .and_then(|repo| repo.bundles.clone()),
        );
        loop {
            let event = tokio::select! {
                _ = cleanup.poll() => continue,
                event = file_rx.recv() => match event {
                    Some(event) => event,
                    None => break,
                },
                _ = retry_watches.tick() => {
                    let mut restored = refresh_library_watches(
                        &mut watcher, &self.library_paths, &mut library_watches,
                    );
                    restored.extend(refresh_metadata_watches(
                        &mut watcher, &self.metadata_location.central_paths(),
                        &self.library_paths, &mut metadata_watches,
                    ));
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
        if let Some(task) = cleanup.task.take() {
            report_cleanup(task.await);
        }
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
            if let Some(data) = self.runtime_data_policy.persistent_data_dir() {
                let managed = data.join("library");
                let absolute = std::path::absolute(path).unwrap_or_else(|_| path.clone());
                let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| absolute.clone());
                if [
                    std::path::absolute(&managed).ok(),
                    std::fs::canonicalize(&managed).ok(),
                ]
                .into_iter()
                .flatten()
                .any(|root| absolute.starts_with(&root) || resolved.starts_with(&root))
                {
                    return false;
                }
            }
            if self
                .metadata_location
                .central_paths()
                .iter()
                .any(|root| path.starts_with(root) || root.starts_with(path))
            {
                return true;
            }
            let filename = path.file_name().and_then(|s| s.to_str());

            if LibraryItemFormat::from_path(path).is_some() || is_custom_path(path) {
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

/// Identity of an available library root. It changes when the root is removed,
/// replaced, or a filesystem is mounted or unmounted over it.
fn root_identity(path: &Path) -> std::io::Result<(u64, u64)> {
    let metadata = crate::source::scanner::library_root_metadata(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok((metadata.dev(), metadata.ino()))
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        Ok((0, 0))
    }
}

/// Registration and last failure are separate so failed watches are retried
/// without repeating the same warning every five seconds.
#[derive(Default)]
struct LibraryWatches {
    registered: HashMap<PathBuf, (u64, u64)>,
    failures: HashMap<PathBuf, String>,
}

impl LibraryWatches {
    fn note_failure(&mut self, root: &Path, message: String) -> bool {
        if self.failures.get(root) == Some(&message) {
            return false;
        }
        warn!("{message} Watching will be retried automatically.");
        self.failures.insert(root.to_path_buf(), message);
        true
    }
}

/// Returns roots that were lost, replaced or recovered so reconciliation runs.
fn refresh_library_watches(
    watcher: &mut impl Watcher,
    library_paths: &[PathBuf],
    state: &mut LibraryWatches,
) -> Vec<PathBuf> {
    let mut changed = Vec::new();
    for root in library_paths {
        let current = match root_identity(root) {
            Ok(identity) => identity,
            Err(error) => {
                if state.registered.remove(root).is_some() {
                    let _ = watcher.unwatch(root);
                    changed.push(root.clone());
                }
                state.note_failure(
                    root,
                    crate::source::scanner::library_access_warning(root, &error),
                );
                continue;
            }
        };
        let previous = state.registered.get(root).copied();
        if previous == Some(current) {
            continue;
        }
        if previous.is_some() {
            let _ = watcher.unwatch(root);
            state.registered.remove(root);
            info!("Library root was replaced: '{}'", root.display());
            changed.push(root.clone());
        }
        match watcher.watch(root, RecursiveMode::Recursive) {
            Ok(()) => {
                state.registered.insert(root.clone(), current);
                if state.failures.remove(root).is_some() {
                    info!(
                        "Library access and watching restored for '{}'; reconciling books",
                        root.display()
                    );
                }
                if previous.is_none() {
                    changed.push(root.clone());
                }
            }
            Err(error) => {
                state.note_failure(root, format!(
                    "Cannot watch library path '{}': {error}. Check filesystem access permissions and watcher resource limits.",
                    root.display()
                ));
            }
        }
    }
    changed
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

    #[tokio::test(start_paused = true)]
    async fn hourly_cleanup_expires_without_events_and_retries_failures() {
        use crate::store::bundles::{BundleStore, Retention};
        let dir = tempfile::tempdir().unwrap();
        let store = std::sync::Arc::new(BundleStore::open(dir.path(), Retention::Days(1)).unwrap());
        let retired = store.root.join("retired").join(format!(
            "100-0123456789abcdef0123456789abcdef-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::write(&retired, b"cannot remove as directory").unwrap();
        let mut cleanup = RetiredCleanup::new(Some(store));
        assert!(futures::poll!(Box::pin(cleanup.poll())).is_pending());
        tokio::time::advance(Duration::from_secs(3599)).await;
        assert!(futures::poll!(Box::pin(cleanup.poll())).is_pending());
        tokio::time::advance(Duration::from_secs(1)).await;
        cleanup.poll().await;
        cleanup.task.take().unwrap().await.unwrap().unwrap();
        assert!(retired.exists());
        std::fs::remove_file(&retired).unwrap();
        std::fs::create_dir(&retired).unwrap();
        tokio::time::advance(Duration::from_secs(3600)).await;
        cleanup.poll().await;
        cleanup.task.take().unwrap().await.unwrap().unwrap();
        assert!(!retired.exists());
    }

    #[tokio::test(start_paused = true)]
    async fn hourly_cleanup_never_overlaps_and_disables_forever() {
        use crate::store::bundles::{BundleStore, Retention};
        let dir = tempfile::tempdir().unwrap();
        let store =
            std::sync::Arc::new(BundleStore::open(dir.path(), Retention::Immediate).unwrap());
        let mut cleanup = RetiredCleanup::new(Some(store));
        let (release, blocked) = std::sync::mpsc::channel();
        cleanup.task = Some(tokio::task::spawn_blocking(move || {
            blocked.recv().unwrap();
            Ok(())
        }));
        let id = cleanup.task.as_ref().unwrap().id();
        tokio::time::advance(Duration::from_secs(3600)).await;
        cleanup.poll().await;
        assert_eq!(cleanup.task.as_ref().unwrap().id(), id);
        release.send(()).unwrap();
        cleanup.task.take().unwrap().await.unwrap().unwrap();
        let forever_dir = tempfile::tempdir().unwrap();
        let forever =
            std::sync::Arc::new(BundleStore::open(forever_dir.path(), Retention::Forever).unwrap());
        for mut disabled in [
            RetiredCleanup::new(None),
            RetiredCleanup::new(Some(forever)),
        ] {
            tokio::time::advance(Duration::from_secs(7200)).await;
            assert!(futures::poll!(Box::pin(disabled.poll())).is_pending());
            assert!(disabled.task.is_none());
        }
    }

    #[test]
    fn library_roots_are_rewatched_and_reported_when_they_return_or_are_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("books");
        let roots = vec![root.clone()];
        let mut watcher =
            notify::PollWatcher::new(|_: notify::Result<Event>| {}, Config::default()).unwrap();
        let mut registered = LibraryWatches::default();
        assert!(refresh_library_watches(&mut watcher, &roots, &mut registered).is_empty());
        assert_eq!(registered.failures.len(), 1);
        let missing = registered.failures[&root].clone();
        assert!(!registered.note_failure(&root, missing));
        assert!(registered.note_failure(&root, "changed failure".into()));
        std::fs::create_dir(&root).unwrap();
        assert_eq!(
            refresh_library_watches(&mut watcher, &roots, &mut registered),
            roots
        );
        assert!(refresh_library_watches(&mut watcher, &roots, &mut registered).is_empty());
        assert!(registered.failures.is_empty());
        std::fs::remove_dir(&root).unwrap();
        assert_eq!(
            refresh_library_watches(&mut watcher, &roots, &mut registered),
            roots
        );
        assert!(registered.registered.is_empty());
        // A replaced directory (like a remount over the mount point) is rewatched.
        std::fs::create_dir(&root).unwrap();
        refresh_library_watches(&mut watcher, &roots, &mut registered);
        let moved = dir.path().join("old");
        std::fs::rename(&root, &moved).unwrap();
        std::fs::create_dir(&root).unwrap();
        #[cfg(unix)]
        assert_eq!(
            refresh_library_watches(&mut watcher, &roots, &mut registered),
            roots
        );
        assert!(registered.registered.contains_key(&root));
    }

    #[cfg(unix)]
    #[test]
    fn library_permission_recovery_reconciles_without_identity_change() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("books");
        std::fs::create_dir(&root).unwrap();
        let roots = vec![root.clone()];
        let mut watcher =
            notify::PollWatcher::new(|_: notify::Result<Event>| {}, Config::default()).unwrap();
        let mut state = LibraryWatches::default();
        refresh_library_watches(&mut watcher, &roots, &mut state);
        let before = state.registered[&root];
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o0)).unwrap();
        if std::fs::read_dir(&root).is_ok() {
            std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
            eprintln!("Skipping permission fixture: runner bypasses mode bits");
            return;
        }
        let lost = refresh_library_watches(&mut watcher, &roots, &mut state);
        let repeated = refresh_library_watches(&mut watcher, &roots, &mut state);
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(lost, roots);
        assert!(repeated.is_empty());
        assert!(state.registered.is_empty());
        assert!(state.failures[&root].contains("permissions"));
        assert_eq!(
            refresh_library_watches(&mut watcher, &roots, &mut state),
            roots
        );
        assert_eq!(state.registered[&root], before);
        assert!(state.failures.is_empty());
        assert!(refresh_library_watches(&mut watcher, &roots, &mut state).is_empty());
    }

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
