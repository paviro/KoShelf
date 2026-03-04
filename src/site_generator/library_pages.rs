//! Content list and detail page generation (books + comics).

use super::SiteGenerator;
use crate::contracts::mappers;
use crate::koreader::BookStatistics;
use crate::models::{BookStatus, ContentType, LibraryItem, StatisticsData};
use crate::runtime::ContractSnapshot;
use crate::templates::{ItemDetailMarkdownTemplate, ItemDetailTemplate, LibraryListTemplate};
use anyhow::Result;
use askama::Template;
use log::{info, warn};
use std::collections::HashSet;
use std::fs;

use super::utils::UiContext;

#[derive(Debug, Default)]
struct StatusBuckets {
    reading: Vec<LibraryItem>,
    completed: Vec<LibraryItem>,
    abandoned: Vec<LibraryItem>,
    unread: Vec<LibraryItem>,
}

impl StatusBuckets {
    fn from_items(items: &[LibraryItem]) -> Self {
        let mut out = Self::default();

        for item in items {
            match item.status() {
                BookStatus::Reading => out.reading.push(item.clone()),
                BookStatus::Complete => out.completed.push(item.clone()),
                BookStatus::Abandoned => out.abandoned.push(item.clone()),
                BookStatus::Unknown => {
                    // If it has KoReader metadata, treat as reading; otherwise it's unread.
                    if item.koreader_metadata.is_some() {
                        out.reading.push(item.clone());
                    } else {
                        out.unread.push(item.clone());
                    }
                }
            }
        }

        out.reading
            .sort_by(|a, b| a.book_info.title.cmp(&b.book_info.title));
        out.completed
            .sort_by(|a, b| a.book_info.title.cmp(&b.book_info.title));
        out.abandoned
            .sort_by(|a, b| a.book_info.title.cmp(&b.book_info.title));
        out.unread
            .sort_by(|a, b| a.book_info.title.cmp(&b.book_info.title));

        out
    }
}

impl SiteGenerator {
    fn content_dir(&self, content_type: ContentType) -> std::path::PathBuf {
        match content_type {
            ContentType::Book => self.books_dir(),
            ContentType::Comic => self.comics_dir(),
        }
    }

