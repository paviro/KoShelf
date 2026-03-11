use crate::contracts::common::{ApiMeta, ContentTypeFilter};
use crate::contracts::recap::{
    CompletionYearResponse, CompletionYearsResponse, RecapSummaryResponse,
};
use crate::domain::meta::fallback_meta;
use crate::runtime::ContractSnapshot;

pub fn completion_years(
    snapshot: &ContractSnapshot,
    content_type: ContentTypeFilter,
) -> CompletionYearsResponse {
    snapshot
        .completion_years
        .get(content_type.as_str())
        .cloned()
        .unwrap_or_else(|| empty_completion_years_response(fallback_meta(snapshot), content_type))
}

pub fn completion_year(
    snapshot: &ContractSnapshot,
    content_type: ContentTypeFilter,
    year_key: &str,
    year_value: i32,
) -> CompletionYearResponse {
    let meta = snapshot
        .completion_years
        .get(content_type.as_str())
        .map(|years| years.meta.clone())
        .unwrap_or_else(|| fallback_meta(snapshot));

    snapshot
        .completion_years_by_key
        .get(content_type.as_str())
        .and_then(|years| years.get(year_key))
        .cloned()
        .unwrap_or_else(|| empty_completion_year_response(meta, content_type, year_value))
}

fn empty_recap_summary_response() -> RecapSummaryResponse {
    RecapSummaryResponse {
        total_items: 0,
        total_time_seconds: 0,
        total_time_days: 0,
        total_time_hours: 0,
        longest_session_hours: 0,
        longest_session_minutes: 0,
        average_session_hours: 0,
        average_session_minutes: 0,
        active_days: 0,
        active_days_percentage: 0.0,
        longest_streak: 0,
        best_month: None,
    }
}

fn empty_completion_years_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
) -> CompletionYearsResponse {
    CompletionYearsResponse {
        meta,
        content_type,
        available_years: vec![],
        latest_year: None,
    }
}

fn empty_completion_year_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    year: i32,
) -> CompletionYearResponse {
    CompletionYearResponse {
        meta,
        content_type,
        year,
        summary: empty_recap_summary_response(),
        months: vec![],
        items: vec![],
        share_assets: None,
    }
}
