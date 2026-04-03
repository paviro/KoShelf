use crate::app::config::{CommonArgs, SiteConfig, parse_time_to_seconds};
use crate::pipeline::ingest::{load_reading_data, update_library};
use crate::pipeline::media::{self, resolve_media_dirs};
use crate::pipeline::recap::regenerate_share_images;
use crate::server::api::responses::site::{PasswordPolicy, SiteAuth, SiteCapabilities, SiteData};
use crate::shelf::time_config::TimeConfig;
use crate::source::scanner::MetadataLocation;
use crate::store::lifecycle::{
    RuntimeDataPathOptions, RuntimeDataPolicy, resolve_runtime_data_policy,
};
use crate::store::memory::ReadingData;
use crate::store::sqlite::repo::LibraryRepository;
use crate::store::sqlite::{open_library_pool, run_library_migrations};
use anyhow::{Context, Result};
use log::info;
use std::collections::HashSet;
use std::path::PathBuf;

fn metadata_location(common: &CommonArgs) -> MetadataLocation {
    if let Some(ref docsettings_path) = common.docsettings_path {
        MetadataLocation::DocSettings(docsettings_path.clone())
    } else if let Some(ref hashdocsettings_path) = common.hashdocsettings_path {
        MetadataLocation::HashDocSettings(hashdocsettings_path.clone())
    } else {
        MetadataLocation::InBookFolder
    }
}

fn resolve_data_policy(common: &CommonArgs) -> RuntimeDataPolicy {
    resolve_runtime_data_policy(&RuntimeDataPathOptions {
        data_path: common.data_path.clone(),
    })
}

fn build_site_config(
    common: &CommonArgs,
    output_dir: PathBuf,
    is_internal_server: bool,
    auth_enabled: bool,
    writeback_enabled: bool,
    include_files: bool,
    runtime_data_policy: RuntimeDataPolicy,
) -> Result<SiteConfig> {
    let heatmap_scale_max =
        parse_time_to_seconds(&common.heatmap_scale_max).with_context(|| {
            format!(
                "Invalid heatmap-scale-max format: {}",
                common.heatmap_scale_max
            )
        })?;

    let min_time_per_day = if let Some(ref min_time_str) = common.min_time_per_day {
        parse_time_to_seconds(min_time_str)
            .with_context(|| format!("Invalid min-time-per-day format: {}", min_time_str))?
    } else {
        None
    };

    Ok(SiteConfig {
        output_dir,
        site_title: common.title.clone(),
        include_unread: common.include_unread,
        library_paths: common.library_path.clone(),
        metadata_location: metadata_location(common),
        statistics_db_path: common.statistics_db.clone(),
        heatmap_scale_max,
        time_config: TimeConfig::from_cli(&common.timezone, &common.day_start_time)?,
        min_pages_per_day: common.min_pages_per_day,
        min_time_per_day,
        include_all_stats: common.include_all_stats,
        is_internal_server,
        language: common.language.clone(),
        use_stable_page_metadata: !common.ignore_stable_page_metadata,
        auth_enabled,
        writeback_enabled,
        include_files,
        runtime_data_policy,
    })
}

/// State produced by the shared pipeline initialization.
pub(crate) struct PipelineState {
    pub config: SiteConfig,
    pub repo: LibraryRepository,
    pub reading_data: Option<ReadingData>,
    pub has_reading_data: bool,
    pub site_data: SiteData,
    pub generated_at: String,
    pub _runtime_temp_dir: Option<tempfile::TempDir>,
}

/// Run the shared pipeline: DB setup, library update, statistics, recap images, site metadata.
pub(crate) async fn initialize_pipeline(
    common: &CommonArgs,
    output_dir: PathBuf,
    is_internal_server: bool,
    auth_enabled: bool,
    writeback_enabled: bool,
    include_files: bool,
) -> Result<PipelineState> {
    let mut runtime_data_policy = resolve_data_policy(common);
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

    // In ephemeral mode, use a separate temp directory for runtime data.
    let runtime_temp_dir = if !runtime_data_policy.is_persistent() {
        let tmp =
            tempfile::tempdir().context("Failed to create temporary runtime data directory")?;
        runtime_data_policy.set_resolved_data_dir(tmp.path().to_path_buf());
        Some(tmp)
    } else {
        None
    };

    let config = build_site_config(
        common,
        output_dir,
        is_internal_server,
        auth_enabled,
        writeback_enabled,
        include_files,
        runtime_data_policy,
    )?;

    if is_internal_server {
        let data_dir = config
            .runtime_data_policy
            .persistent_data_dir()
            .context("Serve mode requires --data-path")?;
        std::fs::create_dir_all(data_dir).with_context(|| {
            format!("Failed to create runtime data directory at {:?}", data_dir)
        })?;
    }

    if let Some(db_path) = config.runtime_data_policy.library_db_path() {
        info!("Runtime library DB path: {:?}", db_path);
    }

    // ── 1. Create DB ─────────────────────────────────────────────────
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

    // ── 2. Create media directories ──────────────────────────────────
    let media_dirs = resolve_media_dirs(&config.output_dir, is_internal_server);
    media::create_media_directories(&media_dirs)?;

    // ── 3. Update library ────────────────────────────────────────────
    if !config.library_paths.is_empty() {
        update_library(
            &config,
            &repo,
            &media_dirs.covers_dir,
            &media_dirs.files_dir,
        )
        .await?;

        match repo.load_all_item_ids().await {
            Ok(ids) => {
                let id_set: HashSet<String> = ids.into_iter().collect();
                media::cleanup_stale_covers_by_ids(&id_set, &media_dirs.covers_dir)?;
                if config.is_internal_server {
                    media::cleanup_stale_files_by_ids(&id_set, &media_dirs.files_dir)?;
                }
            }
            Err(e) => log::warn!("Failed to load item IDs for cover cleanup: {}", e),
        }
    }

    // ── 4. Load statistics ───────────────────────────────────────────
    let reading_data = load_reading_data(&config, &repo).await?;
    let has_reading_data = reading_data
        .as_ref()
        .is_some_and(|rd| !rd.stats_data.page_stats.is_empty());

    // ── 5. Generate recap images ─────────────────────────────────────
    if let Some(ref rd) = reading_data {
        regenerate_share_images(
            &rd.stats_data,
            &repo,
            &rd.page_scaling,
            &media_dirs.recap_dir,
            &config.time_config,
            true,
        )
        .await?;
    }

    // ── 6. Build site metadata ───────────────────────────────────────
    let generated_at = config.time_config.now_rfc3339();
    let (has_books, has_comics) = repo.query_content_type_flags().await?;
    let auth = if config.auth_enabled {
        Some(SiteAuth {
            authenticated: false,
            password_policy: PasswordPolicy {
                min_chars: crate::server::auth::password::MIN_PASSWORD_CHARS,
            },
        })
    } else {
        None
    };

    let site_data = SiteData {
        title: config.site_title.clone(),
        language: config.language.clone(),
        capabilities: SiteCapabilities {
            has_books,
            has_comics,
            has_reading_data,
            has_files: is_internal_server || config.include_files,
            has_writeback: config.writeback_enabled,
        },
        auth,
    };

    Ok(PipelineState {
        config,
        repo,
        reading_data,
        has_reading_data,
        site_data,
        generated_at,
        _runtime_temp_dir: runtime_temp_dir,
    })
}
