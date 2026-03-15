//! Response types for `/api/reading/*` endpoints.

use serde::Serialize;
use std::collections::BTreeMap;

use super::library::LibraryContentType;

// ── GET /api/reading/summary ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ReadingSummaryData {
    pub range: ResolvedRange,
    pub overview: ReadingOverview,
    pub streaks: ReadingStreaks,
    pub heatmap_config: HeatmapConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedRange {
    pub from: String,
    pub to: String,
    pub tz: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadingOverview {
    pub reading_time_sec: i64,
    pub pages_read: i64,
    pub sessions: i64,
    pub completions: i64,
    pub items_completed: i64,
    pub longest_reading_time_in_day_sec: i64,
    pub most_pages_in_day: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_session_duration_sec: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longest_session_duration_sec: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadingStreaks {
    pub current: StreakData,
    pub longest: StreakData,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreakData {
    pub days: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeatmapConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_scale_sec: Option<i64>,
}

// ── GET /api/reading/metrics ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ReadingMetricsData {
    pub metrics: Vec<String>,
    pub group_by: String,
    pub scope: String,
    pub points: Vec<MetricPoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricPoint {
    pub key: String,
    #[serde(flatten)]
    pub values: BTreeMap<String, i64>,
}

// ── GET /api/reading/available-periods ────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ReadingAvailablePeriodsData {
    pub source: String,
    pub group_by: String,
    pub periods: Vec<PeriodEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PeriodEntry {
    pub key: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading_time_sec: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages_read: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completions: Option<i64>,
}

// ── GET /api/reading/calendar ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ReadingCalendarData {
    pub month: String,
    pub events: Vec<ReadingCalendarEvent>,
    pub items: BTreeMap<String, CalendarItemRef>,
    pub stats_by_scope: CalendarStatsByScope,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadingCalendarEvent {
    pub item_ref: String,
    pub start: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    pub reading_time_sec: i64,
    pub pages_read: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalendarItemRef {
    pub title: String,
    pub authors: Vec<String>,
    pub content_type: LibraryContentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_cover: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalendarStatsByScope {
    pub all: CalendarScopeStats,
    pub books: CalendarScopeStats,
    pub comics: CalendarScopeStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalendarScopeStats {
    pub items_read: usize,
    pub pages_read: i64,
    pub reading_time_sec: i64,
    pub active_days_percentage: u8,
}

// ── GET /api/reading/completions ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ReadingCompletionsData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<CompletionGroup>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<CompletionItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<CompletionsSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_assets: Option<CompletionsShareAssets>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletionGroup {
    pub key: String,
    pub items_finished: usize,
    pub reading_time_sec: i64,
    pub items: Vec<CompletionItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletionItem {
    pub title: String,
    pub authors: Vec<String>,
    pub start_date: String,
    pub end_date: String,
    pub reading_time_sec: i64,
    pub session_count: i64,
    pub pages_read: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calendar_length_days: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_session_duration_sec: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_cover: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<LibraryContentType>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletionsSummary {
    pub total_items: usize,
    pub total_reading_time_sec: i64,
    pub longest_session_duration_sec: i64,
    pub average_session_duration_sec: i64,
    pub active_days: usize,
    pub active_days_percentage: u8,
    pub longest_streak_days: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_month: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletionsShareAssets {
    pub story_url: String,
    pub square_url: String,
    pub banner_url: String,
}
