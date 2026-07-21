use crate::source::koreader::types::{PageStat, StatBook, StatisticsData};
use crate::source::sqlite_snapshot::copy_sqlite_snapshot;
use anyhow::{Context, Result};
use log::{debug, info, warn};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tempfile::TempDir;

/// Reads KOReader's `statistics.sqlite3` database to extract book metadata and page-level reading history.
pub struct StatisticsParser;

/// Clear a read-only bit inherited from the source file via `fs::copy`.
fn make_writable(path: &Path) -> Result<()> {
    let mut permissions = std::fs::metadata(path)
        .with_context(|| format!("Failed to read permissions of {:?}", path))?
        .permissions();
    if permissions.readonly() {
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        std::fs::set_permissions(path, permissions)
            .with_context(|| format!("Failed to make {:?} writable", path))?;
    }
    Ok(())
}

impl StatisticsParser {
    /// Copy the database to a temp file (to avoid locking the live DB) and parse all books and page stats.
    #[cfg(test)]
    pub async fn parse<P: AsRef<Path>>(path: P) -> Result<StatisticsData> {
        Self::parse_merged(&[path.as_ref().to_path_buf()]).await
    }

    /// Parse one or more statistics databases, merging additional ones into the
    /// first using KOReader's own sync semantics (statistics.koplugin `onSync`):
    /// books are matched by (title, authors, md5), raw `page_stat_data` rows are
    /// merged with MAX(duration) on identical (id_book, page, start_time), and
    /// per-book totals are recomputed afterwards. Merging raw rows (rather than
    /// the rescaled `page_stat` view) lets the view rescale reading history to a
    /// single pagination even when devices disagree on page counts.
    pub async fn parse_merged(paths: &[PathBuf]) -> Result<StatisticsData> {
        let (primary, rest) = paths
            .split_first()
            .context("No statistics database paths provided")?;

        info!("Opening statistics database: {:?}", primary);

        let temp_dir = TempDir::new().with_context(|| "Failed to create temporary directory")?;
        let temp_db_path = temp_dir.path().join("statistics.db");

        debug!(
            "Copying database to temporary directory: {:?}",
            temp_db_path
        );
        copy_sqlite_snapshot(primary, &temp_db_path)?;
        if !rest.is_empty() {
            // fs::copy preserves permission bits, but the merge writes to the
            // temp copies — a read-only source (e.g. Syncthing permission
            // sync) must not make them read-only too.
            make_writable(&temp_db_path)?;
        }

        // The temp copy is opened writable only when there are databases to merge in.
        let mode = if rest.is_empty() { "ro" } else { "rw" };
        let url = format!("sqlite:{}?mode={}", temp_db_path.display(), mode);
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

        if !rest.is_empty() {
            Self::ensure_mergeable_schema(&pool, "main", primary).await?;

            for (index, income) in rest.iter().enumerate() {
                info!("Merging statistics database: {:?}", income);
                let income_copy = temp_dir.path().join(format!("income_{index}.db"));
                copy_sqlite_snapshot(income, &income_copy)?;
                make_writable(&income_copy)?;
                Self::merge_attached_db(&pool, &income_copy, income).await?;
            }

            // KOReader's final sync step: recompute totals from the merged page
            // stats. Books without raw rows (history trimmed in KOReader) keep
            // their stored totals instead of being zeroed out.
            sqlx::query(
                "UPDATE book SET (total_read_pages, total_read_time) = \
                 (SELECT count(DISTINCT page), sum(duration) FROM page_stat WHERE id_book = book.id) \
                 WHERE EXISTS (SELECT 1 FROM page_stat WHERE id_book = book.id)",
            )
            .execute(&pool)
            .await
            .context("Failed to recompute book totals after merging statistics databases")?;
        }

        let mut books = Self::parse_books(&pool).await?;
        let mut page_stats = Self::parse_page_stats(&pool).await?;
        pool.close().await;

        Self::deduplicate_by_md5(&mut books, &mut page_stats);

        let mut stats_by_md5 = HashMap::new();
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

    /// Merging needs `page_stat_data` (raw rows with total_pages and the
    /// UNIQUE(id_book, page, start_time) constraint), present since KOReader
    /// schema 20201010 (release 2020.10).
    async fn ensure_mergeable_schema(pool: &SqlitePool, schema: &str, source: &Path) -> Result<()> {
        let row = sqlx::query(
            "SELECT 1 FROM pragma_table_list \
             WHERE schema = ?1 AND name = 'page_stat_data' AND type = 'table'",
        )
        .bind(schema)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("Failed to inspect schema of {:?}", source))?;
        if row.is_none() {
            anyhow::bail!(
                "Statistics database {:?} uses an unsupported KOReader schema (missing page_stat_data table). \
                 Merging multiple databases requires KOReader 2020.10 or newer.",
                source
            );
        }
        Ok(())
    }

