use crate::contracts::calendar::{ActivityMonthResponse, ActivityMonthsResponse};
use crate::contracts::common::ContentTypeFilter;
use crate::contracts::reading::{
    ReadingAvailablePeriodsData, ReadingCalendarData, ReadingCompletionsData, ReadingMetricsData,
    ReadingSummaryData,
};
use crate::contracts::recap::{CompletionYearResponse, CompletionYearsResponse};
use crate::contracts::statistics::{
    ActivityWeekResponse, ActivityWeeksResponse, ActivityYearDailyResponse,
    ActivityYearSummaryResponse,
};
use crate::domain::reading::queries::{
    ReadingAvailablePeriodsQuery, ReadingCalendarQuery, ReadingCompletionsQuery,
    ReadingMetricsQuery, ReadingSummaryQuery,
};
use crate::domain::reading::{
    activity, available_periods, calendar, completions, metrics, summary,
};
use crate::runtime::{ContractSnapshot, ReadingData};

#[derive(Debug, Default, Clone, Copy)]
pub struct ReadingService;

impl ReadingService {
    // ── Legacy snapshot-based methods ────────────────────────────────────

    pub fn activity_weeks(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
    ) -> ActivityWeeksResponse {
        activity::activity_weeks(snapshot, content_type)
    }

    pub fn activity_week(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        week_key: &str,
    ) -> ActivityWeekResponse {
        activity::activity_week(snapshot, content_type, week_key)
    }

    pub fn activity_year_daily(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        year_key: &str,
        year_value: i32,
    ) -> ActivityYearDailyResponse {
        activity::activity_year_daily(snapshot, content_type, year_key, year_value)
    }

    pub fn activity_year_summary(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        year_key: &str,
        year_value: i32,
    ) -> ActivityYearSummaryResponse {
        activity::activity_year_summary(snapshot, content_type, year_key, year_value)
    }

    pub fn activity_months(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
    ) -> ActivityMonthsResponse {
        calendar::activity_months(snapshot, content_type)
    }

    pub fn activity_month(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        month_key: &str,
    ) -> ActivityMonthResponse {
        calendar::activity_month(snapshot, content_type, month_key)
    }

    pub fn completion_years(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
    ) -> CompletionYearsResponse {
        completions::completion_years(snapshot, content_type)
    }

    pub fn completion_year(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        year_key: &str,
        year_value: i32,
    ) -> CompletionYearResponse {
        completions::completion_year(snapshot, content_type, year_key, year_value)
    }

    // ── New on-demand reading endpoints ─────────────────────────────────

    pub fn summary(reading_data: &ReadingData, query: ReadingSummaryQuery) -> ReadingSummaryData {
        summary::summary(reading_data, query)
    }

    pub fn metrics(reading_data: &ReadingData, query: ReadingMetricsQuery) -> ReadingMetricsData {
        metrics::metrics(reading_data, query)
    }

    pub fn available_periods(
        reading_data: &ReadingData,
        query: ReadingAvailablePeriodsQuery,
    ) -> ReadingAvailablePeriodsData {
        available_periods::available_periods(reading_data, query)
    }

    pub fn calendar(
        reading_data: &ReadingData,
        query: ReadingCalendarQuery,
    ) -> ReadingCalendarData {
        calendar::reading_calendar(reading_data, query)
    }

    pub fn completions(
        reading_data: &ReadingData,
        query: ReadingCompletionsQuery,
    ) -> ReadingCompletionsData {
        completions::reading_completions(reading_data, query)
    }
}
