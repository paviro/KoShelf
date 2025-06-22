use anyhow::{Result, Context};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::path::Path;
use log::{info, warn, debug};
use std::collections::HashMap;
use chrono::{NaiveDate, Duration, Datelike, DateTime, Utc};
use crate::models::{ReadingStats, WeeklyStats, DailyStats};
use std::fs;
use tempfile::TempDir;

/// Data structure representing a book entry from the statistics database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatBook {
    pub id: i64,
    pub title: String,
    pub authors: String,
    pub notes: Option<i64>,
    pub last_open: Option<i64>,
    pub highlights: Option<i64>,
    pub pages: Option<i64>,
    pub series: String,
    pub language: String,
    pub md5: String,
    pub total_read_time: Option<i64>,
    pub total_read_pages: Option<i64>,
}

impl StatBook {
    /// Calculate additional statistics for this book from page stats
    pub fn calculate_session_stats(&self, page_stats: &[PageStat]) -> BookSessionStats {
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

/// Additional statistics calculated for a book from its reading sessions
#[derive(Debug, Clone)]
pub struct BookSessionStats {
    pub session_count: i64,
    pub average_session_duration: Option<i64>, // in seconds
    pub longest_session_duration: Option<i64>, // in seconds
    pub last_read_date: Option<String>,
    pub reading_speed: Option<f64>, // pages per hour
}

/// Data structure representing a page stat entry from the statistics database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageStat {
    pub id_book: i64,
    pub page: i64,
    pub start_time: i64,
    pub duration: i64,
    pub total_pages: i64,
}

/// Main container for KoReader statistics data
#[derive(Debug, Clone)]
pub struct StatisticsData {
    pub books: Vec<StatBook>,
    pub page_stats: Vec<PageStat>,
    pub stats_by_md5: std::collections::HashMap<String, StatBook>,
}

/// Parser for KoReader statistics database
pub struct StatisticsParser;

impl StatisticsParser {
    /// Parse the statistics database from the given path
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<StatisticsData> {
        info!("Opening statistics database: {:?}", path.as_ref());
        
        // Create a temporary directory for the database copy
        let temp_dir = TempDir::new().with_context(|| "Failed to create temporary directory")?;
        let temp_db_path = temp_dir.path().join("statistics.db");
        
        // Copy the database to the temporary directory
        debug!("Copying database to temporary directory: {:?}", temp_db_path);
        fs::copy(path.as_ref(), &temp_db_path)
            .with_context(|| format!("Failed to copy database from {:?} to {:?}", path.as_ref(), temp_db_path))?;
        
        // Open the temporary database copy with read-only access
        let conn = Connection::open_with_flags(
            &temp_db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY
        ).with_context(|| format!("Failed to open temporary statistics database: {:?}", temp_db_path))?;
        
        // Parse books
        let books = Self::parse_books(&conn)?;
        
        // Parse page stats
        let page_stats = Self::parse_page_stats(&conn)?;
        
        // Create MD5 lookup map
        let mut stats_by_md5 = std::collections::HashMap::new();
        for stat_book in &books {
            stats_by_md5.insert(stat_book.md5.clone(), stat_book.clone());
        }
        
        let stats_data = StatisticsData {
            books,
            page_stats,
            stats_by_md5,
        };
        
        info!("Found {} books and {} page stats in the statistics database!", 
            stats_data.books.len(), stats_data.page_stats.len());
            
        Ok(stats_data)
    }
    
    /// Parse book entries from the database
    fn parse_books(conn: &Connection) -> Result<Vec<StatBook>> {
        let mut stmt = conn.prepare("SELECT id, title, authors, notes, last_open, highlights, pages, series, language, md5, total_read_time, total_read_pages FROM book")?;
        
        let book_iter = stmt.query_map([], |row| {
            Ok(StatBook {
                id: row.get(0)?,
                title: row.get(1)?,
                authors: row.get(2)?,
                notes: row.get(3)?,
                last_open: row.get(4)?,
                highlights: row.get(5)?,
                pages: row.get(6)?,
                series: row.get(7)?,
                language: row.get(8)?,
                md5: row.get(9)?,
                total_read_time: row.get(10)?,
                total_read_pages: row.get(11)?,
            })
        })?;
        
        let mut books = Vec::new();
        for book in book_iter {
            match book {
                Ok(book) => books.push(book),
                Err(e) => warn!("Failed to parse book entry: {}", e),
            }
        }
        
        Ok(books)
    }
    
    /// Parse page stat entries from the database
    fn parse_page_stats(conn: &Connection) -> Result<Vec<PageStat>> {
        let mut stmt = conn.prepare("SELECT id_book, page, start_time, duration, total_pages FROM page_stat_data")?;
        
        let stat_iter = stmt.query_map([], |row| {
            Ok(PageStat {
                id_book: row.get(0)?,
                page: row.get(1)?,
                start_time: row.get(2)?,
                duration: row.get(3)?,
                total_pages: row.get(4)?,
            })
        })?;
        
        let mut page_stats = Vec::new();
        for stat in stat_iter {
            match stat {
                Ok(stat) => page_stats.push(stat),
                Err(e) => warn!("Failed to parse page stat entry: {}", e),
            }
        }
        
        Ok(page_stats)
    }
    
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
        
        // Create daily activity data for heatmap
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
        
        ReadingStats {
            total_read_time,
            total_page_reads,
            longest_read_time_in_day,
            most_pages_in_day,
            weeks,
            daily_activity,
        }
    }
} 