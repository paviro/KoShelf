use anyhow::{Context, Result};
use clap::Parser;
use log::info;
use regex::Regex;
use std::path::PathBuf;

mod calendar;
mod comic_parser;
mod config;
mod epub_parser;
mod fb2_parser;
mod file_watcher;
mod i18n;
mod library_scanner;
mod lua_parser;
mod mobi_parser;
mod models;
mod partial_md5;
mod read_completion_analyzer;
mod session_calculator;
mod share_image;
mod site_generator;
mod statistics;
mod statistics_parser;
mod templates;
mod time_config;
mod utils;
mod version_notifier;
mod web_server;

#[cfg(test)]
mod tests;

use crate::config::SiteConfig;
use crate::file_watcher::FileWatcher;
use crate::library_scanner::MetadataLocation;
use crate::site_generator::SiteGenerator;
use crate::time_config::TimeConfig;
use crate::version_notifier::create_version_notifier;
use crate::web_server::WebServer;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path(s) to folders containing ebooks (EPUB, FB2, MOBI) and/or comics (CBZ, CBR) with KoReader metadata.
    /// Can be specified multiple times. (optional if statistics_db is provided)
    #[arg(short = 'i', visible_short_alias = 'b', long, alias = "books-path", display_order = 1, action = clap::ArgAction::Append)]
    library_path: Vec<PathBuf>,

    /// Path to KOReader's docsettings folder (for users who store metadata separately). Requires --books-path. Mutually exclusive with --hashdocsettings-path.
    #[arg(long, display_order = 2)]
    docsettings_path: Option<PathBuf>,

    /// Path to KOReader's hashdocsettings folder (for users who store metadata by hash). Requires --books-path. Mutually exclusive with --docsettings-path.
    #[arg(long, display_order = 3)]
    hashdocsettings_path: Option<PathBuf>,

    /// Path to the statistics.sqlite3 file for additional reading stats (optional if books_path is provided)
    #[arg(short, long, display_order = 4)]
    statistics_db: Option<PathBuf>,

    /// Output directory for the generated static site (if not provided, starts web server with file watching)
    #[arg(short, long, display_order = 5)]
    output: Option<PathBuf>,

    /// Port for web server mode (default: 3000)
    #[arg(short, long, default_value = "3000", display_order = 6)]
    port: u16,

    /// Enable file watching with static output (requires --output)
    #[arg(short, long, default_value = "false", display_order = 7)]
    watch: bool,

    /// Site title
    #[arg(short, long, default_value = "KoShelf", display_order = 8)]
    title: String,

    /// Include unread books (EPUBs without KoReader metadata) in the generated site
    #[arg(long, default_value = "false", display_order = 9)]
    include_unread: bool,

    /// Maximum value for heatmap color intensity scaling (e.g., "auto", "1h", "1h30m", "45min"). Values above this will still be shown but use the highest color intensity.
    #[arg(long, default_value = "auto", display_order = 10)]
    heatmap_scale_max: String,

    /// Timezone to interpret timestamps (IANA name, e.g., "Australia/Sydney"). Defaults to system local timezone.
    #[arg(long, display_order = 11)]
    timezone: Option<String>,

    /// Logical day start time (HH:MM). Defaults to 00:00.
    #[arg(long, value_name = "HH:MM", display_order = 12)]
    day_start_time: Option<String>,

    /// Minimum pages read per day to be counted in statistics (optional)
    #[arg(long, display_order = 13)]
    min_pages_per_day: Option<u32>,

    /// Minimum reading time per day to be counted in statistics (e.g., "15m", "1h"). (optional)
    #[arg(long, display_order = 14)]
    min_time_per_day: Option<String>,

    /// Include statistics for all books in the database, not just those in --books-path.
    /// By default, when --books-path is provided, statistics are filtered to only include
    /// books present in that directory. Use this flag to include all statistics.
    #[arg(long, default_value = "false", display_order = 15)]
    include_all_stats: bool,

    /// Language for UI translations. Use full locale (e.g., en_US, de_DE) for correct date formatting. Use --list-languages to see available options
    #[arg(long, short = 'l', default_value = "en_US", display_order = 16)]
    language: String,

    /// List all supported languages and exit
    #[arg(long, display_order = 17)]
    list_languages: bool,

    /// Print GitHub repository URL
    #[arg(long, display_order = 18)]
    github: bool,
}