    fn content_slug(content_type: ContentType) -> &'static str {
        match content_type {
            ContentType::Book => "books",
            ContentType::Comic => "comics",
        }
    }

    fn data_content_dir(&self, content_type: ContentType) -> std::path::PathBuf {
        match content_type {
            ContentType::Book => self.data_books_dir(),
            ContentType::Comic => self.data_comics_dir(),
        }
    }

    async fn generate_content_list(
        &self,
        content_type: ContentType,
        items: &[LibraryItem],
        render_to_root: bool,
        ui: &UiContext,
    ) -> Result<()> {
        info!(
            "Generating {} list page...",
            Self::content_slug(content_type)
        );

        let buckets = StatusBuckets::from_items(items);

        let template = LibraryListTemplate {
            site_title: self.site_title.clone(),
            details_base_path: match content_type {
                ContentType::Book => "/books/".to_string(),
                ContentType::Comic => "/comics/".to_string(),
            },
            reading_books: buckets.reading,
            completed_books: buckets.completed,
            abandoned_books: buckets.abandoned,
            unread_books: buckets.unread,
            version: self.get_version(),
            last_updated: self.get_last_updated(),
            navbar_items: self.create_navbar_items_with_recap(
                match content_type {
                    ContentType::Book => "books",
                    ContentType::Comic => "comics",
                },
                ui.recap_latest_href.as_deref(),
                ui.nav,
            ),
            translation: self.t(),
        };

        let html = template.render()?;

        let output_path = if render_to_root {
            self.output_dir.join("index.html")
        } else {
            self.content_dir(content_type).join("index.html")
        };
        self.write_minify_html(output_path, &html)?;
        Ok(())
    }

    async fn generate_content_pages(
        &self,
        content_type: ContentType,
        items: &[LibraryItem],
        stats_data: &mut Option<StatisticsData>,
        ui: &UiContext,
        snapshot: &mut ContractSnapshot,
    ) -> Result<()> {
        info!(
            "Generating {} detail pages...",
            Self::content_slug(content_type)
        );

        for item in items {
            // Try to find matching statistics by MD5
            let item_stats = stats_data.as_ref().and_then(|stats| {
                item.koreader_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.partial_md5_checksum.as_ref())
                    .and_then(|md5| stats.stats_by_md5.get(md5))
                    .cloned()
            });

            // Calculate session statistics if we have item stats
            let session_stats = match (stats_data.as_ref(), &item_stats) {
                (Some(stats), Some(stat)) => Some(stat.calculate_session_stats(
                    &stats.page_stats,
                    &self.time_config,
                    &self.translations,
                )),
                _ => None,
            };

            let search_base_path = match content_type {
                ContentType::Comic if ui.nav.has_books => "/comics/".to_string(),
                _ => "/".to_string(),
            };

            let template = ItemDetailTemplate {
                site_title: self.site_title.clone(),
                book: item.clone(),
                book_stats: item_stats.clone(),
                session_stats: session_stats.clone(),
                search_base_path: search_base_path.clone(),
                version: self.get_version(),
                last_updated: self.get_last_updated(),
                navbar_items: self.create_navbar_items_with_recap(
                    match content_type {
                        ContentType::Book => "books",
                        ContentType::Comic => "comics",
                    },
                    ui.recap_latest_href.as_deref(),
                    ui.nav,
                ),
                translation: self.t(),
            };

            let html = template.render()?;
            let item_dir = self.content_dir(content_type).join(&item.id);
            fs::create_dir_all(&item_dir)?;
            let item_path = item_dir.join("index.html");
            self.write_minify_html(item_path, &html)?;

            // Generate Markdown export
            let md_template = ItemDetailMarkdownTemplate {
                book: item.clone(),
                book_stats: item_stats.clone(),
                session_stats: session_stats.clone(),
                version: self.get_version(),
                last_updated: self.get_last_updated(),
            };
            let markdown = md_template.render()?;
            let md_path = item_dir.join("details.md");
            self.write_registered_string(md_path, &markdown)?;

            // New contract-based static data output for frontend `/data` mode.
            let contract_meta = mappers::build_meta(self.get_version(), self.get_last_updated());
            let contract_detail = mappers::map_library_detail_response(
                contract_meta,
                item,
                search_base_path,
                item_stats.clone(),
                session_stats,
            );
            match content_type {
                ContentType::Book => {
                    snapshot
                        .book_details
                        .insert(item.id.clone(), contract_detail);
                }
                ContentType::Comic => {
                    snapshot
                        .comic_details
                        .insert(item.id.clone(), contract_detail);
                }
            }
        }

        Ok(())
    }

    /// Clean up book directories for books that no longer exist in the library
    pub(crate) fn cleanup_stale_books(&self, books: &[LibraryItem]) -> Result<()> {
        self.cleanup_stale_content_dirs(self.books_dir(), books, "book")?;
        self.cleanup_stale_content_data_files(ContentType::Book, books, "book")
    }

    /// Clean up cover files for books that no longer exist in the library
    pub(crate) fn cleanup_stale_covers(&self, books: &[LibraryItem]) -> Result<()> {
        let covers_dir = self.covers_dir();

        // If covers directory doesn't exist, nothing to clean up
        if !covers_dir.exists() {
            return Ok(());
        }

        // Build set of current book IDs
        let current_ids: HashSet<String> = books.iter().map(|b| b.id.clone()).collect();

        // Iterate over existing cover files
        let entries = fs::read_dir(&covers_dir)?;
        for entry in entries.flatten() {
            let path = entry.path();

            // Skip if not a file
            if !path.is_file() {
                continue;
            }

            // Get filename without extension (book ID)
            if let Some(file_stem) = path.file_stem().and_then(|n| n.to_str()) {
                // If this book ID is not in current books, remove the cover
                if !current_ids.contains(file_stem) {
                    info!("Removing stale cover: {:?}", path);
                    if let Err(e) = fs::remove_file(&path) {
                        warn!("Failed to remove stale cover {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Clean up comic directories for comics that no longer exist in the library
    pub(crate) fn cleanup_stale_comics(&self, comics: &[LibraryItem]) -> Result<()> {
        self.cleanup_stale_content_dirs(self.comics_dir(), comics, "comic")?;
        self.cleanup_stale_content_data_files(ContentType::Comic, comics, "comic")
    }

    fn cleanup_stale_content_data_files(
        &self,
        content_type: ContentType,
        items: &[LibraryItem],
        label: &str,
    ) -> Result<()> {
        let data_dir = self.data_content_dir(content_type);
        if !data_dir.exists() {
            return Ok(());
        }

        let current_ids: HashSet<String> = items.iter().map(|item| item.id.clone()).collect();
        let entries = fs::read_dir(&data_dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            if let Some(file_stem) = path.file_stem().and_then(|name| name.to_str())
                && !current_ids.contains(file_stem)
            {
                info!("Removing stale {} data file: {:?}", label, path);
                if let Err(error) = fs::remove_file(&path) {
                    warn!(
                        "Failed to remove stale {} data file {:?}: {}",
                        label, path, error
                    );
                }
            }
        }

        Ok(())
    }

    fn cleanup_stale_content_dirs(
        &self,
        content_dir: std::path::PathBuf,
        items: &[LibraryItem],
        label: &str,
    ) -> Result<()> {
        if !content_dir.exists() {
            return Ok(());
        }

        let current_ids: HashSet<String> = items.iter().map(|b| b.id.clone()).collect();
        let entries = fs::read_dir(&content_dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str())
                && !current_ids.contains(dir_name)
            {
                info!("Removing stale {} directory: {:?}", label, path);
                if let Err(e) = fs::remove_dir_all(&path) {
                    warn!(
                        "Failed to remove stale {} directory {:?}: {}",
                        label, path, e
                    );
                }
            }
        }
        Ok(())
    }

    // Public wrappers kept for readability at call sites.
    pub(crate) async fn generate_book_list(
        &self,
        books: &[LibraryItem],
        ui: &UiContext,
    ) -> Result<()> {
        self.generate_content_list(ContentType::Book, books, true, ui)
            .await
    }

    pub(crate) async fn generate_comic_list(
        &self,
        comics: &[LibraryItem],
        render_to_root: bool,
        ui: &UiContext,
    ) -> Result<()> {
        self.generate_content_list(ContentType::Comic, comics, render_to_root, ui)
            .await
    }

    pub(crate) async fn generate_book_pages(
        &self,
        books: &[LibraryItem],
        stats_data: &mut Option<StatisticsData>,
        ui: &UiContext,
        snapshot: &mut ContractSnapshot,
    ) -> Result<()> {
        self.generate_content_pages(ContentType::Book, books, stats_data, ui, snapshot)
            .await
    }

    pub(crate) async fn generate_comic_pages(
        &self,
        comics: &[LibraryItem],
        stats_data: &mut Option<StatisticsData>,
        ui: &UiContext,
        snapshot: &mut ContractSnapshot,
    ) -> Result<()> {
        self.generate_content_pages(ContentType::Comic, comics, stats_data, ui, snapshot)
            .await
    }
}
