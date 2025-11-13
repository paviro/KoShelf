use crate::models::*;
use crate::templates::*;
use crate::statistics_parser::StatisticsParser;
use crate::statistics::{BookStatistics};
use crate::book_scanner::scan_books;
use anyhow::{Result, Context};
use askama::Template;
use std::fs;
use std::path::{Path, PathBuf};
use log::info;
use minify_html::{Cfg, minify};
use std::time::SystemTime;
use webp;
use futures::future;
use crate::calendar::CalendarGenerator;
use crate::time_config::TimeConfig;
use std::collections::{BTreeMap, HashMap};

pub struct SiteGenerator {
    output_dir: PathBuf,
    site_title: String,
    include_unread: bool,
    books_path: Option<PathBuf>,
    statistics_db_path: Option<PathBuf>,
    heatmap_scale_max: Option<u32>,
    time_config: TimeConfig,
}

impl SiteGenerator {
    pub fn new(
        output_dir: PathBuf, 
        site_title: String, 
        include_unread: bool, 
        books_path: Option<PathBuf>, 
        statistics_db_path: Option<PathBuf>,
        heatmap_scale_max: Option<u32>,
        time_config: TimeConfig,
    ) -> Self {
        Self {
            output_dir,
            site_title,
            include_unread,
            books_path,
            statistics_db_path,
            heatmap_scale_max,
            time_config,
        }
    }

    // Path constants to avoid duplication
    fn books_dir(&self) -> PathBuf { self.output_dir.join("books") }
    fn calendar_dir(&self) -> PathBuf { self.output_dir.join("calendar") }
    fn statistics_dir(&self) -> PathBuf { self.output_dir.join("statistics") }
    fn recap_dir(&self) -> PathBuf { self.output_dir.join("recap") }
    fn assets_dir(&self) -> PathBuf { self.output_dir.join("assets") }
    fn covers_dir(&self) -> PathBuf { self.assets_dir().join("covers") }
    fn css_dir(&self) -> PathBuf { self.assets_dir().join("css") }
    fn js_dir(&self) -> PathBuf { self.assets_dir().join("js") }
    fn json_dir(&self) -> PathBuf { self.assets_dir().join("json") }
    fn statistics_json_dir(&self) -> PathBuf { self.json_dir().join("statistics") }
    fn calendar_json_dir(&self) -> PathBuf { self.json_dir().join("calendar") }
    
    pub async fn generate(&self) -> Result<()> {
        info!("Generating static site in: {:?}", self.output_dir);
        
        // Scan books if books_path is provided
        let books = if let Some(ref books_path) = self.books_path {
            scan_books(books_path).await?
        } else {
            Vec::new()
        };
        
        // After loading statistics if path is provided
        let mut stats_data = if let Some(ref stats_path) = self.statistics_db_path {
            if stats_path.exists() {
                let mut data = StatisticsParser::parse(stats_path)?;
                crate::statistics::StatisticsCalculator::populate_completions(&mut data, &self.time_config);
                Some(data)
            } else {
                info!("Statistics database not found: {:?}", stats_path);
                None
            }
        } else {
            None
        };

        // Compute latest recap href (if any completions are available)
        let recap_latest_href: Option<String> = stats_data.as_ref().and_then(|sd| {
            let mut years: Vec<i32> = Vec::new();
            for b in &sd.books {
                if let Some(cs) = &b.completions {
                    for c in &cs.entries {
                        if c.end_date.len() >= 4 {
                            if let Ok(y) = c.end_date[0..4].parse::<i32>() {
                                if !years.contains(&y) {
                                    years.push(y);
                                }
                            }
                        }
                    }
                }
            }
            if years.is_empty() {
                None
            } else {
                years.sort_by(|a, b| b.cmp(a));
                Some(format!("/recap/{}/", years[0]))
            }
        });
        
        // Create output directories based on what we're generating
        self.create_directories(&books, &stats_data).await?;
        
        // Copy static assets
        self.copy_static_assets(&books, &stats_data).await?;
        
        // Generate book covers
        self.generate_covers(&books).await?;

        // Generate individual book pages
        self.generate_book_pages(&books, &mut stats_data, recap_latest_href.clone()).await?;
        
        if !books.is_empty() {
            // Generate book list page at index.html
            self.generate_book_list(&books, recap_latest_href.clone()).await?;
        }
        
        if let Some(ref mut stats_data) = stats_data {
            // Generate statistics page (render to root if no books)
            self.generate_statistics_page(stats_data, books.is_empty(), recap_latest_href.clone()).await?;
            
            // Generate calendar page if we have statistics data
            self.generate_calendar_page(stats_data, &books, recap_latest_href.clone()).await?;

            // Generate recap pages (static yearly)
            self.generate_recap_pages(stats_data, &books).await?;
        }

        info!("Static site generation completed!");

        Ok(())
    }
    
