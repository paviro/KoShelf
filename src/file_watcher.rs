use crate::models::*;
use crate::epub_parser::EpubParser;
use crate::lua_parser::LuaParser;
use crate::site_generator::SiteGenerator;
use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Instant};
use log::{info, warn, debug};
use crate::utils::generate_book_id;

pub struct FileWatcher {
    books_path: PathBuf,
    site_dir: PathBuf,
    site_title: String,
    include_unread: bool,
    books: Arc<Mutex<HashMap<PathBuf, Book>>>,
    epub_parser: EpubParser,
    lua_parser: LuaParser,
}

impl FileWatcher {
    pub async fn new(
        books_path: PathBuf,
        site_dir: PathBuf,
        site_title: String,
        include_unread: bool,
        initial_books: Vec<Book>,
    ) -> Result<Self> {
        let books = Arc::new(Mutex::new(
            initial_books.into_iter().map(|b| (b.epub_path.clone(), b)).collect()
        ));
        let epub_parser = EpubParser::new();
        let lua_parser = LuaParser::new();
        Ok(Self {
            books_path,
            site_dir,
            site_title,
            include_unread,
            books,
            epub_parser,
            lua_parser,
        })
    }
    
    pub async fn run(self) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Set up file watcher
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                match result {
                    Ok(event) => {
                        if let Err(e) = tx.send(event) {
                            warn!("Failed to send file event: {}", e);
                        }
                    }
                    Err(e) => warn!("File watcher error: {}", e),
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )?;
        
        watcher.watch(&self.books_path, RecursiveMode::Recursive)?;
        
        info!("File watcher started for directory: {:?}", self.books_path);
        
        // Debounce events to avoid rebuilding too frequently
        let mut last_rebuild = Instant::now();
        let rebuild_delay = Duration::from_millis(500);
        
        while let Some(event) = rx.recv().await {
            debug!("File event: {:?}", event);
            
            // Log all events for EPUB files to debug
            for path in &event.paths {
                if path.extension().and_then(|s| s.to_str()) == Some("epub") {
                    info!("EPUB event - Kind: {:?}, Path: {:?}", event.kind, path);
                }
            }
            
            // Always process Remove events, but debounce Create/Modify events
            let should_debounce = !matches!(event.kind, EventKind::Remove(_));
            
            if should_debounce && last_rebuild.elapsed() < rebuild_delay {
                info!("Debouncing event: {:?}", event.kind);
                continue;
            }
            
            if self.should_process_event(&event) {
                if let Err(e) = self.handle_file_event(event).await {
                    warn!("Error handling file event: {}", e);
                } else {
                    last_rebuild = Instant::now();
                    // Small delay to allow file operations to complete
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
        
        Ok(())
    }
    
    fn should_process_event(&self, event: &Event) -> bool {
        match &event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                event.paths.iter().any(|path| {
                    let extension = path.extension().and_then(|s| s.to_str());
                    extension == Some("epub") || 
                    path.file_name().and_then(|s| s.to_str()) == Some("metadata.epub.lua")
                })
            }
            _ => false,
        }
    }
    
