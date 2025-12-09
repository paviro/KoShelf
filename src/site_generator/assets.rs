//! Asset management: directory creation, static file copying, and cover generation.

use super::SiteGenerator;
use crate::models::{Book, StatisticsData};
use anyhow::{Context, Result};
use futures::future;
use log::info;
use std::fs;
use std::time::SystemTime;

impl SiteGenerator {
    pub(crate) async fn create_directories(&self, books: &[Book], stats_data: &Option<StatisticsData>) -> Result<()> {
        fs::create_dir_all(&self.output_dir)?;
        
        // Only create books directory if we have books
        if !books.is_empty() {
            fs::create_dir_all(self.books_dir())?;
            fs::create_dir_all(self.covers_dir())?;
        }
        
        // Always create assets directories for CSS/JS/icons as they're always needed
        fs::create_dir_all(self.css_dir())?;
        fs::create_dir_all(self.js_dir())?;
        fs::create_dir_all(self.icons_dir())?;
        
        // Create directories based on what we have
        if stats_data.is_some() {
            // Create JSON directories for statistics data
            fs::create_dir_all(self.json_dir())?;
            fs::create_dir_all(self.statistics_json_dir())?;
            fs::create_dir_all(self.calendar_json_dir())?;
            // Recap pages directory (static HTML)
            fs::create_dir_all(self.recap_dir())?;
            
            // Create calendar directory
            fs::create_dir_all(self.calendar_dir())?;
            
            // Only create statistics directory if we have books (if no books, stats render to root)
            if !books.is_empty() {
                fs::create_dir_all(self.statistics_dir())?;
            }
        }
        
        Ok(())
    }
    
    /// Helper to write a static asset and register it in the cache manifest.
    fn write_asset(&self, path: std::path::PathBuf, content: &[u8]) -> Result<()> {
        self.cache_manifest.register_file(&path, &self.output_dir, content);
        fs::write(path, content)?;
        Ok(())
    }
    
    pub(crate) async fn copy_static_assets(&self, books: &[Book], stats_data: &Option<StatisticsData>) -> Result<()> {
        // Write the pre-compiled CSS (always needed for basic styling)
        let css_content = include_str!(concat!(env!("OUT_DIR"), "/compiled_style.css"));
        self.write_asset(self.css_dir().join("style.css"), css_content.as_bytes())?;
        
        // Copy book-related JavaScript files only if we have books
        if !books.is_empty() {
            let js_content = include_str!("../../assets/book_list.js");
            self.write_asset(self.js_dir().join("book_list.js"), js_content.as_bytes())?;

            let js_content = include_str!("../../assets/book_detail.js");
            self.write_asset(self.js_dir().join("book_detail.js"), js_content.as_bytes())?;
            
            let lazy_loading_content = include_str!("../../assets/lazy-loading.js");
            self.write_asset(self.js_dir().join("lazy-loading.js"), lazy_loading_content.as_bytes())?;

            let section_toggle_content = include_str!("../../assets/section-toggle.js");
            self.write_asset(self.js_dir().join("section-toggle.js"), section_toggle_content.as_bytes())?;
        }
        
        // Copy statistics-related JavaScript files only if we have stats data
        if stats_data.is_some() {
            let stats_js_content = include_str!("../../assets/statistics.js");
            self.write_asset(self.js_dir().join("statistics.js"), stats_js_content.as_bytes())?;
            
            let heatmap_js_content = include_str!("../../assets/heatmap.js");
            self.write_asset(self.js_dir().join("heatmap.js"), heatmap_js_content.as_bytes())?;

            let calendar_css = include_str!(concat!(env!("OUT_DIR"), "/event-calendar.min.css"));
            self.write_asset(self.css_dir().join("event-calendar.min.css"), calendar_css.as_bytes())?;

            let calendar_js = include_str!(concat!(env!("OUT_DIR"), "/event-calendar.min.js"));
            self.write_asset(self.js_dir().join("event-calendar.min.js"), calendar_js.as_bytes())?;
            
            let calendar_map = include_str!(concat!(env!("OUT_DIR"), "/event-calendar.min.js.map"));
            self.write_asset(self.js_dir().join("event-calendar.min.js.map"), calendar_map.as_bytes())?;
            
            let calendar_init_js_content = include_str!("../../assets/calendar.js");
            self.write_asset(self.js_dir().join("calendar.js"), calendar_init_js_content.as_bytes())?;

            // Storage utility
            let storage_js_content = include_str!("../../assets/storage-manager.js");
            self.write_asset(self.js_dir().join("storage-manager.js"), storage_js_content.as_bytes())?;

            // Recap small UI logic
            let recap_js_content = include_str!("../../assets/recap.js");
            self.write_asset(self.js_dir().join("recap.js"), recap_js_content.as_bytes())?;
        }
        
        // PWA files (manifest and service worker are static, don't cache them)
        let manifest_content = include_str!("../../assets/manifest.json");
        fs::write(self.output_dir.join("manifest.json"), manifest_content)?;
        
        let sw_content = include_str!("../../assets/service-worker.js");
        fs::write(self.output_dir.join("service-worker.js"), sw_content)?;

        // PWA client-side script (handles update notifications)
        let pwa_js_content = include_str!("../../assets/pwa.js");
        self.write_asset(self.js_dir().join("pwa.js"), pwa_js_content.as_bytes())?;
        
        // Generate version.txt for lightweight polling (plain timestamp, ~25 bytes)
        let version = chrono::Local::now().to_rfc3339();
        fs::write(self.output_dir.join("version.txt"), &version)?;
        
        // Generate server-mode.txt based on CLI mode
        let server_mode = if self.is_internal_server { "internal" } else { "external" };
        fs::write(self.output_dir.join("server-mode.txt"), server_mode)?;
        
        // PWA icons
        let icon_192 = include_bytes!("../../assets/icons/icon-192.png");
        self.write_asset(self.icons_dir().join("icon-192.png"), icon_192)?;
        
        let icon_512 = include_bytes!("../../assets/icons/icon-512.png");
        self.write_asset(self.icons_dir().join("icon-512.png"), icon_512)?;
        
        Ok(())
    }
    
    pub(crate) async fn generate_covers(&self, books: &[Book]) -> Result<()> {
        info!("Generating book covers...");
        
        // Collect all cover generation tasks and their paths for manifest registration
        let mut tasks = Vec::new();
        let mut all_cover_paths = Vec::new();
        
        for book in books {
            if let Some(ref cover_data) = book.epub_info.cover_data {
                let cover_path = self.covers_dir().join(format!("{}.webp", book.id));
                let epub_path = book.epub_path.clone();
                let cover_data = cover_data.clone();
                
                // Track all covers for manifest registration
                all_cover_paths.push(cover_path.clone());

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
        
        // Register all covers in cache manifest
        for cover_path in &all_cover_paths {
            if let Ok(content) = fs::read(cover_path) {
                self.cache_manifest.register_file(cover_path, &self.output_dir, &content);
            }
        }
        
        Ok(())
    }
}
