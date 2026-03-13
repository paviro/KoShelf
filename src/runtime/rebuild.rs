//! Rebuild orchestration: targeted (incremental) and full rebuild pipelines.
//!
//! Extracted from the file watcher so the rebuild logic is testable and
//! the watcher module is limited to event setup, debouncing, and dispatch.

use crate::config::SiteConfig;
use crate::contracts::site::{SiteCapabilities, SiteData};
use crate::domain::library::upsert_single_item;
use crate::infra::scanner::{MetadataLocation, scan_specific_files};
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::koreader::{StatisticsCalculator, StatisticsParser};
use crate::models::{BookStatus, LibraryItemFormat, ReadingData};
use crate::runtime::export::{ExportConfig, export_data_files};
use crate::runtime::ingest::build_covers_by_md5;
use crate::runtime::media::{self, resolve_media_dirs};
use crate::runtime::{SharedReadingDataStore, SharedSiteStore, UpdateNotifier};
use anyhow::Result;
use log::{debug, info, warn};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Summary of a rebuild pass, returned to the caller for logging/observability.
pub struct RebuildResult {
    pub upserted: u64,
    pub deleted: u64,
    pub stats_reloaded: bool,
    pub published_revision: Option<u64>,
}

/// Targeted rebuild: process only changed paths using the library DB.
pub async fn targeted_rebuild(
    accumulated_paths: HashSet<PathBuf>,
    config: &SiteConfig,
    repo: &LibraryRepository,
    site_store: Option<&SharedSiteStore>,
    reading_data_store: Option<&SharedReadingDataStore>,
    update_notifier: Option<&UpdateNotifier>,
) -> Result<RebuildResult> {
    info!(
        "Starting targeted rebuild for {} changed paths",
        accumulated_paths.len()
    );

    let stats_changed = config
        .statistics_db_path
        .as_ref()
        .is_some_and(|sp| accumulated_paths.contains(sp));
    let library_changed = accumulated_paths
        .iter()
        .any(|p| LibraryItemFormat::from_path(p).is_some());

    let media_dirs = resolve_media_dirs(&config.output_dir, config.is_internal_server);
    if let Err(e) = media::create_media_directories(&media_dirs) {
        warn!("Failed to create media directories: {}", e);
    }

    // ── 1. Classify paths ────────────────────────────────────────────
    let mut parse_paths: HashSet<PathBuf> = HashSet::new();
    let mut delete_book_paths: Vec<String> = Vec::new();

    for path in &accumulated_paths {
        if LibraryItemFormat::from_path(path).is_some() {
            if path.exists() {
                parse_paths.insert(path.clone());
            } else {
                delete_book_paths.push(path.to_string_lossy().to_string());
            }
        } else if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if LibraryItemFormat::is_metadata_file(filename) {
                if let Some(book_path) =
                    derive_book_path_from_metadata_path(path, &config.metadata_location, repo).await
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
    }

    // ── 2. Delete removed items ──────────────────────────────────────
    let mut deleted_count = 0u64;
    for book_path_str in &delete_book_paths {
        match repo.find_fingerprint_by_book_path(book_path_str).await {
            Ok(Some(fp)) => {
                if let Err(e) = repo.delete_item(&fp.item_id).await {
                    warn!("Failed to delete item {}: {}", fp.item_id, e);
                } else {
                    let cover_path = media_dirs.covers_dir.join(format!("{}.webp", fp.item_id));
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
                warn!("Failed to look up fingerprint for {}: {}", book_path_str, e);
            }
        }
    }

    // ── 3. Parse changed/new files ───────────────────────────────────
    let parse_list: Vec<PathBuf> = parse_paths.into_iter().collect();
    let scanned = scan_specific_files(&parse_list, &config.metadata_location).await;

    // ── 4. Filter + upsert ──────────────────────────────────────────
    let mut upserted_items = Vec::new();
    for si in &scanned {
        if si.item.koreader_metadata.is_none()
            && !(config.include_unread && si.item.status() == BookStatus::Unknown)
        {
            continue;
        }

        if let Err(e) = upsert_single_item(
            repo,
            &si.item,
            si.metadata_path.as_deref(),
            config.use_stable_page_metadata,
            &config.time_config,
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

    // ── 5. Generate covers for changed items ─────────────────────────
    if !upserted_items.is_empty()
        && let Err(e) = media::generate_covers(&upserted_items, &media_dirs.covers_dir).await
    {
        warn!("Failed to generate covers: {}", e);
    }

    // ── 6. Load all item IDs (used for stats filtering + covers map) ──
    let all_item_ids: Option<Vec<String>> = match repo.load_all_item_ids().await {
        Ok(ids) => Some(ids),
        Err(e) => {
            warn!("Failed to load item IDs: {}", e);
            None
        }
    };

    // ── 7. Stats reload if affected ──────────────────────────────────
    let mut reading_data: Option<ReadingData> = None;
    let mut stats_reloaded = false;

    if stats_changed
        && let Some(ref stats_path) = config.statistics_db_path
        && stats_path.exists()
    {
        match StatisticsParser::parse(stats_path).await {
            Ok(mut data) => {
                if config.min_pages_per_day.is_some() || config.min_time_per_day.is_some() {
                    StatisticsCalculator::filter_stats(
                        &mut data,
                        &config.time_config,
                        config.min_pages_per_day,
                        config.min_time_per_day,
                    );
                }

                if !config.include_all_stats
                    && let Some(ref ids) = all_item_ids
                {
                    let md5s: HashSet<String> = ids.iter().cloned().collect();
                    StatisticsCalculator::filter_to_library(&mut data, &md5s);
                }

                StatisticsCalculator::populate_completions(&mut data, &config.time_config);

                let covers_by_md5 = all_item_ids
                    .as_ref()
                    .map(|ids| build_covers_by_md5(ids.iter()))
                    .unwrap_or_default();

                let rd = ReadingData {
                    stats_data: data,
                    time_config: config.time_config.clone(),
                    heatmap_scale_max: config.heatmap_scale_max,
                    covers_by_md5,
                };

                if let Some(store) = reading_data_store {
                    store.replace(rd.clone());
                }

                reading_data = Some(rd);
                stats_reloaded = true;
            }
            Err(e) => warn!("Failed to reload statistics: {}", e),
        }
    }

    // ── 7b. Refresh covers in existing ReadingData if library changed ──
    if reading_data.is_none()
        && library_changed
        && let Some(store) = reading_data_store
        && let Some(mut rd) = store.get().map(|rd| rd.as_ref().clone())
    {
        rd.covers_by_md5 = all_item_ids
            .as_ref()
            .map(|ids| build_covers_by_md5(ids.iter()))
            .unwrap_or_default();
        store.replace(rd);
    }

    // ── 8. Refresh SiteStore from DB ─────────────────────────────────
    let generated_at = config.time_config.now_rfc3339();

    match repo.query_content_type_flags().await {
        Ok((has_books, has_comics)) => {
            let has_reading_data = reading_data.is_some()
                || reading_data_store
                    .and_then(|s| s.get())
                    .is_some_and(|rd| !rd.stats_data.page_stats.is_empty());

            let site_data = SiteData {
                title: config.site_title.clone(),
                language: config.language.clone(),
                capabilities: SiteCapabilities {
                    has_books,
                    has_comics,
                    has_reading_data,
                },
            };

            if let Some(site_store) = site_store {
                site_store.replace(site_data);
            }
        }
        Err(e) => warn!("Failed to query content type flags: {}", e),
    }

    // ── 9. SSE broadcast ─────────────────────────────────────────────
    let published_revision = if let Some(notifier) = update_notifier {
        let update = notifier.publish(generated_at.clone());
        info!("Published data_changed event, revision {}", update.revision);
        Some(update.revision)
    } else {
        None
    };

    // ── 10. Static data re-export ────────────────────────────────────
    if !config.is_internal_server {
        let store_rd = reading_data_store.and_then(|s| s.get());
        let rd_ref = reading_data.as_ref().or(store_rd.as_deref());

        let export_config = ExportConfig {
            site_title: config.site_title.clone(),
            language: config.language.clone(),
        };
        if let Err(e) = export_data_files(
            &config.output_dir.join("data"),
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

    Ok(RebuildResult {
        upserted: upserted_items.len() as u64,
        deleted: deleted_count,
        stats_reloaded,
        published_revision,
    })
}

/// Full rebuild: re-ingest everything (fallback when no library DB is available).
pub async fn full_rebuild(
    config: &SiteConfig,
    site_store: Option<&SharedSiteStore>,
    reading_data_store: Option<&SharedReadingDataStore>,
    update_notifier: Option<&UpdateNotifier>,
) -> Result<RebuildResult> {
    info!("Starting full rebuild (no library DB available)");

    let ingest_result = crate::runtime::ingest(config).await?;
    let reading_data = ingest_result.reading_data(config);

    let media_dirs = resolve_media_dirs(&config.output_dir, config.is_internal_server);

    if let Err(e) = media::create_media_directories(&media_dirs) {
        warn!("Failed to create media directories: {}", e);
    }

    if !config.is_internal_server
        && let Err(e) =
            media::sync_static_frontend(&config.output_dir, ingest_result.has_reading_data())
    {
        warn!("Failed to sync static frontend: {}", e);
    }

    if let Err(e) =
        media::generate_covers(&ingest_result.filtered_items, &media_dirs.covers_dir).await
    {
        warn!("Failed to generate covers: {}", e);
    }

    if let Err(e) =
        media::cleanup_stale_covers(&ingest_result.filtered_items, &media_dirs.covers_dir)
    {
        warn!("Failed to cleanup stale covers: {}", e);
    }

    let generated_at = config.time_config.now_rfc3339();

    let site_data = SiteData::from_items(
        &ingest_result.filtered_items,
        ingest_result.has_reading_data(),
        &config.site_title,
        &config.language,
    );

    if let Some(store) = site_store {
        store.replace(site_data);
    }

    if let Some(store) = reading_data_store
        && let Some(ref rd) = reading_data
    {
        store.replace(rd.clone());
    }

    let published_revision = if let Some(notifier) = update_notifier {
        let update = notifier.publish(generated_at);
        info!("Published data_changed event, revision {}", update.revision);
        Some(update.revision)
    } else {
        None
    };

    info!("Full rebuild completed successfully");

    Ok(RebuildResult {
        upserted: ingest_result.filtered_items.len() as u64,
        deleted: 0,
        stats_reloaded: reading_data.is_some(),
        published_revision,
    })
}

// ── Path derivation helpers ──────────────────────────────────────────────

/// Derive the book file path from a KOReader metadata file path.
///
/// For `InBookFolder`: `/library/Title.sdr/metadata.epub.lua` → `/library/Title.epub`
/// For `DocSettings`/`HashDocSettings`: look up by metadata_path in DB fingerprints.
pub(crate) async fn derive_book_path_from_metadata_path(
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
pub(crate) fn derive_book_path_from_sdr_path(sdr_path: &Path) -> Option<PathBuf> {
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
