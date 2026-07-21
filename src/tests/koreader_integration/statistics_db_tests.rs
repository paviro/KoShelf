use super::lua_mocks::compose_lua_mocks;
use super::*;
use crate::source::koreader::StatisticsParser;
use mlua::{Lua, LuaOptions, StdLib, Table};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{AssertSqlSafe, Executor, Row, SqlitePool};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Debug)]
struct StatisticsLuaArtifacts {
    statements: Vec<String>,
    totals_query: String,
}

#[derive(Debug, PartialEq, Eq)]
struct ColumnDescriptor {
    name: String,
    data_type: String,
    notnull: bool,
    default_value: Option<String>,
}

impl ColumnDescriptor {
    fn new(name: &str, data_type: &str, notnull: bool, default_value: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            data_type: data_type.to_string(),
            notnull,
            default_value: default_value.map(|s| s.to_string()),
        }
    }
}

struct TestDatabase {
    _temp_dir: TempDir,
    pool: SqlitePool,
    path: PathBuf,
}

impl TestDatabase {
    async fn new(statements: &[String]) -> Self {
        let temp_dir =
            TempDir::new().expect("Failed to create temporary directory for statistics DB");
        let path = temp_dir.path().join("statistics.sqlite3");

        let url = format!("sqlite:{}?mode=rwc", path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .expect("Failed to open temporary statistics DB");

        for stmt in statements {
            // These schema statements are captured from KoReader's Lua test fixture,
            // so they are inherently runtime SQL rather than literals.
            pool.execute(AssertSqlSafe(stmt.clone()))
                .await
                .unwrap_or_else(|err| {
                    panic!(
                        "Failed to execute KoReader schema statement:\n{}\nError: {}",
                        stmt, err
                    )
                });
        }

        pool.execute("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE;")
            .await
            .expect("Failed to normalize SQLite journal mode for test database");

        Self {
            _temp_dir: temp_dir,
            pool,
            path,
        }
    }
}

#[tokio::test]
async fn test_statistics_schema_matches_koreader() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db = TestDatabase::new(&artifacts.statements).await;

    let data_columns = fetch_table_info(&db.pool, "page_stat_data").await;
    assert_eq!(
        data_columns,
        vec![
            ColumnDescriptor::new("id_book", "INTEGER", false, None),
            ColumnDescriptor::new("page", "INTEGER", true, Some("0")),
            ColumnDescriptor::new("start_time", "INTEGER", true, Some("0")),
            ColumnDescriptor::new("duration", "INTEGER", true, Some("0")),
            ColumnDescriptor::new("total_pages", "INTEGER", true, Some("0")),
        ],
        "KoReader page_stat_data schema changed. Update parser expectations."
    );

    let view_columns = fetch_table_info(&db.pool, "page_stat").await;
    let view_column_names: Vec<&str> = view_columns.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        view_column_names,
        vec!["id_book", "page", "start_time", "duration"],
        "page_stat view columns changed unexpectedly"
    );

    let view_sql: String =
        sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE type='view' AND name='page_stat'")
            .fetch_one(&db.pool)
            .await
            .expect("Unable to read page_stat view SQL");

    let lowered = view_sql.to_lowercase();
    for snippet in [
        "create view",
        "first_page + idx - 1 as page",
        "duration / (last_page - first_page + 1) as duration",
        "((page - 1) * pages) / total_pages + 1 as first_page",
        "join (select number as idx from numbers)",
    ] {
        assert!(
            lowered.contains(snippet),
            "Expected KoReader view definition to contain '{}', actual SQL:\n{}",
            snippet,
            view_sql
        );
    }
}

