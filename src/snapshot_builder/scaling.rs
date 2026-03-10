//! Synthetic stable-page scaling helpers.

use crate::models::{
    CalendarMonths, ContentType, LibraryItem, ReadingStats, StatisticsData, StreakInfo,
};
use crate::time_config::TimeConfig;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PageScaling {
    enabled: bool,
    factor_by_book_id: HashMap<i64, f64>,
    factor_by_md5: HashMap<String, f64>,
}

#[derive(Debug, Default)]
struct MonthlyPageTotals {
    all: HashMap<String, i64>,
    books: HashMap<String, i64>,
    comics: HashMap<String, i64>,
}

impl PageScaling {
    pub fn from_inputs(
        enabled: bool,
        items: &[LibraryItem],
        stats_data: Option<&StatisticsData>,
    ) -> Self {
        if !enabled {
            return Self {
                enabled: false,
                factor_by_book_id: HashMap::new(),
                factor_by_md5: HashMap::new(),
            };
        }

        let Some(stats_data) = stats_data else {
            return Self {
                enabled: true,
                factor_by_book_id: HashMap::new(),
                factor_by_md5: HashMap::new(),
            };
        };

        let md5_to_item: HashMap<String, &LibraryItem> = items
            .iter()
            .filter_map(|item| {
                item.koreader_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.partial_md5_checksum.as_ref())
                    .map(|md5| (md5.to_lowercase(), item))
            })
            .collect();

        let mut factor_by_book_id: HashMap<i64, f64> = HashMap::new();
        let mut factor_by_md5: HashMap<String, f64> = HashMap::new();

        for stat_book in &stats_data.books {
            let md5_key = stat_book.md5.to_lowercase();
            let Some(item) = md5_to_item.get(&md5_key) else {
                continue;
            };

            let Some(stable_total) = item.synthetic_scaling_page_total() else {
                continue;
            };

            let rendered_total = stat_book.pages.filter(|pages| *pages > 0).or_else(|| {
                item.koreader_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.doc_pages.map(i64::from))
                    .filter(|pages| *pages > 0)
            });

            let Some(rendered_total) = rendered_total else {
                continue;
            };

            let factor = stable_total as f64 / rendered_total as f64;
            if !factor.is_finite() || factor <= 0.0 {
                continue;
            }

