use std::collections::{BTreeMap, HashMap, HashSet};

use super::calendar::{
    ActivityMonthResponse, ActivityMonthsResponse, CalendarEventResponse, CalendarItemResponse,
    CalendarMonthlyStats,
};
use super::common::{ApiMeta, ContentTypeFilter};
use super::library::{
    LibraryAnnotation, LibraryContentType, LibraryDetailItem, LibraryDetailResponse,
    LibraryDetailStatistics, LibraryIdentifier, LibraryListItem, LibraryListResponse,
    LibraryStatus,
};
use super::recap::{
    CompletionYearResponse, CompletionYearsResponse, RecapItemResponse, RecapMonthResponse,
    RecapShareAssets, RecapSummaryResponse,
};
use super::site::{SiteCapabilities, SiteResponse};
use super::statistics::{
    ActivityHeatmapConfig, ActivityOverview, ActivityStreaks, ActivityWeekResponse,
    ActivityWeeksResponse, ActivityYearDailyResponse, ActivityYearSummaryResponse, AvailableWeek,
    MonthlyAggregate, YearlySummary,
};
use crate::models::{BookStatus, ContentType, LibraryItem};

pub fn build_meta(version: impl Into<String>, generated_at: impl Into<String>) -> ApiMeta {
    ApiMeta {
        version: version.into(),
        generated_at: generated_at.into(),
    }
}

pub fn map_site_response(
    meta: ApiMeta,
    title: impl Into<String>,
    language: impl Into<String>,
    capabilities: SiteCapabilities,
) -> SiteResponse {
    SiteResponse {
        meta,
        title: title.into(),
        language: language.into(),
        capabilities,
    }
}

pub fn map_library_content_type(content_type: ContentType) -> LibraryContentType {
    match content_type {
        ContentType::Book => LibraryContentType::Book,
        ContentType::Comic => LibraryContentType::Comic,
    }
}

pub fn map_library_status(status: BookStatus) -> LibraryStatus {
    match status {
        BookStatus::Reading => LibraryStatus::Reading,
        BookStatus::Complete => LibraryStatus::Complete,
        BookStatus::Abandoned => LibraryStatus::Abandoned,
        BookStatus::Unknown => LibraryStatus::Unknown,
    }
}

pub fn map_library_list_item(item: &LibraryItem) -> LibraryListItem {
    LibraryListItem {
        id: item.id.clone(),
        title: item.book_info.title.clone(),
        authors: item.book_info.authors.clone(),
        series: item.series_display(),
        status: map_library_status(item.status()),
        progress_percentage: item.progress_percentage(),
        rating: item.rating(),
        annotation_count: item.annotation_count(),
        cover_url: format!("/assets/covers/{}.webp", item.id),
        content_type: map_library_content_type(item.content_type()),
    }
}

pub fn map_library_list_response(meta: ApiMeta, items: &[LibraryItem]) -> LibraryListResponse {
    let mut mapped_items: Vec<LibraryListItem> = items.iter().map(map_library_list_item).collect();
    mapped_items.sort_by(|a, b| a.title.cmp(&b.title).then_with(|| a.id.cmp(&b.id)));

    LibraryListResponse {
        meta,
        items: mapped_items,
    }
}

fn map_library_identifier(identifier: crate::models::Identifier) -> LibraryIdentifier {
    let display_scheme = identifier.display_scheme();
    let url = identifier.url();

    LibraryIdentifier {
        scheme: identifier.scheme,
        value: identifier.value,
        display_scheme,
        url,
    }
}

fn map_library_annotation(annotation: &crate::models::Annotation) -> LibraryAnnotation {
    LibraryAnnotation {
        chapter: annotation.chapter.clone(),
        datetime: annotation.datetime.clone(),
        pageno: annotation.pageno,
        text: annotation.text.clone(),
        note: annotation.note.clone(),
    }
}