#[tokio::test]
async fn test_statistics_totals_query_matches_parser() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db = TestDatabase::new(&artifacts.statements).await;
    seed_sample_statistics(&db.pool).await;

    let query = format_percent_d(&artifacts.totals_query, &[1]);
    // This query is captured from KoReader's Lua code and formatted with a
    // numeric book id above, so it cannot be represented as a local SQL literal.
    let row = sqlx::query(AssertSqlSafe(query))
        .fetch_one(&db.pool)
        .await
        .expect("KoReader totals query failed");
    let expected_pages: i64 = row.get(0);
    let expected_duration: i64 = row.get(1);

    let stats_data = StatisticsParser::parse(&db.path)
        .await
        .expect("Rust parser failed to read statistics DB");
    let mut unique_pages = HashSet::new();
    let mut total_duration = 0;
    for stat in stats_data.page_stats.iter().filter(|ps| ps.id_book == 1) {
        unique_pages.insert(stat.page);
        total_duration += stat.duration;
    }

    assert_eq!(
        unique_pages.len() as i64,
        expected_pages,
        "Rust unique page counting diverged from KoReader SQL"
    );
    assert_eq!(
        total_duration, expected_duration,
        "Rust duration aggregation diverged from KoReader SQL"
    );
}

async fn fetch_table_info(pool: &SqlitePool, table: &str) -> Vec<ColumnDescriptor> {
    let pragma = match table {
        "page_stat_data" => "PRAGMA table_info(page_stat_data)",
        "page_stat" => "PRAGMA table_info(page_stat)",
        _ => unreachable!("unexpected statistics table: {table}"),
    };
    let rows = sqlx::query(pragma)
        .fetch_all(pool)
        .await
        .expect("Failed to inspect schema");

    rows.iter()
        .map(|row| ColumnDescriptor {
            name: row.get(1),
            data_type: row.get(2),
            notnull: row.get::<i32, _>(3) != 0,
            default_value: row.get(4),
        })
        .collect()
}

