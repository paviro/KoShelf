//! Recap share image generation and page scaling helpers.
//!
//! Computes yearly completion summaries and renders share images to disk.
//! The reading domain service handles completion data on demand — this module
//! only generates the visual share assets.

use crate::pipeline::share::{ShareFormat, ShareImageData, generate_share_image};
use crate::server::api::responses::library::LibraryContentType;
use crate::shelf::models::ContentType;
use crate::shelf::statistics::StatisticsCalculator;
use crate::shelf::statistics::compute::scaling::PageScaling;
use crate::shelf::statistics::types::{MonthRecap, RecapItem, YearlySummary};
use crate::shelf::time_config::TimeConfig;
use crate::source::koreader::types::{DailyStats, PageStat, ReadingStats, StatisticsData};
use crate::store::sqlite::repo::LibraryRepository;
use anyhow::Result;
use chrono::Datelike;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::time::Instant;

// ── Recap generation ────────────────────────────────────────────────────

type YearMonthItems = HashMap<i32, BTreeMap<String, Vec<RecapItem>>>;

/// Parse completion end date into `(year, year_month)` where `year_month` is `YYYY-MM`.
fn completion_year_and_month(end_date: &str) -> Option<(i32, String)> {
    let year_str = end_date.get(0..4)?;
    let year_month = end_date.get(0..7)?.to_string();
    let year = year_str.parse::<i32>().ok()?;
    Some((year, year_month))
}

