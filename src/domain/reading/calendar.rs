use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::NaiveDate;

use crate::contracts::reading::{
    CalendarItemRef, CalendarScopeStats, CalendarStatsByScope, ReadingCalendarData,
    ReadingCalendarEvent,
};
use crate::domain::reading::queries::ReadingCalendarQuery;
use crate::domain::reading::scaling::PageScaling;
use crate::domain::reading::shared;
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::infra::stores::ReadingData;
use crate::models::ContentType;
use crate::source::koreader::types::{PageStat, StatisticsData};
use crate::time_config::TimeConfig;

/// Compute the calendar response for a specific month from reading data.
pub async fn reading_calendar(
    reading_data: &ReadingData,
    repo: &LibraryRepository,
    query: ReadingCalendarQuery,
) -> ReadingCalendarData {
    let time_config = shared::resolve_time_config(&reading_data.time_config, query.tz);
    let (month_from, month_to) = month_date_range(&query.month);

    // stats_by_scope always covers all three scopes regardless of query.scope.
    let stats_by_scope = compute_stats_by_scope(
        &reading_data.stats_data,
        &time_config,
        month_from,
        month_to,
        &reading_data.page_scaling,
    );

    // Events are filtered by the requested scope.
    let scoped_stats = shared::filter_stats_by_scope(&reading_data.stats_data, query.scope);
    let (events, items) = build_events_and_items(
        &scoped_stats,
        &time_config,
        month_from,
        month_to,
        repo,
        &reading_data.page_scaling,
    )
    .await;

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
    page_scaling: &PageScaling,
) -> CalendarStatsByScope {
    let days_in_month = (month_to - month_from).num_days() + 1;

    CalendarStatsByScope {
        all: compute_scope_stats(
            stats_data,
            time_config,
            month_from,
            month_to,
            days_in_month,
            page_scaling,
        ),
        books: compute_scope_stats(
            &stats_data.filtered_by_content_type(ContentType::Book),
            time_config,
            month_from,
            month_to,
            days_in_month,
            page_scaling,
        ),
        comics: compute_scope_stats(
            &stats_data.filtered_by_content_type(ContentType::Comic),
            time_config,
            month_from,
            month_to,
            days_in_month,
            page_scaling,
        ),
    }
}

fn compute_scope_stats(
    stats_data: &StatisticsData,
    time_config: &TimeConfig,
    from: NaiveDate,
    to: NaiveDate,
    days_in_month: i64,
    page_scaling: &PageScaling,
) -> CalendarScopeStats {
    let mut unique_books: HashSet<i64> = HashSet::new();
    let mut unique_dates: HashSet<NaiveDate> = HashSet::new();
    let mut scaled_pages: f64 = 0.0;
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
        scaled_pages += page_scaling.factor_for_book_id(ps.id_book);
        total_time += ps.duration;
    }

    let total_pages = super::scaling::round_pages(scaled_pages);

    let active_days_percentage = if days_in_month > 0 {
        (unique_dates.len() as f64 / days_in_month as f64 * 100.0).round() as u8
    } else {
        0
    };

    CalendarScopeStats {
        items_read: unique_books.len(),
        pages_read: total_pages,
        reading_time_sec: total_time,
        active_days_percentage,
    }
}

