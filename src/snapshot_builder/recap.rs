//! Yearly recap payload computation with share images.

use super::SnapshotBuilder;
use super::utils::completion_year_and_month;
use crate::contracts::{common::ContentTypeFilter, mappers, recap::RecapShareAssets};
use crate::models::{
    ContentType, DailyStats, LibraryItem, MonthRecap, PageStat, ReadingStats, RecapItem,
    StatisticsData, YearlySummary,
};
use crate::runtime::ContractSnapshot;
use anyhow::Result;
use chrono::Datelike;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::time::{Instant, SystemTime};

type YearMonthItems = HashMap<i32, BTreeMap<String, Vec<RecapItem>>>;

struct MonthlyRecaps {
    all: Vec<MonthRecap>,
    books: Vec<MonthRecap>,
    comics: Vec<MonthRecap>,
}

fn build_monthly_recaps(
    months_map: BTreeMap<String, Vec<RecapItem>>,
    month_hours_all: &HashMap<String, i64>,
    month_hours_books: &HashMap<String, i64>,
    month_hours_comics: &HashMap<String, i64>,
) -> MonthlyRecaps {
    // Keep the same behavior as before:
    // - month iteration order is determined by the BTreeMap key order (YYYY-MM ascending)
    // - items are sorted by end_date descending within each month
    // - per-type recaps are only emitted when there are items for that type
    let mut monthly: BTreeMap<String, MonthRecap> = BTreeMap::new();
    let mut monthly_books: BTreeMap<String, MonthRecap> = BTreeMap::new();
    let mut monthly_comics: BTreeMap<String, MonthRecap> = BTreeMap::new();

    for (ym, mut items) in months_map {
        if items.is_empty() {
            continue;
        }

        // Sort items by end_date descending (Newest first)
        items.sort_by(|a, b| b.end_date.cmp(&a.end_date));

        let month_label = if let Ok(date) =
            chrono::NaiveDate::parse_from_str(&format!("{}-01", ym), "%Y-%m-%d")
        {
            date.format("%B").to_string()
        } else {
            ym.clone()
        };

        let hours = *month_hours_all.get(&ym).unwrap_or(&0);
        let month_recap = MonthRecap {
            month_key: ym.clone(),
            month_label: month_label.clone(),
            books_finished: items.len(),
            hours_read_seconds: hours,
            items: items.clone(),
        };
        monthly.insert(ym.clone(), month_recap);

        // Books-only month recap
        let mut items_books: Vec<RecapItem> = items
            .iter()
            .filter(|it| it.content_type == Some(ContentType::Book))
            .cloned()
            .collect();
        if !items_books.is_empty() {
            items_books.sort_by(|a, b| b.end_date.cmp(&a.end_date));
            let hours_books = *month_hours_books.get(&ym).unwrap_or(&0);
            monthly_books.insert(
                ym.clone(),
                MonthRecap {
                    month_key: ym.clone(),
                    month_label: month_label.clone(),
                    books_finished: items_books.len(),
                    hours_read_seconds: hours_books,
                    items: items_books,
                },
            );
        }

        // Comics-only month recap
        let mut items_comics: Vec<RecapItem> = items
            .iter()
            .filter(|it| it.content_type == Some(ContentType::Comic))
            .cloned()
            .collect();
        if !items_comics.is_empty() {
            items_comics.sort_by(|a, b| b.end_date.cmp(&a.end_date));
            let hours_comics = *month_hours_comics.get(&ym).unwrap_or(&0);
            monthly_comics.insert(
                ym.clone(),
                MonthRecap {
                    month_key: ym.clone(),
                    month_label: month_label.clone(),
                    books_finished: items_comics.len(),
                    hours_read_seconds: hours_comics,
                    items: items_comics,
                },
            );
        }
    }

    MonthlyRecaps {
        all: monthly.into_values().collect(),
        books: monthly_books.into_values().collect(),
        comics: monthly_comics.into_values().collect(),
    }
}

fn build_md5_to_item(items: &[LibraryItem]) -> HashMap<String, &LibraryItem> {
    let mut md5_to_book: HashMap<String, &LibraryItem> = HashMap::new();
    for book in items {
        if let Some(md5) = book
            .koreader_metadata
            .as_ref()
            .and_then(|m| m.partial_md5_checksum.as_ref())
        {
            md5_to_book.insert(md5.clone(), book);
        }
    }
    md5_to_book
}

