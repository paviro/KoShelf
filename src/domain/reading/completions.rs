use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::NaiveDate;

use crate::contracts::reading::{
    CompletionGroup, CompletionItem, CompletionsShareAssets, CompletionsSummary,
    ReadingCompletionsData,
};
use crate::domain::reading::queries::{
    CompletionsGroupBy, CompletionsSelector, ReadingCompletionsQuery,
};
use crate::domain::reading::shared;
use crate::koreader::session;
use crate::models::{PageStat, StatisticsData};
use crate::runtime::ReadingData;
use crate::time_config::TimeConfig;

/// Compute the completions response from reading data and a validated query.
pub fn reading_completions(
    reading_data: &ReadingData,
    query: ReadingCompletionsQuery,
) -> ReadingCompletionsData {
    let time_config = shared::resolve_time_config(&reading_data.time_config, query.tz);
    let stats = shared::filter_stats_by_scope(&reading_data.stats_data, query.scope);

    // Resolve optional date range and year from selector.
    let (range, resolved_year) = resolve_completions_range(&query.selector);

    // Collect completion items, sorted by end_date descending.
    let mut all_items = collect_completion_items(&stats, range.as_ref());
    all_items.sort_by(|a, b| b.end_date.cmp(&a.end_date));

    let total_items = all_items.len();

    // Optional summary (computed before items are moved into groups).
    let summary = if query.includes.has_summary() {
        Some(compute_completions_summary(
            &stats,
            &time_config,
            range.as_ref(),
            total_items,
        ))
    } else {
        None
    };

    // Build groups or flat items based on group_by.
    let (groups, items) = match query.group_by {
        CompletionsGroupBy::Month => (Some(build_month_groups(all_items)), None),
        CompletionsGroupBy::None => (None, Some(all_items)),
    };

    // Optional share assets (only meaningful when a single year is resolved).
    let share_assets = if query.includes.has_share_assets() {
        resolved_year.map(|year| CompletionsShareAssets {
            story_url: format!("/assets/recap/{year}_share_story.webp"),
            square_url: format!("/assets/recap/{year}_share_square.webp"),
            banner_url: format!("/assets/recap/{year}_share_banner.webp"),
        })
    } else {
        None
    };

    ReadingCompletionsData {
        groups,
        items,
        summary,
        share_assets,
    }
}

// ── Range resolution ────────────────────────────────────────────────────────

/// Resolve the optional date range for the completions query.
///
/// Returns `(range, resolved_year)` where `range` is `None` for
/// `Default` (meaning "all completions, no date filter") and `resolved_year`
/// is `Some` only for `Year` selectors (used for share asset URLs).
fn resolve_completions_range(
    selector: &CompletionsSelector,
) -> (Option<(NaiveDate, NaiveDate)>, Option<i32>) {
    match selector {
        CompletionsSelector::Year(y) => {
            let from = NaiveDate::from_ymd_opt(*y, 1, 1).unwrap();
            let to = NaiveDate::from_ymd_opt(*y, 12, 31).unwrap();
            (Some((from, to)), Some(*y))
        }
        CompletionsSelector::Range(r) => (Some((r.from, r.to)), None),
        CompletionsSelector::Default => (None, None),
    }
}

// ── Item collection ─────────────────────────────────────────────────────────

/// Collect completion items, optionally filtered to `[from, to]`.
/// When `range` is `None`, all completions are included.
fn collect_completion_items(
    stats: &StatisticsData,
    range: Option<&(NaiveDate, NaiveDate)>,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    for book in &stats.books {
        let Some(ref completions) = book.completions else {
            continue;
        };
        for entry in &completions.entries {
            let Ok(end_date) = NaiveDate::parse_from_str(&entry.end_date, "%Y-%m-%d") else {
                continue;
            };
            if let Some((from, to)) = range
                && (end_date < *from || end_date > *to)
            {
                continue;
            }

            items.push(CompletionItem {
                title: book.title.clone(),
                authors: shared::parse_authors(&book.authors),
                start_date: entry.start_date.clone(),
                end_date: entry.end_date.clone(),
                reading_time_sec: entry.reading_time,
                session_count: entry.session_count,
                pages_read: entry.pages_read,
                calendar_length_days: entry.calendar_length_days(),
                average_speed: entry.average_speed(),
                average_session_duration_sec: entry.avg_session_duration(),
                // Library enrichment fields — populated when library linking is available.
                rating: None,
                review_note: None,
                series: None,
                item_id: None,
                item_cover: None,
                content_type: Some(shared::to_library_content_type(book.content_type)),
            });
        }
    }

    items
}

