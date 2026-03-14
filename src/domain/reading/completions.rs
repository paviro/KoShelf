//! Reading completions: detection algorithm and query-layer response building.
//!
//! # Completion Detection
//!
//! Detects when a user has completed reading a book by analyzing page statistics
//! from KOReader. The core challenge is distinguishing between:
//! - A single read-through of a book
//! - Multiple re-reads of the same book
//! - Abandoned reads that were later restarted
//!
//! ## Algorithm Overview
//!
//! 1. **Page stats are sorted by time** and grouped into "reading progressions"
//!
//! 2. **A progression becomes a valid completion** when:
//!    - At least `min_completion_percentage` (78%) of pages were visited
//!      (we never have stats for all pages even if all were read)
//!    - Pages from the beginning (`min_early_percentage`, first 20%) were read
//!    - Pages from the end (`min_late_percentage`, last 2%) were read
//!
//! 3. **Progressions are split** when ALL conditions are met:
//!    - Reading jumps backwards to early pages (within `min_early_percentage`)
//!    - The remaining reading from that point would form a valid completion on its own
//!
//! ## Key Behaviors
//!
//! - **Re-reading a chapter then continuing**: No split occurs because the remaining
//!   pages from the restart wouldn't form a complete read (missing the middle parts).
//!
//! - **Abandoned reads**: If a user starts reading, stops partway, then restarts from
//!   the beginning and finishes, a split occurs. Only the second portion (the successful
//!   read-through) counts as a completion with accurate reading time.
//!
//! - **True re-reads**: When a user finishes a book, then starts again from the beginning
//!   and reads through again, two separate completions are detected.

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::NaiveDate;
use log::debug;

use super::sessions;
use crate::contracts::reading::{
    CompletionGroup, CompletionItem, CompletionsShareAssets, CompletionsSummary,
    ReadingCompletionsData,
};
use crate::domain::reading::queries::{
    CompletionsGroupBy, CompletionsSelector, ReadingCompletionsQuery,
};
use crate::domain::reading::shared;
use crate::source::koreader::types::{
    BookCompletions, PageStat, ReadCompletion, StatBook, StatisticsData,
};
use crate::store::memory::ReadingData;
use crate::store::sqlite::repo::LibraryRepository;
use crate::time_config::TimeConfig;

/// Compute the completions response from reading data and a validated query.
pub async fn reading_completions(
    reading_data: &ReadingData,
    repo: &LibraryRepository,
    query: ReadingCompletionsQuery,
) -> ReadingCompletionsData {
    let time_config = shared::resolve_time_config(&reading_data.time_config, query.tz);
    let stats = shared::filter_stats_by_scope(&reading_data.stats_data, query.scope);

    let (range, resolved_year) = resolve_completions_range(&query.selector);
    let mut all_items =
        collect_completion_items(&stats, range.as_ref(), &reading_data.page_scaling);
    enrich_completion_items(&mut all_items, repo).await;

    all_items.sort_by(|a, b| b.end_date.cmp(&a.end_date));

    let total_items = all_items.len();

    // Must be computed before items are moved into groups.
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

    let (groups, items) = match query.group_by {
        CompletionsGroupBy::Month => {
            let month_reading_time =
                compute_month_reading_times(&stats, &time_config, range.as_ref());
            (
                Some(build_month_groups(all_items, &month_reading_time)),
                None,
            )
        }
        CompletionsGroupBy::None => (None, Some(all_items)),
    };

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
/// Library enrichment fields are left as `None` — populated later via `enrich_completion_items`.
fn collect_completion_items(
    stats: &StatisticsData,
    range: Option<&(NaiveDate, NaiveDate)>,
    page_scaling: &crate::domain::reading::scaling::PageScaling,
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

            let pages_read = page_scaling.scale_pages_for_md5(&book.md5, entry.pages_read);
            let average_speed = if entry.reading_time > 0 && pages_read > 0 {
                Some(pages_read as f64 / (entry.reading_time as f64 / 3600.0))
            } else {
                None
            };

            items.push(CompletionItem {
                title: book.title.clone(),
                authors: shared::parse_authors(&book.authors),
                start_date: entry.start_date.clone(),
                end_date: entry.end_date.clone(),
                reading_time_sec: entry.reading_time,
                session_count: entry.session_count,
                pages_read,
                calendar_length_days: entry.calendar_length_days(),
                average_speed,
                average_session_duration_sec: entry.avg_session_duration(),
                rating: None,
                review_note: None,
                series: None,
                item_id: Some(book.md5.clone()),
                item_cover: None,
                content_type: Some(shared::to_library_content_type(book.content_type)),
            });
        }
    }

    items
}