pub fn map_library_detail_response(
    meta: ApiMeta,
    item: &LibraryItem,
    search_base_path: impl Into<String>,
    item_stats: Option<crate::models::StatBook>,
    session_stats: Option<crate::models::BookSessionStats>,
) -> LibraryDetailResponse {
    let highlights = item
        .annotations()
        .iter()
        .filter(|annotation| annotation.is_highlight())
        .map(map_library_annotation)
        .collect::<Vec<_>>();
    let bookmarks = item
        .annotations()
        .iter()
        .filter(|annotation| annotation.is_bookmark())
        .map(map_library_annotation)
        .collect::<Vec<_>>();
    let completions = item_stats
        .as_ref()
        .and_then(|stats| stats.completions.clone());

    LibraryDetailResponse {
        meta,
        item: LibraryDetailItem {
            id: item.id.clone(),
            title: item.book_info.title.clone(),
            authors: item.book_info.authors.clone(),
            series: item.series_display(),
            status: map_library_status(item.status()),
            progress_percentage: item.progress_percentage(),
            rating: item.rating(),
            cover_url: format!("/assets/covers/{}.webp", item.id),
            content_type: map_library_content_type(item.content_type()),
            language: item.language().cloned(),
            publisher: item.publisher().cloned(),
            description: item.book_info.description.clone(),
            review_note: item.review_note().cloned(),
            pages: item.doc_pages(),
            search_base_path: search_base_path.into(),
            subjects: item.subjects().clone(),
            identifiers: item
                .identifiers()
                .into_iter()
                .map(map_library_identifier)
                .collect(),
        },
        highlights,
        bookmarks,
        statistics: LibraryDetailStatistics {
            item_stats,
            session_stats,
            completions,
        },
    }
}

fn map_activity_heatmap_config(max_scale_seconds: i64) -> ActivityHeatmapConfig {
    ActivityHeatmapConfig {
        max_scale_seconds: (max_scale_seconds > 0).then_some(max_scale_seconds),
    }
}

fn map_activity_overview(stats: &crate::models::ReadingStats) -> ActivityOverview {
    ActivityOverview {
        total_read_time: stats.total_read_time,
        total_page_reads: stats.total_page_reads,
        longest_read_time_in_day: stats.longest_read_time_in_day,
        most_pages_in_day: stats.most_pages_in_day,
        average_session_duration: stats.average_session_duration,
        longest_session_duration: stats.longest_session_duration,
        total_completions: stats.total_completions,
        items_completed: stats.books_completed,
        most_completions: stats.most_completions,
    }
}

pub fn map_activity_weeks_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    available_years: Vec<i32>,
    stats: &crate::models::ReadingStats,
    max_scale_seconds: i64,
) -> ActivityWeeksResponse {
    let mut available_weeks: Vec<AvailableWeek> = stats
        .weeks
        .iter()
        .map(|week| AvailableWeek {
            week_key: week.start_date.clone(),
            start_date: week.start_date.clone(),
            end_date: week.end_date.clone(),
        })
        .collect();
    available_weeks.sort_by(|a, b| b.week_key.cmp(&a.week_key));
    available_weeks.dedup_by(|a, b| a.week_key == b.week_key);

    ActivityWeeksResponse {
        meta,
        content_type,
        available_years,
        available_weeks,
        overview: map_activity_overview(stats),
        streaks: ActivityStreaks {
            longest: stats.longest_streak.clone(),
            current: stats.current_streak.clone(),
        },
        heatmap_config: map_activity_heatmap_config(max_scale_seconds),
    }
}

fn week_end_date_from_key(week_key: &str) -> String {
    chrono::NaiveDate::parse_from_str(week_key, "%Y-%m-%d")
        .map(|start| {
            (start + chrono::Duration::days(6))
                .format("%Y-%m-%d")
                .to_string()
        })
        .unwrap_or_else(|_| week_key.to_string())
}

fn zero_weekly_stats(week_key: &str) -> crate::models::WeeklyStats {
    crate::models::WeeklyStats {
        start_date: week_key.to_string(),
        end_date: week_end_date_from_key(week_key),
        read_time: 0,
        pages_read: 0,
        avg_pages_per_day: 0.0,
        avg_read_time_per_day: 0.0,
        longest_session_duration: None,
        average_session_duration: None,
    }
}

pub fn map_activity_week_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    week_key: impl Into<String>,
    stats: Option<&crate::models::WeeklyStats>,
    all_daily_activity: &[crate::models::DailyStats],
) -> ActivityWeekResponse {
    let week_key = week_key.into();
    let resolved_stats = stats
        .cloned()
        .unwrap_or_else(|| zero_weekly_stats(&week_key));
    let daily_activity = week_daily_activity(
        &resolved_stats.start_date,
        &resolved_stats.end_date,
        all_daily_activity,
    );

    ActivityWeekResponse {
        meta,
        content_type,
        week_key: week_key.clone(),
        stats: resolved_stats,
        daily_activity,
    }
}

