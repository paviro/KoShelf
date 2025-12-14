//! Content list and detail page generation (books + comics).

use super::SiteGenerator;
use crate::models::{BookStatus, ContentType, LibraryItem, StatisticsData};
use crate::koreader::BookStatistics;
use crate::templates::{ItemDetailMarkdownTemplate, ItemDetailTemplate, LibraryListTemplate};
use anyhow::Result;
use askama::Template;
use log::{info, warn};
use std::collections::HashSet;
use std::fs;

use super::utils::NavContext;

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

    fn write_list_manifest(
        &self,
        content_type: ContentType,
        buckets: &StatusBuckets,
    ) -> Result<()> {
        use serde_json::json;

        let slug = Self::content_slug(content_type);
        let to_manifest_entry = |b: &LibraryItem| {
            json!({
                "id": b.id.clone(),
                "title": b.book_info.title.clone(),
                "authors": b.book_info.authors.clone(),
                "json_path": format!("/{}/{}/details.json", slug, b.id),
                "html_path":  format!("/{}/{}/index.html",  slug, b.id),
            })
        };

        let manifest = json!({
            "content_type": match content_type { ContentType::Book => "book", ContentType::Comic => "comic" },
            "reading": buckets.reading.iter().map(to_manifest_entry).collect::<Vec<_>>(),
            "completed": buckets.completed.iter().map(to_manifest_entry).collect::<Vec<_>>(),
            "abandoned": buckets.abandoned.iter().map(to_manifest_entry).collect::<Vec<_>>(),
            "new": buckets.unread.iter().map(to_manifest_entry).collect::<Vec<_>>(),
            "generated_at": self.get_last_updated(),
        });

        fs::create_dir_all(self.content_dir(content_type))?;
        let list_json_path = self.content_dir(content_type).join("list.json");
        let list_json_content = serde_json::to_string_pretty(&manifest)?;
        self.cache_manifest.register_file(
            &list_json_path,
            &self.output_dir,
            list_json_content.as_bytes(),
        );
        fs::write(list_json_path, list_json_content)?;
        Ok(())
    }

    async fn generate_content_list(
        &self,
        content_type: ContentType,
        items: &[LibraryItem],
        render_to_root: bool,
        recap_latest_href: Option<String>,
        nav: NavContext,
    ) -> Result<()> {
        info!(
            "Generating {} list page...",
            Self::content_slug(content_type)
        );

        let buckets = StatusBuckets::from_items(items);
        self.write_list_manifest(content_type, &buckets)?;

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
                recap_latest_href.as_deref(),
                nav,
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
        recap_latest_href: Option<String>,
        nav: NavContext,
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

            let template = ItemDetailTemplate {
                site_title: self.site_title.clone(),
                book: item.clone(),
                book_stats: item_stats.clone(),
                session_stats: session_stats.clone(),
                search_base_path: match content_type {
                    ContentType::Comic if nav.has_books => "/comics/".to_string(),
                    _ => "/".to_string(),
                },
                version: self.get_version(),
                last_updated: self.get_last_updated(),
                navbar_items: self.create_navbar_items_with_recap(
                    match content_type {
                        ContentType::Book => "books",
                        ContentType::Comic => "comics",
                    },
                    recap_latest_href.as_deref(),
                    nav,
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
            self.cache_manifest
                .register_file(&md_path, &self.output_dir, markdown.as_bytes());
            fs::write(md_path, markdown)?;

            // Generate JSON export (not used by the frontend code - only for the user's convenience)
            let mut root = serde_json::Map::new();
            root.insert(
                "content_type".to_string(),
                serde_json::Value::String(match content_type {
                    ContentType::Book => "book".to_string(),
                    ContentType::Comic => "comic".to_string(),
                }),
            );

            let mut item_obj = serde_json::Map::new();
            item_obj.insert(
                "title".to_string(),
                serde_json::Value::String(item.book_info.title.clone()),
            );
            item_obj.insert(
                "authors".to_string(),
                serde_json::to_value(item.book_info.authors.clone())?,
            );
            item_obj.insert(
                "series".to_string(),
                serde_json::to_value(item.series_display())?,
            );
            item_obj.insert(
                "language".to_string(),
                serde_json::to_value(item.language())?,
            );
            item_obj.insert(
                "publisher".to_string(),
                serde_json::to_value(item.publisher())?,
            );
            item_obj.insert(
                "description".to_string(),
                serde_json::to_value(item.book_info.description.clone())?,
            );
            item_obj.insert("rating".to_string(), serde_json::to_value(item.rating())?);
            item_obj.insert(
                "review_note".to_string(),
                serde_json::to_value(item.review_note().cloned())?,
            );
            item_obj.insert(
                "status".to_string(),
                serde_json::Value::String(item.status().to_string()),
            );
            item_obj.insert(
                "progress_percentage".to_string(),
                serde_json::to_value(item.progress_percentage())?,
            );
            item_obj.insert(
                "subjects".to_string(),
                serde_json::to_value(item.subjects())?,
            );
            item_obj.insert(
                "identifiers".to_string(),
                serde_json::to_value(
                    item.identifiers()
                        .iter()
                        .map(|id| {
                            serde_json::json!({
                                "scheme": id.scheme,
                                "value": id.value,
                                "display_scheme": id.display_scheme(),
                                "url": id.url()
                            })
                        })
                        .collect::<Vec<_>>(),
                )?,
            );

            let item_key = match content_type {
                ContentType::Book => "book",
                ContentType::Comic => "comic",
            };
            root.insert(item_key.to_string(), serde_json::Value::Object(item_obj));

            let annotations = item
                .koreader_metadata
                .as_ref()
                .map(|m| m.annotations.clone())
                .unwrap_or_default();
            let bookmarks = item
                .koreader_metadata
                .as_ref()
                .map(|m| {
                    m.annotations
                        .iter()
                        .filter(|a| a.is_bookmark())
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            root.insert(
                "annotations".to_string(),
                serde_json::to_value(annotations)?,
            );
            root.insert("bookmarks".to_string(), serde_json::to_value(bookmarks)?);

            root.insert(
                "statistics".to_string(),
                serde_json::json!({
                    "item_stats": item_stats,
                    "session_stats": session_stats,
                    "completions": item_stats.as_ref().and_then(|s| s.completions.as_ref()),
                }),
            );
            root.insert(
                "export_info".to_string(),
                serde_json::json!({
                    "generated_by": "KoShelf",
                    "version": self.get_version(),
                    "generated_at": self.get_last_updated(),
                }),
            );

            let json_str = serde_json::to_string_pretty(&serde_json::Value::Object(root))?;
            let json_path = item_dir.join("details.json");
            self.cache_manifest
                .register_file(&json_path, &self.output_dir, json_str.as_bytes());
            fs::write(json_path, json_str)?;
        }

        Ok(())
    }

    /// Clean up book directories for books that no longer exist in the library
    pub(crate) fn cleanup_stale_books(&self, books: &[LibraryItem]) -> Result<()> {
        self.cleanup_stale_content_dirs(self.books_dir(), books, "book")
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
        self.cleanup_stale_content_dirs(self.comics_dir(), comics, "comic")
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
                && !current_ids.contains(dir_name) {
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
        recap_latest_href: Option<String>,
        nav: NavContext,
    ) -> Result<()> {
        self.generate_content_list(ContentType::Book, books, true, recap_latest_href, nav)
            .await
    }

    pub(crate) async fn generate_comic_list(
        &self,
        comics: &[LibraryItem],
        render_to_root: bool,
        recap_latest_href: Option<String>,
        nav: NavContext,
    ) -> Result<()> {
        self.generate_content_list(
            ContentType::Comic,
            comics,
            render_to_root,
            recap_latest_href,
            nav,
        )
        .await
    }

    pub(crate) async fn generate_book_pages(
        &self,
        books: &[LibraryItem],
        stats_data: &mut Option<StatisticsData>,
        recap_latest_href: Option<String>,
        nav: NavContext,
    ) -> Result<()> {
        self.generate_content_pages(ContentType::Book, books, stats_data, recap_latest_href, nav)
            .await
    }

    pub(crate) async fn generate_comic_pages(
        &self,
        comics: &[LibraryItem],
        stats_data: &mut Option<StatisticsData>,
        recap_latest_href: Option<String>,
        nav: NavContext,
    ) -> Result<()> {
        self.generate_content_pages(
            ContentType::Comic,
            comics,
            stats_data,
            recap_latest_href,
            nav,
        )
        .await
    }
}
