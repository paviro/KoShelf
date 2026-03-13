//! Shared library ingest: scans the library and loads statistics in a single pass.
//!
//! Both the library DB pipeline and media asset generation (covers, recap images)
//! consume the same scan results.  This module eliminates the previous double-scan
//! that occurred when the snapshot builder and library build pipeline each called
//! `scan_library()` independently.

use crate::config::SiteConfig;
use crate::koreader::{StatisticsCalculator, StatisticsParser};
use crate::library::scan_library;
use crate::models::merge_precedence::normalize_partial_md5;
use crate::models::{BookStatus, ContentType, LibraryItem, StatisticsData};
use crate::runtime::ReadingData;
use anyhow::Result;
use log::info;
use std::collections::{HashMap, HashSet};

/// Output of the shared ingest step.
pub struct IngestResult {
    /// All items returned by the library scanner (unfiltered).
    pub raw_items: Vec<LibraryItem>,
    /// Items after applying `include_unread` filtering.
    pub filtered_items: Vec<LibraryItem>,
    /// MD5 hashes from the raw scan (used for stats filtering).
    pub library_md5s: HashSet<String>,
    /// Processed statistics data (filtered, tagged with content types, completions populated).
    pub stats_data: Option<StatisticsData>,
}

impl IngestResult {
    /// Package the processed statistics into a `ReadingData` suitable for domain services.
    pub fn reading_data(&self, config: &SiteConfig) -> Option<ReadingData> {
        self.stats_data.as_ref().map(|sd| {
            let covers_by_md5 = build_covers_by_md5(self.filtered_items.iter().map(|i| &i.id));
            ReadingData {
                stats_data: sd.clone(),
                time_config: config.time_config.clone(),
                heatmap_scale_max: config.heatmap_scale_max,
                covers_by_md5,
            }
        })
    }

    /// Whether the library contains reading data (statistics with page stats).
    pub fn has_reading_data(&self) -> bool {
        self.stats_data
            .as_ref()
            .is_some_and(|sd| !sd.page_stats.is_empty())
    }
}

/// Scan the library and load statistics in one pass.
///
/// Returns both raw and filtered items so that:
/// - The library DB pipeline receives raw items (it does its own filtering).
/// - Cover generation, recap images, and site metadata use filtered items.
pub async fn ingest(config: &SiteConfig) -> Result<IngestResult> {
    let (raw_items, library_md5s) = if !config.library_paths.is_empty() {
        scan_library(&config.library_paths, &config.metadata_location).await?
    } else {
        (Vec::new(), HashSet::new())
    };

    // Filter items based on include_unread setting.
    let filtered_items: Vec<LibraryItem> = raw_items
        .iter()
        .filter(|item| {
            if item.koreader_metadata.is_some() {
                return true;
            }
            config.include_unread && item.status() == BookStatus::Unknown
        })
        .cloned()
        .collect();

    // Load statistics if path is provided.
    let mut stats_data = if let Some(ref stats_path) = config.statistics_db_path {
        if stats_path.exists() {
            let mut data = StatisticsParser::parse(stats_path).await?;

            if config.min_pages_per_day.is_some() || config.min_time_per_day.is_some() {
                StatisticsCalculator::filter_stats(
                    &mut data,
                    &config.time_config,
                    config.min_pages_per_day,
                    config.min_time_per_day,
                );
            }

            if !config.include_all_stats && !raw_items.is_empty() {
                StatisticsCalculator::filter_to_library(&mut data, &library_md5s);
            }

            StatisticsCalculator::populate_completions(&mut data, &config.time_config);
            info!(
                "Statistics: {} books, {} with completions",
                data.books.len(),
                data.books
                    .iter()
                    .filter(|b| b.completions.is_some())
                    .count()
            );
            Some(data)
        } else {
            info!("Statistics database not found: {:?}", stats_path);
            None
        }
    } else {
        None
    };

    // Tag statistics entries by content type using MD5 → LibraryItem lookup.
    if let Some(ref mut sd) = stats_data {
        let mut md5_to_content_type: HashMap<String, ContentType> = HashMap::new();
        for item in &filtered_items {
            if let Some(canonical_md5) = normalize_partial_md5(&item.id) {
                md5_to_content_type.insert(canonical_md5, item.content_type());
            } else {
                log::debug!(
                    "Item {:?} has non-canonical id '{}'; stats content_type tagging may be incomplete",
                    item.file_path,
                    item.id
                );
            }
        }
        sd.tag_content_types(&md5_to_content_type);
    }

    Ok(IngestResult {
        raw_items,
        filtered_items,
        library_md5s,
        stats_data,
    })
}

/// Build an md5 → cover URL map from item IDs.
pub fn build_covers_by_md5<'a>(ids: impl Iterator<Item = &'a String>) -> HashMap<String, String> {
    ids.map(|id| (id.clone(), format!("/assets/covers/{}.webp", id)))
        .collect()
}
