use crate::cli::{Cli, parse_time_to_seconds};
use crate::config::SiteConfig;
use crate::contracts::common::ApiMeta;
use crate::contracts::site::{SiteCapabilities, SiteResponse};
use crate::domain::library::{LibraryBuildMode, LibraryBuildPipeline};
use crate::infra::lifecycle::{
    RuntimeDataPathOptions, RuntimeDataPolicy, resolve_runtime_data_policy,
};
use crate::infra::sqlite::library_db::open_library_pool;
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::infra::sqlite::migrations::run_library_migrations;
use crate::library::MetadataLocation;
use crate::runtime::export::{ExportConfig, export_data_files};
use crate::runtime::media::{self, resolve_media_dirs};
use crate::runtime::{ReadingDataStore, RuntimeObservability, SiteStore, UpdateNotifier};
use crate::server::WebServer;
use crate::time_config::TimeConfig;
use anyhow::{Context, Result};
use log::info;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

enum RunMode {
    StaticExport,
    WatchStatic,
    Serve,
}

struct OutputPlan {
    output_dir: std::path::PathBuf,
    /// Keep alive for Serve mode so the temp directory is cleaned up on exit.
    _temp_dir: Option<TempDir>,
    mode: RunMode,
}

fn plan_output(cli: &Cli, runtime_data_policy: &RuntimeDataPolicy) -> Result<OutputPlan> {
    match (&cli.output, cli.watch) {
        (Some(dir), false) => Ok(OutputPlan {
            output_dir: dir.clone(),
            _temp_dir: None,
            mode: RunMode::StaticExport,
        }),
        (Some(dir), true) => Ok(OutputPlan {
            output_dir: dir.clone(),
            _temp_dir: None,
            mode: RunMode::WatchStatic,
        }),
        (None, _) => match runtime_data_policy.persistent_data_dir() {
            Some(data_dir) => Ok(OutputPlan {
                output_dir: data_dir.to_path_buf(),
                _temp_dir: None,
                mode: RunMode::Serve,
            }),
            None => {
                // In ephemeral server mode, use a temp runtime-data directory cleaned up on exit.
                let tmp =
                    tempfile::tempdir().context("Failed to create temporary output directory")?;
                Ok(OutputPlan {
                    output_dir: tmp.path().to_path_buf(),
                    _temp_dir: Some(tmp),
                    mode: RunMode::Serve,
                })
            }
        },
    }
}

fn metadata_location(cli: &Cli) -> MetadataLocation {
    if let Some(ref docsettings_path) = cli.docsettings_path {
        MetadataLocation::DocSettings(docsettings_path.clone())
    } else if let Some(ref hashdocsettings_path) = cli.hashdocsettings_path {
        MetadataLocation::HashDocSettings(hashdocsettings_path.clone())
    } else {
        MetadataLocation::InBookFolder
    }
}

fn resolve_runtime_data_policy_for_run(cli: &Cli) -> RuntimeDataPolicy {
    let cli_overrides = RuntimeDataPathOptions {
        data_dir: cli.data_dir.clone(),
    };

    resolve_runtime_data_policy(&cli_overrides)
}

/// Build a `SiteResponse` from the current library items and reading data availability.
fn build_site_response(
    items: &[crate::models::LibraryItem],
    has_reading_data: bool,
    site_title: &str,
    language: &str,
    time_config: &TimeConfig,
) -> SiteResponse {
    SiteResponse {
        meta: ApiMeta {
            version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: time_config.now_rfc3339(),
        },
        title: site_title.to_string(),
        language: language.to_string(),
        capabilities: SiteCapabilities {
            has_books: items.iter().any(|item| item.is_book()),
            has_comics: items.iter().any(|item| item.is_comic()),
            has_reading_data,
        },
    }
}

