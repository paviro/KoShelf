use std::path::PathBuf;
use anyhow::Result;
use log::{debug, info};
use walkdir;

use crate::models::Book;
use crate::epub_parser::EpubParser;
use crate::lua_parser::LuaParser;
use crate::utils::generate_book_id;

pub async fn scan_books(books_path: &PathBuf) -> Result<Vec<Book>> {
    info!("Scanning books in directory: {:?}", books_path);
    let epub_parser = EpubParser::new();
    let lua_parser = LuaParser::new();
    
    let mut books = Vec::new();
    
    // Walk through all epub files
    for entry in walkdir::WalkDir::new(books_path) {
        let entry = entry?;
        let path = entry.path();
        
		if path
			.extension()
			.and_then(|s| s.to_str())
			.map(|ext| ext.eq_ignore_ascii_case("epub"))
			.unwrap_or(false)
		{
            debug!("Processing: {:?}", path);
            
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
    
    info!("Found {} books!", books.len());
    Ok(books)
} 