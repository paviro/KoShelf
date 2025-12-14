//! Site generator module - orchestrates static site generation for KoShelf.
//!
//! This module is split into submodules for maintainability:
//! - `assets`: Directory creation, static assets, and cover generation
//! - `library_pages`: Library list and detail page generation (books + comics)
//! - `statistics`: Statistics page and JSON export
//! - `calendar`: Calendar page generation
//! - `recap`: Yearly recap page generation
//! - `cache_manifest`: PWA cache manifest generation
//! - `utils`: Utility functions (minification, navbar, version info)

mod assets;
mod cache_manifest;
mod calendar;
mod library_pages;
mod recap;
mod statistics;
mod utils;

pub use cache_manifest::CacheManifestBuilder;

use crate::config::SiteConfig;
use crate::i18n::Translations;
use crate::library::scan_library;
use crate::models::{BookStatus, ContentType, LibraryItem, StatisticsData};
use crate::koreader::{calculate_partial_md5, StatisticsCalculator, StatisticsParser};
use anyhow::Result;
use log::info;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use utils::{NavContext, UiContext};

#[derive(Debug)]
struct GenerationContext {
    all_items: Vec<LibraryItem>,
    books: Vec<LibraryItem>,
    comics: Vec<LibraryItem>,
    stats_data: Option<StatisticsData>,
    recap_latest_href: Option<String>,
    nav: NavContext,
}

pub struct SiteGenerator {
    config: SiteConfig,
    /// Cache manifest builder for PWA smart caching
    cache_manifest: Arc<CacheManifestBuilder>,
    /// Translations for i18n
    translations: Rc<Translations>,
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

        // Load translations for the configured language
        let translations = Rc::new(Translations::load(&config.language).unwrap_or_else(|e| {
            log::warn!(
                "Failed to load translations for '{}': {}. Falling back to English.",
                config.language,
                e
            );
            Translations::load("en").expect("English translations must exist")
        }));