    async fn handle_file_event(&self, event: Event) -> Result<()> {
        let mut books_changed = false;
        let mut books = self.books.lock().await;
        
        for path in &event.paths {
            match &event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    if path.extension().and_then(|s| s.to_str()) == Some("epub") {
                        books_changed |= self.handle_epub_change(&mut books, path).await;
                    } else if path.file_name().and_then(|s| s.to_str()) == Some("metadata.epub.lua") {
                        books_changed |= self.handle_metadata_change(&mut books, path).await;
                    }
                }
                EventKind::Remove(_) => {
                    if path.extension().and_then(|s| s.to_str()) == Some("epub") {
                        books_changed |= self.handle_epub_removal(&mut books, path);
                    } else if path.file_name().and_then(|s| s.to_str()) == Some("metadata.epub.lua") {
                        books_changed |= self.handle_metadata_deletion(&mut books, path).await;
                    }
                }
                _ => {}
            }
        }
        
        if books_changed {
            self.rebuild_site(&books).await?;
        }
        
        Ok(())
    }
    
    async fn handle_epub_change(&self, books: &mut HashMap<PathBuf, Book>, path: &Path) -> bool {
        info!("EPUB file changed: {:?}", path);
        match Self::parse_book(&self.epub_parser, &self.lua_parser, path).await {
            Ok(book) => {
                books.insert(path.to_path_buf(), book);
                true
            }
            Err(e) => {
                // If parsing fails and we had this book before, check if file was deleted
                if books.contains_key(path) && !path.exists() {
                    info!("EPUB file appears to have been deleted: {:?}", path);
                    self.handle_epub_removal(books, path)
                } else {
                    warn!("Failed to parse changed EPUB {:?}: {}", path, e);
                    false
                }
            }
        }
    }
    
    fn handle_epub_removal(&self, books: &mut HashMap<PathBuf, Book>, path: &Path) -> bool {
        info!("EPUB file deleted: {:?}", path);
        if let Some(removed_book) = books.remove(path) {
            info!("Removed book '{}' (ID: {}) from memory", removed_book.epub_info.title, removed_book.id);
            self.cleanup_book_files(&removed_book);
            true
        } else {
            debug!("Book not found in memory for deletion: {:?}", path);
            false
        }
    }
    
    async fn handle_metadata_change(&self, books: &mut HashMap<PathBuf, Book>, path: &Path) -> bool {
        info!("Metadata file changed: {:?}", path);
        if let Some(epub_path) = self.find_epub_for_metadata(path) {
            match Self::parse_book(&self.epub_parser, &self.lua_parser, &epub_path).await {
                Ok(book) => {
                    books.insert(epub_path, book);
                    true
                }
                Err(e) => {
                    warn!("Failed to reparse EPUB after metadata change {:?}: {}", epub_path, e);
                    false
                }
            }
        } else {
            false
        }
    }
    
    async fn handle_metadata_deletion(&self, books: &mut HashMap<PathBuf, Book>, path: &Path) -> bool {
        info!("Metadata file deleted: {:?}", path);
        if let Some(epub_path) = self.find_epub_for_metadata(path) {
            match Self::parse_book(&self.epub_parser, &self.lua_parser, &epub_path).await {
                Ok(book) => {
                    books.insert(epub_path, book);
                    true
                }
                Err(e) => {
                    warn!("Failed to reparse EPUB after metadata deletion {:?}: {}", epub_path, e);
                    false
                }
            }
        } else {
            false
        }
    }
    
    fn cleanup_book_files(&self, book: &Book) {
        let cover_path = self.site_dir.join("assets/covers").join(format!("{}.jpg", book.id));
        let book_dir = self.site_dir.join("books").join(&book.id);
        
        if let Err(e) = std::fs::remove_file(&cover_path) {
            if cover_path.exists() {
                warn!("Failed to delete cover image {:?}: {}", cover_path, e);
            }
        } else {
            info!("Deleted cover image: {:?}", cover_path);
        }
        
        if let Err(e) = std::fs::remove_dir_all(&book_dir) {
            if book_dir.exists() {
                warn!("Failed to delete book directory {:?}: {}", book_dir, e);
            }
        } else {
            info!("Deleted book directory: {:?}", book_dir);
        }
    }
    
    async fn rebuild_site(&self, books: &HashMap<PathBuf, Book>) -> Result<()> {
        info!("Rebuilding site due to file changes");
        let books_vec: Vec<Book> = books.values().cloned().collect();
        
        let site_generator = SiteGenerator::new(
            self.site_dir.clone(),
            self.site_title.clone(),
            self.include_unread,
        );
        site_generator.generate(books_vec).await?;
        info!("Site rebuild completed");
        Ok(())
    }
    
    fn find_epub_for_metadata(&self, metadata_path: &Path) -> Option<PathBuf> {
        // metadata.epub.lua is in a .sdr directory, find the corresponding epub
        let sdr_dir = metadata_path.parent()?;
        let sdr_name = sdr_dir.file_name()?.to_str()?;
        
        if !sdr_name.ends_with(".sdr") {
            return None;
        }
        
        let epub_name = &sdr_name[..sdr_name.len() - 4]; // Remove ".sdr"
        let epub_path = sdr_dir.parent()?.join(format!("{}.epub", epub_name));
        
        if epub_path.exists() {
            Some(epub_path)
        } else {
            None
        }
    }
    
    async fn parse_book(epub_parser: &EpubParser, lua_parser: &LuaParser, path: &Path) -> Result<Book> {
        // Parse epub
        let epub_info = epub_parser.parse(path).await?;
        
        // Look for corresponding .sdr directory and metadata
        let epub_stem = path.file_stem().unwrap().to_str().unwrap();
        let sdr_path = path.parent().unwrap().join(format!("{}.sdr", epub_stem));
        let metadata_path = sdr_path.join("metadata.epub.lua");
        
        let koreader_metadata = if metadata_path.exists() {
            match lua_parser.parse(&metadata_path).await {
                Ok(metadata) => Some(metadata),
                Err(e) => {
                    warn!("Failed to parse metadata {:?}: {}", metadata_path, e);
                    None
                }
            }
        } else {
            None
        };
        
        Ok(Book {
            id: generate_book_id(&epub_info.title),
            epub_info,
            koreader_metadata,
            epub_path: path.to_path_buf(),
        })
    }
} 