async fn seed_sample_statistics(pool: &SqlitePool) {
    sqlx::query(
        "INSERT INTO book (id, title, authors, notes, last_open, highlights, pages, series, language, md5, total_read_time, total_read_pages)
         VALUES (?1, ?2, ?3, NULL, NULL, NULL, ?4, NULL, ?5, ?6, NULL, NULL)",
    )
    .bind(1_i64)
    .bind("Test Book")
    .bind("Test Author")
    .bind(300_i64)
    .bind("en")
    .bind("test-md5")
    .execute(pool)
    .await
    .expect("Failed to seed book row");

    let inserts: [(i64, i64, i64, i64, i64); 2] = [(1, 1, 1_000, 90, 100), (1, 25, 2_000, 60, 120)];

    for (id_book, page, start_time, duration, total_pages) in inserts {
        sqlx::query(
            "INSERT INTO page_stat_data (id_book, page, start_time, duration, total_pages)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(id_book)
        .bind(page)
        .bind(start_time)
        .bind(duration)
        .bind(total_pages)
        .execute(pool)
        .await
        .expect("Failed to seed page_stat_data row");
    }
}

fn load_statistics_artifacts(lua: &Lua, root: &Path) -> StatisticsLuaArtifacts {
    let script = build_statistics_lua_script(root);
    let result: Table = lua
        .load(&script)
        .eval()
        .expect("Failed to bootstrap KoReader statistics module");

    let executed_table: Table = result
        .get("executed_sql")
        .expect("Missing executed_sql results from Lua");
    let statements = executed_table
        .sequence_values::<String>()
        .collect::<mlua::Result<Vec<_>>>()
        .expect("Failed to collect KoReader schema statements");
    let totals_query: String = result
        .get("totals_query")
        .expect("Failed to capture KoReader totals query");

    StatisticsLuaArtifacts {
        statements,
        totals_query,
    }
}

fn format_percent_d(template: &str, values: &[i64]) -> String {
    let mut formatted = template.to_string();
    for value in values {
        if let Some(idx) = formatted.find("%d") {
            formatted.replace_range(idx..idx + 2, &value.to_string());
        } else {
            panic!(
                "Template '{}' is missing enough placeholders for provided values",
                template
            );
        }
    }

    assert!(
        !formatted.contains("%d"),
        "Unused %d placeholders remain after formatting SQL: {}",
        formatted
    );
    formatted
}

#[tokio::test]
async fn test_statistics_parser_deduplicates_by_md5() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db = TestDatabase::new(&artifacts.statements).await;

    // Seed two books with the same md5 but different authors (simulates metadata edit)
    for (id, authors, last_open, read_time, read_pages) in [
        (1_i64, "Original Author", 100_i64, 5000_i64, 80_i64),
        (2_i64, "Updated Author\nTranslator", 200_i64, 100_i64, 2_i64),
    ] {
        sqlx::query(
            "INSERT INTO book (id, title, authors, notes, last_open, highlights, pages, series, language, md5, total_read_time, total_read_pages)
             VALUES (?1, ?2, ?3, 0, ?4, 0, 300, NULL, 'en', 'shared-md5', ?5, ?6)",
        )
        .bind(id)
        .bind("Shared Book")
        .bind(authors)
        .bind(last_open)
        .bind(read_time)
        .bind(read_pages)
        .execute(&db.pool)
        .await
        .expect("Failed to seed book");
    }

    // Seed page_stats under both book IDs
    for (id_book, page, start_time, duration, total_pages) in [
        (1_i64, 1_i64, 1000_i64, 60_i64, 300_i64),
        (1, 2, 1060, 60, 300),
        (2, 3, 2000, 30, 300),
    ] {
        sqlx::query(
            "INSERT INTO page_stat_data (id_book, page, start_time, duration, total_pages)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(id_book)
        .bind(page)
        .bind(start_time)
        .bind(duration)
        .bind(total_pages)
        .execute(&db.pool)
        .await
        .expect("Failed to seed page_stat_data");
    }

    let stats = StatisticsParser::parse(&db.path)
        .await
        .expect("Parser failed");

    // Should have exactly one book after deduplication
    assert_eq!(stats.books.len(), 1, "duplicates should be merged into one");
    let book = &stats.books[0];
    assert_eq!(book.md5, "shared-md5");
    assert_eq!(book.id, 2, "canonical = highest last_open");
    assert_eq!(book.authors, "Updated Author\nTranslator");
    assert_eq!(book.total_read_time, Some(5100), "read times summed");
    assert_eq!(book.total_read_pages, Some(82), "read pages summed");

    // All page_stats should reference the canonical book ID
    assert_eq!(stats.page_stats.len(), 3);
    for ps in &stats.page_stats {
        assert_eq!(ps.id_book, 2, "page_stats remapped to canonical id");
    }

    // stats_by_md5 should also reflect the merged entry
    assert_eq!(stats.stats_by_md5.len(), 1);
    assert!(stats.stats_by_md5.contains_key("shared-md5"));
}

async fn seed_book_row(
    pool: &SqlitePool,
    id: i64,
    title: &str,
    authors: &str,
    last_open: i64,
    pages: i64,
    md5: &str,
) {
    seed_book_row_full(pool, id, title, Some(authors), last_open, pages, md5, 0, 0).await;
}

#[allow(clippy::too_many_arguments)]
async fn seed_book_row_full(
    pool: &SqlitePool,
    id: i64,
    title: &str,
    authors: Option<&str>,
    last_open: i64,
    pages: i64,
    md5: &str,
    total_read_time: i64,
    total_read_pages: i64,
) {
    sqlx::query(
        "INSERT INTO book (id, title, authors, notes, last_open, highlights, pages, series, language, md5, total_read_time, total_read_pages)
         VALUES (?1, ?2, ?3, 0, ?4, 0, ?5, NULL, 'en', ?6, ?7, ?8)",
    )
    .bind(id)
    .bind(title)
    .bind(authors)
    .bind(last_open)
    .bind(pages)
    .bind(md5)
    .bind(total_read_time)
    .bind(total_read_pages)
    .execute(pool)
    .await
    .expect("Failed to seed book");
}

async fn seed_page_stat_row(
    pool: &SqlitePool,
    id_book: i64,
    page: i64,
    start_time: i64,
    duration: i64,
    total_pages: i64,
) {
    sqlx::query(
        "INSERT INTO page_stat_data (id_book, page, start_time, duration, total_pages)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(id_book)
    .bind(page)
    .bind(start_time)
    .bind(duration)
    .bind(total_pages)
    .execute(pool)
    .await
    .expect("Failed to seed page_stat_data");
}

#[tokio::test]
async fn test_parse_merged_combines_disjoint_books() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db_a = TestDatabase::new(&artifacts.statements).await;
    let db_b = TestDatabase::new(&artifacts.statements).await;

    seed_book_row(&db_a.pool, 1, "Book A", "Author A", 100, 100, "md5-a").await;
    seed_page_stat_row(&db_a.pool, 1, 1, 1_000, 60, 100).await;
    seed_book_row(&db_b.pool, 1, "Book B", "Author B", 200, 100, "md5-b").await;
    seed_page_stat_row(&db_b.pool, 1, 2, 2_000, 30, 100).await;

    let stats = StatisticsParser::parse_merged(&[db_a.path.clone(), db_b.path.clone()])
        .await
        .expect("merge failed");

    assert_eq!(stats.books.len(), 2, "disjoint books should both survive");
    let book_a = stats.stats_by_md5.get("md5-a").expect("md5-a present");
    assert_eq!(book_a.total_read_time, Some(60));
    assert_eq!(book_a.total_read_pages, Some(1));
    let book_b = stats.stats_by_md5.get("md5-b").expect("md5-b present");
    assert_eq!(book_b.total_read_time, Some(30));
    assert_eq!(book_b.total_read_pages, Some(1));
    assert_eq!(stats.page_stats.len(), 2);
}

#[tokio::test]
async fn test_parse_merged_combines_sessions_of_shared_book() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db_a = TestDatabase::new(&artifacts.statements).await;
    let db_b = TestDatabase::new(&artifacts.statements).await;

    seed_book_row(&db_a.pool, 1, "Shared", "Author", 100, 100, "md5-s").await;
    seed_page_stat_row(&db_a.pool, 1, 1, 1_000, 60, 100).await;
    // Same book on device B under a different autoincrement id
    seed_book_row(&db_b.pool, 7, "Shared", "Author", 200, 100, "md5-s").await;
    seed_page_stat_row(&db_b.pool, 7, 2, 2_000, 30, 100).await;

    let stats = StatisticsParser::parse_merged(&[db_a.path.clone(), db_b.path.clone()])
        .await
        .expect("merge failed");

    assert_eq!(
        stats.books.len(),
        1,
        "same (title, authors, md5) should merge"
    );
    let book = &stats.books[0];
    assert_eq!(book.last_open, Some(200), "most recent last_open wins");
    assert_eq!(
        book.total_read_time,
        Some(90),
        "durations from both devices"
    );
    assert_eq!(
        book.total_read_pages,
        Some(2),
        "distinct pages from both devices"
    );
    assert_eq!(stats.page_stats.len(), 2);
    assert!(stats.page_stats.iter().all(|ps| ps.id_book == book.id));
}

