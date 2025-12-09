//! Site generator module - orchestrates static site generation for KoShelf.
//!
//! This module is split into submodules for maintainability:
//! - `assets`: Directory creation, static assets, and cover generation
//! - `books`: Book list and detail page generation
//! - `statistics`: Statistics page and JSON export
//! - `calendar`: Calendar page generation
//! - `recap`: Yearly recap page generation
//! - `cache_manifest`: PWA cache manifest generation
//! - `utils`: Utility functions (minification, navbar, version info)

mod assets;
mod books;
mod cache_manifest;
mod calendar;
mod recap;
mod statistics;
mod utils;

pub use cache_manifest::CacheManifestBuilder;

use crate::config::SiteConfig;
use crate::statistics_parser::StatisticsParser;
use crate::statistics::StatisticsCalculator;
use crate::book_scanner::scan_books;
use anyhow::Result;
use log::info;
use std::path::PathBuf;
use std::sync::Arc;

pub struct SiteGenerator {
    config: SiteConfig,
    /// Cache manifest builder for PWA smart caching
    cache_manifest: Arc<CacheManifestBuilder>,
}

impl std::ops::Deref for SiteGenerator {
    type Target = SiteConfig;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl SiteGenerator {
    pub fn new(config: SiteConfig) -> Self {
        // Create cache manifest with current timestamp as version
        let version = chrono::Local::now().to_rfc3339();
        let cache_manifest = Arc::new(CacheManifestBuilder::new(version));
        
        Self { config, cache_manifest }
    }

    // Path constants to avoid duplication
    pub(crate) fn books_dir(&self) -> PathBuf { self.output_dir.join("books") }
    pub(crate) fn calendar_dir(&self) -> PathBuf { self.output_dir.join("calendar") }
    pub(crate) fn statistics_dir(&self) -> PathBuf { self.output_dir.join("statistics") }
    pub(crate) fn recap_dir(&self) -> PathBuf { self.output_dir.join("recap") }
    pub(crate) fn assets_dir(&self) -> PathBuf { self.output_dir.join("assets") }
    pub(crate) fn covers_dir(&self) -> PathBuf { self.assets_dir().join("covers") }
    pub(crate) fn css_dir(&self) -> PathBuf { self.assets_dir().join("css") }
    pub(crate) fn js_dir(&self) -> PathBuf { self.assets_dir().join("js") }
    pub(crate) fn icons_dir(&self) -> PathBuf { self.assets_dir().join("icons") }
    pub(crate) fn json_dir(&self) -> PathBuf { self.assets_dir().join("json") }
    pub(crate) fn statistics_json_dir(&self) -> PathBuf { self.json_dir().join("statistics") }
    pub(crate) fn calendar_json_dir(&self) -> PathBuf { self.json_dir().join("calendar") }
    
    pub async fn generate(&self) -> Result<()> {
        info!("Generating static site in: {:?}", self.output_dir);
        
        // Scan books if books_path is provided
        // Also returns the set of MD5 hashes for all books (for statistics filtering)
        let (books, library_md5s) = if let Some(ref books_path) = self.books_path {
            scan_books(books_path, &self.metadata_location).await?
        } else {
            (Vec::new(), std::collections::HashSet::new())
        };
        
        // After loading statistics if path is provided
        let mut stats_data = if let Some(ref stats_path) = self.statistics_db_path {
            if stats_path.exists() {
                let mut data = StatisticsParser::parse(stats_path)?;
                
                // Filter statistics if minimums are set
                if self.min_pages_per_day.is_some() || self.min_time_per_day.is_some() {
                    StatisticsCalculator::filter_stats(
                        &mut data, 
                        &self.time_config, 
                        self.min_pages_per_day, 
                        self.min_time_per_day
                    );
                }
                
                // Filter statistics to library books only (unless --include-all-stats is set)
                // This is skipped if no books_path is provided (can't filter without a library)
                if !self.include_all_stats && !books.is_empty() {
                    StatisticsCalculator::filter_to_library(&mut data, &library_md5s);
                }
                
                StatisticsCalculator::populate_completions(&mut data, &self.time_config);
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
                        if c.end_date.len() >= 4
                            && let Ok(y) = c.end_date[0..4].parse::<i32>()
                            && !years.contains(&y) {
                            years.push(y);
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

        // Write cache manifest for PWA smart caching
        self.cache_manifest.write(self.output_dir.join("cache-manifest.json"))?;

        info!("Static site generation completed!");

        Ok(())
    }
}
