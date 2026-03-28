use crate::app::config::{
    Cli, CliCommand, CommonArgs, ExportArgs, ServeArgs, SiteConfig, parse_time_to_seconds,
    parse_trusted_proxy_nets,
};
use crate::pipeline::export::{ExportConfig, export_data_files};
use crate::pipeline::frontend;
use crate::pipeline::ingest::{load_reading_data, update_library};
use crate::pipeline::media::{self, resolve_media_dirs};
use crate::pipeline::recap::regenerate_share_images;
use crate::pipeline::watcher::FileWatcher;
use crate::server::api::responses::site::{PasswordPolicy, SiteAuth, SiteCapabilities, SiteData};
use crate::server::auth::AuthState;
use crate::server::auth::client_addr::ClientAddrResolver;
use crate::server::auth::password::{
    generate_random_password, generate_token_key, get_stored_auth, hash_password,
    set_password_hash_and_revoke_sessions, set_stored_auth,
};
use crate::server::auth::rate_limit::login_rate_limiter;
use crate::server::auth::session::{cleanup_expired, paseto_key_from_bytes};
use crate::server::{WebServer, WebServerOptions, WriteCoordinator};
use crate::shelf::time_config::TimeConfig;
use crate::source::scanner::MetadataLocation;
use crate::store::lifecycle::{
    KOSHELF_DB_FILENAME, RuntimeDataPathOptions, RuntimeDataPolicy, resolve_runtime_data_policy,
};
use crate::store::memory::{ReadingData, ReadingDataStore, SiteStore, UpdateNotifier};
use crate::store::sqlite::repo::LibraryRepository;
use crate::store::sqlite::{
    open_koshelf_pool, open_library_pool, run_koshelf_migrations, run_library_migrations,
};
use anyhow::{Context, Result};
use clap::CommandFactory;
use log::{info, warn};
use std::collections::{BTreeMap, HashSet};
use std::io::Read as _;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Format a validation error as a clap-style error with usage line and `--help` hint,
/// then exit. This keeps our post-parse validation consistent with clap's own output.
fn exit_validation_error(subcommand: &str, error: anyhow::Error) -> ! {
    Cli::command()
        .find_subcommand(subcommand)
        .expect("known subcommand")
        .clone()
        .error(
            clap::error::ErrorKind::MissingRequiredArgument,
            error.to_string(),
        )
        .exit()
}

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
struct PipelineState {
    config: SiteConfig,
    repo: LibraryRepository,
    reading_data: Option<ReadingData>,
    has_reading_data: bool,
    site_data: SiteData,
    generated_at: String,
    _runtime_temp_dir: Option<tempfile::TempDir>,
}

