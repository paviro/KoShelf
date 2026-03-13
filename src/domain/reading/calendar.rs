use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::NaiveDate;

use crate::contracts::reading::{
    CalendarItemRef, CalendarScopeStats, CalendarStatsByScope, ReadingCalendarData,
    ReadingCalendarEvent,
};
use crate::domain::reading::queries::ReadingCalendarQuery;
use crate::domain::reading::shared;
use crate::infra::stores::ReadingData;
use crate::koreader::types::{PageStat, StatisticsData};
use crate::models::ContentType;
use crate::time_config::TimeConfig;

/// Compute the calendar response for a specific month from reading data.
pub fn reading_calendar(
    reading_data: &ReadingData,
    query: ReadingCalendarQuery,
) -> ReadingCalendarData {
    let time_config = shared::resolve_time_config(&reading_data.time_config, query.tz);
    let (month_from, month_to) = month_date_range(&query.month);

    // stats_by_scope always covers all three scopes regardless of query.scope.
    let stats_by_scope =
        compute_stats_by_scope(&reading_data.stats_data, &time_config, month_from, month_to);

    // Events are filtered by the requested scope.
    let scoped_stats = shared::filter_stats_by_scope(&reading_data.stats_data, query.scope);
    let (events, items) = build_events_and_items(
        &scoped_stats,
        &time_config,
        month_from,
        month_to,
        &reading_data.covers_by_md5,
    );

    ReadingCalendarData {
        month: query.month,
        events,
        items,
        stats_by_scope,
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Parse "YYYY-MM" into the first and last day of the month.
fn month_date_range(month_key: &str) -> (NaiveDate, NaiveDate) {
    let parts: Vec<&str> = month_key.split('-').collect();
    let year: i32 = parts[0].parse().expect("valid year in month key");
    let month: u32 = parts[1].parse().expect("valid month in month key");
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month start");
    let last = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("valid next year")
            - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).expect("valid next month")
            - chrono::Duration::days(1)
    };
    (first, last)
}

/// Compute `CalendarStatsByScope` for all three scopes within [month_from, month_to].
fn compute_stats_by_scope(
    stats_data: &StatisticsData,
    time_config: &TimeConfig,
    month_from: NaiveDate,
    month_to: NaiveDate,
) -> CalendarStatsByScope {
    let days_in_month = (month_to - month_from).num_days() + 1;

    CalendarStatsByScope {
        all: compute_scope_stats(stats_data, time_config, month_from, month_to, days_in_month),
        books: compute_scope_stats(
            &stats_data.filtered_by_content_type(ContentType::Book),
            time_config,
            month_from,
            month_to,
            days_in_month,
        ),
        comics: compute_scope_stats(
            &stats_data.filtered_by_content_type(ContentType::Comic),
            time_config,
            month_from,
            month_to,
            days_in_month,
        ),
    }
}

fn compute_scope_stats(
    stats_data: &StatisticsData,
    time_config: &TimeConfig,
    from: NaiveDate,
    to: NaiveDate,
    days_in_month: i64,
) -> CalendarScopeStats {
    let mut unique_books: HashSet<i64> = HashSet::new();
    let mut unique_dates: HashSet<NaiveDate> = HashSet::new();
    let mut total_pages: i64 = 0;
    let mut total_time: i64 = 0;

    for ps in &stats_data.page_stats {
        if ps.duration <= 0 {
            continue;
        }
        let date = time_config.date_for_timestamp(ps.start_time);
        if date < from || date > to {
            continue;
        }
        unique_books.insert(ps.id_book);
        unique_dates.insert(date);
        total_pages += 1;
        total_time += ps.duration;
    }

    let active_days_percentage = if days_in_month > 0 {
        (unique_dates.len() as f64 / days_in_month as f64) * 100.0
    } else {
        0.0
    };

    CalendarScopeStats {
        items_read: unique_books.len(),
        pages_read: total_pages,
        reading_time_sec: total_time,
        active_days_percentage,
    }
}