/// Build scope-filtered events and item reference map for the month.
///
/// Events are built from the entire reading history so that streaks spanning
/// month boundaries remain intact. Only events that overlap [month_from,
/// month_to] are included in the result — a cross-month event appears
/// identically (with full stats) in both months.
async fn build_events_and_items(
    stats_data: &StatisticsData,
    time_config: &TimeConfig,
    month_from: NaiveDate,
    month_to: NaiveDate,
    repo: &LibraryRepository,
    page_scaling: &PageScaling,
) -> (Vec<ReadingCalendarEvent>, BTreeMap<String, CalendarItemRef>) {
    let book_by_id: HashMap<i64, &crate::source::koreader::types::StatBook> =
        stats_data.books.iter().map(|b| (b.id, b)).collect();

    // No month filtering here — events are built from full history then filtered.
    let mut stats_by_book: HashMap<i64, Vec<&PageStat>> = HashMap::new();
    for ps in &stats_data.page_stats {
        if ps.duration <= 0 {
            continue;
        }
        stats_by_book.entry(ps.id_book).or_default().push(ps);
    }

    let mut all_events = Vec::new();

    for (book_id, page_stats) in &stats_by_book {
        let stat_book = match book_by_id.get(book_id) {
            Some(b) => b,
            None => continue,
        };
        let item_ref_key = stat_book.md5.clone();

        // Accumulate pages as float per day, round once.
        let mut by_day: BTreeMap<NaiveDate, DayAccumulator> = BTreeMap::new();
        for ps in page_stats {
            let date = time_config.date_for_timestamp(ps.start_time);
            let acc = by_day.entry(date).or_default();
            acc.reading_time_sec += ps.duration;
            acc.scaled_pages += page_scaling.factor_for_book_id(ps.id_book);
        }

        let days: Vec<NaiveDate> = by_day.keys().copied().collect();
        merge_into_events(&mut all_events, &item_ref_key, &days, &by_day);
    }

    let events: Vec<ReadingCalendarEvent> = all_events
        .into_iter()
        .filter(|ev| event_overlaps_month(ev, month_from, month_to))
        .collect();

    let mut items: BTreeMap<String, CalendarItemRef> = BTreeMap::new();
    for ev in &events {
        if items.contains_key(&ev.item_ref) {
            continue;
        }
        if let Some(stat_book) = stats_data.stats_by_md5.get(&ev.item_ref) {
            let item_cover = repo
                .get_item(&stat_book.md5)
                .await
                .ok()
                .flatten()
                .map(|detail| detail.cover_url);

            items.insert(
                ev.item_ref.clone(),
                CalendarItemRef {
                    title: stat_book.title.clone(),
                    authors: shared::parse_authors(&stat_book.authors),
                    content_type: shared::to_library_content_type(stat_book.content_type),
                    item_id: Some(stat_book.md5.clone()),
                    item_cover,
                },
            );
        }
    }

    // Sort by start date, then by title for deterministic output.
    let mut events = events;
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
    scaled_pages: f64,
}

impl DayAccumulator {
    fn pages_read(&self) -> i64 {
        super::scaling::round_pages(self.scaled_pages)
    }
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
    let mut span_pages: i64 = by_day.get(&days[0]).map_or(0, |a| a.pages_read());

