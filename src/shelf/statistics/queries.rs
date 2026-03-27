//! Validated query and filter types for `/api/reading/*` endpoints.

use chrono::NaiveDate;
use chrono_tz::Tz;
use std::collections::HashSet;

use crate::server::api::responses::common::ContentTypeFilter;
use crate::server::api::responses::error::ApiErrorCode;

// ── Scope ─────────────────────────────────────────────────────────────────

/// Reading scope — reuses `ContentTypeFilter` directly.
pub type ReadingScope = ContentTypeFilter;

// ── Date range ────────────────────────────────────────────────────────────

/// A validated, inclusive `[from, to]` date range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl DateRange {
    /// Construct after validation that `from <= to`.
    pub fn new(from: NaiveDate, to: NaiveDate) -> Result<Self, (ApiErrorCode, String)> {
        if from > to {
            return Err((
                ApiErrorCode::InvalidQuery,
                "'from' must be less than or equal to 'to'".to_string(),
            ));
        }
        Ok(Self { from, to })
    }

    pub fn from_str(from: &str, to: &str) -> Result<Self, (ApiErrorCode, String)> {
        let from_date = NaiveDate::parse_from_str(from, "%Y-%m-%d").map_err(|_| {
            (
                ApiErrorCode::InvalidQuery,
                "'from' must be a valid date in YYYY-MM-DD format".to_string(),
            )
        })?;
        let to_date = NaiveDate::parse_from_str(to, "%Y-%m-%d").map_err(|_| {
            (
                ApiErrorCode::InvalidQuery,
                "'to' must be a valid date in YYYY-MM-DD format".to_string(),
            )
        })?;
        Self::new(from_date, to_date)
    }
}

// ── Timezone override ─────────────────────────────────────────────────────

/// Parse an optional IANA timezone string into a `chrono_tz::Tz`.
pub fn parse_timezone(value: Option<&str>) -> Result<Option<Tz>, (ApiErrorCode, String)> {
    match value {
        None | Some("") => Ok(None),
        Some(tz_str) => tz_str.parse::<Tz>().map(Some).map_err(|_| {
            (
                ApiErrorCode::InvalidQuery,
                format!(
                    "'tz' must be a valid IANA timezone (e.g. 'America/New_York'), got '{tz_str}'"
                ),
            )
        }),
    }
}

// ── Reading metric ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadingMetric {
    ReadingTimeSec,
    PagesRead,
    Sessions,
    Completions,
    AverageSessionDurationSec,
    LongestSessionDurationSec,
    ActiveDays,
}

query_enum! {
    pub enum ReadingMetric {
        ReadingTimeSec => "reading_time_sec",
        PagesRead => "pages_read",
        Sessions => "sessions",
        Completions => "completions",
        AverageSessionDurationSec => "average_session_duration_sec",
        LongestSessionDurationSec => "longest_session_duration_sec",
        ActiveDays => "active_days",
    }
    field_name: "metric"
}

// ── Group-by for metrics endpoint ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricsGroupBy {
    Total,
    Day,
    Week,
    Month,
    Year,
}

query_enum! {
    pub enum MetricsGroupBy {
        Total => "total",
        Day => "day",
        Week => "week",
        Month => "month",
        Year => "year",
    }
    field_name: "group_by"
}

// ── Available-periods source ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodSource {
    ReadingData,
    Completions,
}

query_enum! {
    pub enum PeriodSource {
        ReadingData => "reading_data",
        Completions => "completions",
    }
    field_name: "source"
}

// ── Group-by for available-periods endpoint ───────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodGroupBy {
    Week,
    Month,
    Year,
}

query_enum! {
    pub enum PeriodGroupBy {
        Week => "week",
        Month => "month",
        Year => "year",
    }
    field_name: "group_by"
}