            factor_by_book_id.insert(stat_book.id, factor);
            factor_by_md5.insert(md5_key, factor);
        }

        Self {
            enabled: true,
            factor_by_book_id,
            factor_by_md5,
        }
    }

    pub fn factor_for_book_id(&self, book_id: i64) -> Option<f64> {
        if !self.enabled {
            return None;
        }
        self.factor_by_book_id.get(&book_id).copied()
    }

    pub fn factor_for_md5(&self, md5: &str) -> Option<f64> {
        if !self.enabled {
            return None;
        }
        self.factor_by_md5.get(&md5.to_lowercase()).copied()
    }

    pub fn scale_pages_for_md5(&self, md5: &str, pages: i64) -> i64 {
        match self.factor_for_md5(md5) {
            Some(factor) => Self::scale_pages_with_factor(pages, factor),
            None => pages,
        }
    }

    pub fn scale_pages_with_factor(pages: i64, factor: f64) -> i64 {
        if pages <= 0 || !factor.is_finite() || factor <= 0.0 {
            return 0;
        }

        Self::round_pages(pages as f64 * factor)
    }

    pub fn apply_to_reading_stats(
        &self,
        source: &StatisticsData,
        reading_stats: &mut ReadingStats,
        time_config: &TimeConfig,
    ) {
        if !self.enabled {
            return;
        }

        let mut daily_pages_f64: HashMap<String, f64> = HashMap::new();

        for stat in &source.page_stats {
            if stat.duration <= 0 {
                continue;
            }

            let factor = self.factor_for_book_id(stat.id_book).unwrap_or(1.0);
            let date_key = time_config
                .date_for_timestamp(stat.start_time)
                .format("%Y-%m-%d")
                .to_string();
            *daily_pages_f64.entry(date_key).or_insert(0.0) += factor;
        }

        for day in &mut reading_stats.daily_activity {
            let scaled_pages = daily_pages_f64.get(&day.date).copied().unwrap_or(0.0);
            day.pages_read = Self::round_pages(scaled_pages);
        }

        reading_stats.total_page_reads = reading_stats
            .daily_activity
            .iter()
            .map(|d| d.pages_read)
            .sum();
        reading_stats.most_pages_in_day = reading_stats
            .daily_activity
            .iter()
            .map(|d| d.pages_read)
            .max()
            .unwrap_or(0);

        let mut weekly_pages: HashMap<String, i64> = HashMap::new();
        for day in &reading_stats.daily_activity {
            let Ok(date) = NaiveDate::parse_from_str(&day.date, "%Y-%m-%d") else {
                continue;
            };

            let week_key = week_start_date_for(date).format("%Y-%m-%d").to_string();
            *weekly_pages.entry(week_key).or_insert(0) += day.pages_read;
        }

        for week in &mut reading_stats.weeks {
            let scaled_pages = weekly_pages.get(&week.start_date).copied().unwrap_or(0);
            week.pages_read = scaled_pages;
            week.avg_pages_per_day = scaled_pages as f64 / 7.0;
        }

        let (longest_streak, current_streak) =
            recompute_streaks(&reading_stats.daily_activity, time_config);
        reading_stats.longest_streak = longest_streak;
        reading_stats.current_streak = current_streak;
    }

    pub fn apply_to_calendar_months(
        &self,
        calendar_months: &mut CalendarMonths,
        source: &StatisticsData,
        time_config: &TimeConfig,
    ) {
        if !self.enabled {
            return;
        }

        for month_data in calendar_months.values_mut() {
            for event in &mut month_data.events {
                event.total_pages_read =
                    self.scale_pages_for_md5(&event.item_id, event.total_pages_read);
            }
        }

        let monthly_totals = self.scaled_monthly_totals(source, time_config);

        for (month_key, month_data) in calendar_months {
            month_data.stats.pages_read = *monthly_totals.all.get(month_key).unwrap_or(&0);
            month_data.stats_books.pages_read = *monthly_totals.books.get(month_key).unwrap_or(&0);
            month_data.stats_comics.pages_read =
                *monthly_totals.comics.get(month_key).unwrap_or(&0);
        }
    }

    fn scaled_monthly_totals(
        &self,
        source: &StatisticsData,
        time_config: &TimeConfig,
    ) -> MonthlyPageTotals {
        let mut totals_all: HashMap<String, f64> = HashMap::new();
        let mut totals_books: HashMap<String, f64> = HashMap::new();
        let mut totals_comics: HashMap<String, f64> = HashMap::new();

        let content_type_by_book_id: HashMap<i64, ContentType> = source
            .books
            .iter()
            .filter_map(|book| {
                book.content_type
                    .map(|content_type| (book.id, content_type))
            })
            .collect();

        for stat in &source.page_stats {
            if stat.duration <= 0 {
                continue;
            }

            let factor = self.factor_for_book_id(stat.id_book).unwrap_or(1.0);
            let month_key = time_config
                .date_for_timestamp(stat.start_time)
                .format("%Y-%m")
                .to_string();

            *totals_all.entry(month_key.clone()).or_insert(0.0) += factor;

            match content_type_by_book_id.get(&stat.id_book) {
                Some(ContentType::Book) => {
                    *totals_books.entry(month_key).or_insert(0.0) += factor;
                }
                Some(ContentType::Comic) => {
                    *totals_comics.entry(month_key).or_insert(0.0) += factor;
                }
                None => {}
            }
        }

        MonthlyPageTotals {
            all: round_map(&totals_all),
            books: round_map(&totals_books),
            comics: round_map(&totals_comics),
        }
    }

    fn round_pages(value: f64) -> i64 {
        if !value.is_finite() || value <= 0.0 {
            return 0;
        }

        value.round() as i64
    }
}