    for day in &days[1..] {
        if *day == span_end + chrono::Duration::days(1) {
            span_end = *day;
            if let Some(acc) = by_day.get(day) {
                span_time += acc.reading_time_sec;
                span_pages += acc.pages_read();
            }
        } else {
            events.push(make_event(
                item_ref, span_start, span_end, span_time, span_pages,
            ));
            span_start = *day;
            span_end = *day;
            span_time = by_day.get(day).map_or(0, |a| a.reading_time_sec);
            span_pages = by_day.get(day).map_or(0, |a| a.pages_read());
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

/// Check whether an event overlaps the [month_from, month_to] range.
fn event_overlaps_month(
    ev: &ReadingCalendarEvent,
    month_from: NaiveDate,
    month_to: NaiveDate,
) -> bool {
    let ev_start =
        NaiveDate::parse_from_str(&ev.start, "%Y-%m-%d").expect("valid event start date");
    let ev_end = match ev.end {
        Some(ref end_str) => {
            // `end` is exclusive (day after last reading day), so subtract 1.
            NaiveDate::parse_from_str(end_str, "%Y-%m-%d").expect("valid event end date")
                - chrono::Duration::days(1)
        }
        None => ev_start, // single-day event
    };
    ev_start <= month_to && ev_end >= month_from
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::common::ContentTypeFilter;
    use crate::domain::reading::PageScaling;
    use crate::source::koreader::types::{PageStat, StatBook, StatisticsData};

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

    fn make_reading_data(stats: StatisticsData) -> ReadingData {
        ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
            page_scaling: PageScaling::disabled(),
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

    #[tokio::test]
    async fn empty_stats_produces_empty_calendar() {
        let repo = crate::infra::sqlite::library_repo::tests::test_repo().await;
        let reading_data = make_reading_data(make_stats_data(vec![], vec![]));
        let query = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::All,
            tz: None,
        };
        let result = reading_calendar(&reading_data, &repo, query).await;
        assert_eq!(result.month, "2026-03");
        assert!(result.events.is_empty());
        assert!(result.items.is_empty());
        assert_eq!(result.stats_by_scope.all.items_read, 0);
    }

    #[tokio::test]
    async fn single_day_event_has_no_end() {
        let repo = crate::infra::sqlite::library_repo::tests::test_repo().await;
        // 2026-03-10 00:00:00 UTC = 1773100800
        let book = make_book(1, "Test Book", "abc123", Some(ContentType::Book));
        let ps = make_page_stat(1, 1773100800, 300);
        let reading_data = make_reading_data(make_stats_data(vec![book], vec![ps]));
        let query = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::All,
            tz: None,
        };
        let result = reading_calendar(&reading_data, &repo, query).await;
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].start, "2026-03-10");
        assert!(result.events[0].end.is_none());
        assert_eq!(result.events[0].reading_time_sec, 300);
        assert_eq!(result.events[0].pages_read, 1);
    }