/// Enrich completion items with library metadata by querying the repository.
///
/// Populates `item_cover`, `rating`, `review_note`, and `series` for items
/// whose MD5 matches a library item.
async fn enrich_completion_items(items: &mut [CompletionItem], repo: &LibraryRepository) {
    let unique_md5s: HashSet<String> = items
        .iter()
        .filter_map(|item| item.item_id.clone())
        .collect();

    let mut cache: HashMap<String, Option<EnrichmentData>> = HashMap::new();
    for md5 in unique_md5s {
        let enrichment = match repo.get_item(&md5).await {
            Ok(Some(detail)) => Some(EnrichmentData {
                cover_url: detail.cover_url,
                rating: detail.rating.map(|r| r as u32),
                review_note: detail.review_note,
                series: detail.series.and_then(|s| format_series(&s)),
            }),
            _ => None,
        };
        cache.insert(md5, enrichment);
    }

    for item in items.iter_mut() {
        if let Some(md5) = &item.item_id
            && let Some(Some(enrichment)) = cache.get(md5)
        {
            item.item_cover = Some(enrichment.cover_url.clone());
            item.rating = enrichment.rating;
            item.review_note = enrichment.review_note.clone();
            item.series = enrichment.series.clone();
        }
    }
}

struct EnrichmentData {
    cover_url: String,
    rating: Option<u32>,
    review_note: Option<String>,
    series: Option<String>,
}

/// Format a `LibrarySeries` as `"Name #Index"` or just `"Name"`.
fn format_series(series: &crate::contracts::library::LibrarySeries) -> Option<String> {
    if series.name.is_empty() {
        return None;
    }
    Some(match &series.index {
        Some(idx) => format!("{} #{}", series.name, idx),
        None => series.name.clone(),
    })
}

// ── Grouping ────────────────────────────────────────────────────────────────

/// Compute total reading time per month from all page stats (not just completed items).
fn compute_month_reading_times(
    stats: &StatisticsData,
    time_config: &TimeConfig,
    range: Option<&(NaiveDate, NaiveDate)>,
) -> HashMap<String, i64> {
    let mut month_times: HashMap<String, i64> = HashMap::new();
    for ps in &stats.page_stats {
        if ps.duration <= 0 {
            continue;
        }
        let date = time_config.date_for_timestamp(ps.start_time);
        if let Some((from, to)) = range
            && (date < *from || date > *to)
        {
            continue;
        }
        let key = shared::bucket_key_month(date);
        *month_times.entry(key).or_insert(0) += ps.duration;
    }
    month_times
}

/// Group completion items by their end-date month key (`YYYY-MM`).
/// `month_reading_times` provides total reading time per month from all reading
/// activity (not just completed items)
fn build_month_groups(
    items: Vec<CompletionItem>,
    month_reading_times: &HashMap<String, i64>,
) -> Vec<CompletionGroup> {
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
            let reading_time_sec = *month_reading_times.get(&key).unwrap_or(&0);
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

    let (avg_session, longest_session) = sessions::session_metrics(&range_page_stats);
    let longest_session_duration_sec = longest_session.unwrap_or(0);
    let average_session_duration_sec = avg_session.unwrap_or(0);

    let mut active_dates: HashSet<NaiveDate> = HashSet::new();
    for ps in &range_page_stats {
        active_dates.insert(time_config.date_for_timestamp(ps.start_time));
    }
    let active_days = active_dates.len();

    let total_days = if let Some((from, to)) = range {
        (*to - *from).num_days() + 1
    } else {
        match (active_dates.iter().min(), active_dates.iter().max()) {
            (Some(first), Some(last)) => (*last - *first).num_days() + 1,
            _ => 0,
        }
    };
    let active_days_percentage = if total_days > 0 {
        (active_days as f64 / total_days as f64 * 100.0).round() as u8
    } else {
        0
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

// ── Completion detection ─────────────────────────────────────────────────────

/// Configuration for read completion detection
#[derive(Debug, Clone)]
pub struct CompletionConfig {
    /// Minimum percentage of book that must be read to count as completion (0.0 - 1.0)
    pub min_completion_percentage: f64,
    /// Minimum percentage of book's beginning that must be read (0.0 - 1.0)
    pub min_early_percentage: f64,
    /// Minimum percentage of book's end that must be read (0.0 - 1.0)
    pub min_late_percentage: f64,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            min_completion_percentage: 0.78, // Must read 78% of the book
            min_early_percentage: 0.20,      // Must read 20% from beginning
            min_late_percentage: 0.02,       // Must read 2% from end
        }
    }
}