        Self {
            config,
            cache_manifest,
            translations,
        }
    }

    /// Get Rc<Translations> for templates to call get()/get_with_num()
    pub(crate) fn t(&self) -> Rc<Translations> {
        Rc::clone(&self.translations)
    }

    // Path constants to avoid duplication
    pub(crate) fn books_dir(&self) -> PathBuf {
        self.output_dir.join("books")
    }
    pub(crate) fn comics_dir(&self) -> PathBuf {
        self.output_dir.join("comics")
    }
    pub(crate) fn calendar_dir(&self) -> PathBuf {
        self.output_dir.join("calendar")
    }
    pub(crate) fn statistics_dir(&self) -> PathBuf {
        self.output_dir.join("statistics")
    }
    pub(crate) fn recap_dir(&self) -> PathBuf {
        self.output_dir.join("recap")
    }
    pub(crate) fn assets_dir(&self) -> PathBuf {
        self.output_dir.join("assets")
    }
    pub(crate) fn covers_dir(&self) -> PathBuf {
        self.assets_dir().join("covers")
    }
    pub(crate) fn css_dir(&self) -> PathBuf {
        self.assets_dir().join("css")
    }
    pub(crate) fn js_dir(&self) -> PathBuf {
        self.assets_dir().join("js")
    }
    pub(crate) fn icons_dir(&self) -> PathBuf {
        self.assets_dir().join("icons")
    }
    pub(crate) fn json_dir(&self) -> PathBuf {
        self.assets_dir().join("json")
    }
    pub(crate) fn statistics_json_dir(&self) -> PathBuf {
        self.json_dir().join("statistics")
    }
    pub(crate) fn calendar_json_dir(&self) -> PathBuf {
        self.json_dir().join("calendar")
    }

    fn recap_latest_href(stats_data: Option<&StatisticsData>) -> Option<String> {
        let sd = stats_data?;
        let mut years: Vec<i32> = Vec::new();
        for b in &sd.books {
            if let Some(cs) = &b.completions {
                for c in &cs.entries {
                    if c.end_date.len() >= 4
                        && let Ok(y) = c.end_date[0..4].parse::<i32>()
                        && !years.contains(&y)
                    {
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
    }

    async fn build_generation_context(&self) -> Result<GenerationContext> {
        // Scan all library paths for books and comics
        // Also returns the set of MD5 hashes for all items (for statistics filtering)
        let (all_items, library_md5s) = if !self.library_paths.is_empty() {
            scan_library(&self.library_paths, &self.metadata_location).await?
        } else {
            (Vec::new(), HashSet::new())
        };

        // Filter items based on include_unread setting
        // Items without KoReader metadata (unread) should only be included if include_unread is true
        let all_items: Vec<_> = all_items
            .into_iter()
            .filter(|item| {
                // Always include items with KoReader metadata
                if item.koreader_metadata.is_some() {
                    return true;
                }
                // For items without metadata, only include if include_unread is true
                // and the item has Unknown status (which is the case for unread items)
                self.include_unread && item.status() == BookStatus::Unknown
            })
            .collect();

        // Separate books and comics
        let books: Vec<_> = all_items.iter().filter(|b| b.is_book()).cloned().collect();
        let comics: Vec<_> = all_items.iter().filter(|b| b.is_comic()).cloned().collect();

        let has_books = !books.is_empty();
        let has_comics = !comics.is_empty();

        // Load statistics if path is provided
        let mut stats_data = if let Some(ref stats_path) = self.statistics_db_path {
            if stats_path.exists() {
                let mut data = StatisticsParser::parse(stats_path)?;

                // Filter statistics if minimums are set
                if self.min_pages_per_day.is_some() || self.min_time_per_day.is_some() {
                    StatisticsCalculator::filter_stats(
                        &mut data,
                        &self.time_config,
                        self.min_pages_per_day,
                        self.min_time_per_day,
                    );
                }

                // Filter statistics to library items only (unless --include-all-stats is set)
                // This is skipped if no library paths provided (can't filter without a library)
                if !self.include_all_stats && !all_items.is_empty() {
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

        let recap_latest_href = Self::recap_latest_href(stats_data.as_ref());

        let nav = NavContext {
            has_books,
            has_comics,
            stats_at_root: stats_data.is_some() && all_items.is_empty(),
        };

        // Tag statistics entries by content type using MD5 -> LibraryItem lookup
        if let Some(ref mut sd) = stats_data {
            let mut md5_to_content_type: HashMap<String, ContentType> = HashMap::new();
            for item in &all_items {
                // Prefer MD5 from KoReader metadata, but fall back to calculating partial MD5 from file.
                let md5 = item
                    .koreader_metadata
                    .as_ref()
                    .and_then(|m| m.partial_md5_checksum.as_ref())
                    .cloned()
                    .or_else(|| calculate_partial_md5(&item.file_path).ok());

                if let Some(md5) = md5 {
                    md5_to_content_type.insert(md5, item.content_type());
                } else {
                    log::debug!(
                        "Could not determine MD5 for {:?}; stats content_type tagging may be incomplete",
                        item.file_path
                    );
                }
            }
            sd.tag_content_types(&md5_to_content_type);
        }

        Ok(GenerationContext {
            all_items,
            books,
            comics,
            stats_data,
            recap_latest_href,
            nav,
        })
    }

    pub async fn generate(&self) -> Result<()> {
        info!("Generating static site in: {:?}", self.output_dir);
        let mut ctx = self.build_generation_context().await?;
        let ui = UiContext {
            recap_latest_href: ctx.recap_latest_href.clone(),
            nav: ctx.nav,
        };

        // Create output directories based on what we're generating
        self.create_directories(&ctx.all_items, &ctx.stats_data).await?;

        // Copy static assets
        self.copy_static_assets(&ctx.all_items, &ctx.stats_data)
            .await?;

        // Generate covers for all items (books and comics)
        self.generate_covers(&ctx.all_items).await?;

        // Clean up stale book directories (for deleted books)
        self.cleanup_stale_books(&ctx.books)?;

        // Clean up stale comic directories (for deleted comics)
        self.cleanup_stale_comics(&ctx.comics)?;

        // Clean up stale covers (for deleted items)
        self.cleanup_stale_covers(&ctx.all_items)?;

        // Generate individual book pages
        self.generate_book_pages(
            &ctx.books,
            &mut ctx.stats_data,
            &ui,
        )
            .await?;

        // Generate individual comic pages (always at /comics/<id>/)
        self.generate_comic_pages(
            &ctx.comics,
            &mut ctx.stats_data,
            &ui,
        )
            .await?;

        // Generate list pages with conditional routing:
        // - If books exist: book list at /
        // - If comics exist AND books exist: comic list at /comics/
        // - If comics exist AND no books: comic list at /
        if ctx.nav.has_books {
            // Generate book list page at index.html
            self.generate_book_list(&ctx.books, &ui)
                .await?;
        }

        if ctx.nav.has_comics {
            // Generate comic list - at root if no books, otherwise at /comics/
            self.generate_comic_list(
                &ctx.comics,
                !ctx.nav.has_books,
                &ui,
            )
                .await?;
        }

        if let Some(ref mut stats_data) = ctx.stats_data {
            // Generate statistics page (render to root if no items at all)
            self.generate_statistics_page(
                stats_data,
                ctx.all_items.is_empty(),
                &ui,
            )
            .await?;

            // Generate calendar page if we have statistics data
            self.generate_calendar_page(
                stats_data,
                &ctx.all_items,
                &ui,
            )
                .await?;

            // Generate recap pages (static yearly)
            self.generate_recap_pages(stats_data, &ctx.all_items, ctx.nav)
                .await?;
        }

        // Write cache manifest for PWA smart caching
        self.cache_manifest
            .write(self.output_dir.join("cache-manifest.json"))?;

        info!("Static site generation completed!");

        Ok(())
    }
}