fn week_daily_activity(
    start_date: &str,
    end_date: &str,
    all_daily_activity: &[crate::models::DailyStats],
) -> Vec<crate::models::DailyStats> {
    all_daily_activity
        .iter()
        .filter(|entry| entry.date.as_str() >= start_date && entry.date.as_str() <= end_date)
        .cloned()
        .collect()
}

fn year_daily_activity(
    year: i32,
    reading_stats: &crate::models::ReadingStats,
) -> Vec<crate::models::DailyStats> {
    let year_prefix = format!("{year}-");
    reading_stats
        .daily_activity
        .iter()
        .filter(|entry| entry.date.starts_with(&year_prefix))
        .cloned()
        .collect()
}

fn year_monthly_aggregates(daily_activity: &[crate::models::DailyStats]) -> Vec<MonthlyAggregate> {
    let mut month_acc: BTreeMap<String, (i64, i64, usize)> = BTreeMap::new();

    for day in daily_activity {
        if day.date.len() < 7 {
            continue;
        }

        let month_key = day.date[0..7].to_string();
        let entry = month_acc.entry(month_key).or_insert((0, 0, 0));
        entry.0 += day.read_time;
        entry.1 += day.pages_read;
        if day.read_time > 0 {
            entry.2 += 1;
        }
    }

    month_acc
        .into_iter()
        .map(
            |(month_key, (read_time, pages_read, active_days))| MonthlyAggregate {
                month_key,
                read_time,
                pages_read,
                active_days,
            },
        )
        .collect()
}

pub fn map_activity_year_daily_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    year: i32,
    reading_stats: &crate::models::ReadingStats,
    max_scale_seconds: i64,
) -> ActivityYearDailyResponse {
    ActivityYearDailyResponse {
        meta,
        content_type,
        year,
        daily_activity: year_daily_activity(year, reading_stats),
        config: Some(map_activity_heatmap_config(max_scale_seconds)),
    }
}

pub fn map_activity_year_summary_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    year: i32,
    reading_stats: &crate::models::ReadingStats,
    completion_counts_by_year: &HashMap<i32, i64>,
    max_scale_seconds: i64,
) -> ActivityYearSummaryResponse {
    let daily_activity = year_daily_activity(year, reading_stats);

    ActivityYearSummaryResponse {
        meta,
        content_type,
        year,
        summary: YearlySummary {
            completed_count: completion_counts_by_year.get(&year).copied().unwrap_or(0),
        },
        monthly_aggregates: year_monthly_aggregates(&daily_activity),
        config: Some(map_activity_heatmap_config(max_scale_seconds)),
    }
}

fn content_type_matches_filter(content_type: ContentType, filter: ContentTypeFilter) -> bool {
    match filter {
        ContentTypeFilter::All => true,
        ContentTypeFilter::Books => content_type == ContentType::Book,
        ContentTypeFilter::Comics => content_type == ContentType::Comic,
    }
}

fn map_calendar_monthly_stats(stats: &crate::models::MonthlyStats) -> CalendarMonthlyStats {
    CalendarMonthlyStats {
        items_read: stats.books_read,
        pages_read: stats.pages_read,
        time_read: stats.time_read,
        days_read_pct: stats.days_read_pct,
    }
}

pub fn map_activity_months_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    mut months: Vec<String>,
) -> ActivityMonthsResponse {
    months.sort_by(|a, b| b.cmp(a));
    months.dedup();

    ActivityMonthsResponse {
        meta,
        content_type,
        months,
    }
}

pub fn map_activity_month_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    month_data: &crate::models::CalendarMonthData,
) -> ActivityMonthResponse {
    let items = month_data
        .books
        .iter()
        .filter_map(|(id, item)| {
            if !content_type_matches_filter(item.content_type, content_type) {
                return None;
            }

            Some((
                id.clone(),
                CalendarItemResponse {
                    title: item.title.clone(),
                    authors: item.authors.clone(),
                    content_type: map_library_content_type(item.content_type),
                    color: item.color.clone(),
                    item_id: item.item_id.clone(),
                    item_cover: item.item_cover.clone(),
                },
            ))
        })
        .collect::<BTreeMap<_, _>>();

    let events = month_data
        .events
        .iter()
        .filter(|event| items.contains_key(&event.item_id))
        .map(|event| CalendarEventResponse {
            start: event.start.clone(),
            end: event.end.clone(),
            total_read_time: event.total_read_time,
            total_pages_read: event.total_pages_read,
            item_id: event.item_id.clone(),
        })
        .collect::<Vec<_>>();

    let stats = match content_type {
        ContentTypeFilter::All => map_calendar_monthly_stats(&month_data.stats),
        ContentTypeFilter::Books => map_calendar_monthly_stats(&month_data.stats_books),
        ContentTypeFilter::Comics => map_calendar_monthly_stats(&month_data.stats_comics),
    };

    ActivityMonthResponse {
        meta,
        content_type,
        events,
        items,
        stats,
    }
}