impl CompletionConfig {
    /// Calculate the early page threshold (pages considered "beginning" of the book)
    fn early_threshold(&self, total_pages: i64) -> i64 {
        (total_pages as f64 * self.min_early_percentage) as i64
    }

    /// Calculate the late page threshold (pages considered "end" of the book)
    fn late_threshold(&self, total_pages: i64) -> i64 {
        (total_pages as f64 * (1.0 - self.min_late_percentage)) as i64
    }
}

/// Represents a reading progression through a book
#[derive(Debug, Clone)]
struct ReadingProgression {
    stats: Vec<PageStat>,
    start_time: i64,
    end_time: i64,
    pages_visited: HashSet<i64>,
    total_reading_time: i64,
}

impl ReadingProgression {
    fn new() -> Self {
        Self {
            stats: Vec::new(),
            start_time: i64::MAX,
            end_time: 0,
            pages_visited: HashSet::new(),
            total_reading_time: 0,
        }
    }

    fn from_stats(stats: &[PageStat]) -> Self {
        let mut progression = Self::new();
        for stat in stats {
            progression.add_stat(stat.clone());
        }
        progression
    }

    fn add_stat(&mut self, stat: PageStat) {
        self.start_time = self.start_time.min(stat.start_time);
        self.end_time = self.end_time.max(stat.start_time + stat.duration);
        self.pages_visited.insert(stat.page);
        self.total_reading_time += stat.duration;
        self.stats.push(stat);
    }

    fn is_empty(&self) -> bool {
        self.stats.is_empty()
    }

    /// Check if this progression qualifies as a valid completion
    fn is_valid_completion(&self, total_pages: i64, config: &CompletionConfig) -> bool {
        if total_pages <= 0 {
            return false;
        }

        let pages_covered = self.pages_visited.len() as i64;
        let completion_percentage = pages_covered as f64 / total_pages as f64;
        if completion_percentage < config.min_completion_percentage {
            return false;
        }

        let early_threshold = config.early_threshold(total_pages);
        let late_threshold = config.late_threshold(total_pages);

        let has_early_pages = self
            .pages_visited
            .iter()
            .any(|&page| page <= early_threshold);
        let has_late_pages = self
            .pages_visited
            .iter()
            .any(|&page| page >= late_threshold);

        has_early_pages && has_late_pages
    }
}

/// Detects book completions by analyzing page statistics against configurable thresholds.
///
/// See the module-level documentation for the full algorithm description.
pub struct ReadCompletionDetector {
    config: CompletionConfig,
    time_config: TimeConfig,
}

impl ReadCompletionDetector {
    pub fn with_config_and_time(config: CompletionConfig, time_config: TimeConfig) -> Self {
        Self {
            config,
            time_config,
        }
    }