    /// Merge one additional statistics database into the primary temp copy.
    /// SQL adapted from KOReader statistics.koplugin `onSync` (2-way merge; the
    /// cached-DB deletion propagation only applies to cloud sync).
    async fn merge_attached_db(pool: &SqlitePool, income_copy: &Path, source: &Path) -> Result<()> {
        sqlx::query("ATTACH DATABASE ?1 AS income_db")
            .bind(income_copy.display().to_string())
            .execute(pool)
            .await
            .with_context(|| format!("Failed to attach statistics database {:?}", source))?;

        let merge_result = Self::run_merge_statements(pool, source).await;

        let detach_result = sqlx::query("DETACH DATABASE income_db")
            .execute(pool)
            .await
            .with_context(|| format!("Failed to detach statistics database {:?}", source));

        merge_result.and(detach_result.map(|_| ()))
    }

    async fn run_merge_statements(pool: &SqlitePool, source: &Path) -> Result<()> {
        Self::ensure_mergeable_schema(pool, "income_db", source).await?;

        // Books are matched NULL-safely rather than by KOReader's plain
        // (title, authors, md5) tuple: modern KOReader can still insert
        // NULL-authors rows (they are distinct under its UNIQUE index, so
        // normalizing them to '' here could violate that index and abort the
        // merge), and plain row-value comparison silently skips NULL columns.
        // The macro splices the shared predicate at compile time so every
        // statement stays a static string literal.
        macro_rules! with_book_match {
            ($before:literal, $after:literal) => {
                concat!(
                    $before,
                    "b.title IS i.title \
                     AND IFNULL(b.authors, '') = IFNULL(i.authors, '') \
                     AND b.md5 IS i.md5",
                    $after
                )
            };
        }

        let statements: [&'static str; 6] = [
            // Most recently opened wins for last_open and pages, so the
            // page_stat view rescales to the pagination of the device the
            // book was last read on (mirrors deduplicate_by_md5's policy).
            with_book_match!(
                "UPDATE book AS b \
                 SET last_open = i.last_open, pages = i.pages \
                 FROM income_db.book AS i \
                 WHERE ",
                " AND IFNULL(i.last_open, 0) > IFNULL(b.last_open, 0)"
            ),
            // Totals are recomputed from merged sessions afterwards, but only
            // for books that still have raw rows; MAX them here so stored
            // legacy totals (raw history trimmed in KOReader) survive.
            with_book_match!(
                "UPDATE book AS b \
                 SET notes = MAX(IFNULL(b.notes, 0), IFNULL(i.notes, 0)), \
                     highlights = MAX(IFNULL(b.highlights, 0), IFNULL(i.highlights, 0)), \
                     total_read_time = MAX(IFNULL(b.total_read_time, 0), IFNULL(i.total_read_time, 0)), \
                     total_read_pages = MAX(IFNULL(b.total_read_pages, 0), IFNULL(i.total_read_pages, 0)) \
                 FROM income_db.book AS i \
                 WHERE ",
                ""
            ),
            // Only the columns KoShelf reads.
            with_book_match!(
                "INSERT INTO book (title, authors, notes, last_open, highlights, pages, md5, \
                                   total_read_time, total_read_pages) \
                 SELECT title, authors, notes, last_open, highlights, pages, md5, \
                        total_read_time, total_read_pages \
                 FROM income_db.book AS i \
                 WHERE NOT EXISTS (SELECT 1 FROM book b WHERE ",
                ")"
            ),
            // Book ids are independent autoincrements per database, so map
            // income ids to primary ids before copying page stats. MIN +
            // GROUP BY guarantees one primary book per income book even if
            // several primary rows match (e.g. NULL- and ''-authors variants).
            with_book_match!(
                "CREATE TEMP TABLE book_id_map AS \
                 SELECT MIN(b.id) AS mid, i.id AS iid \
                 FROM book b \
                 INNER JOIN income_db.book i \
                 ON ",
                " GROUP BY i.id"
            ),
            "INSERT INTO page_stat_data (id_book, page, start_time, duration, total_pages) \
                 SELECT map.mid, page, start_time, duration, total_pages \
                 FROM income_db.page_stat_data \
                 INNER JOIN book_id_map AS map ON id_book = map.iid \
             ON CONFLICT(id_book, page, start_time) DO UPDATE SET \
                 duration = MAX(duration, excluded.duration)",
            "DROP TABLE book_id_map",
        ];

        for statement in statements {
            sqlx::query(statement)
                .execute(pool)
                .await
                .with_context(|| format!("Failed to merge statistics database {:?}", source))?;
        }

        Ok(())
    }

