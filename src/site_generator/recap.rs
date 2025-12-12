//! Yearly recap page generation with share images.

use super::SiteGenerator;
use super::utils::{format_duration, format_day_month};
use chrono::Datelike;
use crate::models::{Book, StatisticsData, RecapItem, MonthRecap, YearlySummary};
use crate::templates::{RecapTemplate, RecapEmptyTemplate};
use anyhow::Result;
use askama::Template;
use log::info;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::time::SystemTime;

impl SiteGenerator {
    pub(crate) async fn generate_recap_pages(&self, stats_data: &mut StatisticsData, books: &[Book]) -> Result<()> {
        info!("Generating recap pages...");

        // Build md5 -> &Book map for cover/link enrichment
        let mut md5_to_book: HashMap<String, &Book> = HashMap::new();
        for book in books {
            if let Some(md5) = book
                .koreader_metadata
                .as_ref()
                .and_then(|m| m.partial_md5_checksum.as_ref())
            {
                md5_to_book.insert(md5.clone(), book);
            }
        }

        // Compute reading stats once to get daily activity for hour totals
        let reading_stats = crate::statistics_parser::StatisticsParser::calculate_stats(stats_data, &self.time_config);

        // Build year -> month (YYYY-MM) -> Vec<RecapItem>
        let mut year_month_items: HashMap<i32, BTreeMap<String, Vec<RecapItem>>> = HashMap::new();
        let mut years: Vec<i32> = Vec::new();

        for sb in &stats_data.books {
            if let Some(comps) = &sb.completions {
                for c in &comps.entries {
                    if c.end_date.len() < 7 { continue; }
                    let year_str = &c.end_date[0..4];
                    let ym = c.end_date[0..7].to_string(); // YYYY-MM
                    let year: i32 = match year_str.parse() { Ok(v) => v, Err(_) => continue };

                    if !years.contains(&year) {
                        years.push(year);
                    }

                    // Enrich from Book when possible
                    let (title, authors, rating, review_note, series_display, book_path, book_cover) = if let Some(book) = md5_to_book.get(&sb.md5) {
                        let title = book.book_info.title.clone();
                        let authors = book.book_info.authors.clone();
                        let rating = book.rating();
                        let review_note = book.review_note().cloned();
                        let series_display = book.series_display();
                        let book_path = Some(format!("/books/{}/index.html", book.id));
                        let book_cover = Some(format!("/assets/covers/{}.webp", book.id));
                        (title, authors, rating, review_note, series_display, book_path, book_cover)
                    } else {
                        // Fallback to StatBook minimal info
                        let title = sb.title.clone();
                        let authors = sb.authors.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>();
                        (title, authors, None, None, None, None, None)
                    };

                    let item = RecapItem {
                        title,
                        authors,
                        start_date: c.start_date.clone(),
                        end_date: c.end_date.clone(),
                        start_display: format_day_month(&c.start_date, &self.translations),
                        end_display: format_day_month(&c.end_date, &self.translations),
                        reading_time: c.reading_time,
                        reading_time_display: format_duration(c.reading_time, &self.translations),
                        session_count: c.session_count,
                        pages_read: c.pages_read,
                        rating,
                        review_note,
                        series_display,
                        book_path,
                        book_cover,
                        star_display: {
                            let mut stars = [false; 5];
                            if let Some(r) = rating {
                                let n = std::cmp::min(r as usize, 5);
                                for star in stars.iter_mut().take(n) { *star = true; }
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

        if years.is_empty() {
            // No completions → render empty state page
            let template = RecapEmptyTemplate {
                site_title: self.site_title.clone(),
                version: self.get_version(),
                last_updated: self.get_last_updated(),
                navbar_items: self.create_navbar_items_with_recap("recap", None),
                translation: self.t(),
            };
            
            let html = template.render()?;
            let recap_dir = self.recap_dir();
            fs::create_dir_all(&recap_dir)?;
            let path = recap_dir.join("index.html");
            self.write_minify_html(path, &html)?;
            
            return Ok(());
        }

        years.sort_by(|a, b| b.cmp(a)); // newest first

        // Pre-compute monthly hours from daily activity: map YYYY-MM -> seconds
        let mut month_hours: HashMap<String, i64> = HashMap::new();
        for day in &reading_stats.daily_activity {
            if day.date.len() >= 7 {
                let ym = day.date[0..7].to_string();
                *month_hours.entry(ym).or_insert(0) += day.read_time;
            }
        }

        // Render each year page
        for (idx, year) in years.iter().enumerate() {
            let months_map = year_month_items.get(year).cloned().unwrap_or_default();
            // Build MonthRecap BTreeMap sorted by month descending (Dec..Jan)
            let mut monthly: BTreeMap<String, MonthRecap> = BTreeMap::new();

            for (ym, mut items) in months_map {
                if items.is_empty() { continue; }
                // Sort items by end_date descending (Newest first)
                items.sort_by(|a, b| b.end_date.cmp(&a.end_date));

                let month_label = if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{}-01", ym), "%Y-%m-%d") {
                    date.format("%B").to_string()
                } else { ym.clone() };

                let hours = *month_hours.get(&ym).unwrap_or(&0);
                let month_recap = MonthRecap {
                    month_key: ym.clone(),
                    month_label,
                    books_finished: items.len(),
                    hours_read_seconds: hours,
                    hours_read_display: format_duration(hours, &self.translations),
                    items,
                };
                monthly.insert(ym, month_recap);
            }

            // Determine prev/next year for controls
            let prev_year = years.get(idx + 1).cloned();
            let next_year = if idx > 0 { years.get(idx - 1).cloned() } else { None };

            // Convert monthly map to a vector in chronological order
            let monthly_vec: Vec<MonthRecap> = monthly.into_values().collect();

            // Latest year href for sidebar
            let latest_href = format!("/recap/{}/", years[0]);

            // ------------------------------------------------------------------
            // Calculate Yearly Summary Stats
            // ------------------------------------------------------------------
            // 1. Basic sums from monthly data
            let mut total_books = 0;
            let mut total_time_seconds = 0;

            for m in &monthly_vec {
                total_books += m.books_finished;
                total_time_seconds += m.hours_read_seconds;
            }

            // 2. Session stats from page_stats (filtered by year)
            // Filter page_stats for the current year
            let year_page_stats: Vec<crate::models::PageStat> = stats_data.page_stats.iter()
                .filter(|ps| {
                    if let Some(dt) = chrono::DateTime::from_timestamp(ps.start_time, 0) {
                        dt.year() == *year
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            // Use session_calculator to aggregate stats into valid sessions (handling gaps)
            let all_sessions = crate::session_calculator::aggregate_session_durations(&year_page_stats);
            
            let session_count = all_sessions.len() as i64;
            let longest_session_duration = all_sessions.iter().max().copied().unwrap_or(0);
            let average_session_duration = if session_count > 0 {
                all_sessions.iter().sum::<i64>() / session_count
            } else {
                0
            };

            // 3. Calculate active reading days for this year
            let year_str = format!("{}", year);
            let active_days: usize = reading_stats.daily_activity.iter()
                .filter(|day| day.date.starts_with(&year_str) && day.read_time > 0)
                .count();
            
            // Calculate percentage based on days in the year (365 or 366 for leap years)
            let days_in_year = if chrono::NaiveDate::from_ymd_opt(*year, 12, 31).is_some_and(|d| d.ordinal() == 366) {
                366.0
            } else {
                365.0
            };
            let active_days_percentage = (active_days as f64 / days_in_year * 100.0).round();

            // 4. Calculate longest streak within this year
            let mut year_reading_dates: Vec<chrono::NaiveDate> = reading_stats.daily_activity.iter()
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

            // 5. Find best month (highest reading time) in this year
            let best_month: Option<(String, i64)> = month_hours.iter()
                .filter(|(ym, _)| ym.starts_with(&year_str))
                .max_by_key(|(_, secs)| *secs)
                .map(|(ym, secs)| (ym.clone(), *secs));
            
            let (best_month_name, best_month_time_display) = if let Some((ym, secs)) = best_month {
                if secs > 0 {
                    let month_name = if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{}-01", ym), "%Y-%m-%d") {
                        Some(date.format("%B").to_string())
                    } else {
                        None
                    };
                    (month_name, Some(format_duration(secs, &self.translations)))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            // Calculate days and hours from total time (drop minutes)
            let total_minutes = total_time_seconds / 60;
            let total_time_days = total_minutes / (24 * 60);
            let total_time_hours = (total_minutes % (24 * 60)) / 60;

            // Calculate hours and minutes for session stats
            let longest_session_total_mins = longest_session_duration / 60;
            let longest_session_hours = longest_session_total_mins / 60;
            let longest_session_minutes = longest_session_total_mins % 60;

            let avg_session_total_mins = average_session_duration / 60;
            let average_session_hours = avg_session_total_mins / 60;
            let average_session_minutes = avg_session_total_mins % 60;

            let summary = YearlySummary {
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
                best_month_name: best_month_name.clone(),
                best_month_time_display,
            };

            let template = RecapTemplate {
                site_title: self.site_title.clone(),
                year: *year,
                available_years: years.clone(),
                prev_year,
                next_year,
                monthly: monthly_vec,
                summary,
                version: self.get_version(),
                last_updated: self.get_last_updated(),
                navbar_items: self.create_navbar_items_with_recap("recap", Some(latest_href.as_str())),
                translation: self.t(),
            };

            let html = template.render()?;
            let year_dir = self.recap_dir().join(format!("{}", year));
            fs::create_dir_all(&year_dir)?;
            let path = year_dir.join("index.html");
            self.write_minify_html(path, &html)?;

            // Generate share images for social media
            let share_data = crate::share_image::ShareImageData {
                year: *year,
                books_read: total_books as u32,
                reading_time_hours: total_time_hours as u32,
                reading_time_days: total_time_days as u32,
                active_days: active_days as u32,
                active_days_percentage: active_days_percentage as u8,
                longest_streak: longest_streak as u32,
                best_month: best_month_name.clone(),
            };

            // Check if we need to regenerate share images (skip if stats DB hasn't changed)
            let stats_db_time = self.statistics_db_path.as_ref()
                .and_then(|p| fs::metadata(p).ok())
                .and_then(|m| m.modified().ok())
                .unwrap_or(SystemTime::UNIX_EPOCH);

            // Generate all three formats in parallel using spawn_blocking (only if needed)
            let share_image_paths: Vec<std::path::PathBuf> = [
                crate::share_image::ShareFormat::Story,
                crate::share_image::ShareFormat::Square,
                crate::share_image::ShareFormat::Banner,
            ]
            .into_iter()
            .map(|format| year_dir.join(format.filename()))
            .collect();

            let share_tasks: Vec<_> = [
                crate::share_image::ShareFormat::Story,
                crate::share_image::ShareFormat::Square,
                crate::share_image::ShareFormat::Banner,
            ]
            .into_iter()
            .filter_map(|format| {
                let output_path = year_dir.join(format.filename());
                
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
                    Some(tokio::task::spawn_blocking(move || {
                        if let Err(e) = crate::share_image::generate_share_image(&share_data, format, &output_path) {
                            log::warn!("Failed to generate share image {:?}: {}", output_path, e);
                        }
                    }))
                } else {
                    None
                }
            })
            .collect();

            // Wait for all share image generation tasks to complete
            for task in share_tasks {
                let _ = task.await;
            }

            // Register share images in cache manifest
            for path in &share_image_paths {
                if let Ok(content) = fs::read(path) {
                    self.cache_manifest.register_file(path, &self.output_dir, &content);
                }
            }
        }

        Ok(())
    }
}
