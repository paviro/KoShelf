use super::cli::{Cli, parse_time_to_seconds};
use super::config::SiteConfig;
use crate::infra::watcher::FileWatcher;
use crate::runtime::export::{ExportConfig, export_data_files};
use crate::runtime::ingest::{load_reading_data, update_library};
use crate::runtime::media::{self, resolve_media_dirs};
use crate::runtime::recap::generate_recap_share_images;
use crate::server::WebServer;
use crate::server::api::responses::site::{SiteCapabilities, SiteData};
use crate::shelf::time_config::TimeConfig;
use crate::source::scanner::MetadataLocation;
use crate::store::lifecycle::{
    RuntimeDataPathOptions, RuntimeDataPolicy, resolve_runtime_data_policy,
};
use crate::store::memory::{ReadingDataStore, SiteStore, UpdateNotifier};
use crate::store::sqlite::migrations::run_library_migrations;
use crate::store::sqlite::pool::open_library_pool;
use crate::store::sqlite::repo::LibraryRepository;
use anyhow::{Context, Result};
use log::info;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

enum RunMode {
    StaticExport,
    WatchStatic,
    Serve,
}

struct OutputPlan {
    output_dir: PathBuf,
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

    let heatmap_scale_max = parse_time_to_seconds(&cli.heatmap_scale_max).with_context(|| {
        format!(
            "Invalid heatmap-scale-max format: {}",
            cli.heatmap_scale_max
        )
    })?;

    let min_time_per_day = if let Some(ref min_time_str) = cli.min_time_per_day {
        parse_time_to_seconds(min_time_str)
            .with_context(|| format!("Invalid min-time-per-day format: {}", min_time_str))?
    } else {
        None
    };

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

    let plan = plan_output(&cli, &runtime_data_policy)?;
    let is_internal_server = matches!(plan.mode, RunMode::Serve);

    // In ephemeral mode, determine where runtime data (library DB) lives.
    let _runtime_temp_dir = if !runtime_data_policy.is_persistent() {
        if is_internal_server {
            // Serve mode: output_dir is already a temp dir; reuse it.
            runtime_data_policy.set_resolved_data_dir(plan.output_dir.clone());
            None
        } else {
            // Static export: use a separate temp dir so library.sqlite
            // doesn't land in the user's output directory.
            let tmp =
                tempfile::tempdir().context("Failed to create temporary runtime data directory")?;
            runtime_data_policy.set_resolved_data_dir(tmp.path().to_path_buf());
            Some(tmp)
        }
    } else {
        None
    };

    if let Some(db_path) = runtime_data_policy.library_db_path() {
        info!("Runtime library DB path: {:?}", db_path);
    }

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

    // ── 1. Create DB (always) ──────────────────────────────────────────
    let db_path = config
        .runtime_data_policy
        .library_db_path()
        .context("Failed to resolve library DB path")?;
    let pool = open_library_pool(&db_path)
        .await
        .context("Failed to open library DB")?;
    run_library_migrations(&pool)
        .await
        .context("Failed to run library DB migrations")?;
    let repo = LibraryRepository::new(pool, config.use_stable_page_metadata);

    // ── 2. Create media directories ────────────────────────────────────
    let media_dirs = resolve_media_dirs(&config.output_dir, is_internal_server);
    media::create_media_directories(&media_dirs)?;

    // ── 3. Update library ─────────────────────────────────────────────
    if !config.library_paths.is_empty() {
        update_library(&config, &repo, &media_dirs.covers_dir).await?;

        // Cleanup stale covers.
        match repo.load_all_item_ids().await {
            Ok(ids) => {
                let id_set: HashSet<String> = ids.into_iter().collect();
                media::cleanup_stale_covers_by_ids(&id_set, &media_dirs.covers_dir)?;
            }
            Err(e) => log::warn!("Failed to load item IDs for cover cleanup: {}", e),
        }
    }

    // ── 4. Load statistics ─────────────────────────────────────────────
    let reading_data = load_reading_data(&config, &repo).await?;
    let has_reading_data = reading_data
        .as_ref()
        .is_some_and(|rd| !rd.stats_data.page_stats.is_empty());

    // ── 5. Sync static frontend ────────────────────────────────────────
    if !is_internal_server {
        media::sync_static_frontend(&config.output_dir, has_reading_data)?;
    }

    // ── 6. Generate recap images ───────────────────────────────────────
    if let Some(ref rd) = reading_data {
        generate_recap_share_images(
            &rd.stats_data,
            &repo,
            &rd.page_scaling,
            &media_dirs.recap_dir,
            config.statistics_db_path.as_deref(),
            &config.time_config,
        )
        .await?;
    }

    // ── 7. Build site metadata from DB ─────────────────────────────────
    let generated_at = config.time_config.now_rfc3339();
    let (has_books, has_comics) = repo.query_content_type_flags().await?;
    let site_data = SiteData {
        title: config.site_title.clone(),
        language: config.language.clone(),
        capabilities: SiteCapabilities {
            has_books,
            has_comics,
            has_reading_data,
        },
    };

    // ── 8. Static data file export ─────────────────────────────────────
    if !is_internal_server {
        let export_config = ExportConfig {
            site_title: cli.title.clone(),
            language: cli.language.clone(),
        };
        export_data_files(
            &plan.output_dir.join("data"),
            &repo,
            reading_data.as_ref(),
            &export_config,
        )
        .await?;
    }

    // ── 9. Enter run mode ──────────────────────────────────────────────
    match plan.mode {
        RunMode::StaticExport => {
            info!("Static export completed.");
            Ok(())
        }

        RunMode::WatchStatic => {
            info!("Watching library changes to refresh static shell/assets and /data export.");
            let file_watcher = FileWatcher::new(config, None, None, None, Some(repo));
            if let Err(e) = file_watcher.run().await {
                log::error!("File watcher error: {}", e);
            }
            Ok(())
        }

        RunMode::Serve => {
            let revision_epoch = format!("serve_{}", &generated_at);
            let initial_generated_at = generated_at;

            let site_store = Arc::new(SiteStore::new());
            site_store.replace(site_data);

            let reading_data_store = Arc::new(ReadingDataStore::new());
            if let Some(rd) = reading_data {
                reading_data_store.replace(rd);
            }

            let update_notifier = UpdateNotifier::new(revision_epoch, initial_generated_at);

            let file_watcher = FileWatcher::new(
                config,
                Some(site_store.clone()),
                Some(reading_data_store.clone()),
                Some(update_notifier.clone()),
                Some(repo.clone()),
            );

            let web_server = WebServer::new(
                plan.output_dir,
                cli.port,
                site_store,
                reading_data_store,
                update_notifier,
                repo,
            );

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