// ── Grouping ────────────────────────────────────────────────────────────────

/// Group completion items by their end-date month key (`YYYY-MM`).
fn build_month_groups(items: Vec<CompletionItem>) -> Vec<CompletionGroup> {
    let mut groups: BTreeMap<String, Vec<CompletionItem>> = BTreeMap::new();

    for item in items {
        // end_date is YYYY-MM-DD; extract YYYY-MM.
        let month_key = item.end_date[..7].to_string();
        groups.entry(month_key).or_default().push(item);
    }

    groups
        .into_iter()
        .map(|(key, group_items)| {
            let items_finished = group_items.len();
            let reading_time_sec: i64 = group_items.iter().map(|i| i.reading_time_sec).sum();
            CompletionGroup {
                key,
                items_finished,
                reading_time_sec,
                items: group_items,
            }
        })
        .collect()
}

// ── Summary computation ─────────────────────────────────────────────────────

/// Compute the optional recap summary from reading activity, optionally within a range.
/// When `range` is `None`, all page stats are included and the span is derived from data.
fn compute_completions_summary(
    stats: &StatisticsData,
    time_config: &TimeConfig,
    range: Option<&(NaiveDate, NaiveDate)>,
    total_items: usize,
) -> CompletionsSummary {
    // Filter page_stats to the resolved range (or include all if no range).
    let range_page_stats: Vec<PageStat> = stats
        .page_stats
        .iter()
        .filter(|ps| {
            if ps.duration <= 0 {
                return false;
            }
            if let Some((from, to)) = range {
                let date = time_config.date_for_timestamp(ps.start_time);
                date >= *from && date <= *to
            } else {
                true
            }
        })
        .cloned()
        .collect();

    let total_reading_time_sec: i64 = range_page_stats.iter().map(|ps| ps.duration).sum();

    // Session metrics.
    let (avg_session, longest_session) = session::session_metrics(&range_page_stats);
    let longest_session_duration_sec = longest_session.unwrap_or(0);
    let average_session_duration_sec = avg_session.unwrap_or(0);

    // Active days.
    let mut active_dates: HashSet<NaiveDate> = HashSet::new();
    for ps in &range_page_stats {
        active_dates.insert(time_config.date_for_timestamp(ps.start_time));
    }
    let active_days = active_dates.len();

    // Compute total_days from explicit range, or derive from data span.
    let total_days = if let Some((from, to)) = range {
        (*to - *from).num_days() + 1
    } else {
        match (active_dates.iter().min(), active_dates.iter().max()) {
            (Some(first), Some(last)) => (*last - *first).num_days() + 1,
            _ => 0,
        }
    };
    let active_days_percentage = if total_days > 0 {
        (active_days as f64 / total_days as f64 * 100.0).round()
    } else {
        0.0
    };

    let longest_streak_days = compute_longest_streak(&active_dates);
    let best_month = compute_best_month(&range_page_stats, time_config);

    CompletionsSummary {
        total_items,
        total_reading_time_sec,
        longest_session_duration_sec,
        average_session_duration_sec,
        active_days,
        active_days_percentage,
        longest_streak_days,
        best_month,
    }
}

fn compute_longest_streak(active_dates: &HashSet<NaiveDate>) -> i64 {
    if active_dates.is_empty() {
        return 0;
    }

    let mut sorted: Vec<NaiveDate> = active_dates.iter().copied().collect();
    sorted.sort();

    let mut max_streak = 1i64;
    let mut current_streak = 1i64;

    for i in 1..sorted.len() {
        if sorted[i] == sorted[i - 1] + chrono::Duration::days(1) {
            current_streak += 1;
            max_streak = max_streak.max(current_streak);
        } else {
            current_streak = 1;
        }
    }

    max_streak
}

