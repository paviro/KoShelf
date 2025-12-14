use crate::koreader::statistics::StatisticsCalculator;
use crate::models::{PageStat, StatBook, StatisticsData};
use crate::time_config::TimeConfig;
use std::collections::HashMap;

#[test]
fn test_filter_stats_per_book_per_day() {
    // Comprehensive test for per-book-per-day filtering
    // Tests multiple scenarios:
    // - Day 1: Book 1 (10 pages, 600s) passes pages threshold, Book 2 (5 pages, 300s) fails both
    // - Day 2: Book 1 (2 pages, 3600s) passes time threshold, Book 2 (15 pages, 200s) passes pages threshold
    // - Day 3: Book 1 (3 pages, 100s) fails both, Book 2 (4 pages, 500s) fails both

    let day1_ts = 1672531200; // 2023-01-01 00:00:00 UTC
    let day2_ts = 1672617600; // 2023-01-02 00:00:00 UTC
    let day3_ts = 1672704000; // 2023-01-03 00:00:00 UTC
    let mut page_stats = Vec::new();

    // Day 1: Book 1 - 10 pages, 600s (passes pages threshold)
    for i in 0..10 {
        page_stats.push(PageStat {
            id_book: 1,
            page: i,
            start_time: day1_ts + i * 60,
            duration: 60,
        });
    }

    // Day 1: Book 2 - 5 pages, 300s (fails both thresholds)
    for i in 0..5 {
        page_stats.push(PageStat {
            id_book: 2,
            page: i,
            start_time: day1_ts + i * 60,
            duration: 60,
        });
    }

    // Day 2: Book 1 - 2 pages, 3600s (passes time threshold)
    for i in 0..2 {
        page_stats.push(PageStat {
            id_book: 1,
            page: i + 10,
            start_time: day2_ts + i * 1800,
            duration: 1800,
        });
    }

    // Day 2: Book 2 - 15 pages, 200s (passes pages threshold)
    for i in 0..15 {
        page_stats.push(PageStat {
            id_book: 2,
            page: i + 5,
            start_time: day2_ts + i * 10,
            duration: 13,
        });
    }

    // Day 3: Book 1 - 3 pages, 100s (fails both)
    for i in 0..3 {
        page_stats.push(PageStat {
            id_book: 1,
            page: i + 12,
            start_time: day3_ts + i * 30,
            duration: 33,
        });
    }

    // Day 3: Book 2 - 4 pages, 500s (fails both)
    for i in 0..4 {
        page_stats.push(PageStat {
            id_book: 2,
            page: i + 20,
            start_time: day3_ts + i * 100,
            duration: 125,
        });
    }

    let mut data = StatisticsData {
        books: vec![
            StatBook {
                id: 1,
                title: "Book 1".to_string(),
                authors: "Author 1".to_string(),
                notes: None,
                last_open: None,
                highlights: None,
                pages: None,
                md5: "abc".to_string(),
                content_type: None,
                total_read_time: None,
                total_read_pages: None,
                completions: None,
            },
            StatBook {
                id: 2,
                title: "Book 2".to_string(),
                authors: "Author 2".to_string(),
                notes: None,
                last_open: None,
                highlights: None,
                pages: None,
                md5: "def".to_string(),
                content_type: None,
                total_read_time: None,
                total_read_pages: None,
                completions: None,
            },
        ],
        page_stats,
        stats_by_md5: HashMap::new(),
    };

    let time_config = TimeConfig::new(None, 0);
    // Filter: min 8 pages OR min 1800s (30 mins)
    StatisticsCalculator::filter_stats(&mut data, &time_config, Some(8), Some(1800));

    // Expected results:
    // Day 1: Book 1 kept (10 pages), Book 2 filtered (5 pages, 300s)
    // Day 2: Book 1 kept (3600s), Book 2 kept (15 pages)
    // Day 3: Both filtered
    // Total: 10 + 2 + 15 = 27 pages

    assert_eq!(data.page_stats.len(), 27, "Should have 27 pages remaining");

    let mut book1_day1 = 0;
    let mut book1_day2 = 0;
    let mut book1_day3 = 0;
    let mut book2_day1 = 0;
    let mut book2_day2 = 0;
    let mut book2_day3 = 0;

    for stat in &data.page_stats {
        match (stat.id_book, stat.start_time) {
            (1, ts) if ts >= day1_ts && ts < day2_ts => book1_day1 += 1,
            (1, ts) if ts >= day2_ts && ts < day3_ts => book1_day2 += 1,
            (1, ts) if ts >= day3_ts => book1_day3 += 1,
            (2, ts) if ts >= day1_ts && ts < day2_ts => book2_day1 += 1,
            (2, ts) if ts >= day2_ts && ts < day3_ts => book2_day2 += 1,
            (2, ts) if ts >= day3_ts => book2_day3 += 1,
            _ => {}
        }
    }

    assert_eq!(
        book1_day1, 10,
        "Book 1 Day 1: 10 pages (passes pages threshold)"
    );
    assert_eq!(
        book1_day2, 2,
        "Book 1 Day 2: 2 pages (passes time threshold)"
    );
    assert_eq!(book1_day3, 0, "Book 1 Day 3: filtered (fails both)");
    assert_eq!(book2_day1, 0, "Book 2 Day 1: filtered (fails both)");
    assert_eq!(
        book2_day2, 15,
        "Book 2 Day 2: 15 pages (passes pages threshold)"
    );
    assert_eq!(book2_day3, 0, "Book 2 Day 3: filtered (fails both)");
}
