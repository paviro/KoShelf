use serde::{Deserialize, Serialize};

use super::common::{ApiMeta, ContentTypeFilter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableWeek {
    pub week_key: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityOverview {
    pub total_read_time: i64,
    pub total_page_reads: i64,
    pub longest_read_time_in_day: i64,
    pub most_pages_in_day: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_session_duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longest_session_duration: Option<i64>,
    pub total_completions: i64,
    pub items_completed: i64,
    pub most_completions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStreaks {
    pub longest: crate::models::StreakInfo,
    pub current: crate::models::StreakInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityHeatmapConfig {
    pub max_scale_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityWeeksResponse {
    pub meta: ApiMeta,
    pub content_type: ContentTypeFilter,
    pub available_years: Vec<i32>,
    pub available_weeks: Vec<AvailableWeek>,
    pub overview: ActivityOverview,
    pub streaks: ActivityStreaks,
    pub heatmap_config: ActivityHeatmapConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityWeekResponse {
    pub meta: ApiMeta,
    pub content_type: ContentTypeFilter,
    pub week_key: String,
    #[serde(flatten)]
    pub stats: crate::models::WeeklyStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlySummary {
    pub completed_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyAggregate {
    pub month_key: String,
    pub read_time: i64,
    pub pages_read: i64,
    pub active_days: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityYearDailyResponse {
    pub meta: ApiMeta,
    pub content_type: ContentTypeFilter,
    pub year: i32,
    pub daily_activity: Vec<crate::models::DailyStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ActivityHeatmapConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityYearSummaryResponse {
    pub meta: ApiMeta,
    pub content_type: ContentTypeFilter,
    pub year: i32,
    pub summary: YearlySummary,
    pub monthly_aggregates: Vec<MonthlyAggregate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ActivityHeatmapConfig>,
}
