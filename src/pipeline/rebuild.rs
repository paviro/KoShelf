//! Rebuild orchestration: targeted (incremental) rebuild pipeline.
//!
//! Extracted from the file watcher so the rebuild logic is testable and
//! the watcher module is limited to event setup, debouncing, and dispatch.

use crate::app::config::SiteConfig;
use crate::pipeline::export::{ExportConfig, export_data_files};
use crate::pipeline::ingest::{
    delete_item_for_book_path, ingest_items, load_reading_data, sync_library,
};
use crate::pipeline::media::{self, resolve_media_dirs};
use crate::pipeline::recap::regenerate_share_images;
use crate::server::api::responses::site::{SiteCapabilities, SiteData};
use crate::shelf::models::LibraryItemFormat;
use crate::source::FileFingerprint;
use crate::source::scanner::{CollectedItem, MetadataLocation};
use crate::source::sqlite_snapshot::is_sqlite_db_or_companion;
use crate::store::memory::{SharedReadingDataStore, SharedSiteStore, UpdateNotifier};
use crate::store::sqlite::repo::LibraryRepository;
use anyhow::Result;
use log::{debug, info, warn};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Targeted rebuild: process only changed paths using the library DB.
pub async fn rebuild(
    accumulated_paths: HashSet<PathBuf>,
    config: &SiteConfig,
    repo: &LibraryRepository,
    site_store: Option<&SharedSiteStore>,
    reading_data_store: Option<&SharedReadingDataStore>,
    update_notifier: Option<&UpdateNotifier>,
) -> Result<()> {
    info!(
        "Starting targeted rebuild for {} changed paths",
        accumulated_paths.len()
    );

    let stats_changed = config.statistics_db_paths.iter().any(|db_path| {
        accumulated_paths
            .iter()
            .any(|path| is_sqlite_db_or_companion(path, db_path))
    });
    let full_library_sync_required = requires_full_library_sync(&accumulated_paths, config);

    let media_dirs = resolve_media_dirs(&config.output_dir, config.is_internal_server);
    if let Err(e) = media::create_media_directories(&media_dirs) {
        warn!("Failed to create media directories: {}", e);
    }

    // ── 1. Classify paths ────────────────────────────────────────────
    let mut parse_items: Vec<CollectedItem> = Vec::new();
    let mut delete_book_paths: Vec<String> = Vec::new();

    let library_update = if full_library_sync_required {
        Some(sync_library(config, repo, &media_dirs).await?)
    } else {
        for path in &accumulated_paths {
            if let Some(format) = LibraryItemFormat::from_path(path) {
                if path.exists() {
                    parse_items.push(CollectedItem {
                        path: path.clone(),
                        format,
                        kobo_hints: None,
                    });
                } else {
                    delete_book_paths.push(path.to_string_lossy().to_string());
                }
            } else if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if LibraryItemFormat::is_metadata_path(path) {
                    if let Some(book_path) =
                        derive_book_path_from_metadata_path(path, &config.metadata_location, repo)
                            .await
                        && book_path.exists()
                    {
                        if metadata_fingerprint_matches_db(&book_path, repo).await {
                            debug!(
                                "Skipping re-ingest for {:?}: metadata fingerprint already matches DB",
                                path
                            );
                        } else if let Some(format) = LibraryItemFormat::from_path(&book_path) {
                            parse_items.push(CollectedItem {
                                path: book_path,
                                format,
                                kobo_hints: None,
                            });
                        }
                    }
                } else if filename.ends_with(".sdr")
                    && let Some(book_path) = derive_book_path_from_sdr_path(path)
                    && book_path.exists()
                {
                    if metadata_fingerprint_matches_db(&book_path, repo).await {
                        debug!(
                            "Skipping re-ingest for {:?}: metadata fingerprint already matches DB",
                            path
                        );
                    } else if let Some(format) = LibraryItemFormat::from_path(&book_path) {
                        parse_items.push(CollectedItem {
                            path: book_path,
                            format,
                            kobo_hints: None,
                        });
                    }
                }
            }
        }
        None
    };
    if let Some(update) = &library_update {
        debug!(
            "Full library sync result: {} unchanged, {} changed, {} added, {} removed",
            update.unchanged, update.changed, update.added, update.removed
        );
    }

    // ── 2. Delete removed items ──────────────────────────────────────
    let mut deleted_count = 0u64;
    for book_path_str in &delete_book_paths {
        if delete_item_for_book_path(repo, book_path_str, &media_dirs, config.is_internal_server)
            .await
        {
            deleted_count += 1;
        }
    }

    // ── 3. Ingest changed/new paths ──────────────────────────────────
    let ingest_stats = ingest_items(&parse_items, config, repo, &media_dirs).await?;

    // ── 4. Stats reload if affected ──────────────────────────────────
    let mut stats_reloaded = false;
    let needs_stats_reload = stats_changed
        || ingest_stats.stats_invalidated > 0
        || deleted_count > 0
        || library_update
            .as_ref()
            .and_then(|update| update.ingest_stats)
            .is_some_and(|stats| stats.stats_invalidated > 0)
        || library_update
            .as_ref()
            .is_some_and(|update| update.removed > 0);

    if needs_stats_reload {
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

    // ── 4b. Regenerate share images if stats changed ────────────────
    if stats_reloaded
        && let Some(rd) = reading_data_store.and_then(|s| s.get())
        && let Err(e) = regenerate_share_images(
            &rd.stats_data,
            repo,
            &rd.page_scaling,
            &media_dirs.recap_dir,
            &config.time_config,
            false,
        )
        .await
    {
        warn!("Failed to regenerate share images: {}", e);
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
                    has_files: config.is_internal_server || config.include_files,

                    has_writeback: config.writeback_enabled,
                },
                auth: None,
            };

            if let Some(site_store) = site_store {
                site_store.replace(site_data);
            }
        }
        Err(e) => warn!("Failed to query content type flags: {}", e),
    }

    // ── 6. SSE broadcast (only when something actually changed) ────
    let data_changed = ingest_stats.upserted > 0
        || deleted_count > 0
        || stats_reloaded
        || library_update.as_ref().is_some_and(|update| {
            update.removed > 0
                || update
                    .ingest_stats
                    .is_some_and(|stats| stats.upserted > 0 || stats.stats_invalidated > 0)
        });
    if data_changed && let Some(notifier) = update_notifier {
        let update = notifier.publish(generated_at.clone());
        info!("Published data_changed event, revision {}", update.revision);
    }

    // ── 7. Static data re-export ────────────────────────────────────
    if !config.is_internal_server {
        let store_rd = reading_data_store.and_then(|s| s.get());
        let rd_ref = store_rd.as_deref();

        let export_config = ExportConfig {
            site_title: config.site_title.clone(),
            language: config.language.clone(),
            include_files: config.include_files,
        };
        if let Err(e) = export_data_files(
            &config.output_dir.join("data"),
            &config.output_dir,
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

    Ok(())
}

// ── Fingerprint helpers ──────────────────────────────────────────────────

/// Check whether the on-disk metadata fingerprint matches what's stored in the DB.
///
/// Returns `true` when the fingerprints match, indicating the change was already
/// processed by a write handler and re-ingestion can be skipped.
async fn metadata_fingerprint_matches_db(book_path: &Path, repo: &LibraryRepository) -> bool {
    let book_str = book_path.to_string_lossy();
    let stored = match repo.find_fingerprint_by_book_path(&book_str).await {
        Ok(Some(fp)) => fp,
        _ => return false,
    };

    let (meta_path, stored_size, stored_modified) = match (
        &stored.metadata_path,
        stored.metadata_size_bytes,
        stored.metadata_modified_unix_ms,
    ) {
        (Some(p), Some(s), Some(m)) => (p.clone(), s, m),
        _ => return false,
    };

    let disk = match FileFingerprint::capture(Path::new(&meta_path)) {
        Ok(fp) => fp,
        Err(_) => return false,
    };

    disk.size_bytes as i64 == stored_size && disk.modified_unix_ms as i64 == stored_modified
}

fn is_extensionless_path(path: &Path) -> bool {
    path.extension().is_none()
}

fn requires_full_library_sync(accumulated_paths: &HashSet<PathBuf>, config: &SiteConfig) -> bool {
    let kobo_db_changed = config.kobo_db_path.as_ref().is_some_and(|db_path| {
        accumulated_paths
            .iter()
            .any(|path| is_sqlite_db_or_companion(path, db_path))
    });
    let kobo_extensionless_changed = config.kobo_db_path.is_some()
        && accumulated_paths
            .iter()
            .any(|path| is_extensionless_path(path.as_path()));

    kobo_db_changed || kobo_extensionless_changed
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
            let sdr_dir = metadata_path.parent()?;
            let sdr_name = sdr_dir.file_name()?.to_str()?;
            let book_stem = sdr_name.strip_suffix(".sdr")?;
            let book_filename = LibraryItemFormat::book_filename_from_sidecar(book_stem, filename)?;
            let grandparent = sdr_dir.parent()?;
            Some(grandparent.join(book_filename))
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

    // A `<name>.fb2.sdr` sidecar can also belong to a `<name>.fb2.zip` book
    // (KOReader strips only the final suffix).
    if book_stem.to_lowercase().ends_with(".fb2") {
        let candidate = parent.join(format!("{}.zip", book_stem));
        if candidate.exists() {
            return Some(candidate);
        }
    }

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

#[cfg(test)]
mod tests {
    use super::{derive_book_path_from_sdr_path, requires_full_library_sync};
    use crate::app::config::SiteConfig;
    use crate::shelf::models::LibraryItemFormat;
    use crate::shelf::time_config::TimeConfig;
    use crate::source::scanner::MetadataLocation;
    use crate::store::lifecycle::{RuntimeDataPathOptions, resolve_runtime_data_policy};
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    fn test_config(output_dir: &Path, kobo_db_path: Option<PathBuf>) -> SiteConfig {
        let mut runtime_data_policy =
            resolve_runtime_data_policy(&RuntimeDataPathOptions::default());
        runtime_data_policy.set_resolved_data_dir(output_dir.join("runtime"));

        SiteConfig {
            output_dir: output_dir.to_path_buf(),
            site_title: "KoShelf".to_string(),
            include_unread: true,
            library_paths: vec![output_dir.join("library")],
            metadata_location: MetadataLocation::InBookFolder,
            statistics_db_paths: vec![],
            kobo_db_path,
            heatmap_scale_max: None,
            time_config: TimeConfig::from_cli(&None, &None).expect("time config"),
            min_pages_per_day: None,
            min_time_per_day: None,
            include_all_stats: false,
            is_internal_server: false,
            language: "en_US".to_string(),
            use_stable_page_metadata: true,
            auth_enabled: false,
            writeback_enabled: false,
            include_files: false,
            runtime_data_policy,
        }
    }

    #[test]
    fn rebuild_requires_full_library_sync_for_kobo_db_or_extensionless_changes() {
        let dir = tempfile::tempdir().expect("temp dir");
        let kobo_db_path = dir.path().join("KoboReader.sqlite");
        let config = test_config(dir.path(), Some(kobo_db_path.clone()));

        let mut db_changed = HashSet::new();
        db_changed.insert(kobo_db_path);
        assert!(requires_full_library_sync(&db_changed, &config));

        let mut extensionless_changed = HashSet::new();
        extensionless_changed.insert(dir.path().join("library").join("kobo-content-id"));
        assert!(requires_full_library_sync(&extensionless_changed, &config));

        let mut regular_book_changed = HashSet::new();
        regular_book_changed.insert(dir.path().join("library").join("book.epub"));
        assert!(!requires_full_library_sync(&regular_book_changed, &config));
    }

    #[test]
    fn sidecar_derives_book_filename_by_metadata_filename() {
        assert_eq!(
            LibraryItemFormat::book_filename_from_sidecar("Book.fb2", "metadata.zip.lua"),
            Some("Book.fb2.zip".to_string())
        );
        // A plain FB2 whose name itself ends in .fb2 (Book.fb2.fb2) shares the
        // sidecar stem but carries metadata.fb2.lua, not metadata.zip.lua.
        assert_eq!(
            LibraryItemFormat::book_filename_from_sidecar("Book.fb2", "metadata.fb2.lua"),
            Some("Book.fb2.fb2".to_string())
        );
        assert_eq!(
            LibraryItemFormat::book_filename_from_sidecar("Book", "metadata.fb2.lua"),
            Some("Book.fb2".to_string())
        );
        assert_eq!(
            LibraryItemFormat::book_filename_from_sidecar("Book", "metadata.zip.lua"),
            None
        );
    }

    #[test]
    fn sdr_path_derivation_finds_fb2_zip_book() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join("Book.fb2.zip");
        std::fs::write(&book_path, b"book").expect("book file");
        let sdr_path = dir.path().join("Book.fb2.sdr");
        std::fs::create_dir_all(&sdr_path).expect("sdr dir");

        assert_eq!(derive_book_path_from_sdr_path(&sdr_path), Some(book_path));
    }

    #[test]
    fn sdr_path_derivation_finds_double_extension_fb2_book() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join("Book.fb2.fb2");
        std::fs::write(&book_path, b"book").expect("book file");
        let sdr_path = dir.path().join("Book.fb2.sdr");
        std::fs::create_dir_all(&sdr_path).expect("sdr dir");

        assert_eq!(derive_book_path_from_sdr_path(&sdr_path), Some(book_path));
    }
}