#[tokio::test]
async fn test_parse_merged_overlapping_sessions_keep_max_duration() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db_a = TestDatabase::new(&artifacts.statements).await;
    let db_b = TestDatabase::new(&artifacts.statements).await;

    seed_book_row(&db_a.pool, 1, "Shared", "Author", 100, 100, "md5-s").await;
    seed_page_stat_row(&db_a.pool, 1, 1, 1_000, 60, 100).await;
    // Same session recorded on both devices with differing durations
    seed_book_row(&db_b.pool, 1, "Shared", "Author", 100, 100, "md5-s").await;
    seed_page_stat_row(&db_b.pool, 1, 1, 1_000, 90, 100).await;

    let stats = StatisticsParser::parse_merged(&[db_a.path.clone(), db_b.path.clone()])
        .await
        .expect("merge failed");

    assert_eq!(stats.books.len(), 1);
    assert_eq!(
        stats.page_stats.len(),
        1,
        "identical session must not double count"
    );
    assert_eq!(stats.page_stats[0].duration, 90, "MAX(duration) wins");
    assert_eq!(stats.books[0].total_read_time, Some(90));
    assert_eq!(stats.books[0].total_read_pages, Some(1));
}

#[tokio::test]
async fn test_parse_merged_same_db_twice_matches_single_parse() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db = TestDatabase::new(&artifacts.statements).await;

    seed_book_row(&db.pool, 1, "Book", "Author", 100, 100, "md5-a").await;
    seed_page_stat_row(&db.pool, 1, 1, 1_000, 60, 100).await;
    seed_page_stat_row(&db.pool, 1, 2, 2_000, 30, 100).await;

    let single = StatisticsParser::parse(&db.path)
        .await
        .expect("parse failed");
    let merged = StatisticsParser::parse_merged(&[db.path.clone(), db.path.clone()])
        .await
        .expect("merge failed");

    assert_eq!(merged.books.len(), single.books.len());
    assert_eq!(merged.page_stats.len(), single.page_stats.len());
    assert_eq!(
        merged.books[0].total_read_time,
        Some(90),
        "merging a database with itself must be a no-op"
    );
    assert_eq!(merged.books[0].total_read_pages, Some(2));
}

