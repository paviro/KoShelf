//! Book/comic detail payload computation.

use super::SnapshotBuilder;
use crate::contracts::mappers;
use crate::koreader::BookStatistics;
use crate::models::{ContentType, LibraryItem, StatisticsData};
use crate::runtime::ContractSnapshot;
use anyhow::Result;
use log::{info, warn};
use std::collections::HashSet;
use std::fs;

impl SnapshotBuilder {
    fn content_slug(content_type: ContentType) -> &'static str {
        match content_type {
            ContentType::Book => "books",
            ContentType::Comic => "comics",
        }
    }

    pub(crate) fn compute_content_detail_data(
        &self,
        content_type: ContentType,
        items: &[LibraryItem],
        stats_data: &mut Option<StatisticsData>,
        snapshot: &mut ContractSnapshot,
    ) -> Result<()> {
        info!(
            "Computing {} detail data...",
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
                ContentType::Book => "/books".to_string(),
                ContentType::Comic => "/comics".to_string(),
            };

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
}