/// Build scope-filtered events and item reference map for the month.
fn build_events_and_items(
    stats_data: &StatisticsData,
    time_config: &TimeConfig,
    month_from: NaiveDate,
    month_to: NaiveDate,
    covers_by_md5: &std::collections::HashMap<String, String>,
) -> (Vec<ReadingCalendarEvent>, BTreeMap<String, CalendarItemRef>) {
    // Build book ID -> StatBook lookup.
    let book_by_id: HashMap<i64, &crate::koreader::types::StatBook> =
        stats_data.books.iter().map(|b| (b.id, b)).collect();

    // Group page stats by book, filter to month range.
    let mut stats_by_book: HashMap<i64, Vec<&PageStat>> = HashMap::new();
    for ps in &stats_data.page_stats {
        if ps.duration <= 0 {
            continue;
        }
        let date = time_config.date_for_timestamp(ps.start_time);
        if date < month_from || date > month_to {
            continue;
        }
        stats_by_book.entry(ps.id_book).or_default().push(ps);
    }

    let mut events = Vec::new();
    let mut items: BTreeMap<String, CalendarItemRef> = BTreeMap::new();

    for (book_id, page_stats) in &stats_by_book {
        let stat_book = match book_by_id.get(book_id) {
            Some(b) => b,
            None => continue,
        };
        let item_ref_key = stat_book.md5.clone();

        // Ensure item reference exists.
        if !items.contains_key(&item_ref_key) {
            items.insert(
                item_ref_key.clone(),
                CalendarItemRef {
                    title: stat_book.title.clone(),
                    authors: shared::parse_authors(&stat_book.authors),
                    content_type: shared::to_library_content_type(stat_book.content_type),
                    item_id: Some(stat_book.md5.clone()),
                    item_cover: covers_by_md5.get(&stat_book.md5).cloned(),
                },
            );
        }

        // Group page stats by day.
        let mut by_day: BTreeMap<NaiveDate, DayAccumulator> = BTreeMap::new();
        for ps in page_stats {
            let date = time_config.date_for_timestamp(ps.start_time);
            let acc = by_day.entry(date).or_default();
            acc.reading_time_sec += ps.duration;
            acc.pages_read += 1;
        }

        // Merge consecutive days into event spans.
        let days: Vec<NaiveDate> = by_day.keys().copied().collect();
        merge_into_events(&mut events, &item_ref_key, &days, &by_day);
    }

    // Sort events deterministically: by start date, then by item title for stability.
    events.sort_by(|a, b| {
        a.start.cmp(&b.start).then_with(|| {
            let title_a = items
                .get(&a.item_ref)
                .map(|i| i.title.as_str())
                .unwrap_or("");
            let title_b = items
                .get(&b.item_ref)
                .map(|i| i.title.as_str())
                .unwrap_or("");
            title_a.cmp(title_b)
        })
    });

    (events, items)
}

#[derive(Default)]
struct DayAccumulator {
    reading_time_sec: i64,
    pages_read: i64,
}

/// Merge consecutive reading days into event spans.
fn merge_into_events(
    events: &mut Vec<ReadingCalendarEvent>,
    item_ref: &str,
    days: &[NaiveDate],
    by_day: &BTreeMap<NaiveDate, DayAccumulator>,
) {
    if days.is_empty() {
        return;
    }

    let mut span_start = days[0];
    let mut span_end = days[0];
    let mut span_time: i64 = by_day.get(&days[0]).map_or(0, |a| a.reading_time_sec);
    let mut span_pages: i64 = by_day.get(&days[0]).map_or(0, |a| a.pages_read);

    for day in &days[1..] {
        if *day == span_end + chrono::Duration::days(1) {
            span_end = *day;
            if let Some(acc) = by_day.get(day) {
                span_time += acc.reading_time_sec;
                span_pages += acc.pages_read;
            }
        } else {
            events.push(make_event(
                item_ref, span_start, span_end, span_time, span_pages,
            ));
            span_start = *day;
            span_end = *day;
            span_time = by_day.get(day).map_or(0, |a| a.reading_time_sec);
            span_pages = by_day.get(day).map_or(0, |a| a.pages_read);
        }
    }

    events.push(make_event(
        item_ref, span_start, span_end, span_time, span_pages,
    ));
}

