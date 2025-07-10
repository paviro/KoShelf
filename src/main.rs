use clap::Parser;
use std::path::PathBuf;
use anyhow::{Result, Context};
use log::info;

mod models;
mod epub_parser;
mod lua_parser;
mod site_generator;
mod templates;
mod web_server;
mod file_watcher;
mod book_scanner;
mod utils;
mod session_calculator;
mod statistics_parser;
mod statistics;
mod calendar;
mod read_completion_analyzer;

use crate::site_generator::SiteGenerator;
use crate::web_server::WebServer;
use crate::file_watcher::FileWatcher;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the folder containing epub files and KoReader metadata (optional if statistics_db is provided)
    #[arg(short, long)]
    books_path: Option<PathBuf>,

    /// Path to the statistics.sqlite3 file for additional reading stats (optional if books_path is provided)
    #[arg(short, long)]
    statistics_db: Option<PathBuf>,
    
    /// Output directory for the generated static site (if not provided, starts web server with file watching)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Maximum value for heatmap color intensity scaling (e.g., "auto", "1h", "1h30m", "45min"). Values above this will still be shown but use the highest color intensity.
    #[arg(long, default_value = "auto")]
    heatmap_scale_max: String,
    
    /// Enable file watching with static output (requires --output)
    #[arg(short, long, default_value = "false")]
    watch: bool,
    
    /// Site title
    #[arg(short, long, default_value = "KoShelf")]
    title: String,
    
    /// Include unread books (EPUBs without KoReader metadata) in the generated site
    #[arg(long, default_value = "false")]
    include_unread: bool,
    
    /// Port for web server mode (default: 3000)
    #[arg(short, long, default_value = "3000")]
    port: u16,
    
    /// Print GitHub repository URL
    #[arg(long)]
    github: bool,
}

// Parse time format strings like "1h", "1h30m", "45min" into seconds
fn parse_time_to_seconds(time_str: &str) -> Result<Option<u32>> {
    if time_str == "auto" {
        return Ok(None);
    }
    
    let time_str = time_str.to_lowercase();
    let mut total_seconds = 0u32;
    
    // Handle hours
    if let Some(h_pos) = time_str.find('h') {
        let hours_str = &time_str[..h_pos];
        if let Ok(hours) = hours_str.parse::<u32>() {
            total_seconds += hours * 3600;
        } else {
            anyhow::bail!("Invalid hour format in: {}", time_str);
        }
        
        // Check for minutes after hours
        let remaining = &time_str[h_pos + 1..];
        if !remaining.is_empty() {
            // Remove common minute suffixes and parse
            let remaining = remaining.replace("min", "").replace('m', "");
            if !remaining.is_empty() {
                if let Ok(minutes) = remaining.parse::<u32>() {
                    if minutes >= 60 {
                        anyhow::bail!("Minutes cannot be 60 or more: {}", time_str);
                    }
                    total_seconds += minutes * 60;
                } else {
                    anyhow::bail!("Invalid minute format in: {}", time_str);
                }
            }
        }
    } else {
        // Only minutes specified
        let minutes_str = time_str.replace("min", "").replace('m', "");
        if let Ok(minutes) = minutes_str.parse::<u32>() {
            total_seconds = minutes * 60;
        } else {
            anyhow::bail!("Invalid time format: {}", time_str);
        }
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
        .init();
    let cli = Cli::parse();
    
    // Handle --github flag
    if cli.github {
        println!("https://github.com/paviro/KOShelf");
        return Ok(());
    }
    
    info!("Starting KOShelf...");
    
    // Require at least one of books_path or statistics_db
    if cli.books_path.is_none() && cli.statistics_db.is_none() {
        anyhow::bail!("Either --books-path or --statistics-db (or both) must be provided");
    }
    
    // Validate books path if provided
    if let Some(ref books_path) = cli.books_path {
        if !books_path.exists() {
            anyhow::bail!("Books path does not exist: {:?}", books_path);
        }
        if !books_path.is_dir() {
            anyhow::bail!("Books path is not a directory: {:?}", books_path);
        }
    }

    // Validate include-unread option
    if cli.include_unread && cli.books_path.is_none() {
        anyhow::bail!("--include-unread can only be used when --books-path is provided");
    }

    // Validate watch option
    if cli.watch && cli.output.is_none() {
        info!("--watch specified without --output. Note that file watching is enabled by default when no output directory is specified (web server mode)");
    }

    // Validate port option
    if cli.output.is_some() && cli.port != 3000 {
        anyhow::bail!("--port can only be used in web server mode (without --output)");
    }

    // Validate statistics database if provided
    if let Some(ref stats_path) = cli.statistics_db {
        if !stats_path.exists() {
            anyhow::bail!("Statistics database does not exist: {:?}", stats_path);
        }
    }

    // Parse heatmap scale max
    let heatmap_scale_max = parse_time_to_seconds(&cli.heatmap_scale_max)
        .with_context(|| format!("Invalid heatmap-scale-max format: {}", cli.heatmap_scale_max))?;

    // Determine output directory
    let (output_dir, is_static_site) = match &cli.output {
        Some(output_dir) => (output_dir.clone(), !cli.watch),
        None => {
            let temp_dir = tempfile::tempdir()?;
            (temp_dir.path().to_path_buf(), false)
        }
    };

    // Create site generator - it will handle book scanning and stats loading internally
    let site_generator = SiteGenerator::new(
        output_dir.clone(), 
        cli.title.clone(), 
        cli.include_unread,
        cli.books_path.clone(),
        cli.statistics_db.clone(),
        heatmap_scale_max,
    );
    
    // Generate initial site
    site_generator.generate().await?;

    if is_static_site {
        return Ok(());
    }

    // Web server mode or watch mode
    if cli.output.is_some() && cli.watch {
        info!("Starting file watcher mode for static output");
        
        // Start file watcher only
        let file_watcher = FileWatcher::new(
            cli.books_path.clone(),
            output_dir,
            cli.title.clone(),
            cli.include_unread,
            cli.statistics_db.clone(),
            heatmap_scale_max,
        ).await?;
        
        // Run file watcher
        if let Err(e) = file_watcher.run().await {
            log::error!("File watcher error: {}", e);
        }
    } else {
        // Start file watcher
        let file_watcher = FileWatcher::new(
            cli.books_path.clone(),
            output_dir.clone(),
            cli.title.clone(),
            cli.include_unread,
            cli.statistics_db.clone(),
            heatmap_scale_max,
        ).await?;

        // Start web server
        let web_server = WebServer::new(output_dir, cli.port);

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