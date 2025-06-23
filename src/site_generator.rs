use crate::models::*;
use crate::templates::*;
use crate::statistics_parser::StatisticsParser;
use crate::statistics::{BookStatistics, StatisticsCalculator};
use crate::book_scanner::scan_books;
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
    books_path: Option<PathBuf>,
    statistics_db_path: Option<PathBuf>,
}

impl SiteGenerator {
    pub fn new(
        output_dir: PathBuf, 
        site_title: String, 
        include_unread: bool, 
        books_path: Option<PathBuf>, 
        statistics_db_path: Option<PathBuf>
    ) -> Self {
        Self {
            output_dir,
            site_title,
            include_unread,
            books_path,
            statistics_db_path,
        }
    }
    
    pub async fn generate(&self) -> Result<()> {
        info!("Generating static site in: {:?}", self.output_dir);
        
        // Scan books if books_path is provided
        let books = if let Some(ref books_path) = self.books_path {
            scan_books(books_path).await?
        } else {
            Vec::new()
        };
        
        // Load statistics if path is provided
        let stats_data = if let Some(ref stats_path) = self.statistics_db_path {
            if stats_path.exists() {
                Some(StatisticsParser::parse(stats_path)?)
            } else {
                info!("Statistics database not found: {:?}", stats_path);
                None
            }
        } else {
            None
        };
        
        // Create output directories based on what we're generating
        self.create_directories(&books, &stats_data).await?;
        
        // Copy static assets
        self.copy_static_assets(&books, &stats_data).await?;
        
        // Generate book covers
        self.generate_covers(&books).await?;
        
        // Generate individual book pages
        self.generate_book_pages(&books, &stats_data).await?;
        
        if !books.is_empty() {
            // Generate book list page at index.html
            self.generate_book_list(&books).await?;
        }
        
        if let Some(ref stats_data) = stats_data {
            // Generate statistics page (render to root if no books)
            self.generate_statistics_page(stats_data, books.is_empty()).await?;
            
            // Generate calendar page if we have statistics data
            self.generate_calendar_page(stats_data, &books).await?;
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
    
    async fn create_directories(&self, books: &[Book], stats_data: &Option<StatisticsData>) -> Result<()> {
        fs::create_dir_all(&self.output_dir)?;
        
        // Only create books directory if we have books
        if !books.is_empty() {
            fs::create_dir_all(self.output_dir.join("books"))?;
            fs::create_dir_all(self.output_dir.join("assets/covers"))?;
        }
        
        // Always create assets directories for CSS/JS as they're always needed
        fs::create_dir_all(self.output_dir.join("assets/css"))?;
        fs::create_dir_all(self.output_dir.join("assets/js"))?;
        
        // Only create JSON directory if we have statistics data
        if stats_data.is_some() {
            fs::create_dir_all(self.output_dir.join("assets/json"))?;
        }
        
        // Create directories based on what we have
        if stats_data.is_some() {
            // Always create calendar directory when we have stats
            fs::create_dir_all(self.output_dir.join("calendar"))?;
            
            // Only create statistics directory if we have books (if no books, stats render to root)
            if !books.is_empty() {
                fs::create_dir_all(self.output_dir.join("statistics"))?;
            }
        }
        
        Ok(())
    }
    
    async fn copy_static_assets(&self, books: &[Book], stats_data: &Option<StatisticsData>) -> Result<()> {
        // Write the pre-compiled CSS (always needed for basic styling)
        let css_content = include_str!(concat!(env!("OUT_DIR"), "/compiled_style.css"));
        fs::write(self.output_dir.join("assets/css/style.css"), css_content)?;
        
        // Copy book-related JavaScript files only if we have books
        if !books.is_empty() {
            let js_content = include_str!("../assets/book_list.js");
            fs::write(self.output_dir.join("assets/js/book_list.js"), js_content)?;
            
            let lazy_loading_content = include_str!("../assets/lazy-loading.js");
            fs::write(self.output_dir.join("assets/js/lazy-loading.js"), lazy_loading_content)?;
        }
        
        // Copy statistics-related JavaScript files only if we have stats data
        if stats_data.is_some() {
            let stats_js_content = include_str!("../assets/statistics.js");
            fs::write(self.output_dir.join("assets/js/statistics.js"), stats_js_content)?;
            
            let heatmap_js_content = include_str!("../assets/heatmap.js");
            fs::write(self.output_dir.join("assets/js/heatmap.js"), heatmap_js_content)?;

            let calendar_css = include_str!(concat!(env!("OUT_DIR"), "/event-calendar.min.css"));
            fs::write(self.output_dir.join("assets/css/event-calendar.min.css"), calendar_css)?;

            let calendar_js = include_str!(concat!(env!("OUT_DIR"), "/event-calendar.min.js"));
            fs::write(self.output_dir.join("assets/js/event-calendar.min.js"), calendar_js)?;
            
            let calendar_map = include_str!(concat!(env!("OUT_DIR"), "/event-calendar.min.js.map"));
            fs::write(self.output_dir.join("assets/js/event-calendar.min.js.map"), calendar_map)?;
            
            let calendar_init_js_content = include_str!("../assets/calendar.js");
            fs::write(self.output_dir.join("assets/js/calendar.js"), calendar_init_js_content)?;
        }
        
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
    
    async fn generate_book_list(&self, books: &[Book]) -> Result<()> {
        info!("Generating book list page...");
        
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
    
    async fn generate_book_pages(&self, books: &[Book], stats_data: &Option<StatisticsData>) -> Result<()> {
        info!("Generating book detail pages...");
        
        for book in books {
            // Try to find matching statistics by MD5
            let book_stats = if let Some(stats) = stats_data {
                // Try to match using the partial_md5_checksum from KoReader metadata
                book.koreader_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.partial_md5_checksum.as_ref())
                    .and_then(|md5| stats.stats_by_md5.get(md5))
                    .cloned()
            } else {
                None
            };
            
            // Calculate session statistics if we have book stats
            let session_stats = if let (Some(stats), Some(book_stat)) = (stats_data, &book_stats) {
                Some(book_stat.calculate_session_stats(&stats.page_stats))
            } else {
                None
            };
            
            let template = BookTemplate {
                site_title: self.site_title.clone(),
                book: book.clone(),
                book_stats,
                session_stats,
                version: self.get_version(),
                last_updated: self.get_last_updated(),
                navbar_items: self.create_navbar_items("books"),
            };
            
            let html = template.render()?;
            let book_dir = self.output_dir.join("books").join(&book.id);
            fs::create_dir_all(&book_dir)?;
            let book_path = book_dir.join("index.html");
            fs::write(book_path, html)?;
        }
        
        Ok(())
    }
    
    async fn generate_statistics_page(&self, stats_data: &StatisticsData, render_to_root: bool) -> Result<()> {
        if render_to_root {
            info!("Generating statistics page at root index...");
        } else {
            info!("Generating statistics page...");
        }
        
        // Calculate reading stats from the parsed data
        let reading_stats = StatisticsParser::calculate_stats(stats_data);
        
        // Export daily activity data grouped by year as separate JSON files and get available years
        let available_years = self.export_daily_activity_by_year(&reading_stats.daily_activity).await?;

        // Export individual week data as separate JSON files
        for (index, week) in reading_stats.weeks.iter().enumerate() {
            let week_json = serde_json::to_string_pretty(&week)?;
            fs::write(self.output_dir.join("assets/json").join(format!("week_{}.json", index)), week_json)?;
        }
        
        // Create the template with appropriate navbar
        let template = StatsTemplate {
            site_title: self.site_title.clone(),
            reading_stats: reading_stats.clone(),
            available_years,
            version: self.get_version(),
            last_updated: self.get_last_updated(),
            navbar_items: self.create_navbar_items("statistics"),
        };
        
        // Render and write the template
        let html = template.render()?;
        
        if render_to_root {
            // Write directly to index.html
            fs::write(self.output_dir.join("index.html"), html)?;
        } else {
            // Create stats directory and write the index file
            let stats_dir = self.output_dir.join("statistics");
            fs::create_dir_all(&stats_dir)?;
            fs::write(stats_dir.join("index.html"), html)?;
        }
        
        Ok(())
    }

    // Export daily activity data grouped by year as separate JSON files and return available years
    async fn export_daily_activity_by_year(&self, daily_activity: &[crate::models::DailyStats]) -> Result<Vec<i32>> {
        use std::collections::HashMap;
        
        // Group daily stats by year
        let mut activity_by_year: HashMap<i32, Vec<&crate::models::DailyStats>> = HashMap::new();
        
        for day_stat in daily_activity {
            // Extract year from date (format: yyyy-mm-dd)
            if let Some(year_str) = day_stat.date.get(0..4) {
                if let Ok(year) = year_str.parse::<i32>() {
                    activity_by_year.entry(year).or_default().push(day_stat);
                }
            }
        }
        
        // Collect available years before consuming the map
        let mut available_years: Vec<i32> = activity_by_year.keys().cloned().collect();
        available_years.sort_by(|a, b| b.cmp(a)); // Sort descending (newest first)
        
        // Export each year's data to a separate file
        for (year, year_data) in activity_by_year {
            let filename = format!("daily_activity_{}.json", year);
            let file_path = self.output_dir.join("assets/json").join(filename);
            
            let json = serde_json::to_string_pretty(&year_data)?;
            fs::write(file_path, json)?;
        }
        
        Ok(available_years)
    }

    // Create default navbar items
    fn create_navbar_items(&self, current_page: &str) -> Vec<NavItem> {
        let mut items = Vec::new();
        
        // Add books navigation item only if we have a books path configured
        if self.books_path.is_some() {
            items.push(NavItem {
                label: "Books".to_string(),
                href: "/".to_string(),
                icon_svg: "M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253".to_string(),
                is_active: current_page == "books",
            });
        }
        
        // Add stats navigation item if we have a stats database path configured
        if self.statistics_db_path.is_some() {
            let stats_href = if self.books_path.is_some() {
                "/statistics/".to_string()  // Books exist, stats go to subfolder
            } else {
                "/".to_string()  // No books, stats are at root
            };
            
            items.push(NavItem {
                label: "Statistics".to_string(),
                href: stats_href,
                icon_svg: "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z".to_string(),
                is_active: current_page == "statistics",
            });
        }
        
        // Add calendar navigation item if we have statistics data
        if self.statistics_db_path.is_some() {
            items.push(NavItem {
                label: "Calendar".to_string(),
                href: "/calendar/".to_string(),  // Calendar always goes to /calendar/
                icon_svg: "M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z".to_string(),
                is_active: current_page == "calendar",
            });
        }
        
        items
    }
    
    async fn generate_calendar_page(&self, stats_data: &StatisticsData, books: &[Book]) -> Result<()> {
        info!("Generating calendar page...");
        
        // Generate calendar data from statistics data only
        let calendar_data = StatisticsCalculator::generate_calendar_events(stats_data, books);
        
        // Export calendar data as JSON
        let data_json = serde_json::to_string_pretty(&calendar_data)?;
        fs::write(self.output_dir.join("assets/json/calendar_data.json"), data_json)?;
        
        // Create the template
        let template = CalendarTemplate {
            site_title: self.site_title.clone(),
            version: self.get_version(),
            last_updated: self.get_last_updated(),
            navbar_items: self.create_navbar_items("calendar"),
        };
        
        // Render and write the template
        let html = template.render()?;
        
        // Calendar always goes to its own directory
        let calendar_dir = self.output_dir.join("calendar");
        fs::create_dir_all(&calendar_dir)?;
        fs::write(calendar_dir.join("index.html"), html)?;
        
        Ok(())
    }
} 