fn make_event(
    item_ref: &str,
    start: NaiveDate,
    end: NaiveDate,
    reading_time_sec: i64,
    pages_read: i64,
) -> ReadingCalendarEvent {
    let end_field = if start == end {
        None
    } else {
        Some(
            (end + chrono::Duration::days(1))
                .format("%Y-%m-%d")
                .to_string(),
        )
    };
    ReadingCalendarEvent {
        item_ref: item_ref.to_string(),
        start: start.format("%Y-%m-%d").to_string(),
        end: end_field,
        reading_time_sec,
        pages_read,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::common::ContentTypeFilter;
    use crate::koreader::types::{PageStat, StatBook, StatisticsData};

    fn make_stats_data(books: Vec<StatBook>, page_stats: Vec<PageStat>) -> StatisticsData {
        let stats_by_md5 = books.iter().map(|b| (b.md5.clone(), b.clone())).collect();
        StatisticsData {
            books,
            page_stats,
            stats_by_md5,
        }
    }

    fn make_book(id: i64, title: &str, md5: &str, content_type: Option<ContentType>) -> StatBook {
        StatBook {
            id,
            title: title.to_string(),
            authors: "Author A".to_string(),
            notes: None,
            last_open: None,
            highlights: None,
            pages: None,
            md5: md5.to_string(),
            content_type,
            total_read_time: None,
            total_read_pages: None,
            completions: None,
        }
    }

    fn make_page_stat(id_book: i64, start_time: i64, duration: i64) -> PageStat {
        PageStat {
            id_book,
            page: 1,
            start_time,
            duration,
        }
    }

    #[test]
    fn month_date_range_regular_month() {
        let (from, to) = month_date_range("2026-03");
        assert_eq!(from, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
        assert_eq!(to, NaiveDate::from_ymd_opt(2026, 3, 31).unwrap());
    }

    #[test]
    fn month_date_range_february_non_leap() {
        let (from, to) = month_date_range("2026-02");
        assert_eq!(from, NaiveDate::from_ymd_opt(2026, 2, 1).unwrap());
        assert_eq!(to, NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
    }

    #[test]
    fn month_date_range_december() {
        let (from, to) = month_date_range("2026-12");
        assert_eq!(from, NaiveDate::from_ymd_opt(2026, 12, 1).unwrap());
        assert_eq!(to, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    }

    #[test]
    fn empty_stats_produces_empty_calendar() {
        let stats = make_stats_data(vec![], vec![]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
            covers_by_md5: std::collections::HashMap::new(),
        };
        let query = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::All,
            tz: None,
        };
        let result = reading_calendar(&reading_data, query);
        assert_eq!(result.month, "2026-03");
        assert!(result.events.is_empty());
        assert!(result.items.is_empty());
        assert_eq!(result.stats_by_scope.all.items_read, 0);
    }

    #[test]
    fn single_day_event_has_no_end() {
        // 2026-03-10 00:00:00 UTC = 1773100800
        let book = make_book(1, "Test Book", "abc123", Some(ContentType::Book));
        let ps = make_page_stat(1, 1773100800, 300);
        let stats = make_stats_data(vec![book], vec![ps]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
            covers_by_md5: std::collections::HashMap::new(),
        };
        let query = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::All,
            tz: None,
        };
        let result = reading_calendar(&reading_data, query);
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].start, "2026-03-10");
        assert!(result.events[0].end.is_none());
        assert_eq!(result.events[0].reading_time_sec, 300);
        assert_eq!(result.events[0].pages_read, 1);
    }

    #[test]
    fn consecutive_days_merge_into_span() {
        let book = make_book(1, "Test Book", "abc123", Some(ContentType::Book));
        // Day 1: 2026-03-10 00:00:00 UTC
        let ps1 = make_page_stat(1, 1773100800, 200);
        // Day 2: 2026-03-11 00:00:00 UTC
        let ps2 = make_page_stat(1, 1773100800 + 86400, 300);
        let stats = make_stats_data(vec![book], vec![ps1, ps2]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
            covers_by_md5: std::collections::HashMap::new(),
        };
        let query = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::All,
            tz: None,
        };
        let result = reading_calendar(&reading_data, query);
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].start, "2026-03-10");
        assert_eq!(result.events[0].end, Some("2026-03-12".to_string()));
        assert_eq!(result.events[0].reading_time_sec, 500);
        assert_eq!(result.events[0].pages_read, 2);
    }

    #[test]
    fn gap_creates_separate_events() {
        let book = make_book(1, "Test Book", "abc123", Some(ContentType::Book));
        // Day 1: 2026-03-10
        let ps1 = make_page_stat(1, 1773100800, 200);
        // Day 3: 2026-03-12 (gap on day 2)
        let ps2 = make_page_stat(1, 1773100800 + 86400 * 2, 300);
        let stats = make_stats_data(vec![book], vec![ps1, ps2]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
            covers_by_md5: std::collections::HashMap::new(),
        };
        let query = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::All,
            tz: None,
        };
        let result = reading_calendar(&reading_data, query);
        assert_eq!(result.events.len(), 2);
        assert_eq!(result.events[0].start, "2026-03-10");
        assert!(result.events[0].end.is_none());
        assert_eq!(result.events[1].start, "2026-03-12");
        assert!(result.events[1].end.is_none());
    }

    #[test]
    fn stats_by_scope_computed_for_all_scopes() {
        let book = make_book(1, "A Book", "abc", Some(ContentType::Book));
        let comic = make_book(2, "A Comic", "def", Some(ContentType::Comic));
        // Both on 2026-03-10
        let ps1 = make_page_stat(1, 1773100800, 100);
        let ps2 = make_page_stat(2, 1773100800, 200);
        let stats = make_stats_data(vec![book, comic], vec![ps1, ps2]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
            covers_by_md5: std::collections::HashMap::new(),
        };
        let query = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::Books, // Only book events
            tz: None,
        };
        let result = reading_calendar(&reading_data, query);

        // Events filtered to books only.
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.items.len(), 1);

        // But stats_by_scope covers all scopes.
        assert_eq!(result.stats_by_scope.all.items_read, 2);
        assert_eq!(result.stats_by_scope.all.reading_time_sec, 300);
        assert_eq!(result.stats_by_scope.books.items_read, 1);
        assert_eq!(result.stats_by_scope.books.reading_time_sec, 100);
        assert_eq!(result.stats_by_scope.comics.items_read, 1);
        assert_eq!(result.stats_by_scope.comics.reading_time_sec, 200);
    }

    #[test]
    fn page_stats_outside_month_are_excluded() {
        let book = make_book(1, "Test Book", "abc123", Some(ContentType::Book));
        // 2026-02-28 (outside March)
        let ps_outside = make_page_stat(1, 1773100800 - 86400 * 10, 999);
        // 2026-03-10 (inside March)
        let ps_inside = make_page_stat(1, 1773100800, 100);
        let stats = make_stats_data(vec![book], vec![ps_outside, ps_inside]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
            covers_by_md5: std::collections::HashMap::new(),
        };
        let query = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::All,
            tz: None,
        };
        let result = reading_calendar(&reading_data, query);
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].reading_time_sec, 100);
        assert_eq!(result.stats_by_scope.all.reading_time_sec, 100);
    }
}
