use crate::models::*;
use super::partial_md5::calculate_partial_md5;
use crate::time_config::TimeConfig;
use chrono::Datelike;
/// Provides utilities for generating calendar-related data (events, monthly payloads, stats).
use chrono::{Duration, NaiveDate};
use std::collections::{BTreeMap, HashMap};

pub struct CalendarGenerator;

impl CalendarGenerator {
    /// Generate per-month calendar payload (events, books, stats) directly.
    /// Returns a `CalendarMonths` map keyed by "YYYY-MM".
    pub fn generate_calendar_months(
        stats_data: &StatisticsData,
        books: &[LibraryItem],
        time_config: &TimeConfig,
    ) -> CalendarMonths {
        // Group page stats by book ID first
        let mut book_sessions: HashMap<i64, Vec<&PageStat>> = HashMap::new();
        for stat in &stats_data.page_stats {
            if stat.duration > 0 {
                book_sessions.entry(stat.id_book).or_default().push(stat);
            }
        }

        let mut calendar_events = Vec::new();
        let mut calendar_items: BTreeMap<String, CalendarItem> = BTreeMap::new();

        // Build a lookup map from partial MD5 checksum to the corresponding book detail path,
        // cover path, and content type.
        //
        // Prefer MD5 from KOReader metadata, but fall back to calculating a compatible partial MD5
        // from the file. This enables calendar → detail linking even for items without metadata.
        let md5_to_book_info: HashMap<String, (String, String, ContentType)> = books
            .iter()
            .filter_map(|b| {
                let md5 = b
                    .koreader_metadata
                    .as_ref()
                    .and_then(|m| m.partial_md5_checksum.as_ref())
                    .cloned()
                    .or_else(|| calculate_partial_md5(&b.file_path).ok());

                md5.map(|md5| {
                    (
                        md5.to_lowercase(),
                        (
                            match b.content_type() {
                                ContentType::Book => format!("/books/{}/", b.id),
                                ContentType::Comic => format!("/comics/{}/", b.id),
                            },
                            format!("/assets/covers/{}.webp", b.id),
                            b.content_type(),
                        ),
                    )
                })
            })
            .collect();

        for (book_id, sessions) in book_sessions {
            // Retrieve the corresponding book metadata
            let stat_book = match stats_data.books.iter().find(|b| b.id == book_id) {
                Some(book) => book,
                None => continue,
            };

            if sessions.is_empty() {
                continue;
            }

            // Generate a unique item ID for the calendar system
            let calendar_item_id = stat_book.md5.clone();

            // Create the item metadata entry if we haven't already
            if !calendar_items.contains_key(&calendar_item_id) {
                let authors = Self::parse_authors(&stat_book.authors);
                let (item_path, item_cover, content_type_from_library) = md5_to_book_info
                    .get(&stat_book.md5.to_lowercase())
                    .map(|(path, cover, ct)| (Some(path.clone()), Some(cover.clone()), Some(*ct)))
                    .unwrap_or((None, None, None));

                let content_type = content_type_from_library
                    .or(stat_book.content_type)
                    .unwrap_or(ContentType::Book);

                let calendar_item =
                    CalendarItem::new(stat_book.title.clone(), authors, content_type, item_path, item_cover);
                calendar_items.insert(calendar_item_id.clone(), calendar_item);
            }

            // Sort sessions by start time globally for this book
            let mut sorted_sessions = sessions;
            sorted_sessions.sort_by_key(|s| s.start_time);

            // Build sessions grouped per day (NaiveDate)
            let mut sessions_by_day: HashMap<NaiveDate, Vec<&PageStat>> = HashMap::new();
            for session in &sorted_sessions {
                if let Ok(date) = NaiveDate::parse_from_str(
                    &Self::timestamp_to_date_string(session.start_time, time_config),
                    "%Y-%m-%d",
                ) {
                    sessions_by_day.entry(date).or_default().push(session);
                }
            }

            let mut days: Vec<NaiveDate> = sessions_by_day.keys().cloned().collect();
            if days.is_empty() {
                continue;
            }
            days.sort();

            // Process reading days and create calendar events
            Self::create_calendar_events_from_reading_days(
                &mut calendar_events,
                &calendar_item_id,
                &days,
                &sessions_by_day,
            );
        }

        // Sort events deterministically by start date then item title to ensure consistent ordering across builds
        calendar_events.sort_by(|a, b| {
            use std::cmp::Ordering;
            match a.start.cmp(&b.start) {
                Ordering::Equal => {
                    let title_a = calendar_items
                        .get(&a.item_id)
                        .map(|bk| bk.title.as_str())
                        .unwrap_or("");
                    let title_b = calendar_items
                        .get(&b.item_id)
                        .map(|bk| bk.title.as_str())
                        .unwrap_or("");
                    title_a.cmp(title_b)
                }
                other => other,
            }
        });

        //------------------------------------------------------------------
        // Build per-month payloads ----------------------------------------
        //------------------------------------------------------------------

        // Pre-compute monthly statistics once.
        let monthly_stats_map = Self::build_monthly_stats(stats_data, time_config);
        let monthly_stats_books_map = Self::build_monthly_stats(
            &stats_data.filtered_by_content_type(ContentType::Book),
            time_config,
        );
        let monthly_stats_comics_map = Self::build_monthly_stats(
            &stats_data.filtered_by_content_type(ContentType::Comic),
            time_config,
        );

        let mut months_map: std::collections::BTreeMap<String, CalendarMonthData> =
            std::collections::BTreeMap::new();

        // Helper for a zeroed MonthlyStats value
        let zero_stats = MonthlyStats {
            books_read: 0,
            pages_read: 0,
            time_read: 0,
            days_read_pct: 0,
        };

        for ev in &calendar_events {
            // Parse start date
            let start_date = chrono::NaiveDate::parse_from_str(&ev.start, "%Y-%m-%d").unwrap();

            // Determine exclusive end date
            let end_exclusive = if let Some(ref end_str) = ev.end {
                chrono::NaiveDate::parse_from_str(end_str, "%Y-%m-%d").unwrap()
            } else {
                start_date + chrono::Duration::days(1)
            };

            // Iterate months overlapped by this event
            let mut iter_date = start_date;
            while iter_date < end_exclusive {
                let year_month = format!("{}-{:02}", iter_date.year(), iter_date.month());

                // Ensure entry exists
                let month_entry =
                    months_map
                        .entry(year_month.clone())
                        .or_insert_with(|| CalendarMonthData {
                            events: Vec::new(),
                            books: std::collections::BTreeMap::new(),
                            stats: monthly_stats_map
                                .get(&year_month)
                                .cloned()
                                .unwrap_or(zero_stats.clone()),
                            stats_books: monthly_stats_books_map
                                .get(&year_month)
                                .cloned()
                                .unwrap_or(zero_stats.clone()),
                            stats_comics: monthly_stats_comics_map
                                .get(&year_month)
                                .cloned()
                                .unwrap_or(zero_stats.clone()),
                        });

                // Push event (clone so that events can live in multiple months if spanning)
                month_entry.events.push(ev.clone());

                // Add corresponding item metadata
                if let Some(item_meta) = calendar_items.get(&ev.item_id) {
                    month_entry
                        .books
                        .entry(ev.item_id.clone())
                        .or_insert(item_meta.clone());
                }

                // Move iter_date to first day of next month
                iter_date = if iter_date.month() == 12 {
                    chrono::NaiveDate::from_ymd_opt(iter_date.year() + 1, 1, 1).unwrap()
                } else {
                    chrono::NaiveDate::from_ymd_opt(iter_date.year(), iter_date.month() + 1, 1)
                        .unwrap()
                };
            }
        }

        months_map
    }

