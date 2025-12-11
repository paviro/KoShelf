//! Site configuration module - bundles generator/watcher configuration.

use crate::book_scanner::MetadataLocation;
use crate::time_config::TimeConfig;
use std::path::PathBuf;

/// Configuration for site generation and file watching.
#[derive(Clone)]
pub struct SiteConfig {
    /// Output directory for the generated site
    pub output_dir: PathBuf,
    /// Title for the generated site
    pub site_title: String,
    /// Whether to include unread books
    pub include_unread: bool,
    /// Path to the books directory (optional)
    pub books_path: Option<PathBuf>,
    /// Where to look for KoReader metadata
    pub metadata_location: MetadataLocation,
    /// Path to the statistics database (optional)
    pub statistics_db_path: Option<PathBuf>,
    /// Maximum value for heatmap scale (optional)
    pub heatmap_scale_max: Option<u32>,
    /// Time zone configuration
    pub time_config: TimeConfig,
    /// Minimum pages per day for statistics filtering (optional)
    pub min_pages_per_day: Option<u32>,
    /// Minimum time per day in seconds for statistics filtering (optional)
    pub min_time_per_day: Option<u32>,
    /// Whether to include all stats or filter to library books only
    pub include_all_stats: bool,
    /// Whether running with internal web server (enables long-polling)
    pub is_internal_server: bool,
    /// Language for UI translations (e.g., "en_US", "de_DE")
    pub language: String,
}
