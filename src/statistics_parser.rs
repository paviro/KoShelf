use anyhow::{Result, Context};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::path::Path;
use log::{info, warn};
use std::collections::HashMap;
use chrono::{NaiveDate, Duration, Datelike, DateTime, Utc};
use crate::models::{ReadingStats, WeeklyStats, DailyStats};

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
}

/// Parser for KoReader statistics database
pub struct StatisticsParser;

impl StatisticsParser {
    /// Parse the statistics database from the given path
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<StatisticsData> {
        info!("Opening statistics database: {:?}", path.as_ref());
        
        // Use immutable mode without file locks
        // Using URI format to specify immutable=1 mode
        let uri = format!("file:{}?immutable=1&mode=ro", path.as_ref().display());
        let conn = Connection::open_with_flags(
            uri,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI
        ).with_context(|| format!("Failed to open statistics database: {:?}", path.as_ref()))?;
        
        // Parse books
        let books = Self::parse_books(&conn)?;
        
        // Parse page stats
        let page_stats = Self::parse_page_stats(&conn)?;
        
        let stats_data = StatisticsData {
            books,
            page_stats,
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