#[tokio::test]
async fn test_parse_merged_rescales_to_most_recent_pagination() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db_a = TestDatabase::new(&artifacts.statements).await;
    let db_b = TestDatabase::new(&artifacts.statements).await;

    // Device A renders the book at 100 pages, device B (read more recently) at 200.
    seed_book_row(&db_a.pool, 1, "Shared", "Author", 100, 100, "md5-s").await;
    seed_page_stat_row(&db_a.pool, 1, 10, 1_000, 60, 100).await;
    seed_book_row(&db_b.pool, 1, "Shared", "Author", 200, 200, "md5-s").await;
    seed_page_stat_row(&db_b.pool, 1, 30, 2_000, 30, 200).await;

    let stats = StatisticsParser::parse_merged(&[db_a.path.clone(), db_b.path.clone()])
        .await
        .expect("merge failed");

    assert_eq!(stats.books.len(), 1);
    let book = &stats.books[0];
    assert_eq!(
        book.pages,
        Some(200),
        "pages follow the most recently opened device"
    );

    // Device A's page 10/100 rescales to pages 19-20 of 200, splitting its 60s evenly
    // (KOReader page_stat view semantics); device B's row is already in target scale.
    let mut rescaled: Vec<(i64, i64)> = stats
        .page_stats
        .iter()
        .map(|ps| (ps.page, ps.duration))
        .collect();
    rescaled.sort_unstable();
    assert_eq!(rescaled, vec![(19, 30), (20, 30), (30, 30)]);
    assert_eq!(
        book.total_read_time,
        Some(90),
        "duration preserved across rescaling"
    );
    assert_eq!(book.total_read_pages, Some(3));
}

#[tokio::test]
async fn test_parse_merged_then_deduplicates_by_md5() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db_a = TestDatabase::new(&artifacts.statements).await;
    let db_b = TestDatabase::new(&artifacts.statements).await;

    // Same file, but authors metadata was edited on device B → different book identity
    seed_book_row(&db_a.pool, 1, "Book", "Author A", 100, 300, "shared-md5").await;
    seed_page_stat_row(&db_a.pool, 1, 1, 1_000, 60, 300).await;
    seed_book_row(&db_b.pool, 1, "Book", "Author B", 200, 300, "shared-md5").await;
    seed_page_stat_row(&db_b.pool, 1, 2, 2_000, 30, 300).await;

    let stats = StatisticsParser::parse_merged(&[db_a.path.clone(), db_b.path.clone()])
        .await
        .expect("merge failed");

    assert_eq!(stats.books.len(), 1, "same-md5 rows collapse in dedup");
    let book = &stats.books[0];
    assert_eq!(book.authors, "Author B", "canonical = most recently opened");
    assert_eq!(
        book.total_read_time,
        Some(90),
        "summed without double counting"
    );
    assert_eq!(book.total_read_pages, Some(2));
    assert_eq!(stats.page_stats.len(), 2);
    assert!(stats.page_stats.iter().all(|ps| ps.id_book == book.id));
}