fn map_optional_content_type(content_type: Option<ContentType>) -> Option<LibraryContentType> {
    content_type.map(map_library_content_type)
}

fn map_recap_item(item: &crate::models::RecapItem) -> RecapItemResponse {
    RecapItemResponse {
        title: item.title.clone(),
        authors: item.authors.clone(),
        start_date: item.start_date.clone(),
        end_date: item.end_date.clone(),
        reading_time: item.reading_time,
        session_count: item.session_count,
        pages_read: item.pages_read,
        rating: item.rating,
        review_note: item.review_note.clone(),
        series: item.series_display.clone(),
        item_id: item.item_id.clone(),
        item_cover: item.item_cover.clone(),
        content_type: map_optional_content_type(item.content_type),
    }
}

fn map_recap_month(month: &crate::models::MonthRecap) -> RecapMonthResponse {
    RecapMonthResponse {
        month_key: month.month_key.clone(),
        month_label: month.month_label.clone(),
        items_finished: month.books_finished,
        read_time: month.hours_read_seconds,
        items: month.items.iter().map(map_recap_item).collect(),
    }
}

fn map_recap_summary(summary: &crate::models::YearlySummary) -> RecapSummaryResponse {
    RecapSummaryResponse {
        total_items: summary.total_books,
        total_time_seconds: summary.total_time_seconds,
        total_time_days: summary.total_time_days,
        total_time_hours: summary.total_time_hours,
        longest_session_hours: summary.longest_session_hours,
        longest_session_minutes: summary.longest_session_minutes,
        average_session_hours: summary.average_session_hours,
        average_session_minutes: summary.average_session_minutes,
        active_days: summary.active_days,
        active_days_percentage: summary.active_days_percentage,
        longest_streak: summary.longest_streak,
        best_month_name: summary.best_month_name.clone(),
        best_month_time_display: summary.best_month_time_display.clone(),
    }
}

fn sort_desc_unique_years(mut years: Vec<i32>) -> Vec<i32> {
    years.sort_by(|a, b| b.cmp(a));
    years.dedup();
    years
}

pub fn map_completion_years_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    years: Vec<i32>,
) -> CompletionYearsResponse {
    let years = sort_desc_unique_years(years);
    CompletionYearsResponse {
        meta,
        content_type,
        latest_year: years.first().copied(),
        available_years: years,
    }
}

pub fn map_completion_year_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
    year: i32,
    months: &[crate::models::MonthRecap],
    summary: &crate::models::YearlySummary,
    share_assets: Option<RecapShareAssets>,
) -> CompletionYearResponse {
    let mapped_months: Vec<RecapMonthResponse> = months.iter().map(map_recap_month).collect();
    let items: Vec<RecapItemResponse> = mapped_months
        .iter()
        .flat_map(|month| month.items.iter().cloned())
        .collect();

    CompletionYearResponse {
        meta,
        content_type,
        year,
        summary: map_recap_summary(summary),
        months: mapped_months,
        items,
        share_assets,
    }
}

pub fn years_for_content_type(
    year_month_items: &HashMap<i32, BTreeMap<String, Vec<crate::models::RecapItem>>>,
    target: ContentType,
) -> Vec<i32> {
    let mut years: Vec<i32> = year_month_items
        .iter()
        .filter_map(|(year, months)| {
            let has_target = months
                .values()
                .flatten()
                .any(|item| item.content_type == Some(target));
            if has_target { Some(*year) } else { None }
        })
        .collect();
    years.sort_by(|a, b| b.cmp(a));
    years.dedup();
    years
}

pub fn available_years_from_stats(
    stats: &crate::models::ReadingStats,
    completion_counts_by_year: &HashMap<i32, i64>,
) -> Vec<i32> {
    let mut years = HashSet::<i32>::new();

    for day in &stats.daily_activity {
        if let Some(year_str) = day.date.get(0..4)
            && let Ok(year) = year_str.parse::<i32>()
        {
            years.insert(year);
        }
    }

    years.extend(completion_counts_by_year.keys().copied());

    let mut years: Vec<i32> = years.into_iter().collect();
    years.sort_by(|a, b| b.cmp(a));
    years
}
