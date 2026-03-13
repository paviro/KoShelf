use super::scanner::MetadataLocation;
use crate::config::SiteConfig;
use crate::contracts::common::ApiMeta;
use crate::contracts::site::{SiteCapabilities, SiteResponse};
use crate::domain::library::upsert_single_item;
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::koreader::{StatisticsCalculator, StatisticsParser};
use crate::library::scan_specific_files;
use crate::models::{BookStatus, LibraryItemFormat, ReadingData};
use crate::runtime::export::{ExportConfig, export_data_files};
use crate::runtime::ingest::build_covers_by_md5;
use crate::runtime::media::{self, resolve_media_dirs};
use crate::runtime::{
    RuntimeObservability, SharedReadingDataStore, SharedSiteStore, UpdateNotifier,
};
use anyhow::Result;
use log::{debug, info, warn};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, Instant};
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
        site_store: Option<SharedSiteStore>,
        reading_data_store: Option<SharedReadingDataStore>,
        update_notifier: Option<UpdateNotifier>,
        library_repo: Option<LibraryRepository>,
        observability: RuntimeObservability,
    ) -> Self {
        Self {
            config,
            site_store,
            reading_data_store,
            update_notifier,
            library_repo,
            observability,
        }
    }

    pub async fn run(self) -> Result<()> {
        let (file_tx, mut file_rx) = mpsc::unbounded_channel();
        let (rebuild_tx, mut rebuild_rx) = mpsc::unbounded_channel::<WatcherRebuildEvent>();
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
        let observability_clone = self.observability.clone();
        let pending_rebuilds_clone = pending_rebuilds.clone();

        // Spawn delayed rebuild task.
        // NOTE: Statistics loading uses non-Send types (e.g. mlua::Lua, Rc-based translations),
        // so this rebuild loop must not be spawned onto the multithreaded executor.
        let rebuild_task = tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                let rebuild_delay = Duration::from_secs(1);

                while let Some(initial_event) = rebuild_rx.recv().await {
                    let current_depth = decrement_pending_rebuilds(pending_rebuilds_clone.as_ref());
                    observability_clone.set_watcher_queue_depth(current_depth);
                    let queued_at = Instant::now();

                    // Accumulate paths across the debounce window.
                    let mut accumulated_paths: HashSet<PathBuf> =
                        initial_event.paths.into_iter().collect();

                    sleep(rebuild_delay).await;

                    // Drain any additional events that came in during the delay
                    while let Ok(event) = rebuild_rx.try_recv() {
                        let current_depth =
                            decrement_pending_rebuilds(pending_rebuilds_clone.as_ref());
                        observability_clone.set_watcher_queue_depth(current_depth);
                        accumulated_paths.extend(event.paths);
                    }

                    // Derive what changed from the accumulated paths.
                    let stats_changed = config_clone
                        .statistics_db_path
                        .as_ref()
                        .is_some_and(|sp| accumulated_paths.contains(sp));
                    let library_changed = accumulated_paths
                        .iter()
                        .any(|p| LibraryItemFormat::from_path(p).is_some());

                    // ── Targeted rebuild (when library DB is available) ──
                    if let Some(ref repo) = library_repo_clone {
                        info!(
                            "Starting targeted rebuild for {} changed paths",
                            accumulated_paths.len()
                        );

                        let media_dirs = resolve_media_dirs(
                            &config_clone.output_dir,
                            config_clone.is_internal_server,
                        );
                        if let Err(e) = media::create_media_directories(&media_dirs) {
                            warn!("Failed to create media directories: {}", e);
                        }

                        // ── 1. Classify paths ────────────────────────────
                        let mut parse_paths: HashSet<PathBuf> = HashSet::new();
                        let mut delete_book_paths: Vec<String> = Vec::new();

                        for path in &accumulated_paths {
                            if LibraryItemFormat::from_path(path).is_some() {
                                if path.exists() {
                                    parse_paths.insert(path.clone());
                                } else {
                                    delete_book_paths.push(path.to_string_lossy().to_string());
                                }
                            } else if let Some(filename) = path.file_name().and_then(|s| s.to_str())
                            {
                                if LibraryItemFormat::is_metadata_file(filename) {
                                    if let Some(book_path) = derive_book_path_from_metadata_path(
                                        path,
                                        &config_clone.metadata_location,
                                        repo,
                                    )
                                    .await
                                        && book_path.exists()
                                    {
                                        parse_paths.insert(book_path);
                                    }
                                } else if filename.ends_with(".sdr")
                                    && let Some(book_path) = derive_book_path_from_sdr_path(path)
                                    && book_path.exists()
                                {
                                    parse_paths.insert(book_path);
                                }
                            }
                            // Stats DB path: handled by domain check below.
                        }

                        // ── 2. Delete removed items ──────────────────────
                        let mut deleted_count = 0u64;
                        for book_path_str in &delete_book_paths {
                            match repo.find_fingerprint_by_book_path(book_path_str).await {
                                Ok(Some(fp)) => {
                                    if let Err(e) = repo.delete_item(&fp.item_id).await {
                                        warn!("Failed to delete item {}: {}", fp.item_id, e);
                                    } else {
                                        let cover_path = media_dirs
                                            .covers_dir
                                            .join(format!("{}.webp", fp.item_id));
                                        let _ = std::fs::remove_file(&cover_path);
                                        info!(
                                            "Deleted item {} (book removed: {})",
                                            fp.item_id, book_path_str
                                        );
                                        deleted_count += 1;
                                    }
                                }
                                Ok(None) => {
                                    debug!("No DB fingerprint for removed path: {}", book_path_str);
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to look up fingerprint for {}: {}",
                                        book_path_str, e
                                    );
                                }
                            }
                        }

                        // ── 3. Parse changed/new files ───────────────────
                        let parse_list: Vec<PathBuf> = parse_paths.into_iter().collect();
                        let scanned =
                            scan_specific_files(&parse_list, &config_clone.metadata_location).await;

                        // ── 4. Filter + upsert ──────────────────────────
                        let mut upserted_items = Vec::new();
                        for si in &scanned {
                            // Apply include_unread filter.
                            if si.item.koreader_metadata.is_none()
                                && !(config_clone.include_unread
                                    && si.item.status() == BookStatus::Unknown)
                            {
                                continue;
                            }

                            if let Err(e) = upsert_single_item(
                                repo,
                                &si.item,
                                si.metadata_path.as_deref(),
                                config_clone.use_stable_page_metadata,
                                &config_clone.time_config,
                            )
                            .await
                            {
                                warn!("Failed to upsert item {:?}: {}", si.item.file_path, e);
                            } else {
                                upserted_items.push(si.item.clone());
                            }
                        }

                        info!(
                            "Targeted rebuild: {} upserted, {} deleted",
                            upserted_items.len(),
                            deleted_count
                        );

                        // ── 5. Generate covers for changed items ─────────
                        if !upserted_items.is_empty()
                            && let Err(e) =
                                media::generate_covers(&upserted_items, &media_dirs.covers_dir)
                                    .await
                        {
                            warn!("Failed to generate covers: {}", e);
                        }

                        // ── 6. Load all item IDs (used for stats filtering + covers map) ──
                        let all_item_ids: Option<Vec<String>> = match repo.load_all_item_ids().await
                        {
                            Ok(ids) => Some(ids),
                            Err(e) => {
                                warn!("Failed to load item IDs: {}", e);
                                None
                            }
                        };

                        // ── 7. Stats reload if affected ──────────────────
                        let mut reading_data: Option<ReadingData> = None;

                        if stats_changed
                            && let Some(ref stats_path) = config_clone.statistics_db_path
                            && stats_path.exists()
                        {
                            match StatisticsParser::parse(stats_path).await {
                                Ok(mut data) => {
                                    if config_clone.min_pages_per_day.is_some()
                                        || config_clone.min_time_per_day.is_some()
                                    {
                                        StatisticsCalculator::filter_stats(
                                            &mut data,
                                            &config_clone.time_config,
                                            config_clone.min_pages_per_day,
                                            config_clone.min_time_per_day,
                                        );
                                    }

                                    if !config_clone.include_all_stats
                                        && let Some(ref ids) = all_item_ids
                                    {
                                        let md5s: HashSet<String> = ids.iter().cloned().collect();
                                        StatisticsCalculator::filter_to_library(&mut data, &md5s);
                                    }

                                    StatisticsCalculator::populate_completions(
                                        &mut data,
                                        &config_clone.time_config,
                                    );

                                    let covers_by_md5 = all_item_ids
                                        .as_ref()
                                        .map(|ids| build_covers_by_md5(ids.iter()))
                                        .unwrap_or_default();

                                    let rd = ReadingData {
                                        stats_data: data,
                                        time_config: config_clone.time_config.clone(),
                                        heatmap_scale_max: config_clone.heatmap_scale_max,
                                        covers_by_md5,
                                    };

                                    if let Some(ref store) = reading_data_store_clone {
                                        store.replace(rd.clone());
                                    }

                                    reading_data = Some(rd);
                                }
                                Err(e) => warn!("Failed to reload statistics: {}", e),
                            }
                        }

                        // ── 7b. Refresh covers in existing ReadingData if library changed ──
                        if reading_data.is_none()
                            && library_changed
                            && let Some(ref store) = reading_data_store_clone
                            && let Some(mut rd) = store.get().map(|rd| rd.as_ref().clone())
                        {
                            rd.covers_by_md5 = all_item_ids
                                .as_ref()
                                .map(|ids| build_covers_by_md5(ids.iter()))
                                .unwrap_or_default();
                            store.replace(rd);
                        }

                        // ── 8. Refresh SiteStore from DB ─────────────────
                        let generated_at = config_clone.time_config.now_rfc3339();

                        match repo.query_content_type_flags().await {
                            Ok((has_books, has_comics)) => {
                                let has_reading_data = reading_data.is_some()
                                    || reading_data_store_clone
                                        .as_ref()
                                        .and_then(|s| s.get())
                                        .is_some_and(|rd| !rd.stats_data.page_stats.is_empty());

                                let site_response = SiteResponse {
                                    meta: ApiMeta {
                                        version: env!("CARGO_PKG_VERSION").to_string(),
                                        generated_at: generated_at.clone(),
                                    },
                                    title: config_clone.site_title.clone(),
                                    language: config_clone.language.clone(),
                                    capabilities: SiteCapabilities {
                                        has_books,
                                        has_comics,
                                        has_reading_data,
                                    },
                                };

                                if let Some(ref site_store) = site_store_clone {
                                    site_store.replace(site_response);
                                }
                            }
                            Err(e) => warn!("Failed to query content type flags: {}", e),
                        }

                        // ── 9. SSE broadcast ─────────────────────────────
                        if let Some(ref update_notifier) = update_notifier_clone {
                            let update = update_notifier.publish(generated_at.clone());
                            info!("Published data_changed event, revision {}", update.revision,);
                        }

                        // ── 10. Static data re-export ────────────────────
                        if !config_clone.is_internal_server {
                            // Resolve reading_data for export: freshly loaded or from store.
                            let store_rd = reading_data_store_clone.as_ref().and_then(|s| s.get());
                            let rd_ref = reading_data.as_ref().or(store_rd.as_deref());

                            let export_config = ExportConfig {
                                site_title: config_clone.site_title.clone(),
                                language: config_clone.language.clone(),
                            };
                            if let Err(e) = export_data_files(
                                &config_clone.output_dir.join("data"),
                                repo,
                                rd_ref,
                                &export_config,
                            )
                            .await
                            {
                                warn!("Failed to re-export data files: {}", e);
                            }
                        }

                        info!("Targeted rebuild completed successfully");
                        observability_clone.record_watcher_update_latency(queued_at.elapsed());
                        continue;
                    }

                    // ── Fallback: full ingest (no library DB) ────────────
                    info!("Starting full rebuild (no library DB available)");

                    let ingest_result = match crate::runtime::ingest(&config_clone).await {
                        Ok(r) => r,
                        Err(e) => {
                            warn!("Watcher ingest failed: {}", e);
                            continue;
                        }
                    };

                    let reading_data = ingest_result.reading_data(&config_clone);

                    let media_dirs = resolve_media_dirs(
                        &config_clone.output_dir,
                        config_clone.is_internal_server,
                    );

                    if let Err(e) = media::create_media_directories(&media_dirs) {
                        warn!("Failed to create media directories: {}", e);
                    }

                    if !config_clone.is_internal_server
                        && let Err(e) = media::sync_static_frontend(
                            &config_clone.output_dir,
                            ingest_result.has_reading_data(),
                        )
                    {
                        warn!("Failed to sync static frontend: {}", e);
                    }

                    if let Err(e) = media::generate_covers(
                        &ingest_result.filtered_items,
                        &media_dirs.covers_dir,
                    )
                    .await
                    {
                        warn!("Failed to generate covers: {}", e);
                    }

                    if let Err(e) = media::cleanup_stale_covers(
                        &ingest_result.filtered_items,
                        &media_dirs.covers_dir,
                    ) {
                        warn!("Failed to cleanup stale covers: {}", e);
                    }

                    let generated_at = config_clone.time_config.now_rfc3339();

                    let site_response = SiteResponse {
                        meta: ApiMeta {
                            version: env!("CARGO_PKG_VERSION").to_string(),
                            generated_at: generated_at.clone(),
                        },
                        title: config_clone.site_title.clone(),
                        language: config_clone.language.clone(),
                        capabilities: SiteCapabilities {
                            has_books: ingest_result
                                .filtered_items
                                .iter()
                                .any(|item| item.is_book()),
                            has_comics: ingest_result
                                .filtered_items
                                .iter()
                                .any(|item| item.is_comic()),
                            has_reading_data: ingest_result.has_reading_data(),
                        },
                    };

                    if let Some(ref site_store) = site_store_clone {
                        site_store.replace(site_response);
                    }

                    if let Some(ref reading_data_store) = reading_data_store_clone
                        && let Some(rd) = &reading_data
                    {
                        reading_data_store.replace(rd.clone());
                    }

                    if let Some(ref update_notifier) = update_notifier_clone {
                        let update = update_notifier.publish(generated_at);
                        info!("Published data_changed event, revision {}", update.revision,);
                    }

                    info!("Full rebuild completed successfully");
                    observability_clone.record_watcher_update_latency(queued_at.elapsed());
                }
            })
        });

        // Main file event processing loop
        while let Some(event) = file_rx.recv().await {
            if self.is_relevant_event(&event) {
                self.log_file_event(&event);

                let rebuild_event = WatcherRebuildEvent { paths: event.paths };

                if rebuild_tx.send(rebuild_event).is_ok() {
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

// ── Path derivation helpers ──────────────────────────────────────────────

/// Derive the book file path from a KOReader metadata file path.
///
/// For `InBookFolder`: `/library/Title.sdr/metadata.epub.lua` → `/library/Title.epub`
/// For `DocSettings`/`HashDocSettings`: look up by metadata_path in DB fingerprints.
async fn derive_book_path_from_metadata_path(
    metadata_path: &Path,
    metadata_location: &MetadataLocation,
    repo: &LibraryRepository,
) -> Option<PathBuf> {
    match metadata_location {
        MetadataLocation::InBookFolder => {
            let filename = metadata_path.file_name()?.to_str()?;
            let format = LibraryItemFormat::from_metadata_filename(filename)?;
            let sdr_dir = metadata_path.parent()?;
            let sdr_name = sdr_dir.file_name()?.to_str()?;
            let book_stem = sdr_name.strip_suffix(".sdr")?;
            let grandparent = sdr_dir.parent()?;
            Some(grandparent.join(format!("{}.{}", book_stem, format.extension())))
        }
        MetadataLocation::DocSettings(_) | MetadataLocation::HashDocSettings(_) => {
            let meta_str = metadata_path.to_string_lossy();
            match repo.find_book_path_by_metadata_path(&meta_str).await {
                Ok(Some(book_path)) => Some(PathBuf::from(book_path)),
                Ok(None) => {
                    debug!("No DB fingerprint for metadata path: {}", meta_str);
                    None
                }
                Err(e) => {
                    warn!(
                        "Failed to look up book path for metadata {}: {}",
                        meta_str, e
                    );
                    None
                }
            }
        }
    }
}

/// Derive the book file path from a `.sdr` directory path.
///
/// Tries all supported formats to find a matching book file in the parent directory.
/// `/library/Title.sdr` → `/library/Title.epub` (if it exists)
fn derive_book_path_from_sdr_path(sdr_path: &Path) -> Option<PathBuf> {
    let sdr_name = sdr_path.file_name()?.to_str()?;
    let book_stem = sdr_name.strip_suffix(".sdr")?;
    let parent = sdr_path.parent()?;

    for format in [
        LibraryItemFormat::Epub,
        LibraryItemFormat::Fb2,
        LibraryItemFormat::Mobi,
        LibraryItemFormat::Cbz,
        LibraryItemFormat::Cbr,
    ] {
        let candidate = parent.join(format!("{}.{}", book_stem, format.extension()));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}
