use chrono::{Datelike, Duration, NaiveDate};
use log::debug;
use std::collections::{HashMap, HashSet};

use super::completion::{CompletionConfig, ReadCompletionDetector};
use super::session;
use crate::i18n::Translations;
use crate::models::*;
use crate::time_config::TimeConfig;

/// Trait for calculating book session statistics
pub trait BookStatistics {
    fn calculate_session_stats(
        &self,
        page_stats: &[PageStat],
        time_config: &TimeConfig,
        translations: &Translations,
    ) -> BookSessionStats;
}

impl BookStatistics for StatBook {
    /// Calculate additional statistics for this book from page stats
    fn calculate_session_stats(
        &self,
        page_stats: &[PageStat],
        time_config: &TimeConfig,
        translations: &Translations,
    ) -> BookSessionStats {
        let book_sessions: Vec<&PageStat> = page_stats
            .iter()
            .filter(|stat| stat.id_book == self.id && stat.duration > 0)
            .collect();

        // Calculate session statistics using shared helper
        let book_stats: Vec<PageStat> = book_sessions.iter().cloned().cloned().collect();
        let durations = session::session_durations(&book_stats);
        let session_count = durations.len() as i64;
        let longest_session_duration = durations.iter().max().copied();
        let average_session_duration = if !durations.is_empty() {
            Some(durations.iter().sum::<i64>() / session_count)
        } else {
            None
        };

        let last_read_date = book_sessions
            .iter()
            .map(|s| s.start_time)
            .max()
            .map(|timestamp| {
                let date = time_config.date_for_timestamp(timestamp);
                let current_year = time_config.today_date().year();
                let locale = translations.locale();

                // Get appropriate format string from translations
                let format_key = if date.year() == current_year {
                    "datetime.short-current-year"
                } else {
                    "datetime.short-with-year"
                };
                let format_str = translations.get(format_key);

                date.format_localized(&format_str, locale).to_string()
            });

        let reading_speed = if let (Some(total_time), Some(total_pages)) =
            (self.total_read_time, self.total_read_pages)
        {
            if total_time > 0 {
                Some((total_pages as f64 * 3600.0) / total_time as f64) // pages per hour
            } else {
                None
            }
        } else {
            None
        };

        BookSessionStats {
            session_count,
            average_session_duration,
            longest_session_duration,
            last_read_date,
            reading_speed,
        }
    }
}

/// Main statistics calculator
pub struct StatisticsCalculator;

impl StatisticsCalculator {
    /// Calculate reading statistics based on the parsed data and populate completions
    pub fn calculate_stats(
        stats_data: &mut StatisticsData,
        time_config: &TimeConfig,
    ) -> ReadingStats {
        // Initialize overall stats
        let mut total_read_time = 0;
        let mut total_page_reads = 0;

        // Maps to track daily stats
        let mut daily_read_time: HashMap<String, i64> = HashMap::new();
        let mut daily_page_reads: HashMap<String, i64> = HashMap::new();

        // For weekly stats, we'll organize by ISO week and track page stats for session calculation
        let mut weekly_stats: HashMap<(i32, u32), (i64, i64, Vec<PageStat>)> = HashMap::new(); // (year, week) -> (read_time, pages_read, page_stats)

        // Process all page stats
        for stat in &stats_data.page_stats {
            // Skip invalid durations
            if stat.duration <= 0 {
                continue;
            }

            // Add to overall totals
            total_read_time += stat.duration;
            total_page_reads += 1;

            // Convert unix timestamp to logical local date using configured timezone/day start
            let date = time_config.date_for_timestamp(stat.start_time);
            let date_str = date.format("%Y-%m-%d").to_string();

            // Add to daily stats
            *daily_read_time.entry(date_str.clone()).or_insert(0) += stat.duration;
            *daily_page_reads.entry(date_str).or_insert(0) += 1;

            // Add to weekly stats
            let year = date.year();
            let week = date.iso_week().week();
            let key = (year, week);
            let entry = weekly_stats.entry(key).or_insert((0, 0, Vec::new()));
            entry.0 += stat.duration;
            entry.1 += 1;
            entry.2.push(stat.clone());
        }

        // Find max values for daily stats
        let longest_read_time_in_day = daily_read_time.values().cloned().max().unwrap_or(0);
        let most_pages_in_day = daily_page_reads.values().cloned().max().unwrap_or(0);

        // Calculate overall session statistics
        let (average_session_duration, longest_session_duration) =
            session::session_metrics(&stats_data.page_stats);

        // Calculate completion statistics
        let (total_completions, books_completed, most_completions) =
            Self::calculate_completion_stats(stats_data);

        // Convert weekly stats to WeeklyStats structs
        let weeks = Self::build_weekly_stats(weekly_stats);

        // Create daily activity data for heatmap
        let daily_activity = Self::build_daily_activity(daily_read_time, daily_page_reads);

        // Calculate reading streaks
        let (longest_streak, current_streak) =
            Self::calculate_streaks(&daily_activity, time_config);

        ReadingStats {
            total_read_time,
            total_page_reads,
            longest_read_time_in_day,
            most_pages_in_day,
            average_session_duration,
            longest_session_duration,
            total_completions,
            books_completed,
            most_completions,
            longest_streak,
            current_streak,
            weeks,
            daily_activity,
        }
    }