#[tokio::test]
async fn test_parse_merged_rejects_pre_2020_schema() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db_a = TestDatabase::new(&artifacts.statements).await;
    seed_book_row(&db_a.pool, 1, "Book", "Author", 100, 100, "md5-a").await;

    // Old KOReader schema without page_stat_data
    let old_statements = vec![
        "CREATE TABLE book (id integer PRIMARY KEY autoincrement, title text, authors text, \
         notes integer, last_open integer, highlights integer, pages integer, series text, \
         language text, md5 text, total_read_time integer, total_read_pages integer)"
            .to_string(),
    ];
    let db_old = TestDatabase::new(&old_statements).await;

    let err = StatisticsParser::parse_merged(&[db_a.path.clone(), db_old.path.clone()])
        .await
        .expect_err("pre-2020.10 schema must be rejected");
    let message = format!("{:#}", err);
    assert!(
        message.contains("unsupported KOReader schema"),
        "error should explain the schema problem: {message}"
    );
    assert!(
        message.contains(db_old.path.to_string_lossy().as_ref()),
        "error should name the offending database: {message}"
    );
}

#[tokio::test]
async fn test_parse_merged_matches_null_and_empty_authors() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db_a = TestDatabase::new(&artifacts.statements).await;
    let db_b = TestDatabase::new(&artifacts.statements).await;

    // Same book and same session, but authors is NULL on one device and ''
    // on the other (KOReader treats both as "no author").
    seed_book_row_full(&db_a.pool, 1, "Book", None, 100, 100, "md5-s", 0, 0).await;
    seed_page_stat_row(&db_a.pool, 1, 1, 1_000, 60, 100).await;
    seed_book_row_full(&db_b.pool, 1, "Book", Some(""), 100, 100, "md5-s", 0, 0).await;
    seed_page_stat_row(&db_b.pool, 1, 1, 1_000, 60, 100).await;

    let stats = StatisticsParser::parse_merged(&[db_a.path.clone(), db_b.path.clone()])
        .await
        .expect("merge failed");

    assert_eq!(stats.books.len(), 1, "NULL and '' authors should match");
    assert_eq!(
        stats.page_stats.len(),
        1,
        "identical session must not double count across NULL/'' authors"
    );
    assert_eq!(stats.books[0].total_read_time, Some(60));
}

#[tokio::test]
async fn test_parse_merged_survives_duplicate_null_author_rows() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db_a = TestDatabase::new(&artifacts.statements).await;
    let db_b = TestDatabase::new(&artifacts.statements).await;

    // KOReader's UNIQUE index on (title, authors, md5) treats NULLs as
    // distinct, so a device DB can legitimately hold a NULL-authors and an
    // ''-authors row for the same book. A NULL→'' normalization would
    // violate the index here; the merge must survive this.
    seed_book_row_full(&db_a.pool, 1, "Book", None, 100, 100, "md5-s", 0, 0).await;
    seed_book_row_full(&db_a.pool, 2, "Book", Some(""), 150, 100, "md5-s", 0, 0).await;
    seed_page_stat_row(&db_a.pool, 1, 1, 1_000, 60, 100).await;
    seed_page_stat_row(&db_a.pool, 2, 2, 2_000, 30, 100).await;
    seed_book_row_full(&db_b.pool, 1, "Book", Some(""), 200, 100, "md5-s", 0, 0).await;
    seed_page_stat_row(&db_b.pool, 1, 3, 3_000, 15, 100).await;

    let stats = StatisticsParser::parse_merged(&[db_a.path.clone(), db_b.path.clone()])
        .await
        .expect("merge must not trip the unique index on legacy duplicate rows");

    assert_eq!(stats.books.len(), 1, "same-md5 rows collapse in dedup");
    assert_eq!(stats.page_stats.len(), 3);
    assert_eq!(
        stats.books[0].total_read_time,
        Some(105),
        "sessions from all three rows survive without double counting"
    );
}

