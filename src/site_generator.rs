use crate::models::*;
use crate::templates::*;
use anyhow::{Result, Context};
use askama::Template;
use std::fs;
use std::path::PathBuf;
use log::info;
use std::time::SystemTime;
use chrono::{Local};
use webp;
use futures::future;

pub struct SiteGenerator {
    output_dir: PathBuf,
    site_title: String,
    include_unread: bool,
}

impl SiteGenerator {
    pub fn new(output_dir: PathBuf, site_title: String, include_unread: bool) -> Self {
        Self {
            output_dir,
            site_title,
            include_unread,
        }
    }
    
    pub async fn generate(&self, books: Vec<Book>) -> Result<()> {
        info!("Generating static site in: {:?}", self.output_dir);
        
        // Create output directories
        self.create_directories().await?;
        
        // Copy static assets
        self.copy_static_assets().await?;
        
        // Generate book covers
        self.generate_covers(&books).await?;
        
        // Generate index page
        self.generate_index(&books).await?;
        
        // Generate individual book pages
        self.generate_book_pages(&books).await?;
        
        info!("Static site generation completed!");
        Ok(())
    }
    
    // Get current version from Cargo.toml
    fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
    
    // Get current datetime as formatted string
    fn get_last_updated(&self) -> String {
        Local::now().format("%Y-%m-%d %H:%M").to_string()
    }
    
    async fn create_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.output_dir)?;
        fs::create_dir_all(self.output_dir.join("books"))?;
        fs::create_dir_all(self.output_dir.join("assets/covers"))?;
        fs::create_dir_all(self.output_dir.join("assets/css"))?;
        fs::create_dir_all(self.output_dir.join("assets/js"))?;
        Ok(())
    }
    
    async fn copy_static_assets(&self) -> Result<()> {
        // Write the pre-compiled CSS that was embedded at build time
        let css_content = include_str!(concat!(env!("OUT_DIR"), "/compiled_style.css"));
        fs::write(self.output_dir.join("assets/css/style.css"), css_content)?;
        
        // Copy JavaScript files
        let js_content = include_str!("../assets/script.js");
        fs::write(self.output_dir.join("assets/js/script.js"), js_content)?;
        
        let lazy_loading_content = include_str!("../assets/lazy-loading.js");
        fs::write(self.output_dir.join("assets/js/lazy-loading.js"), lazy_loading_content)?;
        
        Ok(())
    }
    
    async fn generate_covers(&self, books: &[Book]) -> Result<()> {
        info!("Generating book covers...");
        
        // Collect all cover generation tasks
        let mut tasks = Vec::new();
        
        for book in books {
            if let Some(ref cover_data) = book.epub_info.cover_data {
                let cover_path = self.output_dir.join("assets/covers").join(format!("{}.webp", book.id));
                let epub_path = book.epub_path.clone();
                let cover_data = cover_data.clone();

                let should_generate = match (fs::metadata(&epub_path), fs::metadata(&cover_path)) {
                    (Ok(epub_meta), Ok(cover_meta)) => {
                        let epub_time = epub_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        let cover_time = cover_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        epub_time > cover_time
                    }
                    (Ok(_), Err(_)) => true, // Cover missing
                    _ => true, // If we can't get metadata, be safe and regenerate
                };

                if should_generate {
                    // Spawn a task for each cover generation
                    let task = tokio::task::spawn_blocking(move || -> Result<()> {
                        let img = image::load_from_memory(&cover_data)
                            .context("Failed to load cover image")?;
                        
                        // Resize to height of 600px while maintaining aspect ratio
                        let (original_width, original_height) = (img.width(), img.height());
                        let target_height = 600;
                        let target_width = (original_width * target_height) / original_height;
                        
                        let resized = img.resize(target_width, target_height, image::imageops::FilterType::Lanczos3);
                        
                        // Convert to RGB8 format for WebP encoding
                        let rgb_img = resized.to_rgb8();
                        
                        // Use webp crate for better quality control
                        let encoder = webp::Encoder::from_rgb(&rgb_img, rgb_img.width(), rgb_img.height());
                        let webp_data = encoder.encode(50.0);
                        
                        fs::write(&cover_path, &*webp_data)
                            .with_context(|| format!("Failed to save cover: {:?}", cover_path))?;
                        
                        Ok(())
                    });
                    
                    tasks.push(task);
                }
            }
        }
        
        // Wait for all cover generation tasks to complete
        let results = future::join_all(tasks).await;
        
        // Check for any errors
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok(())) => {}, // Success
                Ok(Err(e)) => return Err(e.context(format!("Failed to generate cover {}", i))),
                Err(e) => return Err(anyhow::Error::new(e).context(format!("Task {} panicked", i))),
            }
        }
        
        Ok(())
    }
    
    async fn generate_index(&self, books: &[Book]) -> Result<()> {
        info!("Generating index page...");
        
        let mut reading_books = Vec::new();
        let mut completed_books = Vec::new();
        let mut unread_books = Vec::new();
        
        for book in books {
            match book.status() {
                BookStatus::Reading => reading_books.push(book.clone()),
                BookStatus::Complete => completed_books.push(book.clone()),
                BookStatus::Unknown => {
                    // If it has KoReader metadata, add to reading; otherwise check if we should include unread
                    if book.koreader_metadata.is_some() {
                        reading_books.push(book.clone());
                    } else if self.include_unread {
                        unread_books.push(book.clone());
                    }
                    // If include_unread is false, we skip books without metadata
                }
            }
        }
        
        // Sort by title
        reading_books.sort_by(|a, b| a.epub_info.title.cmp(&b.epub_info.title));
        completed_books.sort_by(|a, b| a.epub_info.title.cmp(&b.epub_info.title));
        unread_books.sort_by(|a, b| a.epub_info.title.cmp(&b.epub_info.title));
        
        let template = IndexTemplate {
            site_title: self.site_title.clone(),
            reading_books,
            completed_books,
            unread_books,
            version: self.get_version(),
            last_updated: self.get_last_updated(),
        };
        
        let html = template.render()?;
        fs::write(self.output_dir.join("index.html"), html)?;
        
        Ok(())
    }
    
    async fn generate_book_pages(&self, books: &[Book]) -> Result<()> {
        info!("Generating book detail pages...");
        
        for book in books {
            let template = BookTemplate {
                site_title: self.site_title.clone(),
                book: book.clone()
            };
            
            let html = template.render()?;
            let book_dir = self.output_dir.join("books").join(&book.id);
            fs::create_dir_all(&book_dir)?;
            let book_path = book_dir.join("index.html");
            fs::write(book_path, html)?;
        }
        
        Ok(())
    }
} 