/// Validate that a (source, group_by) combination is supported.
///
/// Per contract:
/// - `source=reading_data` supports `group_by=week|month|year`
/// - `source=completions` supports `group_by=month|year`
pub fn validate_period_source_group(
    source: PeriodSource,
    group_by: PeriodGroupBy,
) -> Result<(), (ApiErrorCode, String)> {
    match (source, group_by) {
        (PeriodSource::ReadingData, _) => Ok(()),
        (PeriodSource::Completions, PeriodGroupBy::Month | PeriodGroupBy::Year) => Ok(()),
        (PeriodSource::Completions, PeriodGroupBy::Week) => Err((
            ApiErrorCode::InvalidQuery,
            "source 'completions' does not support group_by 'week'; use 'month' or 'year'"
                .to_string(),
        )),
    }
}

// ── Completions group-by ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompletionsGroupBy {
    None,
    #[default]
    Month,
}

impl CompletionsGroupBy {
    pub fn parse(value: Option<&str>) -> Result<Self, (ApiErrorCode, String)> {
        match value {
            None | Some("month") => Ok(Self::Month),
            Some("none") => Ok(Self::None),
            Some(other) => Err((
                ApiErrorCode::InvalidQuery,
                format!("'group_by' must be one of: none, month; got '{other}'"),
            )),
        }
    }
}

// ── Completions year-or-range selector ────────────────────────────────────

/// Mutually exclusive year vs from/to selector for the completions endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionsSelector {
    /// Explicit year — server expands to `year-01-01..year-12-31`.
    Year(i32),
    /// Explicit date range.
    Range(DateRange),
    /// Neither provided — server defaults to latest completion year.
    Default,
}

// ── Completions include tokens ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompletionsIncludeToken {
    Summary,
    ShareAssets,
}

impl crate::shelf::token_set::SetToken for CompletionsIncludeToken {
    fn parse_token(value: &str) -> Option<Self> {
        match value {
            "summary" => Some(Self::Summary),
            "share_assets" => Some(Self::ShareAssets),
            _ => None,
        }
    }

    fn valid_tokens() -> &'static str {
        "summary, share_assets"
    }
}

pub type CompletionsIncludeSet = crate::shelf::token_set::TokenSet<CompletionsIncludeToken>;

impl CompletionsIncludeSet {
    pub fn has_summary(&self) -> bool {
        self.has(CompletionsIncludeToken::Summary)
    }

    pub fn has_share_assets(&self) -> bool {
        self.has(CompletionsIncludeToken::ShareAssets)
    }
}

// ── Composed query types ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ReadingSummaryQuery {
    pub scope: ReadingScope,
    pub range: Option<DateRange>,
    pub tz: Option<Tz>,
}

#[derive(Debug, Clone)]
pub struct ReadingMetricsQuery {
    pub scope: ReadingScope,
    pub metrics: Vec<ReadingMetric>,
    pub group_by: MetricsGroupBy,
    pub range: Option<DateRange>,
    pub tz: Option<Tz>,
}

/// Parse a comma-separated metric string into a `Vec<ReadingMetric>`.
pub fn parse_metrics(value: &str) -> Result<Vec<ReadingMetric>, (ApiErrorCode, String)> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for raw in value.split(',') {
        let token = raw.trim();
        if token.is_empty() {
            continue;
        }
        let metric = ReadingMetric::parse(token)?;
        if seen.insert(metric) {
            result.push(metric);
        }
    }
    if result.is_empty() {
        return Err((
            ApiErrorCode::InvalidQuery,
            "'metric' must contain at least one valid metric".to_string(),
        ));
    }
    Ok(result)
}

#[derive(Debug, Clone)]
pub struct ReadingAvailablePeriodsQuery {
    pub scope: ReadingScope,
    pub source: PeriodSource,
    pub group_by: PeriodGroupBy,
    pub range: Option<DateRange>,
    pub tz: Option<Tz>,
}

#[derive(Debug, Clone)]
pub struct ReadingCalendarQuery {
    pub month: String,
    pub scope: ReadingScope,
    pub tz: Option<Tz>,
}

#[derive(Debug, Clone)]
pub struct ReadingCompletionsQuery {
    pub scope: ReadingScope,
    pub selector: CompletionsSelector,
    pub group_by: CompletionsGroupBy,
    pub includes: CompletionsIncludeSet,
    pub tz: Option<Tz>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── DateRange ─────────────────────────────────────────────────────