    /// Parse authors string into a vector of author names
    fn parse_authors(authors_str: &str) -> Vec<String> {
        if authors_str.is_empty() {
            Vec::new()
        } else {
            authors_str
                .split(&[',', ';'])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
    }

    /// Create calendar events from reading days, grouping consecutive days (spanning month boundaries).
    fn create_calendar_events_from_reading_days(
        calendar_events: &mut Vec<CalendarEvent>,
        calendar_item_id: &str,
        days: &[NaiveDate],
        sessions_by_day: &HashMap<NaiveDate, Vec<&PageStat>>,
    ) {
        if days.is_empty() {
            return;
        }

        let mut span_start = days[0];
        let mut prev_day = days[0];
        let mut span_sessions: Vec<&PageStat> =
            sessions_by_day.get(&days[0]).cloned().unwrap_or_default();

        for day in days.iter().skip(1) {
            let is_consecutive_day = *day == prev_day + Duration::days(1);

            if is_consecutive_day {
                // Same streak (no longer restricted to month)
                if let Some(day_sessions) = sessions_by_day.get(day) {
                    span_sessions.extend(day_sessions);
                }
                prev_day = *day;
            } else {
                // Close previous streak
                Self::create_calendar_event(
                    calendar_events,
                    calendar_item_id,
                    span_start,
                    prev_day,
                    &span_sessions,
                );

                // Start new streak
                span_start = *day;
                prev_day = *day;
                span_sessions = sessions_by_day.get(day).cloned().unwrap_or_default();
            }
        }

        // Push final streak
        Self::create_calendar_event(
            calendar_events,
            calendar_item_id,
            span_start,
            prev_day,
            &span_sessions,
        );
    }

    /// Create a calendar event from reading sessions spanning consecutive days
    fn create_calendar_event(
        calendar_events: &mut Vec<CalendarEvent>,
        calendar_item_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        sessions: &[&PageStat],
    ) {
        let total_read_time: i64 = sessions.iter().map(|s| s.duration).sum();
        let total_pages_read = sessions.len() as i64;

        let end_exclusive = if start_date == end_date {
            None
        } else {
            Some(
                (end_date + Duration::days(1))
                    .format("%Y-%m-%d")
                    .to_string(),
            )
        };

        let event = CalendarEvent::new(
            start_date.format("%Y-%m-%d").to_string(),
            end_exclusive,
            total_read_time,
            total_pages_read,
            calendar_item_id.to_string(),
        );

        calendar_events.push(event);
    }

    /// Convert Unix timestamp to ISO date string (yyyy-mm-dd)
    fn timestamp_to_date_string(timestamp: i64, time_config: &TimeConfig) -> String {
        time_config.format_date(timestamp)
    }

    /// Build per-month reading statistics from raw `StatisticsData`.
    /// Returns a map keyed by "YYYY-MM" → `MonthlyStats`.
    pub fn build_monthly_stats(
        stats_data: &StatisticsData,
        time_config: &TimeConfig,
    ) -> std::collections::HashMap<String, MonthlyStats> {
        use chrono::{Datelike, NaiveDate};
        use std::collections::{HashMap, HashSet};

        #[derive(Default)]
        struct MonthlyStatsAccumulator {
            unique_books: HashSet<i64>,
            unique_dates: HashSet<NaiveDate>,
            total_pages: i64,
            total_time: i64,
        }

        let mut acc_by_month: HashMap<String, MonthlyStatsAccumulator> = HashMap::new();

        // Aggregate raw page statistics into monthly accumulators
        for ps in &stats_data.page_stats {
            if ps.duration <= 0 {
                continue;
            }

            // Convert timestamp to logical local date (yyyy-mm-dd)
            let local_date = time_config.date_for_timestamp(ps.start_time);

            let year_month = format!("{}-{:02}", local_date.year(), local_date.month());

            let acc = acc_by_month.entry(year_month).or_default();
            acc.unique_books.insert(ps.id_book);
            acc.unique_dates.insert(local_date);
            acc.total_pages += 1; // Each PageStat represents one page read
            acc.total_time += ps.duration;
        }

        // Convert accumulators into final MonthlyStats map
        let mut monthly_stats_map: HashMap<String, MonthlyStats> = HashMap::new();

        for (year_month, acc) in acc_by_month {
            // Determine days in the month
            let parts: Vec<&str> = year_month.split('-').collect();
            let (y, m): (i32, u32) = (
                parts[0].parse().unwrap_or(1970),
                parts[1].parse().unwrap_or(1),
            );

            let first_of_month = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
            let first_next_month = if m == 12 {
                NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap()
            } else {
                NaiveDate::from_ymd_opt(y, m + 1, 1).unwrap()
            };
            let days_in_month = (first_next_month - first_of_month).num_days() as usize;

            let days_pct = ((acc.unique_dates.len() * 100) / days_in_month) as u8;

            monthly_stats_map.insert(
                year_month.clone(),
                MonthlyStats {
                    books_read: acc.unique_books.len(),
                    pages_read: acc.total_pages,
                    time_read: acc.total_time,
                    days_read_pct: days_pct,
                },
            );
        }

        monthly_stats_map
    }
}