/// Run the shared pipeline: DB setup, library update, statistics, recap images, site metadata.
async fn initialize_pipeline(
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

async fn run_set_password_command(
    data_path: PathBuf,
    password_arg: Option<String>,
    random: bool,
    overwrite: bool,
) -> Result<()> {
    std::fs::create_dir_all(&data_path).with_context(|| {
        format!(
            "Failed to create data directory for set-password command at {}",
            data_path.display()
        )
    })?;

    let koshelf_db_path = data_path.join(KOSHELF_DB_FILENAME);

    let koshelf_pool = open_koshelf_pool(&koshelf_db_path)
        .await
        .context("Failed to open KoShelf app DB")?;
    run_koshelf_migrations(&koshelf_pool)
        .await
        .context("Failed to run KoShelf app DB migrations")?;

    let stored_auth = get_stored_auth(&koshelf_pool).await?;
    if stored_auth.is_some() && !overwrite {
        info!(
            "Authentication password is already initialized. Re-run with --overwrite to replace it."
        );
        return Ok(());
    }

    let (new_password, is_random_password) = if random {
        (generate_random_password()?, true)
    } else {
        let password = match password_arg {
            Some(value) => value,
            None => {
                let first = rpassword::prompt_password("New password: ")
                    .context("Failed to read password from terminal")?;
                let second = rpassword::prompt_password("Confirm new password: ")
                    .context("Failed to read password confirmation from terminal")?;
                if first != second {
                    anyhow::bail!("Passwords do not match")
                }
                first
            }
        };
        (password, false)
    };

    let new_hash = hash_password(&new_password)?;

    match stored_auth {
        Some((_stored_hash, _stored_token_key)) => {
            set_password_hash_and_revoke_sessions(&koshelf_pool, &new_hash, None).await?;
        }
        None => {
            let token_key = generate_token_key()?;
            set_stored_auth(&koshelf_pool, &new_hash, &token_key).await?;
        }
    }

    if is_random_password {
        eprintln!();
        eprintln!(
            "--------------------------------------------------------------------------------"
        );
        eprintln!("SET-PASSWORD");
        eprintln!(
            "--------------------------------------------------------------------------------"
        );
        eprintln!("Generated authentication password: {}", new_password);
        eprintln!("This password will not be shown again. Save it now.");
        eprintln!(
            "--------------------------------------------------------------------------------"
        );
        eprintln!();
    }

    info!(
        "Authentication password updated successfully for data path {}",
        data_path.display()
    );

    Ok(())
}

async fn run_serve(args: ServeArgs) -> Result<()> {
    if let Err(e) = args.validate() {
        exit_validation_error("serve", e);
    }

    let output_dir = args
        .common
        .data_path
        .clone()
        .context("Serve mode requires --data-path")?;

    let state = initialize_pipeline(
        &args.common,
        output_dir.clone(),
        true,
        args.enable_auth,
        args.enable_writeback,
        false,
    )
    .await?;

    // ── Auth setup ───────────────────────────────────────────────────
    let koshelf_db_path = state
        .config
        .runtime_data_policy
        .koshelf_db_path()
        .context("Failed to resolve KoShelf DB path")?;

    let koshelf_pool = open_koshelf_pool(&koshelf_db_path)
        .await
        .context("Failed to open KoShelf app DB")?;

    run_koshelf_migrations(&koshelf_pool)
        .await
        .context("Failed to run KoShelf app DB migrations")?;

    let trusted_proxies = parse_trusted_proxy_nets(&args.trusted_proxies)?;
    let client_addr_resolver = Arc::new(ClientAddrResolver::new(trusted_proxies));

    let auth_state = if args.enable_auth {
        let token_key_bytes = if let Some((_stored_hash, stored_token_key)) =
            get_stored_auth(&koshelf_pool).await?
        {
            stored_token_key
        } else {
            let generated_password = generate_random_password()?;
            let password_hash = hash_password(&generated_password)?;
            let token_key = generate_token_key()?;
            set_stored_auth(&koshelf_pool, &password_hash, &token_key).await?;

            let data_path_hint = state
                .config
                .runtime_data_policy
                .persistent_data_dir()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<data-path>".to_string());

            eprintln!();
            eprintln!(
                "--------------------------------------------------------------------------------"
            );
            eprintln!("AUTHENTICATION ENABLED (FIRST RUN)");
            eprintln!(
                "--------------------------------------------------------------------------------"
            );
            eprintln!("Generated authentication password: {}", generated_password);
            eprintln!("This password will not be shown again. Save it now.");
            eprintln!(
                "To set a new one later, run: koshelf set-password --data-path {} --overwrite",
                data_path_hint
            );
            eprintln!(
                "--------------------------------------------------------------------------------"
            );
            eprintln!();

            token_key
        };

        cleanup_expired(&koshelf_pool).await?;

        let cleanup_pool = koshelf_pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(24 * 60 * 60));
            interval.tick().await;
            loop {
                interval.tick().await;
                if let Err(error) = cleanup_expired(&cleanup_pool).await {
                    warn!("Failed to clean up expired sessions: {}", error);
                }
            }
        });

        let paseto_key = paseto_key_from_bytes(&token_key_bytes)?;

        Some(AuthState {
            pool: koshelf_pool.clone(),
            token_key: Arc::new(paseto_key),
            login_limiter: Arc::new(login_rate_limiter()),
            client_addr_resolver: client_addr_resolver.clone(),
        })
    } else {
        None
    };

    // ── Start server ─────────────────────────────────────────────────
    let revision_epoch = format!("serve_{}", &state.generated_at);
    let initial_generated_at = state.generated_at;

    let site_store = Arc::new(SiteStore::new());
    site_store.replace(state.site_data);

    let reading_data_store = Arc::new(ReadingDataStore::new());
    if let Some(rd) = state.reading_data {
        reading_data_store.replace(rd);
    }

    let update_notifier = UpdateNotifier::new(revision_epoch, initial_generated_at);

    let write_coordinator = if args.enable_writeback {
        Some(WriteCoordinator::new())
    } else {
        None
    };

    let timezone = state.config.time_config.timezone;

    let file_watcher = FileWatcher::new(
        state.config,
        Some(site_store.clone()),
        Some(reading_data_store.clone()),
        Some(update_notifier.clone()),
        Some(state.repo.clone()),
        write_coordinator.as_ref().map(|wc| wc.recent_writes()),
    );

    let web_server = WebServer::new(WebServerOptions {
        media_cache_dir: output_dir,
        port: args.port,
        site_store,
        reading_data_store,
        update_notifier,
        library_repo: state.repo,
        auth_state,
        write_coordinator,
        timezone,
    });

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

async fn run_export(args: ExportArgs) -> Result<()> {
    if let Err(e) = args.validate() {
        exit_validation_error("export", e);
    }

    let output_dir = args
        .output
        .clone()
        .context("Output directory is required for export")?;

    let state = initialize_pipeline(
        &args.common,
        output_dir.clone(),
        false,
        false,
        false,
        args.include_files,
    )
    .await?;

    // ── Sync static frontend ─────────────────────────────────────────
    frontend::sync_static_frontend(&state.config.output_dir, state.has_reading_data)?;

    // ── Export data files ────────────────────────────────────────────
    let export_config = ExportConfig {
        site_title: state.config.site_title.clone(),
        language: state.config.language.clone(),
        include_files: state.config.include_files,
    };
    export_data_files(
        &output_dir.join("data"),
        &output_dir,
        &state.repo,
        state.reading_data.as_ref(),
        &export_config,
    )
    .await?;

    if args.watch {
        info!("Watching library changes to refresh static shell/assets and /data export.");
        let file_watcher = FileWatcher::new(state.config, None, None, None, Some(state.repo), None);
        if let Err(e) = file_watcher.run().await {
            log::error!("File watcher error: {}", e);
        }
    } else {
        info!("Static export completed.");
    }

    Ok(())
}

