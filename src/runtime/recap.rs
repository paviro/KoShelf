//! Recap share image generation and page scaling helpers.
//!
//! Computes yearly completion summaries and renders share images to disk.
//! The reading domain service handles completion data on demand — this module
//! only generates the visual share assets.

use crate::domain::reading::types::{MonthRecap, RecapItem, YearlySummary};
use crate::koreader::types::{DailyStats, PageStat, ReadingStats, StatisticsData};
use crate::models::LibraryItem;
use crate::time_config::TimeConfig;
use anyhow::Result;
use chrono::Datelike;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime};

// ── Page scaling ────────────────────────────────────────────────────────

/// Scaling factors for converting rendered page numbers to synthetic stable-page
/// equivalents used by KOReader.
#[derive(Debug, Clone)]
struct PageScaling {
    enabled: bool,
    factor_by_md5: HashMap<String, f64>,
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
                factor_by_md5: HashMap::new(),
            };
        }

        let Some(stats_data) = stats_data else {
            return Self {
                enabled: true,
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

            factor_by_md5.insert(md5_key, factor);
        }

        Self {
            enabled: true,
            factor_by_md5,
        }
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

    fn scale_pages_with_factor(pages: i64, factor: f64) -> i64 {
        if pages <= 0 || !factor.is_finite() || factor <= 0.0 {
            return 0;
        }
        Self::round_pages(pages as f64 * factor)
    }

    fn round_pages(value: f64) -> i64 {
        if !value.is_finite() || value <= 0.0 {
            return 0;
        }
        value.round() as i64
    }
}

// ── Recap generation ────────────────────────────────────────────────────

type YearMonthItems = HashMap<i32, BTreeMap<String, Vec<RecapItem>>>;

/// Parse completion end date into `(year, year_month)` where `year_month` is `YYYY-MM`.
fn completion_year_and_month(end_date: &str) -> Option<(i32, String)> {
    let year_str = end_date.get(0..4)?;
    let year_month = end_date.get(0..7)?.to_string();
    let year = year_str.parse::<i32>().ok()?;
    Some((year, year_month))
}

fn build_md5_to_item(items: &[LibraryItem]) -> HashMap<String, &LibraryItem> {
    let mut md5_to_book: HashMap<String, &LibraryItem> = HashMap::new();
    for book in items {
        if let Some(md5) = book
            .koreader_metadata
            .as_ref()
            .and_then(|m| m.partial_md5_checksum.as_ref())
        {
            md5_to_book.insert(md5.to_lowercase(), book);
        }
    }
    md5_to_book
}

fn group_completions_by_year_month(
    stats_data: &StatisticsData,
    md5_to_item: &HashMap<String, &LibraryItem>,
    page_scaling: &PageScaling,
) -> (YearMonthItems, Vec<i32>) {
    let mut year_month_items: YearMonthItems = HashMap::new();
    let mut year_set: HashSet<i32> = HashSet::new();

    for sb in &stats_data.books {
        if let Some(comps) = &sb.completions {
            for c in &comps.entries {
                let Some((year, ym)) = completion_year_and_month(&c.end_date) else {
                    continue;
                };

                year_set.insert(year);

                let (
                    title,
                    authors,
                    rating,
                    review_note,
                    series_display,
                    item_id,
                    item_cover,
                    content_type,
                ) = if let Some(item) = md5_to_item.get(&sb.md5.to_lowercase()) {
                    (
                        item.book_info.title.clone(),
                        item.book_info.authors.clone(),
                        item.rating(),
                        item.review_note().cloned(),
                        item.series_display(),
                        Some(item.id.clone()),
                        Some(format!("/assets/covers/{}.webp", item.id)),
                        Some(item.content_type()),
                    )
                } else {
                    let title = sb.title.clone();
                    let authors = sb
                        .authors
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>();
                    (
                        title,
                        authors,
                        None,
                        None,
                        None,
                        None,
                        None,
                        sb.content_type,
                    )
                };

                let pages_read = page_scaling.scale_pages_for_md5(&sb.md5, c.pages_read);
                let average_speed = if c.reading_time > 0 && pages_read > 0 {
                    Some(pages_read as f64 / (c.reading_time as f64 / 3600.0))
                } else {
                    None
                };

                let item = RecapItem {
                    title,
                    authors,
                    start_date: c.start_date.clone(),
                    end_date: c.end_date.clone(),
                    reading_time: c.reading_time,
                    session_count: c.session_count,
                    pages_read,
                    calendar_length_days: c.calendar_length_days(),
                    rating,
                    review_note,
                    series_display,
                    item_id,
                    item_cover,
                    content_type,
                    average_speed,
                    avg_session_duration: c.avg_session_duration(),
                };

                year_month_items
                    .entry(year)
                    .or_default()
                    .entry(ym.clone())
                    .or_default()
                    .push(item);
            }
        }
    }

    let mut years: Vec<i32> = year_set.into_iter().collect();
    years.sort_by(|a, b| b.cmp(a)); // newest first
    (year_month_items, years)
}

