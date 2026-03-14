//! Rebuild orchestration: targeted (incremental) and full rebuild pipelines.
//!
//! Extracted from the file watcher so the rebuild logic is testable and
//! the watcher module is limited to event setup, debouncing, and dispatch.

use crate::config::SiteConfig;
use crate::contracts::site::{SiteCapabilities, SiteData};
use crate::infra::scanner::{MetadataLocation, collect_paths};
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::infra::stores::{SharedReadingDataStore, SharedSiteStore, UpdateNotifier};
use crate::models::LibraryItemFormat;
use crate::runtime::export::{ExportConfig, export_data_files};
use crate::runtime::ingest::{ingest_paths, load_reading_data};
use crate::runtime::media::{self, resolve_media_dirs};
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

    // ── 3. Ingest changed/new paths ──────────────────────────────────
    let parse_list: Vec<PathBuf> = parse_paths.into_iter().collect();
    let ingest_stats = ingest_paths(&parse_list, config, repo, &media_dirs.covers_dir).await?;

    // ── 4. Stats reload if affected ──────────────────────────────────
    let mut stats_reloaded = false;

    if stats_changed {
        match load_reading_data(config, repo).await {
            Ok(Some(rd)) => {
                if let Some(store) = reading_data_store {
                    store.replace(rd);
                }
                stats_reloaded = true;
            }
            Ok(None) => {}
            Err(e) => warn!("Failed to reload statistics: {}", e),
        }
    }

    // ── 5. Refresh SiteStore from DB ─────────────────────────────────
    let generated_at = config.time_config.now_rfc3339();

    match repo.query_content_type_flags().await {
        Ok((has_books, has_comics)) => {
            let has_reading_data = reading_data_store
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

    // ── 6. SSE broadcast ─────────────────────────────────────────────
    let published_revision = if let Some(notifier) = update_notifier {
        let update = notifier.publish(generated_at.clone());
        info!("Published data_changed event, revision {}", update.revision);
        Some(update.revision)
    } else {
        None
    };

    // ── 7. Static data re-export ────────────────────────────────────
    if !config.is_internal_server {
        let store_rd = reading_data_store.and_then(|s| s.get());
        let rd_ref = store_rd.as_deref();

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
        upserted: ingest_stats.upserted,
        deleted: deleted_count,
        stats_reloaded,
        published_revision,
    })
}

/// Full rebuild: re-ingest everything from scratch.
///
/// Clears the DB, re-scans, and re-ingests all paths.
pub async fn full_rebuild(
    config: &SiteConfig,
    repo: &LibraryRepository,
    site_store: Option<&SharedSiteStore>,
    reading_data_store: Option<&SharedReadingDataStore>,
    update_notifier: Option<&UpdateNotifier>,
) -> Result<RebuildResult> {
    info!("Starting full rebuild");

    let media_dirs = resolve_media_dirs(&config.output_dir, config.is_internal_server);

    if let Err(e) = media::create_media_directories(&media_dirs) {
        warn!("Failed to create media directories: {}", e);
    }

    // Clear DB and re-ingest
    repo.clear_all().await?;

    let paths = collect_paths(&config.library_paths);
    let ingest_stats = ingest_paths(&paths, config, repo, &media_dirs.covers_dir).await?;

    // Cleanup stale covers
    match repo.load_all_item_ids().await {
        Ok(ids) => {
            let id_set: HashSet<String> = ids.into_iter().collect();
            if let Err(e) = media::cleanup_stale_covers_by_ids(&id_set, &media_dirs.covers_dir) {
                warn!("Failed to cleanup stale covers: {}", e);
            }
        }
        Err(e) => warn!("Failed to load item IDs for cover cleanup: {}", e),
    }

    if !config.is_internal_server
        && let Err(e) =
            media::sync_static_frontend(&config.output_dir, config.statistics_db_path.is_some())
    {
        warn!("Failed to sync static frontend: {}", e);
    }

    // Load stats
    let reading_data = load_reading_data(config, repo).await?;

    if let Some(store) = reading_data_store
        && let Some(ref rd) = reading_data
    {
        store.replace(rd.clone());
    }

    let generated_at = config.time_config.now_rfc3339();

    // Build site metadata from DB
    let (has_books, has_comics) = repo
        .query_content_type_flags()
        .await
        .unwrap_or((false, false));
    let has_reading_data = reading_data
        .as_ref()
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

    if let Some(store) = site_store {
        store.replace(site_data);
    }

    let published_revision = if let Some(notifier) = update_notifier {
        let update = notifier.publish(generated_at);
        info!("Published data_changed event, revision {}", update.revision);
        Some(update.revision)
    } else {
        None
    };

    // Static data re-export
    if !config.is_internal_server {
        let export_config = ExportConfig {
            site_title: config.site_title.clone(),
            language: config.language.clone(),
        };
        if let Err(e) = export_data_files(
            &config.output_dir.join("data"),
            repo,
            reading_data.as_ref(),
            &export_config,
        )
        .await
        {
            warn!("Failed to re-export data files: {}", e);
        }
    }

    info!("Full rebuild completed successfully");

    Ok(RebuildResult {
        upserted: ingest_stats.upserted,
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
