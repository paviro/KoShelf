use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;
use log::info;
use walkdir;

mod models;
mod epub_parser;
mod lua_parser;
mod site_generator;
mod templates;

use crate::models::*;
use crate::epub_parser::EpubParser;
use crate::lua_parser::LuaParser;
use crate::site_generator::SiteGenerator;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the folder containing epub files and KoReader metadata
    #[arg(short, long)]
    books_path: PathBuf,
    
    /// Output directory for the generated static site
    #[arg(short, long, default_value = "site")]
    output: PathBuf,
    
    /// Site title
    #[arg(short, long, default_value = "KOReader Library")]
    title: String,
    
    /// Include unread books (EPUBs without KoReader metadata) in the generated site
    #[arg(long, default_value = "false")]
    include_unread: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    
    info!("Starting KoShelf static site generation");
    info!("Books path: {:?}", cli.books_path);
    info!("Output path: {:?}", cli.output);
    
    // Validate input path
    if !cli.books_path.exists() {
        anyhow::bail!("Books path does not exist: {:?}", cli.books_path);
    }
    
    if !cli.books_path.is_dir() {
        anyhow::bail!("Books path is not a directory: {:?}", cli.books_path);
    }
    
    // Parse all books and metadata
    let epub_parser = EpubParser::new();
    let lua_parser = LuaParser::new();
    
    let mut books = Vec::new();
    
    // Walk through all epub files
    for entry in walkdir::WalkDir::new(&cli.books_path) {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("epub") {
            info!("Processing: {:?}", path);
            
            // Parse epub
            let epub_info = match epub_parser.parse(path).await {
                Ok(info) => info,
                Err(e) => {
                    log::warn!("Failed to parse epub {:?}: {}", path, e);
                    continue;
                }
            };
            
            // Look for corresponding .sdr directory and metadata
            let epub_stem = path.file_stem().unwrap().to_str().unwrap();
            let sdr_path = path.parent().unwrap().join(format!("{}.sdr", epub_stem));
            let metadata_path = sdr_path.join("metadata.epub.lua");
            
            let koreader_metadata = if metadata_path.exists() {
                match lua_parser.parse(&metadata_path).await {
                    Ok(metadata) => Some(metadata),
                    Err(e) => {
                        log::warn!("Failed to parse metadata {:?}: {}", metadata_path, e);
                        None
                    }
                }
            } else {
                None
            };
            
            let book = Book {
                id: generate_book_id(&epub_info.title),
                epub_info,
                koreader_metadata,
                epub_path: path.to_path_buf(),
            };
            
            books.push(book);
        }
    }
    
    info!("Found {} books", books.len());
    
    // Generate static site
    let site_generator = SiteGenerator::new(cli.output, cli.title, cli.include_unread);
    site_generator.generate(books).await?;
    
    info!("Static site generated successfully");
    Ok(())
}

fn generate_book_id(title: &str) -> String {
    title
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase()
} 