#[tokio::test]
async fn test_parse_merged_preserves_stored_totals_without_page_stats() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db_a = TestDatabase::new(&artifacts.statements).await;
    let db_b = TestDatabase::new(&artifacts.statements).await;

    // Raw page_stat_data was trimmed in KOReader but the stored totals
    // remain; merging must not zero them out.
    seed_book_row_full(
        &db_a.pool,
        1,
        "Legacy",
        Some("Author"),
        100,
        100,
        "md5-l",
        3_600,
        42,
    )
    .await;
    seed_book_row_full(
        &db_b.pool,
        1,
        "Legacy",
        Some("Author"),
        200,
        100,
        "md5-l",
        7_200,
        80,
    )
    .await;
    seed_book_row_full(
        &db_b.pool,
        2,
        "Fresh",
        Some("Author"),
        200,
        100,
        "md5-f",
        0,
        0,
    )
    .await;
    seed_page_stat_row(&db_b.pool, 2, 1, 1_000, 60, 100).await;

    let stats = StatisticsParser::parse_merged(&[db_a.path.clone(), db_b.path.clone()])
        .await
        .expect("merge failed");

    let legacy = stats.stats_by_md5.get("md5-l").expect("legacy book");
    assert_eq!(
        legacy.total_read_time,
        Some(7_200),
        "stored totals survive (larger device total wins)"
    );
    assert_eq!(legacy.total_read_pages, Some(80));
    let fresh = stats.stats_by_md5.get("md5-f").expect("fresh book");
    assert_eq!(
        fresh.total_read_time,
        Some(60),
        "books with raw rows still get recomputed totals"
    );
}

#[tokio::test]
async fn test_parse_merged_accepts_read_only_sources() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db_a = TestDatabase::new(&artifacts.statements).await;
    let db_b = TestDatabase::new(&artifacts.statements).await;

    seed_book_row(&db_a.pool, 1, "Book A", "Author A", 100, 100, "md5-a").await;
    seed_page_stat_row(&db_a.pool, 1, 1, 1_000, 60, 100).await;
    seed_book_row(&db_b.pool, 1, "Book B", "Author B", 200, 100, "md5-b").await;
    seed_page_stat_row(&db_b.pool, 1, 2, 2_000, 30, 100).await;

    // e.g. Syncthing "sync permissions" from a device where the file is 0444
    for path in [&db_a.path, &db_b.path] {
        let mut permissions = std::fs::metadata(path).expect("db metadata").permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(path, permissions).expect("set read-only");
    }

    let stats = StatisticsParser::parse_merged(&[db_a.path.clone(), db_b.path.clone()])
        .await
        .expect("read-only sources must still merge");

    assert_eq!(stats.books.len(), 2);

    // Restore so TempDir cleanup can delete the files on all platforms.
    for path in [&db_a.path, &db_b.path] {
        let mut permissions = std::fs::metadata(path).expect("db metadata").permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        std::fs::set_permissions(path, permissions).expect("restore permissions");
    }
}

fn build_statistics_lua_script(root: &Path) -> String {
    let mocks = compose_lua_mocks(LUA_STATISTICS_MOCKS_EXTRA);
    LUA_STATISTICS_TEMPLATE
        .replace("__ROOT__", &root.display().to_string())
        .replace("__MOCKS__", &mocks)
}