async fn group_completions_by_year_month(
    stats_data: &StatisticsData,
    repo: &LibraryRepository,
    page_scaling: &PageScaling,
) -> (YearMonthItems, Vec<i32>) {
    let mut year_month_items: YearMonthItems = HashMap::new();
    let mut year_set: HashSet<i32> = HashSet::new();

    // Pre-fetch library items for all unique MD5s.
    let unique_md5s: HashSet<String> = stats_data
        .books
        .iter()
        .filter(|sb| sb.completions.is_some())
        .map(|sb| sb.md5.clone())
        .collect();

    let mut item_cache: HashMap<
        String,
        Option<crate::server::api::responses::library::LibraryDetailItem>,
    > = HashMap::new();
    for md5 in &unique_md5s {
        let detail = repo.get_item(md5).await.ok().flatten();
        item_cache.insert(md5.clone(), detail);
    }

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
                ) = if let Some(Some(detail)) = item_cache.get(&sb.md5) {
                    let series_display = detail.series.as_ref().and_then(|s| {
                        if s.name.is_empty() {
                            None
                        } else {
                            Some(match &s.index {
                                Some(idx) => format!("{} #{}", s.name, idx),
                                None => s.name.clone(),
                            })
                        }
                    });
                    let ct = match detail.content_type {
                        LibraryContentType::Book => ContentType::Book,
                        LibraryContentType::Comic => ContentType::Comic,
                    };
                    (
                        detail.title.clone(),
                        detail.authors.0.clone(),
                        detail.rating,
                        detail.review_note.clone(),
                        series_display,
                        Some(detail.id.clone()),
                        Some(detail.cover_url.clone()),
                        Some(ct),
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
        crate::shelf::statistics::compute::sessions::aggregate_session_durations(&year_page_stats);
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
    let active_days_percentage = (active_days as f64 / days_in_year * 100.0).round() as u8;

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

/// Spawn share image rendering tasks for a single year.
///
/// All three formats are rendered — the caller has already determined
/// this year needs regeneration via fingerprint comparison.
fn spawn_share_tasks_for_year(
    share_data: &ShareImageData,
    recap_dir: &Path,
    progress_tx: &mpsc::Sender<()>,
) -> Vec<tokio::task::JoinHandle<()>> {
    let formats = [ShareFormat::Story, ShareFormat::Square, ShareFormat::Banner];
    let recap_dir_owned = recap_dir.to_path_buf();

    formats
        .into_iter()
        .map(|format| {
            let output_path =
                recap_dir_owned.join(format!("{}_{}", share_data.year, format.filename()));
            let share_data = share_data.clone();
            let tx = progress_tx.clone();
            tokio::task::spawn_blocking(move || {
                if let Err(e) = generate_share_image(&share_data, format, &output_path) {
                    log::warn!("Failed to generate share image {:?}: {}", output_path, e);
                }
                let _ = tx.send(());
            })
        })
        .collect()
}

/// Compute `ShareImageData` for each year that has completions.
async fn compute_share_data_per_year(
    stats_data: &StatisticsData,
    repo: &LibraryRepository,
    page_scaling: &PageScaling,
    time_config: &TimeConfig,
) -> HashMap<i32, ShareImageData> {
    let reading_stats_all = StatisticsCalculator::calculate_stats(stats_data, time_config);

    let (year_month_items, years) =
        group_completions_by_year_month(stats_data, repo, page_scaling).await;

    if years.is_empty() {
        return HashMap::new();
    }

    let month_hours_all = month_hours_for(&reading_stats_all.daily_activity);

    let mut result = HashMap::new();
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

        result.insert(
            *year,
            ShareImageData {
                year: *year,
                books_read: summary.total_books as u32,
                reading_time_hours: summary.total_time_hours as u32,
                reading_time_days: summary.total_time_days as u32,
                active_days: summary.active_days as u32,
                active_days_percentage: summary.active_days_percentage,
                longest_streak: summary.longest_streak as u32,
                best_month: summary.best_month.clone(),
            },
        );
    }

    result
}

fn share_images_exist_on_disk(year: i32, recap_dir: &Path) -> bool {
    [ShareFormat::Story, ShareFormat::Square, ShareFormat::Banner]
        .iter()
        .all(|fmt| {
            recap_dir
                .join(format!("{}_{}", year, fmt.filename()))
                .exists()
        })
}

/// Regenerate share images for years where the content fingerprint changed.
///
/// Compares computed `ShareImageData` fingerprints against DB-stored values.
/// Only renders images for years with changed data. When `show_progress` is
/// true, displays a progress bar (startup); otherwise logs silently (runtime).
pub async fn regenerate_share_images(
    stats_data: &StatisticsData,
    repo: &LibraryRepository,
    page_scaling: &PageScaling,
    recap_dir: &Path,
    time_config: &TimeConfig,
    show_progress: bool,
) -> Result<()> {
    let share_data_by_year =
        compute_share_data_per_year(stats_data, repo, page_scaling, time_config).await;

    let valid_years: Vec<i32> = share_data_by_year.keys().copied().collect();

    if share_data_by_year.is_empty() {
        let empty: HashSet<String> = HashSet::new();
        cleanup_stale_recap_share_assets(&empty, recap_dir)?;
        if let Err(e) = repo.cleanup_stale_share_image_fingerprints(&[]).await {
            warn!("Failed to cleanup stale share image fingerprints: {}", e);
        }
        return Ok(());
    }

    // Compare computed fingerprints with stored values to find years needing render
    let stored_fingerprints = repo
        .load_share_image_fingerprints()
        .await
        .unwrap_or_default();

    let mut years_to_render: Vec<(i32, ShareImageData, String)> = Vec::new();
    for (year, data) in &share_data_by_year {
        let new_fp = data.fingerprint();
        let needs_render = match stored_fingerprints.get(year) {
            Some(stored_fp) => {
                stored_fp != &new_fp || !share_images_exist_on_disk(*year, recap_dir)
            }
            None => true,
        };
        if needs_render {
            years_to_render.push((*year, data.clone(), new_fp));
        }
    }

    if !years_to_render.is_empty() {
        fs::create_dir_all(recap_dir)?;

        let (progress_tx, progress_rx) = mpsc::channel::<()>();
        let mut all_tasks = Vec::new();

        for (_, data, _) in &years_to_render {
            all_tasks.extend(spawn_share_tasks_for_year(data, recap_dir, &progress_tx));
        }

        let total_tasks = all_tasks.len();
        let start = Instant::now();

        if show_progress {
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
                if let Err(e) = task.await {
                    warn!("Share image task failed: {}", e);
                }
            }

            if let Err(e) = progress_task.await {
                warn!("Progress tracking task failed: {}", e);
            }
            pb.finish_and_clear();
        } else {
            drop(progress_tx);
            for task in all_tasks {
                if let Err(e) = task.await {
                    warn!("Share image task failed: {}", e);
                }
            }
        }

        info!(
            "Rendered {} share images in {:.1}s",
            total_tasks,
            start.elapsed().as_secs_f64()
        );

        for (year, _, fp) in &years_to_render {
            if let Err(e) = repo.upsert_share_image_fingerprint(*year, fp).await {
                warn!(
                    "Failed to store share image fingerprint for {}: {}",
                    year, e
                );
            }
        }
    }

    // Cleanup stale assets and fingerprints
    let current_years: HashSet<String> = valid_years.iter().map(|y| y.to_string()).collect();
    cleanup_stale_recap_share_assets(&current_years, recap_dir)?;
    if let Err(e) = repo
        .cleanup_stale_share_image_fingerprints(&valid_years)
        .await
    {
        warn!("Failed to cleanup stale share image fingerprints: {}", e);
    }

    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::koreader::types::StreakInfo;

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