    // Get current version from Cargo.toml
    fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
    
    // Get current datetime as formatted string
    fn get_last_updated(&self) -> String { self.time_config.now_formatted() }

    // Minifies and writes HTML to disk.
    fn write_minify_html<P: AsRef<Path>>(&self, path: P, html: &str) -> Result<()> {
        let cfg = Cfg {
            minify_js: true,
            minify_css: true,
            ..Default::default()
        };

        // Attempt minification; on failure fall back to original HTML
        let minified = String::from_utf8(minify(html.as_bytes(), &cfg)).unwrap_or_else(|_| html.to_string());
        fs::write(path, minified)?;
        Ok(())
    }
    
    async fn create_directories(&self, books: &[Book], stats_data: &Option<StatisticsData>) -> Result<()> {
        fs::create_dir_all(&self.output_dir)?;
        
        // Only create books directory if we have books
        if !books.is_empty() {
            fs::create_dir_all(self.books_dir())?;
            fs::create_dir_all(self.covers_dir())?;
        }
        
        // Always create assets directories for CSS/JS as they're always needed
        fs::create_dir_all(self.css_dir())?;
        fs::create_dir_all(self.js_dir())?;
        
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
    
    async fn copy_static_assets(&self, books: &[Book], stats_data: &Option<StatisticsData>) -> Result<()> {
        // Write the pre-compiled CSS (always needed for basic styling)
        let css_content = include_str!(concat!(env!("OUT_DIR"), "/compiled_style.css"));
        fs::write(self.css_dir().join("style.css"), css_content)?;
        
        // Copy book-related JavaScript files only if we have books
        if !books.is_empty() {
            let js_content = include_str!("../assets/book_list.js");
            fs::write(self.js_dir().join("book_list.js"), js_content)?;

            let js_content = include_str!("../assets/book_detail.js");
            fs::write(self.js_dir().join("book_detail.js"), js_content)?;
            
            let lazy_loading_content = include_str!("../assets/lazy-loading.js");
            fs::write(self.js_dir().join("lazy-loading.js"), lazy_loading_content)?;

            let section_toggle_content = include_str!("../assets/section-toggle.js");
            fs::write(self.js_dir().join("section-toggle.js"), section_toggle_content)?;
        }
        
        // Copy statistics-related JavaScript files only if we have stats data
        if stats_data.is_some() {
            let stats_js_content = include_str!("../assets/statistics.js");
            fs::write(self.js_dir().join("statistics.js"), stats_js_content)?;
            
            let heatmap_js_content = include_str!("../assets/heatmap.js");
            fs::write(self.js_dir().join("heatmap.js"), heatmap_js_content)?;

            let calendar_css = include_str!(concat!(env!("OUT_DIR"), "/event-calendar.min.css"));
            fs::write(self.css_dir().join("event-calendar.min.css"), calendar_css)?;

            let calendar_js = include_str!(concat!(env!("OUT_DIR"), "/event-calendar.min.js"));
            fs::write(self.js_dir().join("event-calendar.min.js"), calendar_js)?;
            
            let calendar_map = include_str!(concat!(env!("OUT_DIR"), "/event-calendar.min.js.map"));
            fs::write(self.js_dir().join("event-calendar.min.js.map"), calendar_map)?;
            
            let calendar_init_js_content = include_str!("../assets/calendar.js");
            fs::write(self.js_dir().join("calendar.js"), calendar_init_js_content)?;

            // Recap small UI logic
            let recap_js_content = include_str!("../assets/recap.js");
            fs::write(self.js_dir().join("recap.js"), recap_js_content)?;
        }
        
        Ok(())
    }
    
    async fn generate_covers(&self, books: &[Book]) -> Result<()> {
        info!("Generating book covers...");
        
        // Collect all cover generation tasks
        let mut tasks = Vec::new();
        
        for book in books {
            if let Some(ref cover_data) = book.epub_info.cover_data {
                let cover_path = self.covers_dir().join(format!("{}.webp", book.id));
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
    
    async fn generate_book_list(&self, books: &[Book], recap_latest_href: Option<String>) -> Result<()> {
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
        
        // ------------------------------------------------------------------
        // Generate books manifest JSON categorized by reading status.
        // NOTE: This manifest is not consumed by the frontend code – it is
        // generated purely for the convenience of users who may want a
        // machine-readable list of all books and their export paths.
        // ------------------------------------------------------------------

        use serde_json::json;

        let to_manifest_entry = |b: &Book| {
            json!({
                "id": b.id.clone(),
                "title": b.epub_info.title.clone(),
                "authors": b.epub_info.authors.clone(),
                "json_path": format!("/books/{}/details.json", b.id),
                "html_path":  format!("/books/{}/index.html",  b.id),
            })
        };

        let reading_json: Vec<_> = reading_books.iter().map(to_manifest_entry).collect();
        let completed_json: Vec<_> = completed_books.iter().map(to_manifest_entry).collect();
        let new_json: Vec<_> = unread_books.iter().map(to_manifest_entry).collect();

        let manifest = json!({
            "reading": reading_json,
            "completed": completed_json,
            "new": new_json,
            "generated_at": self.get_last_updated(),
        });

        fs::write(
            self.books_dir().join("list.json"),
            serde_json::to_string_pretty(&manifest)?,
        )?;

        // ------------------------------------------------------------------
        // Render book list HTML
        // ------------------------------------------------------------------

        let template = IndexTemplate {
            site_title: self.site_title.clone(),
            reading_books,
            completed_books,
            unread_books,
            version: self.get_version(),
            last_updated: self.get_last_updated(),
            navbar_items: self.create_navbar_items_with_recap("books", recap_latest_href.as_deref()),
        };

        let html = template.render()?;
        self.write_minify_html(self.output_dir.join("index.html"), &html)?;
        
        Ok(())
    }
    
    async fn generate_book_pages(&self, books: &[Book], stats_data: &mut Option<StatisticsData>, recap_latest_href: Option<String>) -> Result<()> {
        info!("Generating book detail pages...");
        
        for book in books {
            // Try to find matching statistics by MD5
            let book_stats = stats_data.as_ref().and_then(|stats| {
                // Try to match using the partial_md5_checksum from KoReader metadata
                book.koreader_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.partial_md5_checksum.as_ref())
                    .and_then(|md5| stats.stats_by_md5.get(md5))
                    .cloned()
            });
            
            // Calculate session statistics if we have book stats
            let session_stats = match (stats_data.as_ref(), &book_stats) {
                (Some(stats), Some(book_stat)) => Some(book_stat.calculate_session_stats(&stats.page_stats, &self.time_config)),
                _ => None,
            };
            
            let template = BookTemplate {
                site_title: self.site_title.clone(),
                book: book.clone(),
                book_stats: book_stats.clone(),
                session_stats: session_stats.clone(),
                version: self.get_version(),
                last_updated: self.get_last_updated(),
                navbar_items: self.create_navbar_items_with_recap("books", recap_latest_href.as_deref()),
            };
            
            let html = template.render()?;
            let book_dir = self.books_dir().join(&book.id);
            fs::create_dir_all(&book_dir)?;
            let book_path = book_dir.join("index.html");
            self.write_minify_html(book_path, &html)?;

            // Generate Markdown export
            let md_template = BookMarkdownTemplate {
                book: book.clone(),
                book_stats: book_stats.clone(),
                session_stats: session_stats.clone(),
                version: self.get_version(),
                last_updated: self.get_last_updated(),
            };
            let markdown = md_template.render()?;
            fs::write(book_dir.join("details.md"), markdown)?;

            // Generate JSON export / not used by the frontend code - only for the user's convenience
            let json_data = serde_json::json!({
                "book": {
                    "title": book.epub_info.title,
                    "authors": book.epub_info.authors,
                    "series": book.series_display(),
                    "language": book.language(),
                    "publisher": book.publisher(),
                    "description": book.epub_info.sanitized_description(),
                    "rating": book.rating(),
                    "review_note": book.review_note(),
                    "status": book.status().to_string(),
                    "progress_percentage": book.progress_percentage(),
                    "subjects": book.subjects(),
                    "identifiers": book.identifiers().iter().map(|id| {
                        serde_json::json!({
                            "scheme": id.scheme,
                            "value": id.value,
                            "display_scheme": id.display_scheme(),
                            "url": id.url()
                        })
                    }).collect::<Vec<_>>()
                },
                "annotations": book.koreader_metadata.as_ref().map(|m| &m.annotations).unwrap_or(&vec![]),
                "statistics": {
                    "book_stats": book_stats,
                    "session_stats": session_stats,
                    "completions": book_stats.as_ref().and_then(|stats| stats.completions.as_ref())
                },
                "export_info": {
                    "generated_by": "KoShelf",
                    "version": self.get_version(),
                    "generated_at": self.get_last_updated()
                }
            });
            let json_str = serde_json::to_string_pretty(&json_data)?;
            fs::write(book_dir.join("details.json"), json_str)?;
        }
        
        Ok(())
    }
    
    async fn generate_statistics_page(&self, stats_data: &mut StatisticsData, render_to_root: bool, recap_latest_href: Option<String>) -> Result<()> {
        if render_to_root {
            info!("Generating statistics page at root index...");
        } else {
            info!("Generating statistics page...");
        }
        
        // Calculate reading stats from the parsed data and populate completions
            let reading_stats = StatisticsParser::calculate_stats(stats_data, &self.time_config);
        
        // Export daily activity data grouped by year as separate JSON files and get available years
        let available_years = self.export_daily_activity_by_year(&reading_stats.daily_activity).await?;

        // Export individual week data as separate JSON files
        for (index, week) in reading_stats.weeks.iter().enumerate() {
            let week_json = serde_json::to_string_pretty(&week)?;
            fs::write(self.statistics_json_dir().join(format!("week_{}.json", index)), week_json)?;
        }
        
        // Create the template with appropriate navbar
        let template = StatsTemplate {
            site_title: self.site_title.clone(),
            reading_stats: reading_stats.clone(),
            available_years,
            version: self.get_version(),
            last_updated: self.get_last_updated(),
            navbar_items: self.create_navbar_items_with_recap("statistics", recap_latest_href.as_deref()),
        };
        
        // Render and write the template
        let html = template.render()?;

        if render_to_root {
            // Write directly to index.html
            self.write_minify_html(self.output_dir.join("index.html"), &html)?;
        } else {
            // Create stats directory and write the index file
            let stats_dir = self.output_dir.join("statistics");
            fs::create_dir_all(&stats_dir)?;
            self.write_minify_html(stats_dir.join("index.html"), &html)?;
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
            let file_path = self.statistics_json_dir().join(filename);
            
            // Wrap the data with configuration information
            let json_data = serde_json::json!({
                "data": year_data,
                "config": {
                    "max_scale_seconds": self.heatmap_scale_max
                }
            });
            
            let json = serde_json::to_string_pretty(&json_data)?;
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

    fn create_navbar_items_with_recap(&self, current_page: &str, recap_latest_href: Option<&str>) -> Vec<NavItem> {
        let mut items = self.create_navbar_items(current_page);
        if self.statistics_db_path.is_some() {
            if let Some(href) = recap_latest_href {
                items.push(NavItem {
                    label: "Recap".to_string(),
                    href: href.to_string(),
                    icon_svg: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z".to_string(),
                    is_active: current_page == "recap",
                });
            }
        }
        items
    }
    
    async fn generate_calendar_page(&self, stats_data: &mut StatisticsData, books: &[Book], recap_latest_href: Option<String>) -> Result<()> {
        info!("Generating calendar page...");
        
        // Generate per-month calendar payloads (events + books + stats)
        let calendar_months = CalendarGenerator::generate_calendar_months(stats_data, books, &self.time_config);

        // ------------------------------------------------------------------
        // Write JSON files --------------------------------------------------
        // ------------------------------------------------------------------
        fs::create_dir_all(&self.calendar_json_dir())?;

        // Available months (newest first)
        let mut available_months: Vec<String> = calendar_months.keys().cloned().collect();
        available_months.sort_by(|a, b| b.cmp(a));
        fs::write(
            self.calendar_json_dir().join("available_months.json"),
            serde_json::to_string_pretty(&available_months)?,
        )?;

        // Individual month files
        for (ym, month_data) in &calendar_months {
            let filename = format!("{}.json", ym);
            fs::write(
                self.calendar_json_dir().join(filename),
                serde_json::to_string_pretty(&month_data)?,
            )?;
        }

        // Create the template
        let template = CalendarTemplate {
            site_title: self.site_title.clone(),
            version: self.get_version(),
            last_updated: self.get_last_updated(),
            navbar_items: self.create_navbar_items_with_recap("calendar", recap_latest_href.as_deref()),
        };
        
        // Render and write the template
        let html = template.render()?;

        // Write to the calendar directory (already created in create_directories)
        self.write_minify_html(self.calendar_dir().join("index.html"), &html)?;
        
        Ok(())
    }

    async fn generate_recap_pages(&self, stats_data: &mut StatisticsData, books: &[Book]) -> Result<()> {
        info!("Generating recap pages...");

        let format_duration = |seconds: i64| -> String {
            if seconds <= 0 { return "0m".to_string(); }
            let total_minutes = seconds / 60;
            let days = total_minutes / (24 * 60);
            let hours = (total_minutes % (24 * 60)) / 60;
            let mins = total_minutes % 60;
            let mut parts: Vec<String> = Vec::new();
            if days > 0 { parts.push(format!("{}d", days)); }
            if hours > 0 { parts.push(format!("{}h", hours)); }
            if mins > 0 || parts.is_empty() { parts.push(format!("{}m", mins)); }
            parts.join(" ")
        };

        // Build md5 -> &Book map for cover/link enrichment
        let mut md5_to_book: HashMap<String, &Book> = HashMap::new();
        for book in books {
            if let Some(md5) = book
                .koreader_metadata
                .as_ref()
                .and_then(|m| m.partial_md5_checksum.as_ref())
            {
                md5_to_book.insert(md5.clone(), book);
            }
        }

        // Compute reading stats once to get daily activity for hour totals
        let reading_stats = crate::statistics_parser::StatisticsParser::calculate_stats(stats_data, &self.time_config);

        // Human-friendly date formatter without year, using month name (e.g., "7 Mar")
        let format_day_month = |iso: &str| -> String {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(iso, "%Y-%m-%d") {
                // "%e %b" -> day (space-padded) and abbreviated month name; trim_left for clean day
                let s = date.format("%e %b").to_string();
                s.trim_start().to_string()
            } else {
                iso.to_string()
            }
        };

        // Build year -> month (YYYY-MM) -> Vec<RecapItem>
        let mut year_month_items: HashMap<i32, BTreeMap<String, Vec<crate::models::RecapItem>>> = HashMap::new();
        let mut years: Vec<i32> = Vec::new();

        for sb in &stats_data.books {
            if let Some(comps) = &sb.completions {
                for c in &comps.entries {
                    if c.end_date.len() < 7 { continue; }
                    let year_str = &c.end_date[0..4];
                    let ym = c.end_date[0..7].to_string(); // YYYY-MM
                    let year: i32 = match year_str.parse() { Ok(v) => v, Err(_) => continue };

                    if !years.contains(&year) {
                        years.push(year);
                    }

                    // Enrich from Book when possible
                    let (title, authors, rating, review_note, series_display, book_path, book_cover) = if let Some(book) = md5_to_book.get(&sb.md5) {
                        let title = book.epub_info.title.clone();
                        let authors = book.epub_info.authors.clone();
                        let rating = book.rating();
                        let review_note = book.review_note().cloned();
                        let series_display = book.series_display();
                        let book_path = Some(format!("/books/{}/index.html", book.id));
                        let book_cover = Some(format!("/assets/covers/{}.webp", book.id));
                        (title, authors, rating, review_note, series_display, book_path, book_cover)
                    } else {
                        // Fallback to StatBook minimal info
                        let title = sb.title.clone();
                        let authors = sb.authors.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>();
                        (title, authors, None, None, None, None, None)
                    };

                    let item = crate::models::RecapItem {
                        title,
                        authors,
                        start_date: c.start_date.clone(),
                        end_date: c.end_date.clone(),
                        start_display: format_day_month(&c.start_date),
                        end_display: format_day_month(&c.end_date),
                        reading_time: c.reading_time,
                        reading_time_display: format_duration(c.reading_time),
                        session_count: c.session_count,
                        pages_read: c.pages_read,
                        rating,
                        review_note,
                        series_display,
                        book_path,
                        book_cover,
                        star_display: {
                            let mut stars = [false; 5];
                            if let Some(r) = rating {
                                let n = std::cmp::min(r as usize, 5);
                                for i in 0..n { stars[i] = true; }
                            }
                            stars
                        },
                    };

                    year_month_items
                        .entry(year)
                        .or_default()
                        .entry(ym.clone())
                        .or_default()
                        .push(item);
                }
            }
        }

        if years.is_empty() {
            // No completions → don't generate recap pages
            return Ok(());
        }

        years.sort_by(|a, b| b.cmp(a)); // newest first

        // Pre-compute monthly hours from daily activity: map YYYY-MM -> seconds
        let mut month_hours: HashMap<String, i64> = HashMap::new();
        for day in &reading_stats.daily_activity {
            if day.date.len() >= 7 {
                let ym = day.date[0..7].to_string();
                *month_hours.entry(ym).or_insert(0) += day.read_time;
            }
        }

        // Render each year page
        for (idx, year) in years.iter().enumerate() {
            let months_map = year_month_items.get(year).cloned().unwrap_or_default();
            // Build MonthRecap BTreeMap sorted by month ascending (Jan..Dec)
            let mut monthly: BTreeMap<String, crate::models::MonthRecap> = BTreeMap::new();

            for (ym, mut items) in months_map {
                if items.is_empty() { continue; }
                items.sort_by(|a, b| a.end_date.cmp(&b.end_date));

                let month_label = if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{}-01", ym), "%Y-%m-%d") {
                    date.format("%B").to_string()
                } else { ym.clone() };

                let hours = *month_hours.get(&ym).unwrap_or(&0);
                let month_recap = crate::models::MonthRecap {
                    month_key: ym.clone(),
                    month_label,
                    books_finished: items.len(),
                    hours_read_seconds: hours,
                    hours_read_display: format_duration(hours),
                    items,
                };
                monthly.insert(ym, month_recap);
            }

            // Determine prev/next year for controls
            let prev_year = years.get(idx + 1).cloned();
            let next_year = if idx > 0 { years.get(idx - 1).cloned() } else { None };

            // Convert monthly map to a vector in chronological order
            let monthly_vec: Vec<crate::models::MonthRecap> = monthly.into_values().collect();

            // Latest year href for sidebar
            let latest_href = format!("/recap/{}/", years[0]);

            let template = crate::templates::RecapTemplate {
                site_title: self.site_title.clone(),
                year: *year,
                available_years: years.clone(),
                prev_year,
                next_year,
                monthly: monthly_vec,
                version: self.get_version(),
                last_updated: self.get_last_updated(),
                navbar_items: self.create_navbar_items_with_recap("recap", Some(latest_href.as_str())),
            };

            let html = template.render()?;
            let year_dir = self.recap_dir().join(format!("{}", year));
            fs::create_dir_all(&year_dir)?;
            let path = year_dir.join("index.html");
            self.write_minify_html(path, &html)?;
        }

        Ok(())
    }
} 