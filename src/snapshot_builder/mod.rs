//! Snapshot builder module - orchestrates runtime snapshots and static exports.
//!
//! This module is split into submodules for maintainability:
//! - `assets`: Static shell export (static mode), core snapshot payloads, and cover generation
//! - `library`: Book/comic detail payloads
//! - `statistics`: Statistics payloads
//! - `calendar`: Calendar payloads
//! - `recap`: Yearly recap payloads + share images
//! - `utils`: Miscellaneous helpers (time formatting, version metadata)

mod assets;
mod calendar;
mod library;
mod recap;
mod scaling;
mod statistics;
mod utils;

use crate::config::SiteConfig;
use crate::koreader::{StatisticsCalculator, StatisticsParser, calculate_partial_md5};
use crate::library::scan_library;
use crate::models::{BookStatus, ContentType, LibraryItem, StatisticsData};
use crate::runtime::ContractSnapshot;
use anyhow::Result;
use log::info;
use scaling::PageScaling;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug)]
struct SnapshotInputs {
    all_items: Vec<LibraryItem>,
    books: Vec<LibraryItem>,
    comics: Vec<LibraryItem>,
    stats_data: Option<StatisticsData>,
}

pub struct SnapshotBuilder {
    config: SiteConfig,
}

impl std::ops::Deref for SnapshotBuilder {
    type Target = SiteConfig;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl SnapshotBuilder {
    pub fn new(config: SiteConfig) -> Self {
        Self { config }
    }

    pub(crate) fn assets_dir(&self) -> PathBuf {
        if self.is_internal_server {
            self.output_dir.clone()
        } else {
            self.output_dir.join("assets")
        }
    }
    pub(crate) fn covers_dir(&self) -> PathBuf {
        self.assets_dir().join("covers")
    }
    pub(crate) fn recap_dir(&self) -> PathBuf {
        self.assets_dir().join("recap")
    }

    async fn collect_snapshot_inputs(&self) -> Result<SnapshotInputs> {
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
                    md5_to_content_type.insert(md5.to_lowercase(), item.content_type());
                } else {
                    log::debug!(
                        "Could not determine MD5 for {:?}; stats content_type tagging may be incomplete",
                        item.file_path
                    );
                }
            }
            sd.tag_content_types(&md5_to_content_type);
        }

        Ok(SnapshotInputs {
            all_items,
            books,
            comics,
            stats_data,
        })
    }

    pub async fn refresh_snapshot(&self) -> Result<ContractSnapshot> {
        if self.is_internal_server {
            info!(
                "Refreshing runtime data snapshot and serving media cache from: {:?}",
                self.output_dir
            );
        } else {
            info!(
                "Exporting static shell/assets and /data payloads to: {:?}",
                self.output_dir
            );
        }
        let mut ctx = self.collect_snapshot_inputs().await?;
        let page_scaling = PageScaling::from_inputs(
            self.use_stable_page_metadata,
            &ctx.all_items,
            ctx.stats_data.as_ref(),
        );
        let mut snapshot = ContractSnapshot::default();

        // Create output directories for media and static assets.
        self.create_directories().await?;

        // Compute core payloads and, in static mode, emit embedded frontend files.
        self.compute_core_snapshot_data(&ctx.all_items, &ctx.stats_data, &mut snapshot)?;

        // Generate covers for all items (books and comics)
        self.generate_covers(&ctx.all_items).await?;

        // Clean up stale covers (for deleted items)
        self.cleanup_stale_covers(&ctx.all_items)?;

        // Compute detail payloads for books and comics.
        self.compute_content_detail_data(
            ContentType::Book,
            &ctx.books,
            &mut ctx.stats_data,
            &page_scaling,
            &mut snapshot,
        )?;
        self.compute_content_detail_data(
            ContentType::Comic,
            &ctx.comics,
            &mut ctx.stats_data,
            &page_scaling,
            &mut snapshot,
        )?;

        if let Some(ref mut stats_data) = ctx.stats_data {
            self.compute_statistics_data(stats_data, &page_scaling, &mut snapshot)?;

            self.compute_calendar_data(stats_data, &ctx.all_items, &page_scaling, &mut snapshot)?;

            self.compute_recap_data_and_share_images(
                stats_data,
                &ctx.all_items,
                &page_scaling,
                &mut snapshot,
            )
            .await?;
        }

        if self.is_internal_server {
            info!("Runtime data snapshot refresh completed.");
        } else {
            info!("Static shell/assets and /data export completed.");
        }

        Ok(snapshot)
    }
}