fn compute_best_month(page_stats: &[PageStat], time_config: &TimeConfig) -> Option<String> {
    if page_stats.is_empty() {
        return None;
    }

    let mut month_times: HashMap<String, i64> = HashMap::new();
    for ps in page_stats {
        let date = time_config.date_for_timestamp(ps.start_time);
        let key = shared::bucket_key_month(date);
        *month_times.entry(key).or_insert(0) += ps.duration;
    }

    month_times
        .into_iter()
        .max_by_key(|(_, seconds)| *seconds)
        .filter(|(_, seconds)| *seconds > 0)
        .map(|(key, _)| key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::common::ContentTypeFilter;
    use crate::domain::reading::queries::{CompletionsIncludeSet, DateRange};
    use crate::models::completions::{BookCompletions, ReadCompletion};
    use crate::models::{StatBook, StatisticsData};

    fn make_stats_data(books: Vec<StatBook>, page_stats: Vec<PageStat>) -> StatisticsData {
        let stats_by_md5 = books.iter().map(|b| (b.md5.clone(), b.clone())).collect();
        StatisticsData {
            books,
            page_stats,
            stats_by_md5,
        }
    }

    fn make_book(id: i64, title: &str, md5: &str) -> StatBook {
        StatBook {
            id,
            title: title.to_string(),
            authors: "Author A; Author B".to_string(),
            notes: None,
            last_open: None,
            highlights: None,
            pages: None,
            md5: md5.to_string(),
            content_type: Some(crate::models::ContentType::Book),
            total_read_time: None,
            total_read_pages: None,
            completions: None,
        }
    }

    fn make_completion(
        start: &str,
        end: &str,
        time: i64,
        sessions: i64,
        pages: i64,
    ) -> ReadCompletion {
        ReadCompletion::new(start.to_string(), end.to_string(), time, sessions, pages)
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
    fn empty_stats_produces_empty_completions() {
        let stats = make_stats_data(vec![], vec![]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
        };
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Year(2025),
            group_by: CompletionsGroupBy::Month,
            includes: CompletionsIncludeSet::default(),
            tz: None,
        };
        let result = reading_completions(&reading_data, query);
        assert!(result.groups.unwrap().is_empty());
        assert!(result.items.is_none());
        assert!(result.summary.is_none());
        assert!(result.share_assets.is_none());
    }

    #[test]
    fn completions_grouped_by_month() {
        let mut book = make_book(1, "Test Book", "abc123");
        book.completions = Some(BookCompletions::new(vec![
            make_completion("2025-01-01", "2025-02-15", 3600, 10, 200),
            make_completion("2025-03-01", "2025-04-10", 7200, 20, 400),
        ]));

        let stats = make_stats_data(vec![book], vec![]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
        };
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Year(2025),
            group_by: CompletionsGroupBy::Month,
            includes: CompletionsIncludeSet::default(),
            tz: None,
        };
        let result = reading_completions(&reading_data, query);
        let groups = result.groups.unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].key, "2025-02");
        assert_eq!(groups[0].items_finished, 1);
        assert_eq!(groups[1].key, "2025-04");
        assert_eq!(groups[1].items_finished, 1);
    }

    #[test]
    fn completions_flat_list_when_group_by_none() {
        let mut book = make_book(1, "Test Book", "abc123");
        book.completions = Some(BookCompletions::new(vec![make_completion(
            "2025-01-01",
            "2025-02-15",
            3600,
            10,
            200,
        )]));

        let stats = make_stats_data(vec![book], vec![]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
        };
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Year(2025),
            group_by: CompletionsGroupBy::None,
            includes: CompletionsIncludeSet::default(),
            tz: None,
        };
        let result = reading_completions(&reading_data, query);
        assert!(result.groups.is_none());
        let items = result.items.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Test Book");
        assert_eq!(items[0].authors, vec!["Author A", "Author B"]);
    }

    #[test]
    fn default_selector_returns_all_completions() {
        let mut book = make_book(1, "Old Book", "abc");
        book.completions = Some(BookCompletions::new(vec![
            make_completion("2023-01-01", "2023-06-15", 3600, 5, 100),
            make_completion("2025-01-01", "2025-03-10", 7200, 10, 200),
        ]));

        let stats = make_stats_data(vec![book], vec![]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
        };
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Default,
            group_by: CompletionsGroupBy::None,
            includes: CompletionsIncludeSet::default(),
            tz: None,
        };
        let result = reading_completions(&reading_data, query);
        let items = result.items.unwrap();
        // Default returns all completions across all years.
        assert_eq!(items.len(), 2);
        // Sorted descending by end_date.
        assert_eq!(items[0].end_date, "2025-03-10");
        assert_eq!(items[1].end_date, "2023-06-15");
    }

    #[test]
    fn completions_outside_range_are_excluded() {
        let mut book = make_book(1, "Test Book", "abc");
        book.completions = Some(BookCompletions::new(vec![
            make_completion("2024-06-01", "2024-12-20", 3600, 5, 100),
            make_completion("2025-01-01", "2025-03-10", 7200, 10, 200),
        ]));

        let stats = make_stats_data(vec![book], vec![]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
        };
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Year(2025),
            group_by: CompletionsGroupBy::None,
            includes: CompletionsIncludeSet::default(),
            tz: None,
        };
        let result = reading_completions(&reading_data, query);
        let items = result.items.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].end_date, "2025-03-10");
    }

    #[test]
    fn summary_includes_reading_activity_stats() {
        let mut book = make_book(1, "Test Book", "abc");
        book.completions = Some(BookCompletions::new(vec![make_completion(
            "2025-01-01",
            "2025-02-15",
            3600,
            10,
            200,
        )]));

        // Page stats within 2025: two days of reading.
        // 2025-01-15 00:00:00 UTC ≈ 1736899200
        let ps1 = make_page_stat(1, 1736899200, 1800);
        // 2025-01-16 00:00:00 UTC
        let ps2 = make_page_stat(1, 1736899200 + 86400, 2400);

        let stats = make_stats_data(vec![book], vec![ps1, ps2]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
        };
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Year(2025),
            group_by: CompletionsGroupBy::Month,
            includes: CompletionsIncludeSet::parse(Some("summary")).unwrap(),
            tz: None,
        };
        let result = reading_completions(&reading_data, query);
        let summary = result.summary.unwrap();
        assert_eq!(summary.total_items, 1);
        assert_eq!(summary.total_reading_time_sec, 4200);
        assert_eq!(summary.active_days, 2);
        assert_eq!(summary.longest_streak_days, 2);
        assert!(summary.best_month.is_some());
    }

    #[test]
    fn share_assets_provided_for_year_selector() {
        let stats = make_stats_data(vec![], vec![]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
        };
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Year(2025),
            group_by: CompletionsGroupBy::Month,
            includes: CompletionsIncludeSet::parse(Some("share_assets")).unwrap(),
            tz: None,
        };
        let result = reading_completions(&reading_data, query);
        let assets = result.share_assets.unwrap();
        assert!(assets.story_url.contains("2025"));
    }

    #[test]
    fn share_assets_none_for_range_selector() {
        let stats = make_stats_data(vec![], vec![]);
        let reading_data = ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
        };
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Range(
                DateRange::from_str("2025-01-01", "2025-12-31").unwrap(),
            ),
            group_by: CompletionsGroupBy::Month,
            includes: CompletionsIncludeSet::parse(Some("share_assets")).unwrap(),
            tz: None,
        };
        let result = reading_completions(&reading_data, query);
        assert!(result.share_assets.is_none());
    }

    #[test]
    fn longest_streak_computed_correctly() {
        let mut dates = HashSet::new();
        dates.insert(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        dates.insert(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap());
        dates.insert(NaiveDate::from_ymd_opt(2025, 1, 3).unwrap());
        // Gap
        dates.insert(NaiveDate::from_ymd_opt(2025, 1, 5).unwrap());
        dates.insert(NaiveDate::from_ymd_opt(2025, 1, 6).unwrap());

        assert_eq!(compute_longest_streak(&dates), 3);
    }

    #[test]
    fn longest_streak_empty_returns_zero() {
        assert_eq!(compute_longest_streak(&HashSet::new()), 0);
    }
}
