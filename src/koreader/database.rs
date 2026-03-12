use crate::models::{PageStat, StatBook, StatisticsData};
use anyhow::{Context, Result};
use log::{debug, info, warn};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use tempfile::TempDir;

/// Parser for KoReader statistics database
pub struct StatisticsParser;

impl StatisticsParser {
    /// Parse the statistics database from the given path
    pub async fn parse<P: AsRef<Path>>(path: P) -> Result<StatisticsData> {
        info!("Opening statistics database: {:?}", path.as_ref());

        // Create a temporary directory for the database copy
        let temp_dir = TempDir::new().with_context(|| "Failed to create temporary directory")?;
        let temp_db_path = temp_dir.path().join("statistics.db");

        // Copy the database to the temporary directory
        debug!(
            "Copying database to temporary directory: {:?}",
            temp_db_path
        );
        fs::copy(path.as_ref(), &temp_db_path).with_context(|| {
            format!(
                "Failed to copy database from {:?} to {:?}",
                path.as_ref(),
                temp_db_path
            )
        })?;

        // Open the temporary database copy with read-only access
        let url = format!("sqlite:{}?mode=ro", temp_db_path.display());
        let options = SqliteConnectOptions::from_str(&url)
            .with_context(|| format!("Failed to parse statistics DB URL for {:?}", temp_db_path))?;

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .with_context(|| {
                format!(
                    "Failed to open temporary statistics database: {:?}",
                    temp_db_path
                )
            })?;

        // Parse books
        let books = Self::parse_books(&pool).await?;

        // Parse page stats
        let page_stats = Self::parse_page_stats(&pool).await?;

        pool.close().await;

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

        info!(
            "Found {} books and {} page stats in the statistics database!",
            stats_data.books.len(),
            stats_data.page_stats.len()
        );

        Ok(stats_data)
    }

    /// Parse book entries from the database
    async fn parse_books(pool: &SqlitePool) -> Result<Vec<StatBook>> {
        let rows = sqlx::query(
            "SELECT id, title, authors, notes, last_open, highlights, pages, md5, total_read_time, total_read_pages FROM book",
        )
        .fetch_all(pool)
        .await
        .context("Failed to query book entries")?;

        let mut books = Vec::new();
        for row in rows {
            match Self::row_to_stat_book(&row) {
                Ok(book) => books.push(book),
                Err(e) => warn!("Failed to parse book entry: {}", e),
            }
        }

        Ok(books)
    }

    fn row_to_stat_book(row: &sqlx::sqlite::SqliteRow) -> Result<StatBook> {
        Ok(StatBook {
            id: row.try_get("id").context("id")?,
            title: row.try_get("title").context("title")?,
            authors: row.try_get("authors").context("authors")?,
            notes: row.try_get("notes").context("notes")?,
            last_open: row.try_get("last_open").context("last_open")?,
            highlights: row.try_get("highlights").context("highlights")?,
            pages: row.try_get("pages").context("pages")?,
            md5: row.try_get("md5").context("md5")?,
            content_type: None,
            total_read_time: row.try_get("total_read_time").context("total_read_time")?,
            total_read_pages: row
                .try_get("total_read_pages")
                .context("total_read_pages")?,
            completions: None, // Will be populated later by completion detection
        })
    }

    /// Parse page stat entries from the database
    async fn parse_page_stats(pool: &SqlitePool) -> Result<Vec<PageStat>> {
        // Query the rescaled view so that page numbers are already expressed in the current
        // pagination of each document. The `page_stat` view has the same four columns we
        // actually use (id_book, page, start_time, duration). See KOReader's Lua code for
        // the precise definition.
        let rows = sqlx::query("SELECT id_book, page, start_time, duration FROM page_stat")
            .fetch_all(pool)
            .await
            .context("Failed to query page stat entries")?;

        let mut page_stats = Vec::new();
        for row in rows {
            match Self::row_to_page_stat(&row) {
                Ok(stat) => page_stats.push(stat),
                Err(e) => warn!("Failed to parse page stat entry: {}", e),
            }
        }

        Ok(page_stats)
    }

    fn row_to_page_stat(row: &sqlx::sqlite::SqliteRow) -> Result<PageStat> {
        Ok(PageStat {
            id_book: row.try_get("id_book").context("id_book")?,
            page: row.try_get("page").context("page")?,
            start_time: row.try_get("start_time").context("start_time")?,
            duration: row.try_get("duration").context("duration")?,
        })
    }
}
