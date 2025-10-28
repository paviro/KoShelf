use anyhow::{Result, Context};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use log::{info, warn, debug};
use crate::models::{StatBook, PageStat, StatisticsData};
use crate::statistics::StatisticsCalculator;
use crate::time_config::TimeConfig;
use std::fs;
use tempfile::TempDir;



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
        let mut stmt = conn.prepare("SELECT id, title, authors, notes, last_open, highlights, pages, md5, total_read_time, total_read_pages FROM book")?;
        
        let book_iter = stmt.query_map([], |row| {
            Ok(StatBook {
                id: row.get(0)?,
                title: row.get(1)?,
                authors: row.get(2)?,
                notes: row.get(3)?,
                last_open: row.get(4)?,
                highlights: row.get(5)?,
                pages: row.get(6)?,
                md5: row.get(7)?,
                total_read_time: row.get(8)?,
                total_read_pages: row.get(9)?,
                completions: None, // Will be populated later by completion detection
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
        // Query the rescaled view so that page numbers are already expressed in the current
        // pagination of each document. The `page_stat` view has the same four columns we
        // actually use (id_book, page, start_time, duration). See KOReader's Lua code for
        // the precise definition.
        let mut stmt = conn.prepare("SELECT id_book, page, start_time, duration FROM page_stat")?;
        
        let stat_iter = stmt.query_map([], |row| {
            Ok(PageStat {
                id_book: row.get(0)?,
                page: row.get(1)?,
                start_time: row.get(2)?,
                duration: row.get(3)?,
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
    pub fn calculate_stats(stats_data: &mut StatisticsData, time_config: &TimeConfig) -> crate::models::ReadingStats {
        StatisticsCalculator::calculate_stats(stats_data, time_config)
    }
    

} 