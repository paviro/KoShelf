//! Snapshot builder module - orchestrates runtime snapshots and static exports.
//!
//! Submodules:
//! - `assets`: Static shell export (static mode), site payload, and cover generation
//! - `library`: Stale cover cleanup
//! - `recap`: Recap share image generation
//! - `scaling`: Synthetic stable-page scaling helpers
//! - `utils`: Miscellaneous helpers (time formatting, version metadata)

mod assets;
mod library;
mod recap;
mod scaling;
mod utils;

use crate::config::SiteConfig;
use crate::koreader::{StatisticsCalculator, StatisticsParser};
use crate::library::scan_library;
use crate::models::merge_precedence::normalize_partial_md5;
use crate::models::{BookStatus, ContentType, LibraryItem, StatisticsData};
use crate::runtime::{ContractSnapshot, ReadingData};
use anyhow::Result;
use log::info;
use scaling::PageScaling;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Output of a snapshot refresh: the pre-computed snapshot plus optional
/// processed reading data for on-demand query computation.
pub struct SnapshotResult {
    pub snapshot: ContractSnapshot,
    pub reading_data: Option<ReadingData>,
}

#[derive(Debug)]
struct SnapshotInputs {
    all_items: Vec<LibraryItem>,
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
        let (all_items, library_md5s) = if !self.library_paths.is_empty() {
            scan_library(&self.library_paths, &self.metadata_location).await?
        } else {
            (Vec::new(), HashSet::new())
        };

        // Filter items based on include_unread setting
        let all_items: Vec<_> = all_items
            .into_iter()
            .filter(|item| {
                if item.koreader_metadata.is_some() {
                    return true;
                }
                self.include_unread && item.status() == BookStatus::Unknown
            })
            .collect();

        // Load statistics if path is provided
        let mut stats_data = if let Some(ref stats_path) = self.statistics_db_path {
            if stats_path.exists() {
                let mut data = StatisticsParser::parse(stats_path).await?;

                if self.min_pages_per_day.is_some() || self.min_time_per_day.is_some() {
                    StatisticsCalculator::filter_stats(
                        &mut data,
                        &self.time_config,
                        self.min_pages_per_day,
                        self.min_time_per_day,
                    );
                }

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

        Ok(SnapshotInputs {
            all_items,
            stats_data,
        })
    }

    pub async fn refresh_snapshot(&self) -> Result<SnapshotResult> {
        if self.is_internal_server {
            info!(
                "Refreshing runtime data snapshot and serving media cache from: {:?}",
                self.output_dir
            );
        } else {
            info!("Exporting static shell/assets to: {:?}", self.output_dir);
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

        // Compute site payload and, in static mode, emit embedded frontend files.
        self.compute_core_snapshot_data(&ctx.all_items, &ctx.stats_data, &mut snapshot)?;

        // Generate covers for all items (books and comics).
        self.generate_covers(&ctx.all_items).await?;

        // Clean up stale covers (for deleted items).
        self.cleanup_stale_covers(&ctx.all_items)?;

        // Generate recap share images for each completion year.
        if let Some(ref mut stats_data) = ctx.stats_data {
            self.generate_recap_share_images(stats_data, &ctx.all_items, &page_scaling)
                .await?;
        }

        // Preserve processed stats data for on-demand reading endpoint computation.
        let reading_data = ctx.stats_data.map(|sd| ReadingData {
            stats_data: sd,
            time_config: self.time_config.clone(),
            heatmap_scale_max: self.heatmap_scale_max,
        });

        if self.is_internal_server {
            info!("Runtime data snapshot refresh completed.");
        } else {
            info!("Static shell/assets export completed.");
        }

        Ok(SnapshotResult {
            snapshot,
            reading_data,
        })
    }
}