    /// Detect reading completions for a single book
    pub fn detect_completions(&self, book: &StatBook, page_stats: &[PageStat]) -> BookCompletions {
        debug!(
            "Detecting completions for book: {} (pages: {:?})",
            book.title, book.pages
        );

        let book_stats: Vec<PageStat> = page_stats
            .iter()
            .filter(|stat| stat.id_book == book.id && stat.duration > 0)
            .cloned()
            .collect();

        if book_stats.is_empty() {
            debug!("No valid page stats found for book {}", book.title);
            return BookCompletions::new(Vec::new());
        }

        let total_pages = book.pages.unwrap_or(0);
        if total_pages <= 0 {
            debug!("Book {} has no valid page count", book.title);
            return BookCompletions::new(Vec::new());
        }

        let mut sorted_stats = book_stats;
        sorted_stats.sort_by_key(|stat| stat.start_time);

        let progressions = self.group_into_progressions(&sorted_stats, total_pages);
        debug!(
            "Found {} reading progressions for book {}",
            progressions.len(),
            book.title
        );

        let mut completions = Vec::new();
        let early_threshold = self.config.early_threshold(total_pages);
        let late_threshold = self.config.late_threshold(total_pages);
        let mut best_progression: Option<(f64, usize, bool, bool)> = None; // (coverage%, pages, has_early, has_late)

        for progression in &progressions {
            let pages = progression.pages_visited.len();
            let coverage = pages as f64 / total_pages as f64;
            if best_progression.is_none_or(|(best, _, _, _)| coverage > best) {
                let has_early = progression
                    .pages_visited
                    .iter()
                    .any(|&p| p <= early_threshold);
                let has_late = progression
                    .pages_visited
                    .iter()
                    .any(|&p| p >= late_threshold);
                best_progression = Some((coverage, pages, has_early, has_late));
            }

            if let Some(completion) = self.evaluate_progression(progression, total_pages) {
                completions.push(completion);
            }
        }

        if completions.is_empty() && !progressions.is_empty() {
            if let Some((coverage, pages, has_early, has_late)) = best_progression {
                debug!(
                    "No valid completions for '{}': best progression covered {:.1}% ({}/{} pages, need {:.0}%), early pages: {}, late pages: {}",
                    book.title,
                    coverage * 100.0,
                    pages,
                    total_pages,
                    self.config.min_completion_percentage * 100.0,
                    if has_early { "✓" } else { "✗" },
                    if has_late { "✓" } else { "✗" },
                );
            }
        } else if !completions.is_empty() {
            debug!(
                "Detected {} completion(s) for '{}'",
                completions.len(),
                book.title
            );
        }
        BookCompletions::new(completions)
    }

    /// Group page stats into reading progressions based on re-read detection.
    /// A split occurs when:
    /// 1. Reading restarts from early pages (within min_early_percentage)
    /// 2. The remaining stats from that point would form a valid completion on their own
    ///
    /// This handles both abandoned reads (split off incomplete portion) and true re-reads.
    fn group_into_progressions(
        &self,
        sorted_stats: &[PageStat],
        total_pages: i64,
    ) -> Vec<ReadingProgression> {
        let mut progressions = Vec::new();
        let mut current_progression = ReadingProgression::new();

        let early_page_threshold = self.config.early_threshold(total_pages);

        for (i, stat) in sorted_stats.iter().enumerate() {
            // Split when ALL conditions are met:
            // - Current page is in first 5% of the book
            // - Previous page was beyond early threshold (actual backwards jump)
            // - Current progression already contains early pages
            // - Remaining reading from this point would form a valid completion
            let restart_threshold = (total_pages as f64 * 0.05) as i64;
            let prev_page = if i > 0 { sorted_stats[i - 1].page } else { 0 };
            let is_jumping_back = prev_page > early_page_threshold;
            let already_started_reading = current_progression
                .pages_visited
                .iter()
                .any(|&p| p <= early_page_threshold);

            let is_likely_restart = !current_progression.is_empty()
                && stat.page <= restart_threshold
                && is_jumping_back
                && already_started_reading;

            let should_split = if is_likely_restart {
                let remaining_valid = ReadingProgression::from_stats(&sorted_stats[i..])
                    .is_valid_completion(total_pages, &self.config);
                if remaining_valid {
                    debug!(
                        "Split detected: restarting at page {} (prev page was {})",
                        stat.page, prev_page
                    );
                }
                remaining_valid
            } else {
                false
            };

            if should_split {
                progressions.push(current_progression);
                current_progression = ReadingProgression::new();
            }

            current_progression.add_stat(stat.clone());
        }

        if !current_progression.is_empty() {
            progressions.push(current_progression);
        }

        progressions
    }

