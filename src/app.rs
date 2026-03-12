use crate::cli::{Cli, parse_time_to_seconds};
use crate::config::SiteConfig;
use crate::domain::library::{LibraryBuildMode, LibraryBuildPipeline};
use crate::infra::lifecycle::{
    RuntimeDataPathOptions, RuntimeDataPolicy, resolve_runtime_data_policy,
};
use crate::infra::sqlite::library_db::open_library_pool;
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::infra::sqlite::migrations::run_library_migrations;
use crate::library::{FileWatcher, MetadataLocation, scan_library};
use crate::runtime::{DomainUpdateNotifier, ReadingDataStore, RuntimeObservability, SnapshotStore};
use crate::server::WebServer;
use crate::snapshot_builder::SnapshotBuilder;
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

    // Create site config - bundles all snapshot/export options.
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

    // Create snapshot builder - it handles library scanning and stats loading internally.
    let snapshot_builder = SnapshotBuilder::new(config.clone());
    let observability = RuntimeObservability::default();

    // Build initial in-memory library data.
    let startup_library_build_started_at = Instant::now();
    let initial_result = snapshot_builder.refresh_snapshot().await?;
    observability.record_startup_library_build_duration(startup_library_build_started_at.elapsed());
    info!(
        "Initial library build completed in {} ms",
        observability.snapshot().startup_library_build_duration_ms
    );

    let initial_snapshot = initial_result.snapshot;
    let initial_reading_data = initial_result.reading_data;

    // ── Library build pipeline: populate library.sqlite ────────────────
    //
    // The repository is kept alive so serve-mode handlers can query it.
    // The legacy snapshot path remains during the transition for
    // activity/completion endpoints that haven't migrated yet.
    let library_repo = if let Some(db_path) = config.runtime_data_policy.library_db_path() {
        let db_build_start = Instant::now();
        let pool = open_library_pool(&db_path)
            .await
            .context("Failed to open library DB for build pipeline")?;
        run_library_migrations(&pool)
            .await
            .context("Failed to run library DB migrations")?;

        let repo = LibraryRepository::new(pool);

        // Determine build mode: persistent DB with existing items → incremental.
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

        // Scan library (separate from snapshot builder scan during transition).
        let scanned_items = if !config.library_paths.is_empty() {
            let (items, _library_md5s) =
                scan_library(&config.library_paths, &config.metadata_location).await?;
            items
        } else {
            Vec::new()
        };

        let pipeline = LibraryBuildPipeline::new(
            &repo,
            config.include_unread,
            config.use_stable_page_metadata,
            &config.time_config,
        );

        let result = pipeline.build(build_mode, scanned_items).await?;
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

    if !is_internal_server {
        initial_snapshot.write_to_data_dir(&plan.output_dir.join("data"))?;
    }

    match plan.mode {
        RunMode::StaticExport => Ok(()),

        RunMode::WatchStatic => {
            info!("Watching library changes to refresh static shell/assets and /data export.");
            let file_watcher = FileWatcher::new(
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

            let initial_generated_at = initial_snapshot
                .generated_at()
                .map(str::to_owned)
                .unwrap_or_else(|| config.time_config.now_rfc3339());
            let revision_epoch = format!("serve_{}", initial_generated_at);
            let snapshot_store = Arc::new(SnapshotStore::new());
            snapshot_store.replace(initial_snapshot);

            let reading_data_store = Arc::new(ReadingDataStore::new());
            if let Some(rd) = initial_reading_data {
                reading_data_store.replace(rd);
            }

            let update_notifier = DomainUpdateNotifier::with_observability(
                revision_epoch,
                initial_generated_at,
                observability.clone(),
            );

            // Start file watcher with snapshot updates.
            let file_watcher = FileWatcher::new(
                config,
                Some(snapshot_store.clone()),
                Some(reading_data_store.clone()),
                Some(update_notifier.clone()),
                Some(library_repo.clone()),
                observability,
            );

            // Start web server (runtime media cache is served from `plan.output_dir`).
            let web_server = WebServer::new(
                plan.output_dir,
                cli.port,
                snapshot_store,
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
