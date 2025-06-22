use chrono::{NaiveDate, Duration, Datelike, DateTime, Utc};
use std::collections::HashMap;

use crate::models::{
    StatBook, PageStat, StatisticsData, BookSessionStats, 
    ReadingStats, WeeklyStats, DailyStats, StreakInfo
};

/// Trait for calculating book session statistics
pub trait BookStatistics {
    fn calculate_session_stats(&self, page_stats: &[PageStat]) -> BookSessionStats;
}

impl BookStatistics for StatBook {
    /// Calculate additional statistics for this book from page stats
    fn calculate_session_stats(&self, page_stats: &[PageStat]) -> BookSessionStats {
        let book_sessions: Vec<&PageStat> = page_stats
            .iter()
            .filter(|stat| stat.id_book == self.id && stat.duration > 0)
            .collect();

        // Calculate actual reading sessions by grouping consecutive page reads
        // Pages read within 30 seconds of each other are considered the same session
        let (session_count, average_session_duration, longest_session_duration) = if !book_sessions.is_empty() {
            let mut sessions: Vec<i64> = Vec::new();
            let mut current_session_duration = 0;
            let mut last_end_time = 0;
            let gap_threshold = 30; // seconds
            
            // Sort sessions by start time
            let mut sorted_sessions = book_sessions.clone();
            sorted_sessions.sort_by_key(|s| s.start_time);
            
            for session in sorted_sessions {
                let session_start = session.start_time;
                let session_end = session.start_time + session.duration;
                
                if last_end_time > 0 && session_start - last_end_time <= gap_threshold {
                    // Continue the current session
                    current_session_duration += session.duration;
                } else {
                    // Start a new session
                    if current_session_duration > 0 {
                        sessions.push(current_session_duration);
                    }
                    current_session_duration = session.duration;
                }
                last_end_time = session_end;
            }
            
            // Don't forget the last session
            if current_session_duration > 0 {
                sessions.push(current_session_duration);
            }
            
            let session_count = sessions.len() as i64;
            let longest_session = sessions.iter().max().copied();
            let average_session = if !sessions.is_empty() {
                let total: i64 = sessions.iter().sum();
                Some(total / session_count)
            } else {
                None
            };
            
            (session_count, average_session, longest_session)
        } else {
            (0, None, None)
        };

        let last_read_date = book_sessions
            .iter()
            .map(|s| s.start_time)
            .max()
            .and_then(|timestamp| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp, 0)
                    .map(|dt| {
                        let current_year = chrono::Utc::now().year();
                        let date_year = dt.year();
                        
                        if date_year == current_year {
                            dt.format("%b %d").to_string()
                        } else {
                            dt.format("%b %d %Y").to_string()
                        }
                    })
            });

        let reading_speed = if let (Some(total_time), Some(total_pages)) = 
            (self.total_read_time, self.total_read_pages) {
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
    /// Calculate reading statistics based on the parsed data
    pub fn calculate_stats(stats_data: &StatisticsData) -> ReadingStats {
        // Initialize overall stats
        let mut total_read_time = 0;
        let mut total_page_reads = 0;
        
        // Maps to track daily stats
        let mut daily_read_time: HashMap<String, i64> = HashMap::new();
        let mut daily_page_reads: HashMap<String, i64> = HashMap::new();
        
        // For weekly stats, we'll organize by ISO week
        let mut weekly_stats: HashMap<(i32, u32), (i64, i64)> = HashMap::new(); // (year, week) -> (read_time, pages_read)
        
        // Process all page stats
        for stat in &stats_data.page_stats {
            // Skip invalid durations
            if stat.duration <= 0 {
                continue;
            }
            
            // Add to overall totals
            total_read_time += stat.duration;
            total_page_reads += 1;
            
            // Convert unix timestamp to date
            // Using the approach that handles timestamps safely
            let date_time = DateTime::<Utc>::from_timestamp(stat.start_time, 0)
                .map(|dt| dt.naive_utc())
                .unwrap_or_else(|| {
                    DateTime::<Utc>::from_timestamp(0, 0).unwrap().naive_utc()
                });
            
            let date_str = date_time.date().format("%Y-%m-%d").to_string();
            
            // Add to daily stats
            *daily_read_time.entry(date_str.clone()).or_insert(0) += stat.duration;
            *daily_page_reads.entry(date_str).or_insert(0) += 1;
            
            // Add to weekly stats
            let year = date_time.year();
            let week = date_time.iso_week().week();
            let key = (year, week);
            let entry = weekly_stats.entry(key).or_insert((0, 0));
            entry.0 += stat.duration;
            entry.1 += 1;
        }
        
        // Find max values for daily stats
        let longest_read_time_in_day = daily_read_time.values().cloned().max().unwrap_or(0);
        let most_pages_in_day = daily_page_reads.values().cloned().max().unwrap_or(0);
        
        // Convert weekly stats to WeeklyStats structs
        let weeks = Self::build_weekly_stats(weekly_stats);
        
        // Create daily activity data for heatmap
        let daily_activity = Self::build_daily_activity(daily_read_time, daily_page_reads);
        
        // Calculate reading streaks
        let (longest_streak, current_streak) = Self::calculate_streaks(&daily_activity);
        
        ReadingStats {
            total_read_time,
            total_page_reads,
            longest_read_time_in_day,
            most_pages_in_day,
            longest_streak,
            current_streak,
            weeks,
            daily_activity,
        }
    }
    
    /// Build weekly statistics from raw weekly data
    fn build_weekly_stats(weekly_stats: HashMap<(i32, u32), (i64, i64)>) -> Vec<WeeklyStats> {
        let mut weeks = Vec::new();
        for ((year, week), (read_time, pages_read)) in weekly_stats {
            // Calculate the start date of the week (Monday)
            // This is a simplified calculation
            let start_date_approx = NaiveDate::from_isoywd_opt(year, week, chrono::Weekday::Mon).unwrap_or_else(|| {
                NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or_else(|| NaiveDate::from_ymd_opt(2020, 1, 1).unwrap())
            });
            
            let end_date = start_date_approx + Duration::days(6); // Sunday
            
            let weekly_stat = WeeklyStats {
                start_date: start_date_approx.format("%Y-%m-%d").to_string(),
                end_date: end_date.format("%Y-%m-%d").to_string(),
                read_time,
                pages_read,
                avg_pages_per_day: pages_read as f64 / 7.0,
                avg_read_time_per_day: read_time as f64 / 7.0,
            };
            
            weeks.push(weekly_stat);
        }
        
        // Sort weeks by start date (newest first)
        weeks.sort_by(|a, b| b.start_date.cmp(&a.start_date));
        weeks
    }
    
    /// Build daily activity data from daily stats maps
    fn build_daily_activity(
        daily_read_time: HashMap<String, i64>, 
        daily_page_reads: HashMap<String, i64>
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
    fn calculate_streaks(daily_activity: &[DailyStats]) -> (StreakInfo, StreakInfo) {
        if daily_activity.is_empty() {
            return (
                StreakInfo::new(0, None, None),
                StreakInfo::new(0, None, None)
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
                StreakInfo::new(0, None, None)
            );
        }
        
        // Sort dates chronologically
        let mut sorted_dates = reading_dates;
        sorted_dates.sort();
        
        let today = Utc::now().naive_utc().date();
        
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
        let longest_streak_info = if let Some(&(length, start, end)) = streaks.iter().max_by_key(|&&(len, _, _)| len) {
            StreakInfo::new(
                length,
                Some(start.format("%Y-%m-%d").to_string()),
                Some(end.format("%Y-%m-%d").to_string())
            )
        } else {
            StreakInfo::new(0, None, None)
        };
        
        // Find current streak (streak that includes today or recent days)
        let current_streak_info = if let Some(&last_reading_date) = sorted_dates.last() {
            let days_since_last_read = (today - last_reading_date).num_days();
            
            if days_since_last_read <= 1 {
                // Last read was today or yesterday, find the streak that ends with last_reading_date
                if let Some(&(length, start, _)) = streaks.iter().find(|&&(_, _, end)| end == last_reading_date) {
                    StreakInfo::new(
                        length,
                        Some(start.format("%Y-%m-%d").to_string()),
                        None // Current streak doesn't have an end date
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
} 