fn group_completions_by_year_month(
    stats_data: &StatisticsData,
    md5_to_item: &HashMap<String, &LibraryItem>,
) -> (YearMonthItems, Vec<i32>) {
    // Build year -> month (YYYY-MM) -> Vec<RecapItem>
    let mut year_month_items: YearMonthItems = HashMap::new();
    let mut year_set: HashSet<i32> = HashSet::new();

    for sb in &stats_data.books {
        if let Some(comps) = &sb.completions {
            for c in &comps.entries {
                let Some((year, ym)) = completion_year_and_month(&c.end_date) else {
                    continue;
                };

                year_set.insert(year);

                // Enrich from LibraryItem when possible
                let (
                    title,
                    authors,
                    rating,
                    review_note,
                    series_display,
                    item_id,
                    item_cover,
                    content_type,
                ) = if let Some(item) = md5_to_item.get(&sb.md5) {
                    let title = item.book_info.title.clone();
                    let authors = item.book_info.authors.clone();
                    let rating = item.rating();
                    let review_note = item.review_note().cloned();
                    let series_display = item.series_display();
                    let item_id = Some(item.id.clone());
                    let item_cover = Some(format!("/assets/covers/{}.webp", item.id));
                    (
                        title,
                        authors,
                        rating,
                        review_note,
                        series_display,
                        item_id,
                        item_cover,
                        Some(item.content_type()),
                    )
                } else {
                    // Fallback to StatBook minimal info
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

                let item = RecapItem {
                    title,
                    authors,
                    start_date: c.start_date.clone(),
                    end_date: c.end_date.clone(),
                    reading_time: c.reading_time,
                    session_count: c.session_count,
                    pages_read: c.pages_read,
                    rating,
                    review_note,
                    series_display,
                    item_id,
                    item_cover,
                    content_type,
                    star_display: {
                        let mut stars = [false; 5];
                        if let Some(r) = rating {
                            let n = std::cmp::min(r as usize, 5);
                            for star in stars.iter_mut().take(n) {
                                *star = true;
                            }
                        }
                        stars
                    },
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

fn compute_yearly_summary(
    year: i32,
    monthly: &[MonthRecap],
    month_hours: &HashMap<String, i64>,
    reading_stats: &ReadingStats,
    page_stats: &[PageStat],
    ids_filter: Option<&HashSet<i64>>,
) -> YearlySummary {
    // 1. Basic sums from monthly data
    let mut total_books = 0usize;
    let mut total_time_seconds = 0i64;
    for m in monthly {
        total_books += m.books_finished;
        total_time_seconds += m.hours_read_seconds;
    }

    // 2. Session stats from page_stats (filtered by year and (optional) id set)
    let year_page_stats: Vec<PageStat> = page_stats
        .iter()
        .filter(|ps| {
            if let Some(dt) = chrono::DateTime::from_timestamp(ps.start_time, 0) {
                if dt.year() != year {
                    return false;
                }
                if let Some(ids) = ids_filter {
                    ids.contains(&ps.id_book)
                } else {
                    true
                }
            } else {
                false
            }
        })
        .cloned()
        .collect();

    let all_sessions = crate::koreader::session::aggregate_session_durations(&year_page_stats);
    let session_count = all_sessions.len() as i64;
    let longest_session_duration = all_sessions.iter().max().copied().unwrap_or(0);
    let average_session_duration = if session_count > 0 {
        all_sessions.iter().sum::<i64>() / session_count
    } else {
        0
    };

    // 3. Active days (from daily activity)
    let year_str = format!("{}", year);
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

    // 4. Longest streak within this year (based on daily activity)
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

    // 5. Best month (highest reading time) in this year
    let best_month: Option<(String, i64)> = month_hours
        .iter()
        .filter(|(ym, _)| ym.starts_with(&year_str))
        .max_by_key(|(_, secs)| *secs)
        .map(|(ym, secs)| (ym.clone(), *secs));

    let best_month_name = if let Some((ym, secs)) = best_month {
        if secs > 0 {
            
            chrono::NaiveDate::parse_from_str(&format!("{}-01", ym), "%Y-%m-%d")
                .ok()
                .map(|d| d.format("%B").to_string())
        } else {
            None
        }
    } else {
        None
    };

    // Convert totals into days/hours for display components
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
        best_month_name,
        best_month_time_display: None,
    }
}

fn recap_share_assets_for_year(year: i32) -> RecapShareAssets {
    RecapShareAssets {
        story_url: format!("/assets/recap/{}_share_story.webp", year),
        square_url: format!("/assets/recap/{}_share_square.webp", year),
        banner_url: format!("/assets/recap/{}_share_banner.webp", year),
    }
}

impl SnapshotBuilder {
    fn cleanup_stale_recap_outputs(&self, current_years: &HashSet<i32>) -> Result<()> {
        let valid_years: HashSet<String> =
            current_years.iter().map(|year| year.to_string()).collect();
        self.cleanup_stale_recap_share_assets(&valid_years)?;

        Ok(())
    }

    fn cleanup_stale_recap_share_assets(&self, valid_years: &HashSet<String>) -> Result<()> {
        let assets_recap_dir = self.recap_dir();
        if !assets_recap_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&assets_recap_dir)?;
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

    async fn render_share_images_for_year(&self, year: i32, summary: &YearlySummary) -> Result<()> {
        // Generate share images for social media
        let share_data = crate::share::ShareImageData {
            year,
            books_read: summary.total_books as u32,
            reading_time_hours: summary.total_time_hours as u32,
            reading_time_days: summary.total_time_days as u32,
            active_days: summary.active_days as u32,
            active_days_percentage: summary.active_days_percentage as u8,
            longest_streak: summary.longest_streak as u32,
            best_month: summary.best_month_name.clone(),
        };

        // Regenerate share images only when stats DB mtime differs.
        let stats_db_time = self
            .statistics_db_path
            .as_ref()
            .and_then(|p| fs::metadata(p).ok())
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        // Generate all three formats in parallel using spawn_blocking (only if needed)
        info!("Rendering share images for {}...", year);
        let assets_recap_dir = self.recap_dir();
        fs::create_dir_all(&assets_recap_dir)?;

        let formats = [
            crate::share::ShareFormat::Story,
            crate::share::ShareFormat::Square,
            crate::share::ShareFormat::Banner,
        ];

        let start = Instant::now();

        // Channel to track progress from spawned tasks
        let (progress_tx, progress_rx) = std::sync::mpsc::channel::<()>();

        let share_tasks: Vec<_> = formats
            .into_iter()
            .filter_map(|format| {
                let output_path = assets_recap_dir.join(format!("{}_{}", year, format.filename()));

                // Check if share image needs regeneration
                let should_generate = match fs::metadata(&output_path) {
                    Ok(img_meta) => {
                        let img_time = img_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        stats_db_time > img_time // Only regenerate if stats DB is newer
                    }
                    Err(_) => true, // Image missing, need to generate
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
                        // Signal progress
                        let _ = tx.send(());
                    }))
                } else {
                    None
                }
            })
            .collect();

        let total_tasks = share_tasks.len();

        // Only show progress bar if there's actual work to do
        if total_tasks > 0 {
            // Set up progress bar
            let pb = ProgressBar::new(total_tasks as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg} {bar:30.cyan/blue} {pos}/{len}")
                    .unwrap()
                    .progress_chars("━╸─"),
            );
            pb.set_message(format!("Rendering share images for {}:", year));

            // Drop our sender so the channel closes when all tasks complete
            drop(progress_tx);

            // Spawn a task to update progress bar as tasks complete
            let pb_clone = pb.clone();
            let progress_task = tokio::task::spawn_blocking(move || {
                while progress_rx.recv().is_ok() {
                    pb_clone.inc(1);
                }
            });

            // Wait for all share image generation tasks to complete
            for task in share_tasks {
                let _ = task.await;
            }

            // Wait for progress tracking to finish
            let _ = progress_task.await;

            // Clear the progress bar
            pb.finish_and_clear();

            let elapsed = start.elapsed();
            info!(
                "Rendered share images for {} in {:.1}s",
                year,
                elapsed.as_secs_f64()
            );
        } else {
            drop(progress_tx);
        }

        Ok(())
    }

    pub(crate) async fn compute_recap_data_and_share_images(
        &self,
        stats_data: &mut StatisticsData,
        books: &[LibraryItem],
        snapshot: &mut ContractSnapshot,
    ) -> Result<()> {
        info!("Computing recap data and share assets...");

        // Build md5 -> &LibraryItem map for cover/link enrichment
        let md5_to_book = build_md5_to_item(books);

        // Compute reading stats for All / Books / Comics.
        // These are used for:
        // - monthly hour totals (from daily_activity)
        // - active day counts / streaks (also from daily_activity)
        // - per-year session stats (from page_stats)
        let reading_stats_all =
            crate::koreader::StatisticsCalculator::calculate_stats(stats_data, &self.time_config);
        let mut books_stats_data = stats_data.filtered_by_content_type(ContentType::Book);
        let mut comics_stats_data = stats_data.filtered_by_content_type(ContentType::Comic);
        let reading_stats_books = crate::koreader::StatisticsCalculator::calculate_stats(
            &mut books_stats_data,
            &self.time_config,
        );
        let reading_stats_comics = crate::koreader::StatisticsCalculator::calculate_stats(
            &mut comics_stats_data,
            &self.time_config,
        );

        let book_ids: HashSet<i64> = books_stats_data.books.iter().map(|b| b.id).collect();
        let comic_ids: HashSet<i64> = comics_stats_data.books.iter().map(|b| b.id).collect();

        let (year_month_items, years) = group_completions_by_year_month(stats_data, &md5_to_book);
        let years_books = mappers::years_for_content_type(&year_month_items, ContentType::Book);
        let years_comics = mappers::years_for_content_type(&year_month_items, ContentType::Comic);
        let years_books_set: HashSet<i32> = years_books.iter().copied().collect();
        let years_comics_set: HashSet<i32> = years_comics.iter().copied().collect();

        let recap_meta = mappers::build_meta(self.get_version(), self.get_last_updated());
        snapshot.completion_years.clear();
        snapshot.completion_years.insert(
            ContentTypeFilter::All.as_str().to_string(),
            mappers::map_completion_years_response(
                recap_meta.clone(),
                ContentTypeFilter::All,
                years.clone(),
            ),
        );
        snapshot.completion_years.insert(
            ContentTypeFilter::Books.as_str().to_string(),
            mappers::map_completion_years_response(
                recap_meta.clone(),
                ContentTypeFilter::Books,
                years_books,
            ),
        );
        snapshot.completion_years.insert(
            ContentTypeFilter::Comics.as_str().to_string(),
            mappers::map_completion_years_response(
                recap_meta.clone(),
                ContentTypeFilter::Comics,
                years_comics,
            ),
        );

        if years.is_empty() {
            snapshot.completion_years_by_key.clear();
            let current_years: HashSet<i32> = HashSet::new();
            self.cleanup_stale_recap_outputs(&current_years)?;

            return Ok(());
        }

        // Pre-compute monthly hours from daily activity: map YYYY-MM -> seconds
        let month_hours_all = month_hours_for(&reading_stats_all.daily_activity);
        let month_hours_books = month_hours_for(&reading_stats_books.daily_activity);
        let month_hours_comics = month_hours_for(&reading_stats_comics.daily_activity);

        // Build each year contract payload for all/books/comics filters.
        let mut completion_years_all = HashMap::new();
        let mut completion_years_books = HashMap::new();
        let mut completion_years_comics = HashMap::new();

        for year in years.iter() {
            let months_map = year_month_items.get(year).cloned().unwrap_or_default();
            let monthly = build_monthly_recaps(
                months_map,
                &month_hours_all,
                &month_hours_books,
                &month_hours_comics,
            );

            // ------------------------------------------------------------------
            // Calculate Yearly Summary Stats
            // ------------------------------------------------------------------
            let summary = compute_yearly_summary(
                *year,
                &monthly.all,
                &month_hours_all,
                &reading_stats_all,
                &stats_data.page_stats,
                None,
            );
            let summary_books = compute_yearly_summary(
                *year,
                &monthly.books,
                &month_hours_books,
                &reading_stats_books,
                &stats_data.page_stats,
                Some(&book_ids),
            );
            let summary_comics = compute_yearly_summary(
                *year,
                &monthly.comics,
                &month_hours_comics,
                &reading_stats_comics,
                &stats_data.page_stats,
                Some(&comic_ids),
            );

            self.render_share_images_for_year(*year, &summary).await?;

            let share_assets = recap_share_assets_for_year(*year);
            completion_years_all.insert(
                year.to_string(),
                mappers::map_completion_year_response(
                    recap_meta.clone(),
                    ContentTypeFilter::All,
                    *year,
                    &monthly.all,
                    &summary,
                    Some(share_assets.clone()),
                ),
            );
            if years_books_set.contains(year) {
                completion_years_books.insert(
                    year.to_string(),
                    mappers::map_completion_year_response(
                        recap_meta.clone(),
                        ContentTypeFilter::Books,
                        *year,
                        &monthly.books,
                        &summary_books,
                        Some(share_assets.clone()),
                    ),
                );
            }
            if years_comics_set.contains(year) {
                completion_years_comics.insert(
                    year.to_string(),
                    mappers::map_completion_year_response(
                        recap_meta.clone(),
                        ContentTypeFilter::Comics,
                        *year,
                        &monthly.comics,
                        &summary_comics,
                        Some(share_assets),
                    ),
                );
            }
        }

        snapshot.completion_years_by_key.clear();
        snapshot.completion_years_by_key.insert(
            ContentTypeFilter::All.as_str().to_string(),
            completion_years_all,
        );
        snapshot.completion_years_by_key.insert(
            ContentTypeFilter::Books.as_str().to_string(),
            completion_years_books,
        );
        snapshot.completion_years_by_key.insert(
            ContentTypeFilter::Comics.as_str().to_string(),
            completion_years_comics,
        );

        let current_years: HashSet<i32> = years.iter().copied().collect();
        self.cleanup_stale_recap_outputs(&current_years)?;

        Ok(())
    }
}
