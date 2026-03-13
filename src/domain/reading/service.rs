use crate::contracts::reading::{
    ReadingAvailablePeriodsData, ReadingCalendarData, ReadingCompletionsData, ReadingMetricsData,
    ReadingSummaryData,
};
use crate::domain::reading::queries::{
    ReadingAvailablePeriodsQuery, ReadingCalendarQuery, ReadingCompletionsQuery,
    ReadingMetricsQuery, ReadingSummaryQuery,
};
use crate::domain::reading::{available_periods, calendar, completions, metrics, summary};
use crate::runtime::ReadingData;

#[derive(Debug, Default, Clone, Copy)]
pub struct ReadingService;

impl ReadingService {
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