    /// Parse book entries from the database
    async fn parse_books(pool: &SqlitePool) -> Result<Vec<StatBook>> {
        let rows = sqlx::query(
            "SELECT id, title, IFNULL(authors, '') AS authors, notes, last_open, highlights, pages, md5, total_read_time, total_read_pages FROM book",
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
            completions: None,
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

    /// Merge book entries that share the same md5 (same physical file).
    ///
    /// KoReader indexes books by (title, authors, md5), so editing author
    /// metadata creates a second row for the same file. We pick the most
    /// recently opened entry as canonical, sum reading stats, and remap
    /// page_stats so downstream code sees a single book per md5.
    fn deduplicate_by_md5(books: &mut Vec<StatBook>, page_stats: &mut [PageStat]) {
        let mut md5_groups: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, book) in books.iter().enumerate() {
            md5_groups.entry(book.md5.clone()).or_default().push(idx);
        }

        let mut id_remap: HashMap<i64, i64> = HashMap::new();
        let mut indices_to_remove: HashSet<usize> = HashSet::new();

        for indices in md5_groups.values() {
            if indices.len() <= 1 {
                continue;
            }

            // Canonical = most recently opened, tie-break by highest id
            let &canonical_idx = indices
                .iter()
                .max_by_key(|&&i| (books[i].last_open.unwrap_or(0), books[i].id))
                .unwrap();

            for &idx in indices {
                if idx == canonical_idx {
                    continue;
                }

                if let Some(time) = books[idx].total_read_time {
                    *books[canonical_idx].total_read_time.get_or_insert(0) += time;
                }
                if let Some(pages) = books[idx].total_read_pages {
                    *books[canonical_idx].total_read_pages.get_or_insert(0) += pages;
                }
                if let Some(n) = books[idx].notes {
                    let canon = books[canonical_idx].notes.get_or_insert(0);
                    *canon = (*canon).max(n);
                }
                if let Some(h) = books[idx].highlights {
                    let canon = books[canonical_idx].highlights.get_or_insert(0);
                    *canon = (*canon).max(h);
                }

                id_remap.insert(books[idx].id, books[canonical_idx].id);
                indices_to_remove.insert(idx);
            }
        }

        if indices_to_remove.is_empty() {
            return;
        }

        for stat in page_stats.iter_mut() {
            if let Some(&new_id) = id_remap.get(&stat.id_book) {
                stat.id_book = new_id;
            }
        }

        let mut remove_sorted: Vec<usize> = indices_to_remove.into_iter().collect();
        remove_sorted.sort_unstable_by(|a, b| b.cmp(a));
        for idx in remove_sorted {
            books.remove(idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StatisticsParser;
    use sqlx::Executor;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    #[tokio::test]
    async fn parse_reads_rows_from_wal_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("statistics.sqlite3");
        let url = format!("sqlite:{}?mode=rwc", db_path.display());
        let options = SqliteConnectOptions::from_str(&url).expect("sqlite options");
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("sqlite pool");

        pool.execute("PRAGMA journal_mode=WAL")
            .await
            .expect("wal mode");
        pool.execute("PRAGMA wal_autocheckpoint=0")
            .await
            .expect("disable autocheckpoint");
        pool.execute(
            "CREATE TABLE book (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                authors TEXT NOT NULL,
                notes INTEGER,
                last_open INTEGER,
                highlights INTEGER,
                pages INTEGER,
                md5 TEXT NOT NULL,
                total_read_time INTEGER,
                total_read_pages INTEGER
            )",
        )
        .await
        .expect("book table");
        pool.execute(
            "CREATE TABLE page_stat (
                id_book INTEGER NOT NULL,
                page INTEGER NOT NULL,
                start_time INTEGER NOT NULL,
                duration INTEGER NOT NULL
            )",
        )
        .await
        .expect("page_stat table");
        pool.execute(
            "INSERT INTO book
             (id, title, authors, notes, last_open, highlights, pages, md5, total_read_time, total_read_pages)
             VALUES (1, 'Wal Book', 'Author', 0, 10, 0, 42, 'abc123', 60, 3)",
        )
        .await
        .expect("book row");
        pool.execute(
            "INSERT INTO page_stat (id_book, page, start_time, duration)
             VALUES (1, 2, 1000, 30)",
        )
        .await
        .expect("page_stat row");

        assert!(
            db_path.with_file_name("statistics.sqlite3-wal").exists(),
            "test setup should leave rows in WAL"
        );

        let data = StatisticsParser::parse(&db_path)
            .await
            .expect("parse stats db");

        assert_eq!(data.books.len(), 1);
        assert_eq!(data.books[0].title, "Wal Book");
        assert_eq!(data.page_stats.len(), 1);
        assert_eq!(data.page_stats[0].page, 2);

        pool.close().await;
    }
}