fn recompute_streaks(
    daily_activity: &[crate::models::DailyStats],
    time_config: &TimeConfig,
) -> (StreakInfo, StreakInfo) {
    if daily_activity.is_empty() {
        return (
            StreakInfo::new(0, None, None),
            StreakInfo::new(0, None, None),
        );
    }

    let mut reading_dates: Vec<NaiveDate> = daily_activity
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

    reading_dates.sort();
    reading_dates.dedup();

    let today = time_config.today_date();

    let mut streaks: Vec<(i64, NaiveDate, NaiveDate)> = Vec::new();
    let mut current_streak_start = reading_dates[0];
    let mut current_streak_length = 1i64;

    for i in 1..reading_dates.len() {
        let prev_date = reading_dates[i - 1];
        let curr_date = reading_dates[i];

        if curr_date == prev_date + Duration::days(1) {
            current_streak_length += 1;
        } else {
            streaks.push((current_streak_length, current_streak_start, prev_date));
            current_streak_start = curr_date;
            current_streak_length = 1;
        }
    }

    if let Some(&last_date) = reading_dates.last() {
        streaks.push((current_streak_length, current_streak_start, last_date));
    }

    let longest_streak =
        if let Some(&(length, start, end)) = streaks.iter().max_by_key(|&&(len, _, _)| len) {
            StreakInfo::new(
                length,
                Some(start.format("%Y-%m-%d").to_string()),
                Some(end.format("%Y-%m-%d").to_string()),
            )
        } else {
            StreakInfo::new(0, None, None)
        };

    let current_streak = if let Some(&last_reading_date) = reading_dates.last() {
        let days_since_last_read = (today - last_reading_date).num_days();

        if days_since_last_read <= 1 {
            if let Some(&(length, start, _)) = streaks
                .iter()
                .find(|&&(_, _, end)| end == last_reading_date)
            {
                StreakInfo::new(length, Some(start.format("%Y-%m-%d").to_string()), None)
            } else {
                StreakInfo::new(0, None, None)
            }
        } else {
            StreakInfo::new(0, None, None)
        }
    } else {
        StreakInfo::new(0, None, None)
    };

    (longest_streak, current_streak)
}

fn week_start_date_for(date: NaiveDate) -> NaiveDate {
    let year = date.year();
    let week = date.iso_week().week();

    NaiveDate::from_isoywd_opt(year, week, Weekday::Mon)
        .or_else(|| NaiveDate::from_ymd_opt(year, 1, 1))
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2020, 1, 1).expect("valid fallback date"))
}

