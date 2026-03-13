//! Processed reading data available for on-demand query computation.

use std::collections::HashMap;

use crate::models::StatisticsData;
use crate::time_config::TimeConfig;

/// Bundle of statistics data and time configuration used by reading
/// and library domain services for on-demand query computation.
#[derive(Debug, Clone)]
pub struct ReadingData {
    pub stats_data: StatisticsData,
    pub time_config: TimeConfig,
    pub heatmap_scale_max: Option<u32>,
    /// MD5 → cover URL for library items (e.g. `/assets/covers/{md5}.webp`).
    pub covers_by_md5: HashMap<String, String>,
}