// Parse time format strings like "1h", "1h30m", "45min", "30s" into seconds
fn parse_time_to_seconds(time_str: &str) -> Result<Option<u32>> {
    if time_str.eq_ignore_ascii_case("auto") {
        return Ok(None);
    }

    let re = Regex::new(r"(?i)(\d+)(h|m|min|s)")?;
    let mut total_seconds: u32 = 0;
    let mut matched_any = false;

    for cap in re.captures_iter(time_str) {
        matched_any = true;
        let value: u32 = cap[1].parse()?;
        let unit = &cap[2].to_lowercase();

        match unit.as_str() {
            "h" => total_seconds += value * 3600,
            "m" | "min" => total_seconds += value * 60,
            "s" => total_seconds += value,
            _ => anyhow::bail!("Unknown time unit: {}", unit),
        }
    }

    if !matched_any {
        anyhow::bail!("Invalid time format: {}", time_str);
    }
    if total_seconds == 0 {
        anyhow::bail!("Time cannot be zero: {}", time_str);
    }

    Ok(Some(total_seconds))
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();
    let cli = Cli::parse();

    // Handle --github flag
    if cli.github {
        println!("https://github.com/paviro/KOShelf");
        return Ok(());
    }

    // Handle --list-languages flag
    if cli.list_languages {
        println!("{}", i18n::list_supported_languages());
        return Ok(());
    }

    info!("Starting KOShelf...");

    // Require at least one of library_path or statistics_db
    if cli.library_path.is_empty() && cli.statistics_db.is_none() {
        anyhow::bail!("Either --library-path or --statistics-db (or both) must be provided");
    }

    // Validate library paths if provided
    for library_path in &cli.library_path {
        if !library_path.exists() {
            anyhow::bail!("Library path does not exist: {:?}", library_path);
        }
        if !library_path.is_dir() {
            anyhow::bail!("Library path is not a directory: {:?}", library_path);
        }
    }

    // Validate include-unread option
    if cli.include_unread && cli.library_path.is_empty() {
        anyhow::bail!("--include-unread can only be used when --library-path is provided");
    }

    // Validate docsettings-path and hashdocsettings-path options
    if cli.docsettings_path.is_some() && cli.hashdocsettings_path.is_some() {
        anyhow::bail!(
            "--docsettings-path and --hashdocsettings-path are mutually exclusive. Please use only one."
        );
    }

    if cli.docsettings_path.is_some() && cli.library_path.is_empty() {
        anyhow::bail!("--docsettings-path requires --library-path to be provided");
    }

    if cli.hashdocsettings_path.is_some() && cli.library_path.is_empty() {
        anyhow::bail!("--hashdocsettings-path requires --library-path to be provided");
    }

    // Validate docsettings path if provided
    if let Some(ref docsettings_path) = cli.docsettings_path {
        if !docsettings_path.exists() {
            anyhow::bail!("Docsettings path does not exist: {:?}", docsettings_path);
        }
        if !docsettings_path.is_dir() {
            anyhow::bail!(
                "Docsettings path is not a directory: {:?}",
                docsettings_path
            );
        }
    }

    // Validate hashdocsettings path if provided
    if let Some(ref hashdocsettings_path) = cli.hashdocsettings_path {
        if !hashdocsettings_path.exists() {
            anyhow::bail!(
                "Hashdocsettings path does not exist: {:?}",
                hashdocsettings_path
            );
        }
        if !hashdocsettings_path.is_dir() {
            anyhow::bail!(
                "Hashdocsettings path is not a directory: {:?}",
                hashdocsettings_path
            );
        }
    }

    // Validate watch option
    if cli.watch && cli.output.is_none() {
        info!(
            "--watch specified without --output. Note that file watching is enabled by default when no output directory is specified (web server mode)"
        );
    }

    // Validate port option
    if cli.output.is_some() && cli.port != 3000 {
        anyhow::bail!("--port can only be used in web server mode (without --output)");
    }

    // Validate statistics database if provided
    if let Some(ref stats_path) = cli.statistics_db
        && !stats_path.exists()
    {
        anyhow::bail!("Statistics database does not exist: {:?}", stats_path);
    }

    // Parse heatmap scale max
    let heatmap_scale_max = parse_time_to_seconds(&cli.heatmap_scale_max).with_context(|| {
        format!(
            "Invalid heatmap-scale-max format: {}",
            cli.heatmap_scale_max
        )
    })?;

    // Build time configuration from CLI
    let time_config = TimeConfig::from_cli(&cli.timezone, &cli.day_start_time)?;

    // Determine output directory
    let (output_dir, is_static_site) = match &cli.output {
        Some(output_dir) => (output_dir.clone(), !cli.watch),
        None => {
            let temp_dir = tempfile::tempdir()?;
            (temp_dir.path().to_path_buf(), false)
        }
    };

    // Parse min time per day
    let min_time_per_day = if let Some(ref min_time_str) = cli.min_time_per_day {
        parse_time_to_seconds(min_time_str)
            .with_context(|| format!("Invalid min-time-per-day format: {}", min_time_str))?
    } else {
        None
    };

    // Determine metadata location
    let metadata_location = if let Some(ref docsettings_path) = cli.docsettings_path {
        MetadataLocation::DocSettings(docsettings_path.clone())
    } else if let Some(ref hashdocsettings_path) = cli.hashdocsettings_path {
        MetadataLocation::HashDocSettings(hashdocsettings_path.clone())
    } else {
        MetadataLocation::InBookFolder
    };

    // Determine if we're running with internal web server (not static export)
    let is_internal_server = !is_static_site && cli.output.is_none();

    // Create site config - bundles all generation options
    let config = SiteConfig {
        output_dir: output_dir.clone(),
        site_title: cli.title.clone(),
        include_unread: cli.include_unread,
        library_paths: cli.library_path.clone(),
        metadata_location: metadata_location.clone(),
        statistics_db_path: cli.statistics_db.clone(),
        heatmap_scale_max,
        time_config: time_config.clone(),
        min_pages_per_day: cli.min_pages_per_day,
        min_time_per_day,
        include_all_stats: cli.include_all_stats,
        is_internal_server,
        language: cli.language.clone(),
    };

    // Create site generator - it will handle book scanning and stats loading internally
    let site_generator = SiteGenerator::new(config.clone());

    // Generate initial site
    site_generator.generate().await?;

    if is_static_site {
        return Ok(());
    }

    // Web server mode or watch mode
    if cli.output.is_some() && cli.watch {
        info!("Starting file watcher mode for static output");

        // Start file watcher only (no long-polling needed for static output)
        let file_watcher = FileWatcher::new(config.clone(), None).await?;

        // Run file watcher
        if let Err(e) = file_watcher.run().await {
            log::error!("File watcher error: {}", e);
        }
    } else {
        // Create shared version notifier for long-polling
        let version_notifier = create_version_notifier();

        // Start file watcher with version notifier
        let file_watcher = FileWatcher::new(config, Some(version_notifier.clone())).await?;

        // Start web server with version notifier
        let web_server = WebServer::new(output_dir, cli.port, version_notifier);

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
    }

    Ok(())
}
