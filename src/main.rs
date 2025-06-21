use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;
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
mod statistics_parser;

use crate::site_generator::SiteGenerator;
use crate::web_server::WebServer;
use crate::file_watcher::FileWatcher;
use crate::book_scanner::scan_books;
use crate::statistics_parser::StatisticsParser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the folder containing epub files and KoReader metadata
    #[arg(short, long)]
    books_path: PathBuf,
    
    /// Output directory for the generated static site (if not provided, starts web server with file watching)
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Enable file watching with static output (requires --output)
    #[arg(long, default_value = "false")]
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
    
    /// Path to the statistics.sqlite3 file for additional reading stats
    #[arg(long)]
    statistics_db: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let cli = Cli::parse();
    
    info!("Starting KOShelf...");
    
    // Validate input path
    if !cli.books_path.exists() {
        anyhow::bail!("Books path does not exist: {:?}", cli.books_path);
    }
    if !cli.books_path.is_dir() {
        anyhow::bail!("Books path is not a directory: {:?}", cli.books_path);
    }

    // Validate watch option
    if cli.watch && cli.output.is_none() {
        anyhow::bail!("Watch mode requires an output directory (--output)");
    }

    // Parse statistics if provided
    let stats_data = if let Some(ref stats_path) = cli.statistics_db {
        if !stats_path.exists() {
            anyhow::bail!("Statistics database does not exist: {:?}", stats_path);
        }
        
        Some(StatisticsParser::parse(stats_path)?)
    } else {
        None
    };

    // Determine output directory
    let (output_dir, is_static_site) = match &cli.output {
        Some(output_dir) => (output_dir.clone(), !cli.watch),
        None => {
            let temp_dir = tempfile::tempdir()?;
            (temp_dir.path().to_path_buf(), false)
        }
    };

    // Scan books and generate site
    let books = scan_books(&cli.books_path).await?;
    
    // Create site generator with or without stats
    let mut site_generator = SiteGenerator::new(output_dir.clone(), cli.title.clone(), cli.include_unread);
    
    // Add stats if available
    if let Some(stats) = stats_data {
        site_generator = site_generator.with_stats(stats);
    }
    
    site_generator.generate(books.clone()).await?;

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
            books.clone(),
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
            books.clone(),
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