    /// Evaluate a reading progression to see if it constitutes a completion
    fn evaluate_progression(
        &self,
        progression: &ReadingProgression,
        total_pages: i64,
    ) -> Option<ReadCompletion> {
        if !progression.is_valid_completion(total_pages, &self.config) {
            return None;
        }

        let pages_covered = progression.pages_visited.len() as i64;
        let session_count = sessions::session_count(&progression.stats);
        let start_date = self.time_config.format_date(progression.start_time);
        let end_date = self.time_config.format_date(progression.end_time);

        debug!(
            "Valid completion found: {:.1}% completion, {} sessions, {} pages",
            (pages_covered as f64 / total_pages as f64) * 100.0,
            session_count,
            pages_covered
        );

        Some(ReadCompletion::new(
            start_date,
            end_date,
            progression.total_reading_time,
            session_count,
            pages_covered,
        ))
    }

    /// Detect completions for all books in the statistics data
    pub fn detect_all_completions(
        &self,
        stats_data: &StatisticsData,
    ) -> HashMap<String, BookCompletions> {
        let mut all_completions = HashMap::new();

        for book in &stats_data.books {
            let completions = self.detect_completions(book, &stats_data.page_stats);
            if completions.has_completions() {
                all_completions.insert(book.md5.clone(), completions);
            }
        }

        debug!(
            "Detected completions for {} books out of {}",
            all_completions.len(),
            stats_data.books.len()
        );

        all_completions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::common::ContentTypeFilter;
    use crate::domain::reading::PageScaling;
    use crate::domain::reading::queries::{CompletionsIncludeSet, DateRange};
    use crate::source::koreader::types::{
        BookCompletions, ReadCompletion, StatBook, StatisticsData,
    };

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
            authors: "Author A\nAuthor B".to_string(),
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

    fn make_reading_data(stats: StatisticsData) -> ReadingData {
        ReadingData {
            stats_data: stats,
            time_config: TimeConfig::new(None, 0),
            heatmap_scale_max: None,
            page_scaling: PageScaling::disabled(),
        }
    }

    #[tokio::test]
    async fn empty_stats_produces_empty_completions() {
        let repo = crate::store::sqlite::repo::tests::test_repo().await;
        let reading_data = make_reading_data(make_stats_data(vec![], vec![]));
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Year(2025),
            group_by: CompletionsGroupBy::Month,
            includes: CompletionsIncludeSet::default(),
            tz: None,
        };
        let result = reading_completions(&reading_data, &repo, query).await;
        assert!(result.groups.unwrap().is_empty());
        assert!(result.items.is_none());
        assert!(result.summary.is_none());
        assert!(result.share_assets.is_none());
    }