    /// Build weekly statistics from raw weekly data
    fn build_weekly_stats(
        weekly_stats: HashMap<(i32, u32), (i64, i64, Vec<PageStat>)>,
    ) -> Vec<WeeklyStats> {
        let mut weeks = Vec::new();
        for ((year, week), (read_time, pages_read, page_stats)) in weekly_stats {
            // Calculate the start date of the week (Monday)
            // This is a simplified calculation
            let start_date_approx = NaiveDate::from_isoywd_opt(year, week, chrono::Weekday::Mon)
                .unwrap_or_else(|| {
                    NaiveDate::from_ymd_opt(year, 1, 1)
                        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2020, 1, 1).unwrap())
                });

            let end_date = start_date_approx + Duration::days(6); // Sunday

            // Calculate session statistics for this week
            let (average_session_duration, longest_session_duration) =
                session::session_metrics(&page_stats);

            let weekly_stat = WeeklyStats {
                start_date: start_date_approx.format("%Y-%m-%d").to_string(),
                end_date: end_date.format("%Y-%m-%d").to_string(),
                read_time,
                pages_read,
                avg_pages_per_day: pages_read as f64 / 7.0,
                avg_read_time_per_day: read_time as f64 / 7.0,
                longest_session_duration,
                average_session_duration,
            };