fn round_map(source: &HashMap<String, f64>) -> HashMap<String, i64> {
    let mut out = HashMap::new();
    for (key, value) in source {
        out.insert(key.clone(), PageScaling::round_pages(*value));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        BookInfo, BookStatus, CalendarEvent, CalendarItem, CalendarMonthData, KoReaderMetadata,
        MonthlyStats, PageStat, StatBook, StatisticsData, Summary,
    };
    use crate::time_config::TimeConfig;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn test_item(id: &str, md5: &str, synthetic: bool, stable_total: u32) -> LibraryItem {
        LibraryItem {
            id: id.to_string(),
            book_info: BookInfo {
                title: "Item".to_string(),
                authors: vec!["Author".to_string()],
                description: None,
                language: None,
                publisher: None,
                identifiers: Vec::new(),
                subjects: Vec::new(),
                series: None,
                series_number: None,
                pages: Some(123),
                cover_data: None,
                cover_mime_type: None,
            },
            koreader_metadata: Some(KoReaderMetadata {
                annotations: Vec::new(),
                doc_pages: Some(200),
                doc_path: None,
                doc_props: None,
                pagemap_use_page_labels: Some(true),
                pagemap_chars_per_synthetic_page: synthetic.then_some(1500),
                pagemap_doc_pages: Some(stable_total),
                pagemap_current_page_label: None,
                pagemap_last_page_label: None,
                partial_md5_checksum: Some(md5.to_string()),
                percent_finished: None,
                stats: None,
                summary: Some(Summary {
                    modified: None,
                    note: None,
                    rating: None,
                    status: BookStatus::Reading,
                }),
                text_lang: None,
            }),
            file_path: PathBuf::from("/tmp/item.epub"),
            format: crate::models::LibraryItemFormat::Epub,
        }
    }

    fn test_stat_book(id: i64, md5: &str, pages: i64, content_type: ContentType) -> StatBook {
        StatBook {
            id,
            title: "Stat Book".to_string(),
            authors: "Author".to_string(),
            notes: None,
            last_open: None,
            highlights: None,
            pages: Some(pages),
            md5: md5.to_string(),
            content_type: Some(content_type),
            total_read_time: Some(3600),
            total_read_pages: Some(10),
            completions: None,
        }
    }

    #[test]
    fn builds_factors_for_synthetic_items_only() {
        let item_synthetic = test_item("1", "md5-synth", true, 300);
        let item_publisher_only = test_item("2", "md5-pub", false, 400);

        let books = vec![
            test_stat_book(1, "md5-synth", 200, ContentType::Book),
            test_stat_book(2, "md5-pub", 200, ContentType::Book),
        ];
        let stats_data = StatisticsData {
            books: books.clone(),
            page_stats: Vec::new(),
            stats_by_md5: books
                .into_iter()
                .map(|book| (book.md5.clone(), book))
                .collect(),
        };

        let scaling = PageScaling::from_inputs(
            true,
            &[item_synthetic.clone(), item_publisher_only.clone()],
            Some(&stats_data),
        );
        assert_eq!(scaling.factor_for_book_id(1), Some(1.5));
        assert_eq!(scaling.factor_for_book_id(2), None);
        assert_eq!(scaling.factor_for_md5("md5-synth"), Some(1.5));
        assert_eq!(scaling.factor_for_md5("md5-pub"), None);

        let off = PageScaling::from_inputs(false, &[item_synthetic], Some(&stats_data));
        assert_eq!(off.factor_for_book_id(1), None);
    }

    #[test]
    fn applies_scaled_pages_to_reading_stats() {
        let books = vec![test_stat_book(1, "md5-synth", 200, ContentType::Book)];
        let mut stats_data = StatisticsData {
            books: books.clone(),
            page_stats: vec![
                PageStat {
                    id_book: 1,
                    page: 1,
                    start_time: 1_704_067_200, // 2024-01-01
                    duration: 120,
                },
                PageStat {
                    id_book: 1,
                    page: 2,
                    start_time: 1_704_070_800, // 2024-01-01
                    duration: 60,
                },
                PageStat {
                    id_book: 1,
                    page: 3,
                    start_time: 1_704_153_600, // 2024-01-02
                    duration: 90,
                },
            ],
            stats_by_md5: books
                .into_iter()
                .map(|book| (book.md5.clone(), book))
                .collect(),
        };

        let item = test_item("1", "md5-synth", true, 300);
        let time_config = TimeConfig::new(None, 0);

        let mut reading_stats =
            crate::koreader::StatisticsCalculator::calculate_stats(&mut stats_data, &time_config);
        assert_eq!(reading_stats.total_page_reads, 3);

        let scaling = PageScaling::from_inputs(true, &[item], Some(&stats_data));
        scaling.apply_to_reading_stats(&stats_data, &mut reading_stats, &time_config);

        assert_eq!(reading_stats.daily_activity.len(), 2);
        assert_eq!(reading_stats.daily_activity[0].pages_read, 3);
        assert_eq!(reading_stats.daily_activity[1].pages_read, 2);
        assert_eq!(reading_stats.total_page_reads, 5);
        assert_eq!(reading_stats.most_pages_in_day, 3);
        assert_eq!(reading_stats.weeks[0].pages_read, 5);
        assert!((reading_stats.weeks[0].avg_pages_per_day - (5.0 / 7.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn uses_per_book_factors_for_mixed_daily_totals() {
        let books = vec![
            test_stat_book(1, "md5-a", 200, ContentType::Book),
            test_stat_book(2, "md5-b", 200, ContentType::Book),
        ];
        let mut stats_data = StatisticsData {
            books: books.clone(),
            page_stats: vec![
                PageStat {
                    id_book: 1,
                    page: 1,
                    start_time: 1_704_067_200, // 2024-01-01
                    duration: 60,
                },
                PageStat {
                    id_book: 1,
                    page: 2,
                    start_time: 1_704_070_800, // 2024-01-01
                    duration: 60,
                },
                PageStat {
                    id_book: 2,
                    page: 1,
                    start_time: 1_704_153_600, // 2024-01-02
                    duration: 60,
                },
                PageStat {
                    id_book: 2,
                    page: 2,
                    start_time: 1_704_157_200, // 2024-01-02
                    duration: 60,
                },
            ],
            stats_by_md5: books
                .into_iter()
                .map(|book| (book.md5.clone(), book))
                .collect(),
        };

        let item_a = test_item("1", "md5-a", true, 300); // factor 1.5
        let item_b = test_item("2", "md5-b", true, 100); // factor 0.5
        let time_config = TimeConfig::new(None, 0);

        let mut reading_stats =
            crate::koreader::StatisticsCalculator::calculate_stats(&mut stats_data, &time_config);
        let scaling = PageScaling::from_inputs(true, &[item_a, item_b], Some(&stats_data));
        scaling.apply_to_reading_stats(&stats_data, &mut reading_stats, &time_config);

        assert_eq!(reading_stats.daily_activity.len(), 2);
        assert_eq!(reading_stats.daily_activity[0].pages_read, 3);
        assert_eq!(reading_stats.daily_activity[1].pages_read, 1);
        assert_eq!(reading_stats.total_page_reads, 4);
        assert_eq!(reading_stats.most_pages_in_day, 3);
    }

    #[test]
    fn recomputes_streaks_when_scaled_pages_round_to_zero() {
        let books = vec![test_stat_book(1, "md5-low", 100, ContentType::Book)];
        let timezone = "UTC".parse().expect("timezone should parse");
        let time_config = TimeConfig::new(Some(timezone), 0);
        let today = time_config.today_date();
        let today_timestamp = today
            .and_hms_opt(12, 0, 0)
            .expect("time should be valid")
            .and_utc()
            .timestamp();

        let mut stats_data = StatisticsData {
            books: books.clone(),
            page_stats: vec![PageStat {
                id_book: 1,
                page: 1,
                start_time: today_timestamp,
                duration: 60,
            }],
            stats_by_md5: books
                .into_iter()
                .map(|book| (book.md5.clone(), book))
                .collect(),
        };

        let mut reading_stats =
            crate::koreader::StatisticsCalculator::calculate_stats(&mut stats_data, &time_config);
        assert_eq!(reading_stats.longest_streak.days, 1);
        assert_eq!(reading_stats.current_streak.days, 1);

        let item = test_item("1", "md5-low", true, 40); // factor: 40 / 100 = 0.4
        let scaling = PageScaling::from_inputs(true, &[item], Some(&stats_data));
        scaling.apply_to_reading_stats(&stats_data, &mut reading_stats, &time_config);

        assert_eq!(reading_stats.daily_activity[0].pages_read, 0);
        assert_eq!(reading_stats.longest_streak.days, 0);
        assert_eq!(reading_stats.current_streak.days, 0);
    }

    #[test]
    fn applies_scaled_pages_to_calendar_payloads() {
        let books = vec![test_stat_book(1, "md5-synth", 200, ContentType::Book)];
        let stats_data = StatisticsData {
            books: books.clone(),
            page_stats: vec![
                PageStat {
                    id_book: 1,
                    page: 1,
                    start_time: 1_704_067_200,
                    duration: 120,
                },
                PageStat {
                    id_book: 1,
                    page: 2,
                    start_time: 1_704_070_800,
                    duration: 60,
                },
            ],
            stats_by_md5: books
                .into_iter()
                .map(|book| (book.md5.clone(), book))
                .collect(),
        };

        let item = test_item("1", "md5-synth", true, 300);
        let scaling = PageScaling::from_inputs(true, &[item], Some(&stats_data));
        let time_config = TimeConfig::new(None, 0);

        let mut months: CalendarMonths = BTreeMap::new();
        months.insert(
            "2024-01".to_string(),
            CalendarMonthData {
                events: vec![CalendarEvent::new(
                    "2024-01-01".to_string(),
                    None,
                    180,
                    2,
                    "md5-synth".to_string(),
                )],
                books: BTreeMap::from([(
                    "md5-synth".to_string(),
                    CalendarItem::new(
                        "Title".to_string(),
                        vec!["Author".to_string()],
                        ContentType::Book,
                        Some("1".to_string()),
                        None,
                    ),
                )]),
                stats: MonthlyStats {
                    books_read: 1,
                    pages_read: 2,
                    time_read: 180,
                    days_read_pct: 10,
                },
                stats_books: MonthlyStats {
                    books_read: 1,
                    pages_read: 2,
                    time_read: 180,
                    days_read_pct: 10,
                },
                stats_comics: MonthlyStats {
                    books_read: 0,
                    pages_read: 0,
                    time_read: 0,
                    days_read_pct: 0,
                },
            },
        );

        scaling.apply_to_calendar_months(&mut months, &stats_data, &time_config);

        let month = months.get("2024-01").expect("month should exist");
        assert_eq!(month.events[0].total_pages_read, 3);
        assert_eq!(month.stats.pages_read, 3);
        assert_eq!(month.stats_books.pages_read, 3);
        assert_eq!(month.stats_comics.pages_read, 0);
    }
}