const LUA_STATISTICS_TEMPLATE: &str = r#"
local root = [[__ROOT__]]

__MOCKS__

local executed_sql = {}
local conn = {}

function conn:exec(sql)
    table.insert(executed_sql, sql)
end

local ReaderStatistics = dofile(root .. "/plugins/statistics.koplugin/main.lua")
ReaderStatistics:createDB(conn)

local function find_upvalue(func, target)
    local idx = 1
    while true do
        local name, value = debug.getupvalue(func, idx)
        if not name then
            break
        end
        if name == target then
            return value
        end
        idx = idx + 1
    end
    return nil
end

local function locate_totals_query()
    local candidates = {
        "initData",
        "addBookStatToDB",
        "getPageTimeTotalStats",
        "getPageTimeTotalCompletions",
    }

    for _, name in ipairs(candidates) do
        local func = ReaderStatistics[name]
        if type(func) == "function" then
            local value = find_upvalue(func, "STATISTICS_SQL_BOOK_TOTALS_QUERY")
            if value then
                return value
            end
        end
    end
    return nil
end

local totals_query = locate_totals_query()
assert(totals_query, "Unable to locate STATISTICS_SQL_BOOK_TOTALS_QUERY upvalue in ReaderStatistics")

return {
    executed_sql = executed_sql,
    totals_query = totals_query,
}
"#;

const LUA_STATISTICS_MOCKS_EXTRA: &str = r#"
package.path = table.concat({
    root .. "/?.lua",
    root .. "/?/init.lua",
    root .. "/frontend/?.lua",
    root .. "/frontend/?/init.lua",
    root .. "/plugins/?.lua",
    root .. "/plugins/?/init.lua",
    package.path,
}, ";")

stub_module("ui/bidi", {})

local ButtonDialog = {}
function ButtonDialog:new()
    return setmetatable({}, { __index = self })
end
stub_module("ui/widget/buttondialog", ButtonDialog)

local ConfirmBox = {}
function ConfirmBox:new()
    return setmetatable({}, { __index = self })
end
stub_module("ui/widget/confirmbox", ConfirmBox)

stub_module("device", {
    canUseWAL = function() return true end,
    hasColorScreen = function() return false end,
})

stub_module("dispatcher", {
    registerAction = noop,
})

local DocSettings = {}
function DocSettings:open()
    return {
        readSetting = function(_, _, default) return default end,
    }
end
stub_module("docsettings", DocSettings)

local InfoMessage = {}
function InfoMessage:new(opts)
    return setmetatable(opts or {}, { __index = self })
end
stub_module("ui/widget/infomessage", InfoMessage)

local KeyValuePage = {}
function KeyValuePage:new()
    return setmetatable({}, { __index = self })
end
stub_module("ui/widget/keyvaluepage", KeyValuePage)

stub_module("optmath", {
    round = function(value) return value end,
})

stub_module("readerprogress", {})

stub_module("readhistory", {
    hist = {},
})

stub_module("lua-ljsqlite3/init", {
    open = function()
        error("sqlite access not available in tests")
    end,
})

stub_module("frontend/apps/cloudstorage/syncservice", {
    new = function() return {} end,
})

stub_module("ui/uimanager", {
    show = noop,
    close = noop,
    forceRePaint = noop,
    broadcastEvent = noop,
})

local Widget = {}
Widget.__index = Widget
function Widget:extend(props)
    props = props or {}
    return setmetatable(props, { __index = self })
end
stub_module("ui/widget/widget", Widget)

stub_module("datetime", {})

local gettext = {}
function gettext.pgettext(_, text)
    return text
end
function gettext.ngettext(_, singular, plural, n)
    if n == 1 then
        return singular
    else
        return plural
    end
end
setmetatable(gettext, {
    __call = function(_, text)
        return text
    end,
})
stub_module("gettext", gettext)
"#;
