//! Book list and detail page generation.

use super::SiteGenerator;
use crate::models::{Book, BookStatus, StatisticsData};
use crate::statistics::BookStatistics;
use crate::templates::{IndexTemplate, BookTemplate, BookMarkdownTemplate};
use anyhow::Result;
use askama::Template;
use log::{info, warn};
use std::fs;
use std::collections::HashSet;

impl SiteGenerator {
    pub(crate) async fn generate_book_list(&self, books: &[Book], recap_latest_href: Option<String>) -> Result<()> {
        info!("Generating book list page...");
        
        let mut reading_books = Vec::new();
        let mut completed_books = Vec::new();
        let mut abandoned_books = Vec::new();
        let mut unread_books = Vec::new();
        
        for book in books {
            match book.status() {
                BookStatus::Reading => reading_books.push(book.clone()),
                BookStatus::Complete => completed_books.push(book.clone()),
                BookStatus::Abandoned => abandoned_books.push(book.clone()),
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
        reading_books.sort_by(|a, b| a.book_info.title.cmp(&b.book_info.title));
        completed_books.sort_by(|a, b| a.book_info.title.cmp(&b.book_info.title));
        abandoned_books.sort_by(|a, b| a.book_info.title.cmp(&b.book_info.title));
        unread_books.sort_by(|a, b| a.book_info.title.cmp(&b.book_info.title));
        
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
                "title": b.book_info.title.clone(),
                "authors": b.book_info.authors.clone(),
                "json_path": format!("/books/{}/details.json", b.id),
                "html_path":  format!("/books/{}/index.html",  b.id),
            })
        };

        let reading_json: Vec<_> = reading_books.iter().map(to_manifest_entry).collect();
        let completed_json: Vec<_> = completed_books.iter().map(to_manifest_entry).collect();
        let abandoned_json: Vec<_> = abandoned_books.iter().map(to_manifest_entry).collect();
        let new_json: Vec<_> = unread_books.iter().map(to_manifest_entry).collect();

        let manifest = json!({
            "reading": reading_json,
            "completed": completed_json,
            "abandoned": abandoned_json,
            "new": new_json,
            "generated_at": self.get_last_updated(),
        });

        let list_json_path = self.books_dir().join("list.json");
        let list_json_content = serde_json::to_string_pretty(&manifest)?;
        self.cache_manifest.register_file(&list_json_path, &self.output_dir, list_json_content.as_bytes());
        fs::write(list_json_path, list_json_content)?;

        // ------------------------------------------------------------------
        // Render book list HTML
        // ------------------------------------------------------------------

        let template = IndexTemplate {
            site_title: self.site_title.clone(),
            reading_books,
            completed_books,
            abandoned_books,
            unread_books,
            version: self.get_version(),
            last_updated: self.get_last_updated(),
            navbar_items: self.create_navbar_items_with_recap("books", recap_latest_href.as_deref()),
            translation: self.t(),
        };

        let html = template.render()?;
        self.write_minify_html(self.output_dir.join("index.html"), &html)?;
        
        Ok(())
    }
    
    pub(crate) async fn generate_book_pages(&self, books: &[Book], stats_data: &mut Option<StatisticsData>, recap_latest_href: Option<String>) -> Result<()> {
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
                (Some(stats), Some(book_stat)) => Some(book_stat.calculate_session_stats(&stats.page_stats, &self.time_config, &self.translations)),
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
                translation: self.t(),
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
            let md_path = book_dir.join("details.md");
            self.cache_manifest.register_file(&md_path, &self.output_dir, markdown.as_bytes());
            fs::write(md_path, markdown)?;

            // Generate JSON export / not used by the frontend code - only for the user's convenience
            let json_data = serde_json::json!({
                "book": {
                    "title": book.book_info.title,
                    "authors": book.book_info.authors,
                    "series": book.series_display(),
                    "language": book.language(),
                    "publisher": book.publisher(),
                    "description": book.book_info.description,
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
            let json_path = book_dir.join("details.json");
            self.cache_manifest.register_file(&json_path, &self.output_dir, json_str.as_bytes());
            fs::write(json_path, json_str)?;
        }
        
        Ok(())
    }

    /// Clean up book directories for books that no longer exist in the library
    pub(crate) fn cleanup_stale_books(&self, books: &[Book]) -> Result<()> {
        let books_dir = self.books_dir();
        
        // If books directory doesn't exist, nothing to clean up
        if !books_dir.exists() {
            return Ok(());
        }

        // Build set of current book IDs
        let current_ids: HashSet<String> = books.iter().map(|b| b.id.clone()).collect();
        
        // Iterate over existing book directories
        let entries = fs::read_dir(&books_dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            
            // Skip if not a directory or if it's list.json
            if !path.is_dir() {
                continue;
            }
            
            // Get directory name (book ID)
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                // If this book ID is not in current books, remove the directory
                if !current_ids.contains(dir_name) {
                    info!("Removing stale book directory: {:?}", path);
                    if let Err(e) = fs::remove_dir_all(&path) {
                        warn!("Failed to remove stale book directory {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(())
    }
}
