use crate::models::*;
use crate::templates::*;
use crate::statistics_parser::{StatisticsParser, StatisticsData};
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
    stats_data: Option<StatisticsData>,
}

impl SiteGenerator {
    pub fn new(output_dir: PathBuf, site_title: String, include_unread: bool) -> Self {
        Self {
            output_dir,
            site_title,
            include_unread,
            stats_data: None,
        }
    }
    
    pub fn with_stats(mut self, stats_data: StatisticsData) -> Self {
        self.stats_data = Some(stats_data);
        self
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
        
        // Generate statistics page if we have stats data
        if let Some(ref stats_data) = self.stats_data {
            self.generate_statistics_page(stats_data).await?;
        }
        
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
        fs::create_dir_all(self.output_dir.join("assets/json"))?;
        if self.stats_data.is_some() {
            fs::create_dir_all(self.output_dir.join("statistics"))?;
        }
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
        
        let stats_js_content = include_str!("../assets/statistics.js");
        fs::write(self.output_dir.join("assets/js/statistics.js"), stats_js_content)?;
        
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
            navbar_items: self.create_navbar_items("books"),
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
                book: book.clone(),
            };
            
            let html = template.render()?;
            let book_dir = self.output_dir.join("books").join(&book.id);
            fs::create_dir_all(&book_dir)?;
            let book_path = book_dir.join("index.html");
            fs::write(book_path, html)?;
        }
        
        Ok(())
    }
    
    async fn generate_statistics_page(&self, stats_data: &StatisticsData) -> Result<()> {
        info!("Generating statistics page...");
        
        // Calculate reading stats from the parsed data
        let reading_stats = StatisticsParser::calculate_stats(stats_data);
        
        // Create the template
        let template = StatsTemplate {
            site_title: self.site_title.clone(),
            reading_stats: reading_stats.clone(),
            version: self.get_version(),
            last_updated: self.get_last_updated(),
            navbar_items: self.create_navbar_items("statistics"),
        };
        
        // Render and write the template
        let html = template.render()?;
        
        // Create stats directory and write the index file
        let stats_dir = self.output_dir.join("statistics");
        fs::create_dir_all(&stats_dir)?;
        fs::write(stats_dir.join("index.html"), html)?;
        
        // Export overall stats as JSON for client-side use
        let json = serde_json::to_string_pretty(&reading_stats)?;
        fs::write(self.output_dir.join("assets/json/statistics.json"), json)?;
        
        // Export individual week data as separate JSON files
        for (index, week) in reading_stats.weeks.iter().enumerate() {
            let week_json = serde_json::to_string_pretty(&week)?;
            fs::write(self.output_dir.join("assets/json").join(format!("week_{}.json", index)), week_json)?;
        }
        
        Ok(())
    }

    // Create default navbar items
    fn create_navbar_items(&self, current_page: &str) -> Vec<NavItem> {
        let mut items = vec![
            NavItem {
                label: "Books".to_string(),
                href: "/".to_string(),
                icon_svg: "M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253".to_string(),
                is_active: current_page == "books",
            },
        ];
        
        // Add stats navigation item if we have stats data
        if self.stats_data.is_some() {
            items.push(NavItem {
                label: "Statistics".to_string(),
                href: "/statistics/".to_string(),
                icon_svg: "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z".to_string(),
                is_active: current_page == "statistics",
            });
        }
        
        items
    }
} 