    #[tokio::test]
    async fn completions_grouped_by_month() {
        let repo = crate::store::sqlite::repo::tests::test_repo().await;
        let mut book = make_book(1, "Test Book", "abc123");
        book.completions = Some(BookCompletions::new(vec![
            make_completion("2025-01-01", "2025-02-15", 3600, 10, 200),
            make_completion("2025-03-01", "2025-04-10", 7200, 20, 400),
        ]));

        let reading_data = make_reading_data(make_stats_data(vec![book], vec![]));
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Year(2025),
            group_by: CompletionsGroupBy::Month,
            includes: CompletionsIncludeSet::default(),
            tz: None,
        };
        let result = reading_completions(&reading_data, &repo, query).await;
        let groups = result.groups.unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].key, "2025-02");
        assert_eq!(groups[0].items_finished, 1);
        assert_eq!(groups[1].key, "2025-04");
        assert_eq!(groups[1].items_finished, 1);
    }

    #[tokio::test]
    async fn completions_flat_list_when_group_by_none() {
        let repo = crate::store::sqlite::repo::tests::test_repo().await;
        let mut book = make_book(1, "Test Book", "abc123");
        book.completions = Some(BookCompletions::new(vec![make_completion(
            "2025-01-01",
            "2025-02-15",
            3600,
            10,
            200,
        )]));

        let reading_data = make_reading_data(make_stats_data(vec![book], vec![]));
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Year(2025),
            group_by: CompletionsGroupBy::None,
            includes: CompletionsIncludeSet::default(),
            tz: None,
        };
        let result = reading_completions(&reading_data, &repo, query).await;
        assert!(result.groups.is_none());
        let items = result.items.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Test Book");
        assert_eq!(items[0].authors, vec!["Author A", "Author B"]);
    }

    #[tokio::test]
    async fn default_selector_returns_all_completions() {
        let repo = crate::store::sqlite::repo::tests::test_repo().await;
        let mut book = make_book(1, "Old Book", "abc");
        book.completions = Some(BookCompletions::new(vec![
            make_completion("2023-01-01", "2023-06-15", 3600, 5, 100),
            make_completion("2025-01-01", "2025-03-10", 7200, 10, 200),
        ]));

        let reading_data = make_reading_data(make_stats_data(vec![book], vec![]));
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Default,
            group_by: CompletionsGroupBy::None,
            includes: CompletionsIncludeSet::default(),
            tz: None,
        };
        let result = reading_completions(&reading_data, &repo, query).await;
        let items = result.items.unwrap();
        // Default returns all completions across all years.
        assert_eq!(items.len(), 2);
        // Sorted descending by end_date.
        assert_eq!(items[0].end_date, "2025-03-10");
        assert_eq!(items[1].end_date, "2023-06-15");
    }

    #[tokio::test]
    async fn completions_outside_range_are_excluded() {
        let repo = crate::store::sqlite::repo::tests::test_repo().await;
        let mut book = make_book(1, "Test Book", "abc");
        book.completions = Some(BookCompletions::new(vec![
            make_completion("2024-06-01", "2024-12-20", 3600, 5, 100),
            make_completion("2025-01-01", "2025-03-10", 7200, 10, 200),
        ]));

        let reading_data = make_reading_data(make_stats_data(vec![book], vec![]));
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Year(2025),
            group_by: CompletionsGroupBy::None,
            includes: CompletionsIncludeSet::default(),
            tz: None,
        };
        let result = reading_completions(&reading_data, &repo, query).await;
        let items = result.items.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].end_date, "2025-03-10");
    }

    #[tokio::test]
    async fn summary_includes_reading_activity_stats() {
        let repo = crate::store::sqlite::repo::tests::test_repo().await;
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

        let reading_data = make_reading_data(make_stats_data(vec![book], vec![ps1, ps2]));
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Year(2025),
            group_by: CompletionsGroupBy::Month,
            includes: CompletionsIncludeSet::parse(Some("summary")).unwrap(),
            tz: None,
        };
        let result = reading_completions(&reading_data, &repo, query).await;
        let summary = result.summary.unwrap();
        assert_eq!(summary.total_items, 1);
        assert_eq!(summary.total_reading_time_sec, 4200);
        assert_eq!(summary.active_days, 2);
        assert_eq!(summary.longest_streak_days, 2);
        assert!(summary.best_month.is_some());
    }

    #[tokio::test]
    async fn share_assets_provided_for_year_selector() {
        let repo = crate::store::sqlite::repo::tests::test_repo().await;
        let reading_data = make_reading_data(make_stats_data(vec![], vec![]));
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Year(2025),
            group_by: CompletionsGroupBy::Month,
            includes: CompletionsIncludeSet::parse(Some("share_assets")).unwrap(),
            tz: None,
        };
        let result = reading_completions(&reading_data, &repo, query).await;
        let assets = result.share_assets.unwrap();
        assert!(assets.story_url.contains("2025"));
    }

    #[tokio::test]
    async fn share_assets_none_for_range_selector() {
        let repo = crate::store::sqlite::repo::tests::test_repo().await;
        let reading_data = make_reading_data(make_stats_data(vec![], vec![]));
        let query = ReadingCompletionsQuery {
            scope: ContentTypeFilter::All,
            selector: CompletionsSelector::Range(
                DateRange::from_str("2025-01-01", "2025-12-31").unwrap(),
            ),
            group_by: CompletionsGroupBy::Month,
            includes: CompletionsIncludeSet::parse(Some("share_assets")).unwrap(),
            tz: None,
        };
        let result = reading_completions(&reading_data, &repo, query).await;
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