#[derive(serde::Deserialize)]
struct LicenseGroup {
    license: String,
    text: String,
    dependencies: Vec<LicenseDep>,
}

#[derive(serde::Deserialize)]
struct LicenseDep {
    name: String,
    version: String,
}

fn print_licenses(dependency: Option<String>) -> Result<()> {
    let compressed = include_bytes!(concat!(env!("OUT_DIR"), "/LICENSES.json.gz"));
    let mut decoder = flate2::read::GzDecoder::new(&compressed[..]);
    let mut json = String::new();
    decoder
        .read_to_string(&mut json)
        .expect("Failed to decompress license data");
    let data: Vec<LicenseGroup> =
        serde_json::from_str(&json).expect("Failed to parse embedded license data");

    match dependency {
        None => {
            let mut by_spdx: BTreeMap<&str, Vec<&LicenseDep>> = BTreeMap::new();
            for group in &data {
                by_spdx
                    .entry(&group.license)
                    .or_default()
                    .extend(&group.dependencies);
            }

            for (spdx, deps) in &by_spdx {
                let count = deps.len();
                println!(
                    "{} ({} {})",
                    spdx,
                    count,
                    if count == 1 {
                        "dependency"
                    } else {
                        "dependencies"
                    }
                );
                for dep in deps {
                    if dep.version.is_empty() {
                        println!("  {}", dep.name);
                    } else {
                        println!("  {} {}", dep.name, dep.version);
                    }
                }
                println!();
            }
            println!("Use 'koshelf licenses <dependency>' to view the full license text.");
        }
        Some(query) => {
            let query_lower = query.to_lowercase();

            // Try exact match first, then substring
            let mut matches: Vec<(&LicenseDep, &LicenseGroup)> = Vec::new();
            for group in &data {
                for dep in &group.dependencies {
                    if dep.name.to_lowercase() == query_lower {
                        matches.push((dep, group));
                    }
                }
            }
            if matches.is_empty() {
                for group in &data {
                    for dep in &group.dependencies {
                        if dep.name.to_lowercase().contains(&query_lower) {
                            matches.push((dep, group));
                        }
                    }
                }
            }

            match matches.len() {
                0 => {
                    anyhow::bail!("No dependency found matching '{query}'");
                }
                1 => {
                    let (dep, group) = matches[0];
                    if dep.version.is_empty() {
                        println!("{} — {}\n", dep.name, group.license);
                    } else {
                        println!("{} {} — {}\n", dep.name, dep.version, group.license);
                    }
                    println!("{}", group.text);
                }
                _ => {
                    // Multiple matches — check if they all share the same license group
                    let first_license = &matches[0].1.license;
                    let first_text = &matches[0].1.text;
                    let all_same = matches
                        .iter()
                        .all(|(_, g)| g.license == *first_license && g.text == *first_text);

                    if all_same {
                        for (dep, _) in &matches {
                            if dep.version.is_empty() {
                                println!("{} — {}", dep.name, first_license);
                            } else {
                                println!("{} {} — {}", dep.name, dep.version, first_license);
                            }
                        }
                        println!("\n{}", first_text);
                    } else {
                        println!("Multiple dependencies match '{query}':\n");
                        for (dep, group) in &matches {
                            if dep.version.is_empty() {
                                println!("  {} ({})", dep.name, group.license);
                            } else {
                                println!("  {} {} ({})", dep.name, dep.version, group.license);
                            }
                        }
                        println!("\nPlease specify the exact dependency name.");
                    }
                }
            }
        }
    }
    Ok(())
}

/// Dispatch the parsed CLI command to the appropriate handler.
///
/// `src/main.rs` is responsible for logging init, Clap argument parsing, and config file merging.
pub async fn dispatch(command: CliCommand) -> Result<()> {
    match command {
        CliCommand::Serve(args) => run_serve(args).await,
        CliCommand::Export(args) => run_export(args).await,
        CliCommand::SetPassword {
            data_path,
            password,
            random,
            overwrite,
        } => {
            let Some(resolved_data_path) = data_path else {
                exit_validation_error(
                    "set-password",
                    anyhow::anyhow!(
                        "set-password requires a data path. Provide --data-path, \
                         set KOSHELF_DATA_PATH, or configure koshelf.data_path in your config file"
                    ),
                );
            };
            run_set_password_command(resolved_data_path, password, random, overwrite).await
        }
        CliCommand::ListLanguages => {
            println!("{}", crate::i18n::list_supported_languages());
            Ok(())
        }
        CliCommand::Licenses { dependency } => print_licenses(dependency),
        CliCommand::Github => {
            println!("https://github.com/paviro/KOShelf");
            Ok(())
        }
    }
}