    #[tokio::test]
    async fn consecutive_days_merge_into_span() {
        let repo = crate::infra::sqlite::library_repo::tests::test_repo().await;
        let book = make_book(1, "Test Book", "abc123", Some(ContentType::Book));
        // Day 1: 2026-03-10 00:00:00 UTC
        let ps1 = make_page_stat(1, 1773100800, 200);
        // Day 2: 2026-03-11 00:00:00 UTC
        let ps2 = make_page_stat(1, 1773100800 + 86400, 300);
        let reading_data = make_reading_data(make_stats_data(vec![book], vec![ps1, ps2]));
        let query = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::All,
            tz: None,
        };
        let result = reading_calendar(&reading_data, &repo, query).await;
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].start, "2026-03-10");
        assert_eq!(result.events[0].end, Some("2026-03-12".to_string()));
        assert_eq!(result.events[0].reading_time_sec, 500);
        assert_eq!(result.events[0].pages_read, 2);
    }

    #[tokio::test]
    async fn gap_creates_separate_events() {
        let repo = crate::infra::sqlite::library_repo::tests::test_repo().await;
        let book = make_book(1, "Test Book", "abc123", Some(ContentType::Book));
        // Day 1: 2026-03-10
        let ps1 = make_page_stat(1, 1773100800, 200);
        // Day 3: 2026-03-12 (gap on day 2)
        let ps2 = make_page_stat(1, 1773100800 + 86400 * 2, 300);
        let reading_data = make_reading_data(make_stats_data(vec![book], vec![ps1, ps2]));
        let query = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::All,
            tz: None,
        };
        let result = reading_calendar(&reading_data, &repo, query).await;
        assert_eq!(result.events.len(), 2);
        assert_eq!(result.events[0].start, "2026-03-10");
        assert!(result.events[0].end.is_none());
        assert_eq!(result.events[1].start, "2026-03-12");
        assert!(result.events[1].end.is_none());
    }

    #[tokio::test]
    async fn stats_by_scope_computed_for_all_scopes() {
        let repo = crate::infra::sqlite::library_repo::tests::test_repo().await;
        let book = make_book(1, "A Book", "abc", Some(ContentType::Book));
        let comic = make_book(2, "A Comic", "def", Some(ContentType::Comic));
        // Both on 2026-03-10
        let ps1 = make_page_stat(1, 1773100800, 100);
        let ps2 = make_page_stat(2, 1773100800, 200);
        let reading_data = make_reading_data(make_stats_data(vec![book, comic], vec![ps1, ps2]));
        let query = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::Books, // Only book events
            tz: None,
        };
        let result = reading_calendar(&reading_data, &repo, query).await;

        assert_eq!(result.events.len(), 1); // books only
        assert_eq!(result.items.len(), 1);

        // stats_by_scope covers all scopes regardless of query scope.
        assert_eq!(result.stats_by_scope.all.items_read, 2);
        assert_eq!(result.stats_by_scope.all.reading_time_sec, 300);
        assert_eq!(result.stats_by_scope.books.items_read, 1);
        assert_eq!(result.stats_by_scope.books.reading_time_sec, 100);
        assert_eq!(result.stats_by_scope.comics.items_read, 1);
        assert_eq!(result.stats_by_scope.comics.reading_time_sec, 200);
    }

    #[tokio::test]
    async fn non_overlapping_events_outside_month_are_excluded() {
        let repo = crate::infra::sqlite::library_repo::tests::test_repo().await;
        let book = make_book(1, "Test Book", "abc123", Some(ContentType::Book));
        // 2026-02-28 (outside March, non-consecutive with Mar 10 → separate event)
        let ps_outside = make_page_stat(1, 1773100800 - 86400 * 10, 999);
        // 2026-03-10 (inside March)
        let ps_inside = make_page_stat(1, 1773100800, 100);
        let reading_data =
            make_reading_data(make_stats_data(vec![book], vec![ps_outside, ps_inside]));
        let query = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::All,
            tz: None,
        };
        let result = reading_calendar(&reading_data, &repo, query).await;
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].reading_time_sec, 100);
        assert_eq!(result.stats_by_scope.all.reading_time_sec, 100);
    }

    #[tokio::test]
    async fn cross_month_event_cloned_into_both_months() {
        let repo = crate::infra::sqlite::library_repo::tests::test_repo().await;
        let book = make_book(1, "Test Book", "abc123", Some(ContentType::Book));
        // 2026-02-28 00:00:00 UTC = 1772236800
        let ps_feb = make_page_stat(1, 1772236800, 200);
        // 2026-03-01 00:00:00 UTC = 1772236800 + 86400
        let ps_mar = make_page_stat(1, 1772236800 + 86400, 300);
        let reading_data = make_reading_data(make_stats_data(vec![book], vec![ps_feb, ps_mar]));

        // Query March: the cross-month event should appear with full stats.
        let query_mar = ReadingCalendarQuery {
            month: "2026-03".to_string(),
            scope: ContentTypeFilter::All,
            tz: None,
        };
        let result_mar = reading_calendar(&reading_data, &repo, query_mar).await;
        assert_eq!(result_mar.events.len(), 1);
        assert_eq!(result_mar.events[0].start, "2026-02-28");
        assert_eq!(result_mar.events[0].end, Some("2026-03-02".to_string()));
        assert_eq!(result_mar.events[0].reading_time_sec, 500);
        assert_eq!(result_mar.events[0].pages_read, 2);

        // Query February: the same cross-month event should appear identically.
        let query_feb = ReadingCalendarQuery {
            month: "2026-02".to_string(),
            scope: ContentTypeFilter::All,
            tz: None,
        };
        let result_feb = reading_calendar(&reading_data, &repo, query_feb).await;
        assert_eq!(result_feb.events.len(), 1);
        assert_eq!(result_feb.events[0].start, "2026-02-28");
        assert_eq!(result_feb.events[0].end, Some("2026-03-02".to_string()));
        assert_eq!(result_feb.events[0].reading_time_sec, 500);
        assert_eq!(result_feb.events[0].pages_read, 2);
    }
}