    #[test]
    fn date_range_from_str_accepts_valid_range() {
        let range = DateRange::from_str("2026-01-01", "2026-01-31").unwrap();
        assert_eq!(range.from, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(range.to, NaiveDate::from_ymd_opt(2026, 1, 31).unwrap());
    }

    #[test]
    fn date_range_from_str_accepts_same_day() {
        let range = DateRange::from_str("2026-03-01", "2026-03-01").unwrap();
        assert_eq!(range.from, range.to);
    }

    #[test]
    fn date_range_from_str_rejects_inverted_range() {
        let err = DateRange::from_str("2026-12-31", "2026-01-01").unwrap_err();
        assert_eq!(err.0, ApiErrorCode::InvalidQuery);
    }

    #[test]
    fn date_range_from_str_rejects_bad_date() {
        let err = DateRange::from_str("not-a-date", "2026-01-01").unwrap_err();
        assert_eq!(err.0, ApiErrorCode::InvalidQuery);
    }

    // ── Timezone ──────────────────────────────────────────────────────

    #[test]
    fn parse_timezone_accepts_valid_iana() {
        assert!(parse_timezone(Some("America/New_York")).unwrap().is_some());
        assert!(parse_timezone(Some("UTC")).unwrap().is_some());
    }

    #[test]
    fn parse_timezone_none_and_empty_return_none() {
        assert!(parse_timezone(None).unwrap().is_none());
        assert!(parse_timezone(Some("")).unwrap().is_none());
    }

    #[test]
    fn parse_timezone_rejects_invalid() {
        assert!(parse_timezone(Some("Not/A/Zone")).is_err());
    }

    // ── ReadingMetric ─────────────────────────────────────────────────

    #[test]
    fn reading_metric_parses_all_variants() {
        assert_eq!(
            ReadingMetric::parse("reading_time_sec").unwrap(),
            ReadingMetric::ReadingTimeSec
        );
        assert_eq!(
            ReadingMetric::parse("pages_read").unwrap(),
            ReadingMetric::PagesRead
        );
        assert_eq!(
            ReadingMetric::parse("sessions").unwrap(),
            ReadingMetric::Sessions
        );
        assert_eq!(
            ReadingMetric::parse("completions").unwrap(),
            ReadingMetric::Completions
        );
        assert_eq!(
            ReadingMetric::parse("average_session_duration_sec").unwrap(),
            ReadingMetric::AverageSessionDurationSec
        );
        assert_eq!(
            ReadingMetric::parse("longest_session_duration_sec").unwrap(),
            ReadingMetric::LongestSessionDurationSec
        );
        assert_eq!(
            ReadingMetric::parse("active_days").unwrap(),
            ReadingMetric::ActiveDays
        );
    }

    #[test]
    fn reading_metric_rejects_unknown() {
        assert!(ReadingMetric::parse("unknown").is_err());
    }

    #[test]
    fn reading_metric_round_trips_via_as_str() {
        for name in [
            "reading_time_sec",
            "pages_read",
            "sessions",
            "completions",
            "average_session_duration_sec",
            "longest_session_duration_sec",
            "active_days",
        ] {
            let metric = ReadingMetric::parse(name).unwrap();
            assert_eq!(metric.as_str(), name);
        }
    }

    // ── MetricsGroupBy ────────────────────────────────────────────────

    #[test]
    fn metrics_group_by_parses_valid_values() {
        assert_eq!(
            MetricsGroupBy::parse("total").unwrap(),
            MetricsGroupBy::Total
        );
        assert_eq!(MetricsGroupBy::parse("day").unwrap(), MetricsGroupBy::Day);
        assert_eq!(MetricsGroupBy::parse("week").unwrap(), MetricsGroupBy::Week);
        assert_eq!(
            MetricsGroupBy::parse("month").unwrap(),
            MetricsGroupBy::Month
        );
    }

    #[test]
    fn metrics_group_by_accepts_year() {
        assert_eq!(MetricsGroupBy::parse("year").unwrap(), MetricsGroupBy::Year);
    }

    // ── PeriodSource ──────────────────────────────────────────────────

    #[test]
    fn period_source_parses_valid_values() {
        assert_eq!(
            PeriodSource::parse("reading_data").unwrap(),
            PeriodSource::ReadingData
        );
        assert_eq!(
            PeriodSource::parse("completions").unwrap(),
            PeriodSource::Completions
        );
    }

    #[test]
    fn period_source_rejects_unknown() {
        assert!(PeriodSource::parse("unknown").is_err());
    }

    // ── PeriodGroupBy ─────────────────────────────────────────────────

    #[test]
    fn period_group_by_parses_valid_values() {
        assert_eq!(PeriodGroupBy::parse("week").unwrap(), PeriodGroupBy::Week);
        assert_eq!(PeriodGroupBy::parse("month").unwrap(), PeriodGroupBy::Month);
        assert_eq!(PeriodGroupBy::parse("year").unwrap(), PeriodGroupBy::Year);
    }

    #[test]
    fn period_group_by_rejects_day() {
        assert!(PeriodGroupBy::parse("day").is_err());
    }

    // ── Source/group validation ───────────────────────────────────────

    #[test]
    fn reading_data_supports_all_period_groups() {
        assert!(
            validate_period_source_group(PeriodSource::ReadingData, PeriodGroupBy::Week).is_ok()
        );
        assert!(
            validate_period_source_group(PeriodSource::ReadingData, PeriodGroupBy::Month).is_ok()
        );
        assert!(
            validate_period_source_group(PeriodSource::ReadingData, PeriodGroupBy::Year).is_ok()
        );
    }

    #[test]
    fn completions_supports_month_and_year_only() {
        assert!(
            validate_period_source_group(PeriodSource::Completions, PeriodGroupBy::Month).is_ok()
        );
        assert!(
            validate_period_source_group(PeriodSource::Completions, PeriodGroupBy::Year).is_ok()
        );
        assert!(
            validate_period_source_group(PeriodSource::Completions, PeriodGroupBy::Week).is_err()
        );
    }

    // ── CompletionsGroupBy ────────────────────────────────────────────

    #[test]
    fn completions_group_by_defaults_to_month() {
        assert_eq!(
            CompletionsGroupBy::parse(None).unwrap(),
            CompletionsGroupBy::Month
        );
        assert_eq!(
            CompletionsGroupBy::parse(Some("month")).unwrap(),
            CompletionsGroupBy::Month
        );
    }

    #[test]
    fn completions_group_by_accepts_none_variant() {
        assert_eq!(
            CompletionsGroupBy::parse(Some("none")).unwrap(),
            CompletionsGroupBy::None
        );
    }

    #[test]
    fn completions_group_by_rejects_unknown() {
        assert!(CompletionsGroupBy::parse(Some("day")).is_err());
    }

    // ── CompletionsIncludeSet ─────────────────────────────────────────

    #[test]
    fn completions_include_none_returns_empty() {
        let set = CompletionsIncludeSet::parse(None).unwrap();
        assert!(!set.has_summary());
        assert!(!set.has_share_assets());
    }

    #[test]
    fn completions_include_parses_summary() {
        let set = CompletionsIncludeSet::parse(Some("summary")).unwrap();
        assert!(set.has_summary());
        assert!(!set.has_share_assets());
    }

    #[test]
    fn completions_include_parses_both() {
        let set = CompletionsIncludeSet::parse(Some("summary,share_assets")).unwrap();
        assert!(set.has_summary());
        assert!(set.has_share_assets());
    }

    #[test]
    fn completions_include_rejects_unknown_token() {
        let err = CompletionsIncludeSet::parse(Some("summary,unknown")).unwrap_err();
        assert_eq!(err.0, ApiErrorCode::InvalidQuery);
        assert!(err.1.contains("unknown"));
    }

    #[test]
    fn completions_include_ignores_empty_and_duplicates() {
        let set = CompletionsIncludeSet::parse(Some("summary,,summary,")).unwrap();
        assert!(set.has_summary());
    }
}
