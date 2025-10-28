use std::collections::{HashMap, HashSet};
use log::{debug, info};

use crate::models::*;
use crate::session_calculator;
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
    /// Maximum gap in days between reading sessions within the same completion
    pub max_gap_days: i64,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            min_completion_percentage: 0.75,  // Must read 75% of the book
            min_early_percentage: 0.20,       // Must read 20% from beginning
            min_late_percentage: 0.05,        // Must read 5% from end
            max_gap_days: 180,                // Max 6 months between sessions
        }
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
    
    fn gap_from(&self, timestamp: i64) -> i64 {
        (timestamp - self.end_time).abs()
    }
}

/// Main detector for reading completions
pub struct ReadCompletionDetector {
    config: CompletionConfig,
    time_config: TimeConfig,
}

impl ReadCompletionDetector {
    pub fn with_config_and_time(config: CompletionConfig, time_config: TimeConfig) -> Self { Self { config, time_config } }
    
    /// Detect reading completions for a single book
    pub fn detect_completions(&self, book: &StatBook, page_stats: &[PageStat]) -> BookCompletions {
        debug!("Detecting completions for book: {} (pages: {:?})", book.title, book.pages);
        
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
        
        let total_pages = book.pages.unwrap_or(0) as i64;
        if total_pages <= 0 {
            debug!("Book {} has no valid page count", book.title);
            return BookCompletions::new(Vec::new());
        }
        
        // Sort stats by start time
        let mut sorted_stats = book_stats;
        sorted_stats.sort_by_key(|stat| stat.start_time);
        
        // Group stats into reading progressions
        let progressions = self.group_into_progressions(&sorted_stats);
        debug!("Found {} reading progressions for book {}", progressions.len(), book.title);
        
        // Evaluate each progression for completion
        let mut completions = Vec::new();
        for progression in progressions {
            if let Some(completion) = self.evaluate_progression(&progression, total_pages) {
                completions.push(completion);
            }
        }
        
        debug!("Detected {} completions for book {}", completions.len(), book.title);
        BookCompletions::new(completions)
    }
    
    /// Group page stats into reading progressions based on time gaps
    fn group_into_progressions(&self, sorted_stats: &[PageStat]) -> Vec<ReadingProgression> {
        let mut progressions = Vec::new();
        let mut current_progression = ReadingProgression::new();
        
        for stat in sorted_stats {
            let time_gap_seconds = if current_progression.is_empty() {
                0
            } else {
                current_progression.gap_from(stat.start_time)
            };
            
            // Convert time gap to days for comparison
            let time_gap_days = time_gap_seconds / (24 * 60 * 60);
            
            if time_gap_days > self.config.max_gap_days && !current_progression.is_empty() {
                // Start new progression due to large time gap
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
    fn evaluate_progression(&self, progression: &ReadingProgression, total_pages: i64) -> Option<ReadCompletion> {
        // Calculate completion metrics
        let pages_covered = progression.pages_visited.len() as i64;
        let completion_percentage = pages_covered as f64 / total_pages as f64;
        
        // Check if we meet minimum completion percentage
        if completion_percentage < self.config.min_completion_percentage {
            debug!("Progression doesn't meet minimum completion percentage: {:.2}% < {:.2}%", 
                completion_percentage * 100.0, self.config.min_completion_percentage * 100.0);
            return None;
        }
        
        // Check if we read from early and late parts of the book
        let early_threshold = (total_pages as f64 * self.config.min_early_percentage) as i64;
        let late_threshold = (total_pages as f64 * (1.0 - self.config.min_late_percentage)) as i64;
        
        let has_early_pages = progression.pages_visited.iter().any(|&page| page <= early_threshold);
        let has_late_pages = progression.pages_visited.iter().any(|&page| page >= late_threshold);
        
        if !has_early_pages || !has_late_pages {
            debug!("Progression doesn't span from beginning to end (early: {}, late: {})", 
                has_early_pages, has_late_pages);
            return None;
        }
        
        // Calculate session count
        let session_count = session_calculator::session_count(&progression.stats);
        
        // Convert timestamps to dates
        let start_date = self.time_config.format_date(progression.start_time);
        let end_date = self.time_config.format_date(progression.end_time);
        
        debug!("Valid completion found: {:.1}% completion, {} sessions, {} pages", 
            completion_percentage * 100.0, session_count, pages_covered);
        
        Some(ReadCompletion::new(
            start_date,
            end_date,
            progression.total_reading_time,
            session_count,
            pages_covered,
        ))
    }
    
    /// Detect completions for all books in the statistics data
    pub fn detect_all_completions(&self, stats_data: &StatisticsData) -> HashMap<String, BookCompletions> {
        let mut all_completions = HashMap::new();
        
        for book in &stats_data.books {
            let completions = self.detect_completions(book, &stats_data.page_stats);
            if completions.has_completions() {
                all_completions.insert(book.md5.clone(), completions);
            }
        }
        
        info!("Detected completions for {} books out of {}", 
            all_completions.len(), stats_data.books.len());
        
        all_completions
    }
}

 