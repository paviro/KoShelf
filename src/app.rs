use crate::cli::{Cli, parse_time_to_seconds};
use crate::config::SiteConfig;
use crate::library::{FileWatcher, MetadataLocation};
use crate::runtime::{SnapshotStore, SnapshotUpdateNotifier};
use crate::server::WebServer;
use crate::snapshot_builder::SnapshotBuilder;
use crate::time_config::TimeConfig;
use anyhow::{Context, Result};
use log::info;
use std::sync::Arc;
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

fn plan_output(cli: &Cli) -> Result<OutputPlan> {
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
        (None, _) => {
            // For server mode, use a temp media-cache directory cleaned up on exit.
            let tmp = tempfile::tempdir().context("Failed to create temporary output directory")?;
            Ok(OutputPlan {
                output_dir: tmp.path().to_path_buf(),
                _temp_dir: Some(tmp),
                mode: RunMode::Serve,
            })
        }
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

    // Determine output directory + run mode
    let plan = plan_output(&cli)?;

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
    };

    // Create snapshot builder - it handles library scanning and stats loading internally.
    let snapshot_builder = SnapshotBuilder::new(config.clone());

    // Build initial in-memory snapshot.
    let initial_snapshot = snapshot_builder.refresh_snapshot().await?;
    if !is_internal_server {
        initial_snapshot.write_to_data_dir(&plan.output_dir.join("data"))?;
    }

    match plan.mode {
        RunMode::StaticExport => Ok(()),

        RunMode::WatchStatic => {
            info!("Watching library changes to refresh static shell/assets and /data export.");
            let file_watcher = FileWatcher::new(config, None, None);
            if let Err(e) = file_watcher.run().await {
                log::error!("File watcher error: {}", e);
            }
            Ok(())
        }

        RunMode::Serve => {
            info!(
                "Running in serve mode with runtime snapshot refresh and media cache at: {:?}",
                plan.output_dir
            );
            let initial_generated_at = initial_snapshot
                .generated_at()
                .map(str::to_owned)
                .unwrap_or_else(|| config.time_config.now_formatted());
            let snapshot_store = Arc::new(SnapshotStore::new());
            snapshot_store.replace(initial_snapshot);
            let update_notifier = SnapshotUpdateNotifier::new(initial_generated_at);

            // Start file watcher with snapshot updates.
            let file_watcher =
                FileWatcher::new(config, Some(snapshot_store.clone()), Some(update_notifier.clone()));

            // Start web server (runtime media cache is served from `plan.output_dir`).
            let web_server =
                WebServer::new(plan.output_dir, cli.port, snapshot_store, update_notifier);

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
