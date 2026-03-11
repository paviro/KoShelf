use crate::contracts::calendar::{ActivityMonthResponse, ActivityMonthsResponse};
use crate::contracts::common::ContentTypeFilter;
use crate::contracts::recap::{CompletionYearResponse, CompletionYearsResponse};
use crate::contracts::statistics::{
    ActivityWeekResponse, ActivityWeeksResponse, ActivityYearDailyResponse,
    ActivityYearSummaryResponse,
};
use crate::domain::reading::{calendar, completions, metrics};
use crate::runtime::ContractSnapshot;

#[derive(Debug, Default, Clone, Copy)]
pub struct ReadingService;

impl ReadingService {
    pub fn activity_weeks(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
    ) -> ActivityWeeksResponse {
        metrics::activity_weeks(snapshot, content_type)
    }

    pub fn activity_week(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        week_key: &str,
    ) -> ActivityWeekResponse {
        metrics::activity_week(snapshot, content_type, week_key)
    }

    pub fn activity_year_daily(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        year_key: &str,
        year_value: i32,
    ) -> ActivityYearDailyResponse {
        metrics::activity_year_daily(snapshot, content_type, year_key, year_value)
    }

    pub fn activity_year_summary(
        snapshot: &ContractSnapshot,
        content_type: ContentTypeFilter,
        year_key: &str,
        year_value: i32,
    ) -> ActivityYearSummaryResponse {
        metrics::activity_year_summary(snapshot, content_type, year_key, year_value)
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
}
