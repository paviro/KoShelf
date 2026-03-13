//! Processed reading data available for on-demand query computation.

use crate::models::StatisticsData;
use crate::time_config::TimeConfig;

/// Bundle of statistics data and time configuration used by reading
/// and library domain services for on-demand query computation.
#[derive(Debug, Clone)]
pub struct ReadingData {
    pub stats_data: StatisticsData,
    pub time_config: TimeConfig,
    pub heatmap_scale_max: Option<u32>,
}