/// Run KoShelf with the provided CLI args.
///
/// `src/main.rs` is responsible for logging init and Clap argument parsing.
pub async fn run(cli: Cli) -> Result<()> {
    info!("Starting KOShelf...");

    cli.validate()?;

    // Parse heatmap scale max
    let heatmap_scale_max = parse_time_to_seconds(&cli.heatmap_scale_max).with_context(|| {
        format!(
            "Invalid heatmap-scale-max format: {}",
            cli.heatmap_scale_max
        )
    })?;

    // Parse min time per day
    let min_time_per_day = if let Some(ref min_time_str) = cli.min_time_per_day {
        parse_time_to_seconds(min_time_str)
            .with_context(|| format!("Invalid min-time-per-day format: {}", min_time_str))?
    } else {
        None
    };

    // Build time configuration from CLI
    let time_config = TimeConfig::from_cli(&cli.timezone, &cli.day_start_time)?;

    let mut runtime_data_policy = resolve_runtime_data_policy_for_run(&cli);
    match runtime_data_policy.persistent_data_dir() {
        Some(path) => info!(
            "Runtime data policy: persistent ({:?}, source={})",
            path,
            runtime_data_policy.source.as_str()
        ),
        None => info!(
            "Runtime data policy: ephemeral temp dir (source={})",
            runtime_data_policy.source.as_str()
        ),
    }

    // Determine output directory + run mode
    let plan = plan_output(&cli, &runtime_data_policy)?;

    // In ephemeral mode the temp directory doubles as the runtime data
    // directory (library DB, statistics DB copy, cover cache).
    if !runtime_data_policy.is_persistent() {
        runtime_data_policy.set_resolved_data_dir(plan.output_dir.clone());
    }

    if let Some(db_path) = runtime_data_policy.library_db_path() {
        info!("Runtime library DB path: {:?}", db_path);
    }

    // Determine if we're running with internal web server (enables runtime update events)
    let is_internal_server = matches!(plan.mode, RunMode::Serve);

    // Create site config - bundles all configuration options.
    let config = SiteConfig {
        output_dir: plan.output_dir.clone(),
        site_title: cli.title.clone(),
        include_unread: cli.include_unread,
        library_paths: cli.library_path.clone(),
        metadata_location: metadata_location(&cli),
        statistics_db_path: cli.statistics_db.clone(),
        heatmap_scale_max,
        time_config: time_config.clone(),
        min_pages_per_day: cli.min_pages_per_day,
        min_time_per_day,
        include_all_stats: cli.include_all_stats,
        is_internal_server,
        language: cli.language.clone(),
        use_stable_page_metadata: !cli.ignore_stable_page_metadata,
        runtime_data_policy,
    };

    let observability = RuntimeObservability::default();

    // ── Shared ingest: scan library + load statistics (single pass) ──────
    let startup_started_at = Instant::now();

    if is_internal_server {
        info!(
            "Scanning library and serving media cache from: {:?}",
            config.output_dir
        );
    } else {
        info!(
            "Scanning library and exporting static shell/assets to: {:?}",
            config.output_dir
        );
    }

    let ingest_result = crate::runtime::ingest(&config).await?;
    let reading_data = ingest_result.reading_data(&config);
    let has_reading_data = ingest_result.has_reading_data();

    observability.record_startup_library_build_duration(startup_started_at.elapsed());
    info!(
        "Library scan completed in {} ms ({} items, {} filtered)",
        observability.snapshot().startup_library_build_duration_ms,
        ingest_result.raw_items.len(),
        ingest_result.filtered_items.len(),
    );

    // Destructure so raw_items can be consumed by the library DB pipeline
    // while filtered_items and stats_data remain available.
    let crate::runtime::IngestResult {
        raw_items,
        filtered_items,
        stats_data,
        ..
    } = ingest_result;

    // ── Media assets: covers, recap images, static frontend ─────────────
    let media_dirs = resolve_media_dirs(&config.output_dir, is_internal_server);
    media::create_media_directories(&media_dirs)?;

    if !is_internal_server {
        media::sync_static_frontend(&config.output_dir, has_reading_data)?;
    }

    media::generate_covers(&filtered_items, &media_dirs.covers_dir).await?;
    media::cleanup_stale_covers(&filtered_items, &media_dirs.covers_dir)?;

    if let Some(ref sd) = stats_data {
        crate::runtime::recap::generate_recap_share_images(
            sd,
            &filtered_items,
            config.use_stable_page_metadata,
            &media_dirs.recap_dir,
            config.statistics_db_path.as_deref(),
            &config.time_config,
        )
        .await?;
    }

    // ── Library DB pipeline: populate library.sqlite ─────────────────────
    let library_repo = if let Some(db_path) = config.runtime_data_policy.library_db_path() {
        let db_build_start = Instant::now();
        let pool = open_library_pool(&db_path)
            .await
            .context("Failed to open library DB for build pipeline")?;
        run_library_migrations(&pool)
            .await
            .context("Failed to run library DB migrations")?;

        let repo = LibraryRepository::new(pool);

        let build_mode = if config.runtime_data_policy.is_persistent() {
            let count = repo.count_items().await.unwrap_or(0);
            if count > 0 {
                LibraryBuildMode::Incremental
            } else {
                LibraryBuildMode::Full
            }
        } else {
            LibraryBuildMode::Full
        };

        let pipeline = LibraryBuildPipeline::new(
            &repo,
            config.include_unread,
            config.use_stable_page_metadata,
            &config.time_config,
        );

        let result = pipeline.build(build_mode, raw_items).await?;
        info!(
            "Library DB build ({:?}) completed in {} ms: {} scanned, {} upserted, {} removed, {} collisions",
            build_mode,
            db_build_start.elapsed().as_millis(),
            result.scanned_files,
            result.upserted_items,
            result.removed_items,
            result.collision_count,
        );

        Some(repo)
    } else {
        None
    };

    // ── Static data file export ─────────────────────────────────────────
    if !is_internal_server && let Some(ref repo) = library_repo {
        let export_config = ExportConfig {
            site_title: cli.title.clone(),
            language: cli.language.clone(),
        };
        export_data_files(
            &plan.output_dir.join("data"),
            repo,
            reading_data.as_ref(),
            &export_config,
        )
        .await?;
    }

    // ── Build site metadata ─────────────────────────────────────────────
    let site_response = build_site_response(
        &filtered_items,
        has_reading_data,
        &config.site_title,
        &config.language,
        &config.time_config,
    );

    match plan.mode {
        RunMode::StaticExport => {
            info!("Static export completed.");
            Ok(())
        }

        RunMode::WatchStatic => {
            info!("Watching library changes to refresh static shell/assets and /data export.");
            let file_watcher = crate::library::FileWatcher::new(
                config,
                None,
                None,
                None,
                library_repo,
                observability.clone(),
            );
            if let Err(e) = file_watcher.run().await {
                log::error!("File watcher error: {}", e);
            }
            Ok(())
        }

        RunMode::Serve => {
            let library_repo = library_repo.context("Library DB is required for serve mode")?;

            let revision_epoch = format!("serve_{}", site_response.meta.generated_at.as_str());
            let initial_generated_at = site_response.meta.generated_at.clone();

            let site_store = Arc::new(SiteStore::new());
            site_store.replace(site_response);

            let reading_data_store = Arc::new(ReadingDataStore::new());
            if let Some(rd) = reading_data {
                reading_data_store.replace(rd);
            }

            let update_notifier = UpdateNotifier::with_observability(
                revision_epoch,
                initial_generated_at,
                observability.clone(),
            );

            // Start file watcher with site store updates.
            let file_watcher = crate::library::FileWatcher::new(
                config,
                Some(site_store.clone()),
                Some(reading_data_store.clone()),
                Some(update_notifier.clone()),
                Some(library_repo.clone()),
                observability,
            );

            // Start web server (runtime media cache is served from `plan.output_dir`).
            let web_server = WebServer::new(
                plan.output_dir,
                cli.port,
                site_store,
                reading_data_store,
                update_notifier,
                library_repo,
            );

            // Run both file watcher and web server concurrently
            tokio::select! {
                result = file_watcher.run() => {
                    if let Err(e) = result {
                        log::error!("File watcher error: {}", e);
                    }
                }
                result = web_server.run() => {
                    if let Err(e) = result {
                        log::error!("Web server error: {}", e);
                    }
                }
            }

            Ok(())
        }
    }
}
