use crate::models::{PageStat, StatBook, StatisticsData};
use crate::statistics::StatisticsCalculator;
use crate::time_config::TimeConfig;
use std::collections::HashMap;

fn create_mock_stats() -> StatisticsData {
    // Create 3 days of data:
    // Day 1 (2023-01-01): 5 pages, 10 mins (600s)
    // Day 2 (2023-01-02): 20 pages, 5 mins (300s)
    // Day 3 (2023-01-03): 2 pages, 60 mins (3600s)

    let day1_ts = 1672531200; // 2023-01-01 00:00:00 UTC
    let day2_ts = 1672617600; // 2023-01-02 00:00:00 UTC
    let day3_ts = 1672704000; // 2023-01-03 00:00:00 UTC

    let mut page_stats = Vec::new();

    // Day 1: 5 pages, 10 mins
    for i in 0..5 {
        page_stats.push(PageStat {
            id_book: 1,
            page: i,
            start_time: day1_ts + i * 60, // 1 min per page roughly
            duration: 120,                // 2 mins per page -> total 10 mins
        });
    }

    // Day 2: 20 pages, 5 mins
    for i in 0..20 {
        page_stats.push(PageStat {
            id_book: 1,
            page: i,
            start_time: day2_ts + i * 10,
            duration: 15, // 15s per page -> total 300s (5 mins)
        });
    }

    // Day 3: 2 pages, 60 mins
    for i in 0..2 {
        page_stats.push(PageStat {
            id_book: 1,
            page: i,
            start_time: day3_ts + i * 1800,
            duration: 1800, // 30 mins per page -> total 60 mins
        });
    }

    StatisticsData {
        books: vec![StatBook {
            id: 1,
            title: "Test Book".to_string(),
            authors: "Author".to_string(),
            notes: None,
            last_open: None,
            highlights: None,
            pages: None,
            md5: "abc".to_string(),
            total_read_time: None,
            total_read_pages: None,
            completions: None,
        }],
        page_stats,
        stats_by_md5: HashMap::new(),
    }
}

#[test]
fn test_filter_stats_min_pages() {
    let mut data = create_mock_stats();
    let time_config = TimeConfig::new(None, 0); // UTC by default

    // Filter: min 10 pages
    // Should keep Day 2 (20 pages), remove Day 1 (5) and Day 3 (2)
    StatisticsCalculator::filter_stats(&mut data, &time_config, Some(10), None);

    let remaining_days: Vec<i64> = data.page_stats.iter().map(|s| s.start_time).collect();
    assert!(!remaining_days.is_empty());

    for ts in remaining_days {
        // All timestamps should be from Day 2
        assert!(ts >= 1672617600 && ts < 1672704000);
    }
}

#[test]
fn test_filter_stats_min_time() {
    let mut data = create_mock_stats();
    let time_config = TimeConfig::new(None, 0);

    // Filter: min 30 mins (1800s)
    // Should keep Day 3 (60 mins), remove Day 1 (10 mins) and Day 2 (5 mins)
    StatisticsCalculator::filter_stats(&mut data, &time_config, None, Some(1800));

    let remaining_days: Vec<i64> = data.page_stats.iter().map(|s| s.start_time).collect();
    assert!(!remaining_days.is_empty());

    for ts in remaining_days {
        // All timestamps should be from Day 3
        assert!(ts >= 1672704000);
    }
}

#[test]
fn test_filter_stats_or_logic() {
    let mut data = create_mock_stats();
    let time_config = TimeConfig::new(None, 0);

    // Filter: min 10 pages OR min 30 mins
    // Should keep Day 2 (20 pages) AND Day 3 (60 mins)
    // Should remove Day 1 (5 pages, 10 mins)
    StatisticsCalculator::filter_stats(&mut data, &time_config, Some(10), Some(1800));

    let remaining_days: Vec<i64> = data.page_stats.iter().map(|s| s.start_time).collect();
    assert!(!remaining_days.is_empty());

    let mut has_day2 = false;
    let mut has_day3 = false;
    let mut has_day1 = false;

    for ts in remaining_days {
        if ts >= 1672531200 && ts < 1672617600 {
            has_day1 = true;
        }
        if ts >= 1672617600 && ts < 1672704000 {
            has_day2 = true;
        }
        if ts >= 1672704000 {
            has_day3 = true;
        }
    }

    assert!(!has_day1, "Day 1 should be removed");
    assert!(has_day2, "Day 2 should be kept (pages)");
    assert!(has_day3, "Day 3 should be kept (time)");
}