            weeks.push(weekly_stat);
        }

        // Sort weeks by start date (newest first)
        weeks.sort_by(|a, b| b.start_date.cmp(&a.start_date));
        weeks
    }

    // session_metrics moved to session_calculator

    /// Build daily activity data from daily stats maps
    fn build_daily_activity(
        daily_read_time: HashMap<String, i64>,
        daily_page_reads: HashMap<String, i64>,
    ) -> Vec<DailyStats> {
        let mut daily_activity = Vec::new();
        for (date, read_time) in daily_read_time.iter() {
            let pages_read = daily_page_reads.get(date).cloned().unwrap_or(0);
            daily_activity.push(DailyStats {
                date: date.clone(),
                read_time: *read_time,
                pages_read,
            });
        }

        // Sort daily activity by date (oldest first for chronological display)
        daily_activity.sort_by(|a, b| a.date.cmp(&b.date));
        daily_activity
    }

    /// Calculate reading streaks from daily activity data
    /// Returns (longest_streak_info, current_streak_info)
    fn calculate_streaks(
        daily_activity: &[DailyStats],
        time_config: &TimeConfig,
    ) -> (StreakInfo, StreakInfo) {
        if daily_activity.is_empty() {
            return (
                StreakInfo::new(0, None, None),
                StreakInfo::new(0, None, None),
            );
        }

        // Get all unique dates with reading activity (pages_read > 0)
        let reading_dates: Vec<NaiveDate> = daily_activity
            .iter()
            .filter(|day| day.pages_read > 0)
            .filter_map(|day| NaiveDate::parse_from_str(&day.date, "%Y-%m-%d").ok())
            .collect();

        if reading_dates.is_empty() {
            return (
                StreakInfo::new(0, None, None),
                StreakInfo::new(0, None, None),
            );
        }

        // Sort dates chronologically
        let mut sorted_dates = reading_dates;
        sorted_dates.sort();

        let today = time_config.today_date();

        // Find all streaks and their date ranges
        let mut streaks: Vec<(i64, NaiveDate, NaiveDate)> = Vec::new(); // (length, start_date, end_date)
        let mut current_streak_start = sorted_dates[0];
        let mut current_streak_length = 1i64;

        for i in 1..sorted_dates.len() {
            let prev_date = sorted_dates[i - 1];
            let curr_date = sorted_dates[i];

            // Check if current date is exactly one day after previous date
            if curr_date == prev_date + Duration::days(1) {
                current_streak_length += 1;
            } else {
                // End of current streak, record it
                streaks.push((current_streak_length, current_streak_start, prev_date));
                current_streak_start = curr_date;
                current_streak_length = 1;
            }
        }

        // Don't forget the last streak
        if let Some(&last_date) = sorted_dates.last() {
            streaks.push((current_streak_length, current_streak_start, last_date));
        }

        // Find longest streak
        let longest_streak_info =
            if let Some(&(length, start, end)) = streaks.iter().max_by_key(|&&(len, _, _)| len) {
                StreakInfo::new(
                    length,
                    Some(start.format("%Y-%m-%d").to_string()),
                    Some(end.format("%Y-%m-%d").to_string()),
                )
            } else {
                StreakInfo::new(0, None, None)
            };

        // Find current streak (streak that includes today or recent days)
        let current_streak_info = if let Some(&last_reading_date) = sorted_dates.last() {
            let days_since_last_read = (today - last_reading_date).num_days();

            if days_since_last_read <= 1 {
                // Last read was today or yesterday, find the streak that ends with last_reading_date
                if let Some(&(length, start, _)) = streaks
                    .iter()
                    .find(|&&(_, _, end)| end == last_reading_date)
                {
                    StreakInfo::new(
                        length,
                        Some(start.format("%Y-%m-%d").to_string()),
                        None, // Current streak doesn't have an end date
                    )
                } else {
                    StreakInfo::new(0, None, None)
                }
            } else {
                StreakInfo::new(0, None, None) // No current streak if last read was more than 1 day ago
            }
        } else {
            StreakInfo::new(0, None, None)
        };

        (longest_streak_info, current_streak_info)
    }

    /// Populate completion data for all books in the statistics data
    pub fn populate_completions(stats_data: &mut StatisticsData, time_config: &TimeConfig) {
        let detector = ReadCompletionDetector::with_config_and_time(
            CompletionConfig::default(),
            time_config.clone(),
        );
        let all_completions = detector.detect_all_completions(stats_data);

        // Update each book with its completion data
        for book in &mut stats_data.books {
            if let Some(completions) = all_completions.get(&book.md5) {
                book.completions = Some(completions.clone());
            }
        }

        // Also update the stats_by_md5 map for consistency
        for (md5, book) in &mut stats_data.stats_by_md5 {
            if let Some(completions) = all_completions.get(md5) {
                book.completions = Some(completions.clone());
            }
        }
    }

    /// Calculate completion statistics across all books
    fn calculate_completion_stats(stats_data: &StatisticsData) -> (i64, i64, i64) {
        let mut total_completions = 0i64;
        let mut books_completed = 0i64;
        let mut most_completions = 0i64;

        for book in &stats_data.books {
            if let Some(ref completions) = book.completions
                && completions.total_completions > 0
            {
                total_completions += completions.total_completions as i64;
                books_completed += 1;
                most_completions = most_completions.max(completions.total_completions as i64);
            }
        }

        (total_completions, books_completed, most_completions)
    }
    /// Filter statistics to exclude days that don't meet the minimum requirements per book
    pub fn filter_stats(
        stats_data: &mut StatisticsData,
        time_config: &TimeConfig,
        min_pages: Option<u32>,
        min_time: Option<u32>,
    ) {
        // 1. Aggregate daily totals per book per day
        let mut daily_book_pages: HashMap<(i64, String), i64> = HashMap::new();
        let mut daily_book_time: HashMap<(i64, String), i64> = HashMap::new();

        for stat in &stats_data.page_stats {
            if stat.duration <= 0 {
                continue;
            }

            let date = time_config.date_for_timestamp(stat.start_time);
            let date_str = date.format("%Y-%m-%d").to_string();
            let key = (stat.id_book, date_str);

            *daily_book_pages.entry(key.clone()).or_insert(0) += 1;
            *daily_book_time.entry(key).or_insert(0) += stat.duration;
        }

        // 2. Identify valid (book, date) combinations
        let mut valid_book_dates: HashMap<(i64, String), bool> = HashMap::new();

        // Collect all unique (book, date) combinations
        let mut all_book_dates: Vec<(i64, String)> = daily_book_pages.keys().cloned().collect();
        all_book_dates.extend(daily_book_time.keys().cloned());
        all_book_dates.sort();
        all_book_dates.dedup();

        for book_date in all_book_dates {
            let pages = *daily_book_pages.get(&book_date).unwrap_or(&0);
            let time = *daily_book_time.get(&book_date).unwrap_or(&0);

            let pages_ok = min_pages.is_some_and(|min| pages >= min as i64);
            let time_ok = min_time.is_some_and(|min| time >= min as i64);

            // OR logic: valid if either condition is met
            let is_valid = if min_pages.is_some() && min_time.is_some() {
                pages_ok || time_ok
            } else if min_pages.is_some() {
                pages_ok
            } else {
                time_ok
            };

            valid_book_dates.insert(book_date, is_valid);
        }

        // 3. Filter page_stats based on per-book-per-day validity
        stats_data.page_stats.retain(|stat| {
            if stat.duration <= 0 {
                return false;
            }
            let date = time_config.date_for_timestamp(stat.start_time);
            let date_str = date.format("%Y-%m-%d").to_string();
            let key = (stat.id_book, date_str);
            *valid_book_dates.get(&key).unwrap_or(&false)
        });
    }

    /// Filter statistics to only include books present in the library.
    /// The library_md5s set contains MD5 hashes of books in the scanned library.
    /// This filters out statistics for deleted books or books in other directories.
    pub fn filter_to_library(stats_data: &mut StatisticsData, library_md5s: &HashSet<String>) {
        let original_count = stats_data.books.len();

        // 1. Get IDs of books to keep (those whose MD5 is in library_md5s)
        let mut ids_to_keep: HashSet<i64> = HashSet::new();
        stats_data.books.retain(|book| {
            if library_md5s.contains(&book.md5) {
                ids_to_keep.insert(book.id);
                true
            } else {
                debug!(
                    "Filtering out statistics for book not in library: '{}' by {} (md5: {})",
                    book.title, book.authors, book.md5
                );
                false
            }
        });

        // 2. Filter page_stats to only include entries for kept books
        stats_data
            .page_stats
            .retain(|stat| ids_to_keep.contains(&stat.id_book));

        // 3. Update stats_by_md5 map to only include kept books
        stats_data
            .stats_by_md5
            .retain(|md5, _| library_md5s.contains(md5));

        let filtered_count = original_count - stats_data.books.len();
        log::info!(
            "Filtered statistics to {} books present in library ({} excluded)",
            stats_data.books.len(),
            filtered_count
        );
    }
}
