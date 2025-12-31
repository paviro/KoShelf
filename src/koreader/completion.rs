//! # Reading Completion Detection
//!
//! This module detects when a user has completed reading a book by analyzing page statistics
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
//!    - At least `min_completion_percentage` (75%) of pages were visited
//!    - Pages from the beginning (`min_early_percentage`, first 20%) were read
//!    - Pages from the end (`min_late_percentage`, last 2%) were read
//!
//! 3. **Progressions are split (detecting a re-read)** only when ALL conditions are met:
//!    - The current progression is already a valid completion
//!    - Reading jumps back to early pages (within `min_early_percentage`)
//!    - The remaining reading from that point would form a valid completion on its own
//!
//! ## Key Behaviors
//!
//! - **Re-reading a chapter then continuing**: No split occurs because the remaining
//!   pages from the restart wouldn't form a complete read (missing the middle).
//!
//! - **Abandoned reads**: If a user starts reading, stops partway, then restarts from
//!   the beginning and finishes, only the completed read counts. The abandoned portion
//!   gets included in the progression but doesn't prevent completion detection.
//!
//! - **True re-reads**: When a user finishes a book, then starts again from the beginning
//!   and reads through again, two separate completions are detected.

use log::{debug, info};
use std::collections::{HashMap, HashSet};

use super::session;
use crate::models::*;
use crate::time_config::TimeConfig;

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
            min_completion_percentage: 0.75, // Must read 75% of the book
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
    /// Set `log` to true for detailed debug output (use false during grouping to avoid spam)
    fn is_valid_completion(&self, total_pages: i64, config: &CompletionConfig, log: bool) -> bool {
        if total_pages <= 0 {
            return false;
        }

        // Check minimum completion percentage
        let pages_covered = self.pages_visited.len() as i64;
        let completion_percentage = pages_covered as f64 / total_pages as f64;
        if completion_percentage < config.min_completion_percentage {
            if log {
                debug!(
                    "Progression doesn't meet minimum completion percentage: {:.2}% < {:.2}%",
                    completion_percentage * 100.0,
                    config.min_completion_percentage * 100.0
                );
            }
            return false;
        }

        // Check if we read from early and late parts of the book
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

        if !has_early_pages || !has_late_pages {
            if log {
                debug!(
                    "Progression doesn't span from beginning to end (early: {}, late: {})",
                    has_early_pages, has_late_pages
                );
            }
            return false;
        }

        true
    }
}

/// Main detector for reading completions
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

        // Filter page stats for this book only and with valid durations
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

        // Sort stats by start time
        let mut sorted_stats = book_stats;
        sorted_stats.sort_by_key(|stat| stat.start_time);

        // Group stats into reading progressions
        let progressions = self.group_into_progressions(&sorted_stats, total_pages);
        debug!(
            "Found {} reading progressions for book {}",
            progressions.len(),
            book.title
        );

        // Evaluate each progression for completion
        let mut completions = Vec::new();
        for progression in progressions {
            if let Some(completion) = self.evaluate_progression(&progression, total_pages) {
                completions.push(completion);
            }
        }

        debug!(
            "Detected {} completions for book {}",
            completions.len(),
            book.title
        );
        BookCompletions::new(completions)
    }

    /// Group page stats into reading progressions based on re-read detection.
    /// A split only occurs when:
    /// 1. The current progression is a valid completion
    /// 2. Reading restarts from early pages (within min_early_percentage)
    /// 3. The remaining stats from that point would form a valid completion on their own
    fn group_into_progressions(
        &self,
        sorted_stats: &[PageStat],
        total_pages: i64,
    ) -> Vec<ReadingProgression> {
        let mut progressions = Vec::new();
        let mut current_progression = ReadingProgression::new();

        // Early page threshold for re-read detection
        let early_page_threshold = self.config.early_threshold(total_pages);

        for (i, stat) in sorted_stats.iter().enumerate() {
            // Check if we should start a new progression:
            // Only split if current is a valid completion AND page jumps back to early pages
            // AND the remaining reading from here forms a valid completion
            let should_split = if !current_progression.is_empty()
                && current_progression.is_valid_completion(total_pages, &self.config, false)
                && stat.page <= early_page_threshold
            {
                // Check if remaining stats (from this point forward) form a valid completion
                let remaining_would_complete = ReadingProgression::from_stats(&sorted_stats[i..])
                    .is_valid_completion(total_pages, &self.config, false);

                if remaining_would_complete {
                    debug!(
                        "Re-read detected: page {} is early (threshold: {}), remaining stats form valid completion, splitting",
                        stat.page, early_page_threshold
                    );
                }
                remaining_would_complete
            } else {
                false
            };

            if should_split {
                progressions.push(current_progression);
                current_progression = ReadingProgression::new();
            }

            current_progression.add_stat(stat.clone());
        }

        // Don't forget the last progression
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
        // Use shared validation logic with logging enabled for final evaluation
        if !progression.is_valid_completion(total_pages, &self.config, true) {
            return None;
        }

        let pages_covered = progression.pages_visited.len() as i64;

        // Calculate session count
        let session_count = session::session_count(&progression.stats);

        // Convert timestamps to dates
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

        info!(
            "Detected completions for {} books out of {}",
            all_completions.len(),
            stats_data.books.len()
        );

        all_completions
    }
}