fn month_hours_for(daily: &[DailyStats]) -> HashMap<String, i64> {
    let mut out: HashMap<String, i64> = HashMap::new();
    for day in daily {
        if day.date.len() >= 7 {
            let ym = day.date[0..7].to_string();
            *out.entry(ym).or_insert(0) += day.read_time;
        }
    }
    out
}

fn build_monthly_recaps_all(
    months_map: BTreeMap<String, Vec<RecapItem>>,
    month_hours: &HashMap<String, i64>,
) -> Vec<MonthRecap> {
    let mut monthly: BTreeMap<String, MonthRecap> = BTreeMap::new();

    for (ym, mut items) in months_map {
        if items.is_empty() {
            continue;
        }
        items.sort_by(|a, b| b.end_date.cmp(&a.end_date));
        let hours = *month_hours.get(&ym).unwrap_or(&0);
        monthly.insert(
            ym.clone(),
            MonthRecap {
                month_key: ym,
                books_finished: items.len(),
                hours_read_seconds: hours,
                items,
            },
        );
    }

    monthly.into_values().collect()
}

fn compute_yearly_summary(
    year: i32,
    monthly: &[MonthRecap],
    month_hours: &HashMap<String, i64>,
    reading_stats: &ReadingStats,
    page_stats: &[PageStat],
) -> YearlySummary {
    let year_str = format!("{}", year);
    let year_prefix = format!("{}-", year);

    let total_books = monthly.iter().map(|month| month.books_finished).sum();
    let total_time_seconds: i64 = month_hours
        .iter()
        .filter(|(month_key, _)| month_key.starts_with(&year_prefix))
        .map(|(_, seconds)| *seconds)
        .sum();

    let year_page_stats: Vec<PageStat> = page_stats
        .iter()
        .filter(|ps| {
            if let Some(dt) = chrono::DateTime::from_timestamp(ps.start_time, 0) {
                dt.year() == year
            } else {
                false
            }
        })
        .cloned()
        .collect();

    let all_sessions =
        crate::domain::reading::session_calc::aggregate_session_durations(&year_page_stats);
    let session_count = all_sessions.len() as i64;
    let longest_session_duration = all_sessions.iter().max().copied().unwrap_or(0);
    let average_session_duration = if session_count > 0 {
        all_sessions.iter().sum::<i64>() / session_count
    } else {
        0
    };

    let active_days: usize = reading_stats
        .daily_activity
        .iter()
        .filter(|day| day.date.starts_with(&year_str) && day.read_time > 0)
        .count();

    let days_in_year =
        if chrono::NaiveDate::from_ymd_opt(year, 12, 31).is_some_and(|d| d.ordinal() == 366) {
            366.0
        } else {
            365.0
        };
    let active_days_percentage = (active_days as f64 / days_in_year * 100.0).round();

    let mut year_reading_dates: Vec<chrono::NaiveDate> = reading_stats
        .daily_activity
        .iter()
        .filter(|day| day.date.starts_with(&year_str) && day.read_time > 0)
        .filter_map(|day| chrono::NaiveDate::parse_from_str(&day.date, "%Y-%m-%d").ok())
        .collect();
    year_reading_dates.sort();
    year_reading_dates.dedup();

    let longest_streak = if year_reading_dates.is_empty() {
        0
    } else {
        let mut max_streak = 1i64;
        let mut current_streak = 1i64;
        for i in 1..year_reading_dates.len() {
            let diff = (year_reading_dates[i] - year_reading_dates[i - 1]).num_days();
            if diff == 1 {
                current_streak += 1;
                max_streak = max_streak.max(current_streak);
            } else {
                current_streak = 1;
            }
        }
        max_streak
    };

    let best_month: Option<(String, i64)> = month_hours
        .iter()
        .filter(|(month_key, _)| month_key.starts_with(&year_prefix))
        .max_by_key(|(_, seconds)| *seconds)
        .map(|(month_key, seconds)| (month_key.clone(), *seconds));

    let best_month = best_month.and_then(|(month_key, seconds)| (seconds > 0).then_some(month_key));

    let total_minutes = total_time_seconds / 60;
    let total_time_days = total_minutes / (24 * 60);
    let total_time_hours = (total_minutes % (24 * 60)) / 60;

    let longest_session_total_mins = longest_session_duration / 60;
    let longest_session_hours = longest_session_total_mins / 60;
    let longest_session_minutes = longest_session_total_mins % 60;

    let avg_session_total_mins = average_session_duration / 60;
    let average_session_hours = avg_session_total_mins / 60;
    let average_session_minutes = avg_session_total_mins % 60;

    YearlySummary {
        total_books,
        total_time_seconds,
        total_time_days,
        total_time_hours,
        longest_session_hours,
        longest_session_minutes,
        average_session_hours,
        average_session_minutes,
        active_days,
        active_days_percentage,
        longest_streak,
        best_month,
    }
}

