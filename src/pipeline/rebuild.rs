//! Rebuild orchestration: targeted (incremental) rebuild pipeline.
//!
//! Extracted from the file watcher so the rebuild logic is testable and
//! the watcher module is limited to event setup, debouncing, and dispatch.

use crate::app::config::SiteConfig;
use crate::pipeline::export::{ExportConfig, export_data_files};
use crate::pipeline::ingest::{load_reading_data, reconcile_paths, sync_library};
use crate::pipeline::media::{self, resolve_media_dirs};
use crate::pipeline::recap::regenerate_share_images;
use crate::server::api::responses::site::{SiteCapabilities, SiteData};
use crate::source::sqlite_snapshot::is_sqlite_db_or_companion;
use crate::store::memory::{SharedReadingDataStore, SharedSiteStore, UpdateNotifier};
use crate::store::sqlite::repo::LibraryRepository;
use anyhow::Result;
use log::{info, warn};
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

    let media_dirs = resolve_media_dirs(&config.output_dir);
    if let Err(e) = media::create_media_directories(&media_dirs) {
        warn!("Failed to create media directories: {}", e);
    }

    let update = if full_library_sync_required {
        sync_library(config, repo, &media_dirs).await?
    } else {
        reconcile_paths(config, repo, &media_dirs, &accumulated_paths).await?
    };
    log::debug!(
        "Library reconciliation: {} unchanged, {} changed, {} added, {} removed",
        update.unchanged,
        update.changed,
        update.added,
        update.removed
    );
    let ingest_stats = update.ingest_stats.unwrap_or_default();
    let deleted_count = update.removed;
    let library_update = Some(update);

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
                    has_archive: config.archive_enabled,
                    has_library_management: config.library_management_enabled,
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

    // A reconnected or lost library root requires a full rescan.
    let library_root_changed = accumulated_paths
        .iter()
        .any(|path| config.library_paths.contains(path));

    kobo_db_changed
        || kobo_extensionless_changed
        || library_root_changed
        || accumulated_paths.iter().any(|path| {
            config
                .metadata_location
                .central_paths()
                .iter()
                .any(|root| root.starts_with(path))
        })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::requires_full_library_sync;
    use crate::app::config::SiteConfig;
    #[cfg(test)]
    use crate::shelf::models::LibraryItemFormat;
    use crate::shelf::time_config::TimeConfig;
    use crate::source::scanner::MetadataLocation;
    use crate::store::lifecycle::{RuntimeDataPathOptions, resolve_runtime_data_policy};
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    pub(crate) fn test_config(output_dir: &Path, kobo_db_path: Option<PathBuf>) -> SiteConfig {
        let mut runtime_data_policy =
            resolve_runtime_data_policy(&RuntimeDataPathOptions::default());
        runtime_data_policy.set_resolved_data_dir(output_dir.join("runtime"));

        SiteConfig {
            output_dir: output_dir.to_path_buf(),
            site_title: "KoShelf".to_string(),
            include_unread: true,
            library_paths: vec![output_dir.join("library")],
            metadata_location: MetadataLocation::InBookFolder,
            document_metadata_folder: None,
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
            library_management_enabled: false,
            archive_enabled: true,
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

        std::fs::create_dir_all(dir.path().join("library")).unwrap();
        std::fs::write(dir.path().join("library/book.epub"), b"book").unwrap();
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
            Some("Book.zip".into())
        );
    }

    #[test]
    fn central_files_are_targeted_but_root_changes_require_full_sync() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(dir.path(), None);
        let doc = dir.path().join("doc");
        let hash = dir.path().join("hash");
        config.metadata_location = MetadataLocation::Central {
            docsettings: doc.clone(),
            hashdocsettings: hash.clone(),
        };
        for root in [doc, hash] {
            assert!(requires_full_library_sync(
                &HashSet::from([root.clone()]),
                &config
            ));
            for name in [
                "Book.sdr/metadata.epub.lua",
                "Book.sdr/metadata.epub.lua.old",
                "Book.sdr/custom_metadata.lua",
                "Book.sdr",
            ] {
                assert!(!requires_full_library_sync(
                    &HashSet::from([root.join(name)]),
                    &config
                ));
            }
        }
    }
}