fn cleanup_stale_recap_share_assets(valid_years: &HashSet<String>, recap_dir: &Path) -> Result<()> {
    if !recap_dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(recap_dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if extension != "webp" && extension != "svg" {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some((year_prefix, _)) = file_name.split_once('_') else {
            continue;
        };

        if !year_prefix.chars().all(|ch| ch.is_ascii_digit()) {
            continue;
        }

        if !valid_years.contains(year_prefix) {
            info!("Removing stale recap share asset: {:?}", path);
            if let Err(error) = fs::remove_file(&path) {
                warn!(
                    "Failed to remove stale recap share asset {:?}: {}",
                    path, error
                );
            }
        }
    }

    Ok(())
}

/// Collect share image generation tasks for a single year.
///
/// Returns spawned `JoinHandle`s for images that need (re)generation.
/// Each task sends `()` to `progress_tx` on completion for progress tracking.
fn collect_share_tasks_for_year(
    year: i32,
    summary: &YearlySummary,
    recap_dir: &Path,
    stats_db_time: SystemTime,
    progress_tx: &std::sync::mpsc::Sender<()>,
) -> Vec<tokio::task::JoinHandle<()>> {
    let share_data = crate::share::ShareImageData {
        year,
        books_read: summary.total_books as u32,
        reading_time_hours: summary.total_time_hours as u32,
        reading_time_days: summary.total_time_days as u32,
        active_days: summary.active_days as u32,
        active_days_percentage: summary.active_days_percentage as u8,
        longest_streak: summary.longest_streak as u32,
        best_month: summary.best_month.clone(),
    };

    let formats = [
        crate::share::ShareFormat::Story,
        crate::share::ShareFormat::Square,
        crate::share::ShareFormat::Banner,
    ];

    let recap_dir_owned = recap_dir.to_path_buf();
    formats
        .into_iter()
        .filter_map(|format| {
            let output_path = recap_dir_owned.join(format!("{}_{}", year, format.filename()));

            let should_generate = match fs::metadata(&output_path) {
                Ok(img_meta) => {
                    let img_time = img_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    stats_db_time > img_time
                }
                Err(_) => true,
            };

            if should_generate {
                let share_data = share_data.clone();
                let tx = progress_tx.clone();
                Some(tokio::task::spawn_blocking(move || {
                    if let Err(e) =
                        crate::share::generate_share_image(&share_data, format, &output_path)
                    {
                        log::warn!("Failed to generate share image {:?}: {}", output_path, e);
                    }
                    let _ = tx.send(());
                }))
            } else {
                None
            }
        })
        .collect()
}

/// Generate recap share images for all completion years.
///
/// Computes yearly summaries from the "all" scope and renders share images
/// to the recap assets directory.
pub async fn generate_recap_share_images(
    stats_data: &StatisticsData,
    items: &[LibraryItem],
    use_stable_page_metadata: bool,
    recap_dir: &Path,
    stats_db_path: Option<&Path>,
    time_config: &TimeConfig,
) -> Result<()> {
    let page_scaling = PageScaling::from_inputs(use_stable_page_metadata, items, Some(stats_data));
    let md5_to_book = build_md5_to_item(items);

    let reading_stats_all =
        crate::domain::reading::StatisticsCalculator::calculate_stats(stats_data, time_config);

    let (year_month_items, years) =
        group_completions_by_year_month(stats_data, &md5_to_book, &page_scaling);

    if years.is_empty() {
        let empty: HashSet<String> = HashSet::new();
        cleanup_stale_recap_share_assets(&empty, recap_dir)?;
        return Ok(());
    }

    let month_hours_all = month_hours_for(&reading_stats_all.daily_activity);

    let stats_db_time = stats_db_path
        .and_then(|p| fs::metadata(p).ok())
        .and_then(|m| m.modified().ok())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    fs::create_dir_all(recap_dir)?;

    let (progress_tx, progress_rx) = std::sync::mpsc::channel::<()>();
    let mut all_tasks = Vec::new();

    for year in &years {
        let months_map = year_month_items.get(year).cloned().unwrap_or_default();
        let monthly = build_monthly_recaps_all(months_map, &month_hours_all);

        let summary = compute_yearly_summary(
            *year,
            &monthly,
            &month_hours_all,
            &reading_stats_all,
            &stats_data.page_stats,
        );

        all_tasks.extend(collect_share_tasks_for_year(
            *year,
            &summary,
            recap_dir,
            stats_db_time,
            &progress_tx,
        ));
    }

    let total_tasks = all_tasks.len();

    if total_tasks > 0 {
        let start = Instant::now();
        info!("Rendering share images...");
        let pb = ProgressBar::new(total_tasks as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} {bar:30.cyan/blue} {pos}/{len}")
                .unwrap()
                .progress_chars("━╸─"),
        );
        pb.set_message("Rendering share images:");

        drop(progress_tx);

        let pb_clone = pb.clone();
        let progress_task = tokio::task::spawn_blocking(move || {
            while progress_rx.recv().is_ok() {
                pb_clone.inc(1);
            }
        });

        for task in all_tasks {
            let _ = task.await;
        }

        let _ = progress_task.await;
        pb.finish_and_clear();

        info!(
            "Rendered share images in {:.1}s",
            start.elapsed().as_secs_f64()
        );
    } else {
        drop(progress_tx);
    }

    let current_years: HashSet<String> = years.iter().map(|y| y.to_string()).collect();
    cleanup_stale_recap_share_assets(&current_years, recap_dir)?;

    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::koreader::types::StreakInfo;
    use crate::models::ContentType;

    // ── PageScaling tests ───────────────────────────────────────────────

    mod scaling {
        use super::*;
        use crate::tests::fixtures;

        fn test_item(id: &str, md5: &str, synthetic: bool, stable_total: u32) -> LibraryItem {
            let mut metadata =
                fixtures::koreader_metadata_for_pages(md5, true, synthetic, stable_total);
            metadata.pagemap_current_page_label = None;
            metadata.pagemap_last_page_label = None;
            fixtures::library_item(id, Some(metadata))
        }

        #[test]
        fn builds_factors_for_synthetic_items_only() {
            let item_synthetic = test_item("1", "md5-synth", true, 300);
            let item_publisher_only = test_item("2", "md5-pub", false, 400);

            let books = vec![
                fixtures::stat_book(1, "md5-synth", 200, ContentType::Book),
                fixtures::stat_book(2, "md5-pub", 200, ContentType::Book),
            ];
            let stats_data = fixtures::statistics_data(books.clone(), Vec::new());

            let scaling = PageScaling::from_inputs(
                true,
                &[item_synthetic.clone(), item_publisher_only.clone()],
                Some(&stats_data),
            );
            assert_eq!(scaling.factor_for_md5("md5-synth"), Some(1.5));
            assert_eq!(scaling.factor_for_md5("md5-pub"), None);

            let off = PageScaling::from_inputs(false, &[item_synthetic], Some(&stats_data));
            assert_eq!(off.factor_for_md5("md5-synth"), None);
        }

        #[test]
        fn builds_factors_even_when_page_labels_are_disabled() {
            let mut item = test_item("1", "md5-synth-no-labels", true, 300);
            if let Some(metadata) = item.koreader_metadata.as_mut() {
                metadata.pagemap_use_page_labels = Some(false);
            }

            let books = vec![fixtures::stat_book(
                1,
                "md5-synth-no-labels",
                200,
                ContentType::Book,
            )];
            let stats_data = fixtures::statistics_data(books.clone(), Vec::new());

            let scaling = PageScaling::from_inputs(true, &[item], Some(&stats_data));
            assert_eq!(scaling.factor_for_md5("md5-synth-no-labels"), Some(1.5));
        }
    }

    // ── Recap summary tests ─────────────────────────────────────────────

    fn empty_reading_stats() -> ReadingStats {
        ReadingStats {
            total_read_time: 0,
            total_page_reads: 0,
            longest_read_time_in_day: 0,
            most_pages_in_day: 0,
            average_session_duration: None,
            longest_session_duration: None,
            total_completions: 0,
            books_completed: 0,
            most_completions: 0,
            longest_streak: StreakInfo::new(0, None, None),
            current_streak: StreakInfo::new(0, None, None),
            weeks: Vec::new(),
            daily_activity: Vec::new(),
        }
    }

    #[test]
    fn yearly_summary_includes_months_without_completions() {
        let monthly = vec![MonthRecap {
            month_key: "2025-02".to_string(),
            books_finished: 2,
            hours_read_seconds: 352_800,
            items: Vec::new(),
        }];
        let month_hours = HashMap::from([
            ("2025-01".to_string(), 36_000),
            ("2025-02".to_string(), 352_800),
            ("2024-12".to_string(), 99_999),
        ]);

        let summary =
            compute_yearly_summary(2025, &monthly, &month_hours, &empty_reading_stats(), &[]);

        assert_eq!(summary.total_books, 2);
        assert_eq!(summary.total_time_seconds, 388_800);
        assert_eq!(summary.total_time_days, 4);
        assert_eq!(summary.total_time_hours, 12);
        assert_eq!(summary.best_month.as_deref(), Some("2025-02"));
    }
}
