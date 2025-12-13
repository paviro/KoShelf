//! Yearly recap page generation with share images.

use super::SiteGenerator;
use super::utils::{format_day_month, format_duration};
use crate::models::{
    ContentType, LibraryItem, MonthRecap, RecapItem, StatisticsData, YearlySummary,
};
use crate::templates::{RecapEmptyTemplate, RecapTemplate};
use anyhow::Result;
use askama::Template;
use chrono::Datelike;
use log::info;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::time::SystemTime;

use super::utils::NavContext;

impl SiteGenerator {
    pub(crate) async fn generate_recap_pages(
        &self,
        stats_data: &mut StatisticsData,
        books: &[LibraryItem],
        nav: NavContext,
    ) -> Result<()> {
        info!("Generating recap pages...");
        let show_type_filter = nav.has_books && nav.has_comics;

        // Build md5 -> &LibraryItem map for cover/link enrichment
        let mut md5_to_book: HashMap<String, &LibraryItem> = HashMap::new();
        for book in books {
            if let Some(md5) = book
                .koreader_metadata
                .as_ref()
                .and_then(|m| m.partial_md5_checksum.as_ref())
            {
                md5_to_book.insert(md5.clone(), book);
            }
        }

        // Compute reading stats for All / Books / Comics.
        // These are used for:
        // - monthly hour totals (from daily_activity)
        // - active day counts / streaks (also from daily_activity)
        // - per-year session stats (from page_stats)
        let reading_stats_all = crate::statistics_parser::StatisticsParser::calculate_stats(
            stats_data,
            &self.time_config,
        );
        let mut books_stats_data = stats_data.filtered_by_content_type(ContentType::Book);
        let mut comics_stats_data = stats_data.filtered_by_content_type(ContentType::Comic);
        let reading_stats_books = crate::statistics_parser::StatisticsParser::calculate_stats(
            &mut books_stats_data,
            &self.time_config,
        );
        let reading_stats_comics = crate::statistics_parser::StatisticsParser::calculate_stats(
            &mut comics_stats_data,
            &self.time_config,
        );

        let book_ids: HashSet<i64> = books_stats_data.books.iter().map(|b| b.id).collect();
        let comic_ids: HashSet<i64> = comics_stats_data.books.iter().map(|b| b.id).collect();

        // Build year -> month (YYYY-MM) -> Vec<RecapItem>
        let mut year_month_items: HashMap<i32, BTreeMap<String, Vec<RecapItem>>> = HashMap::new();
        let mut years: Vec<i32> = Vec::new();

        for sb in &stats_data.books {
            if let Some(comps) = &sb.completions {
                for c in &comps.entries {
                    if c.end_date.len() < 7 {
                        continue;
                    }
                    let year_str = &c.end_date[0..4];
                    let ym = c.end_date[0..7].to_string(); // YYYY-MM
                    let year: i32 = match year_str.parse() {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    if !years.contains(&year) {
                        years.push(year);
                    }

                    // Enrich from Book when possible
                    let (
                        title,
                        authors,
                        rating,
                        review_note,
                        series_display,
                        book_path,
                        book_cover,
                        content_type,
                    ) = if let Some(book) = md5_to_book.get(&sb.md5) {
                        let title = book.book_info.title.clone();
                        let authors = book.book_info.authors.clone();
                        let rating = book.rating();
                        let review_note = book.review_note().cloned();
                        let series_display = book.series_display();
                        let book_path = Some(match book.content_type() {
                            ContentType::Book => format!("/books/{}/index.html", book.id),
                            ContentType::Comic => format!("/comics/{}/index.html", book.id),
                        });
                        let book_cover = Some(format!("/assets/covers/{}.webp", book.id));
                        (
                            title,
                            authors,
                            rating,
                            review_note,
                            series_display,
                            book_path,
                            book_cover,
                            Some(book.content_type()),
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

        if years.is_empty() {
            // No completions → render empty state page
            let template = RecapEmptyTemplate {
                site_title: self.site_title.clone(),
                recap_scope: "all".to_string(),
                show_type_filter,
                year: None,
                available_years: Vec::new(),
                version: self.get_version(),
                last_updated: self.get_last_updated(),
                navbar_items: self.create_navbar_items_with_recap("recap", None, nav),
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
        let month_hours_for = |daily: &[crate::models::DailyStats]| -> HashMap<String, i64> {
            let mut out: HashMap<String, i64> = HashMap::new();
            for day in daily {
                if day.date.len() >= 7 {
                    let ym = day.date[0..7].to_string();
                    *out.entry(ym).or_insert(0) += day.read_time;
                }
            }
            out
        };
        let month_hours_all = month_hours_for(&reading_stats_all.daily_activity);
        let month_hours_books = month_hours_for(&reading_stats_books.daily_activity);
        let month_hours_comics = month_hours_for(&reading_stats_comics.daily_activity);

        // (No recap JSON exports needed anymore; we render per-scope pages instead.)

        // Render each year page (All + Books + Comics)
        for year in years.iter() {
            let months_map = year_month_items.get(year).cloned().unwrap_or_default();
            // Build MonthRecap BTreeMap sorted by month descending (Dec..Jan)
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
                    hours_read_display: format_duration(hours, &self.translations),
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
                            hours_read_display: format_duration(hours_books, &self.translations),
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
                            hours_read_display: format_duration(hours_comics, &self.translations),
                            items: items_comics,
                        },
                    );
                }
            }

            // Convert monthly map to a vector in chronological order
            let monthly_vec: Vec<MonthRecap> = monthly.into_values().collect();
            let monthly_vec_books: Vec<MonthRecap> = monthly_books.into_values().collect();
            let monthly_vec_comics: Vec<MonthRecap> = monthly_comics.into_values().collect();

            // Latest year href for sidebar
            let latest_href = format!("/recap/{}/", years[0]);

            // ------------------------------------------------------------------
            // Calculate Yearly Summary Stats
            // ------------------------------------------------------------------
            let compute_summary = |monthly: &[MonthRecap],
                                   month_hours: &HashMap<String, i64>,
                                   reading_stats: &crate::models::ReadingStats,
                                   ids_filter: Option<&HashSet<i64>>|
             -> YearlySummary {
                // 1. Basic sums from monthly data
                let mut total_books = 0usize;
                let mut total_time_seconds = 0i64;
                for m in monthly {
                    total_books += m.books_finished;
                    total_time_seconds += m.hours_read_seconds;
                }

                // 2. Session stats from page_stats (filtered by year and (optional) id set)
                let year_page_stats: Vec<crate::models::PageStat> = stats_data
                    .page_stats
                    .iter()
                    .filter(|ps| {
                        if let Some(dt) = chrono::DateTime::from_timestamp(ps.start_time, 0) {
                            if dt.year() != *year {
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

                let all_sessions =
                    crate::session_calculator::aggregate_session_durations(&year_page_stats);
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

                let days_in_year = if chrono::NaiveDate::from_ymd_opt(*year, 12, 31)
                    .is_some_and(|d| d.ordinal() == 366)
                {
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

                let (best_month_name, best_month_time_display) = if let Some((ym, secs)) =
                    best_month
                {
                    if secs > 0 {
                        let month_name =
                            chrono::NaiveDate::parse_from_str(&format!("{}-01", ym), "%Y-%m-%d")
                                .ok()
                                .map(|d| d.format("%B").to_string());
                        (month_name, Some(format_duration(secs, &self.translations)))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
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
                    best_month_time_display,
                }
            };

            let summary = compute_summary(&monthly_vec, &month_hours_all, &reading_stats_all, None);
            let summary_books = compute_summary(
                &monthly_vec_books,
                &month_hours_books,
                &reading_stats_books,
                Some(&book_ids),
            );
            let summary_comics = compute_summary(
                &monthly_vec_comics,
                &month_hours_comics,
                &reading_stats_comics,
                Some(&comic_ids),
            );

            let template_all = RecapTemplate {
                site_title: self.site_title.clone(),
                recap_scope: "all".to_string(),
                show_type_filter,
                year: *year,
                available_years: years.clone(),
                monthly: monthly_vec.clone(),
                summary: summary.clone(),
                version: self.get_version(),
                last_updated: self.get_last_updated(),
                navbar_items: self.create_navbar_items_with_recap(
                    "recap",
                    Some(latest_href.as_str()),
                    nav,
                ),
                translation: self.t(),
            };

            info!("Generating recap page ({}, all)...", year);
            let html = template_all.render()?;
            let year_dir = self.recap_dir().join(format!("{}", year));
            fs::create_dir_all(&year_dir)?;
            let path = year_dir.join("index.html");
            self.write_minify_html(path, &html)?;

            // Only create per-type recap pages when we actually have both types in the site.
            if show_type_filter {
                // /recap/<year>/books/
                info!("Generating recap page ({}, books)...", year);
                let books_dir = year_dir.join("books");
                fs::create_dir_all(&books_dir)?;
                if monthly_vec_books.is_empty() {
                    let t = RecapEmptyTemplate {
                        site_title: self.site_title.clone(),
                        recap_scope: "books".to_string(),
                        show_type_filter,
                        year: Some(*year),
                        available_years: years.clone(),
                        version: self.get_version(),
                        last_updated: self.get_last_updated(),
                        navbar_items: self.create_navbar_items_with_recap(
                            "recap",
                            Some(latest_href.as_str()),
                            nav,
                        ),
                        translation: self.t(),
                    };
                    let html = t.render()?;
                    self.write_minify_html(books_dir.join("index.html"), &html)?;
                } else {
                    let t = RecapTemplate {
                        site_title: self.site_title.clone(),
                        recap_scope: "books".to_string(),
                        show_type_filter,
                        year: *year,
                        available_years: years.clone(),
                        monthly: monthly_vec_books.clone(),
                        summary: summary_books.clone(),
                        version: self.get_version(),
                        last_updated: self.get_last_updated(),
                        navbar_items: self.create_navbar_items_with_recap(
                            "recap",
                            Some(latest_href.as_str()),
                            nav,
                        ),
                        translation: self.t(),
                    };
                    let html = t.render()?;
                    self.write_minify_html(books_dir.join("index.html"), &html)?;
                }

                // /recap/<year>/comics/
                info!("Generating recap page ({}, comics)...", year);
                let comics_dir = year_dir.join("comics");
                fs::create_dir_all(&comics_dir)?;
                if monthly_vec_comics.is_empty() {
                    let t = RecapEmptyTemplate {
                        site_title: self.site_title.clone(),
                        recap_scope: "comics".to_string(),
                        show_type_filter,
                        year: Some(*year),
                        available_years: years.clone(),
                        version: self.get_version(),
                        last_updated: self.get_last_updated(),
                        navbar_items: self.create_navbar_items_with_recap(
                            "recap",
                            Some(latest_href.as_str()),
                            nav,
                        ),
                        translation: self.t(),
                    };
                    let html = t.render()?;
                    self.write_minify_html(comics_dir.join("index.html"), &html)?;
                } else {
                    let t = RecapTemplate {
                        site_title: self.site_title.clone(),
                        recap_scope: "comics".to_string(),
                        show_type_filter,
                        year: *year,
                        available_years: years.clone(),
                        monthly: monthly_vec_comics.clone(),
                        summary: summary_comics.clone(),
                        version: self.get_version(),
                        last_updated: self.get_last_updated(),
                        navbar_items: self.create_navbar_items_with_recap(
                            "recap",
                            Some(latest_href.as_str()),
                            nav,
                        ),
                        translation: self.t(),
                    };
                    let html = t.render()?;
                    self.write_minify_html(comics_dir.join("index.html"), &html)?;
                }
            }

            // Generate share images for social media
            let share_data = crate::share_image::ShareImageData {
                year: *year,
                books_read: summary.total_books as u32,
                reading_time_hours: summary.total_time_hours as u32,
                reading_time_days: summary.total_time_days as u32,
                active_days: summary.active_days as u32,
                active_days_percentage: summary.active_days_percentage as u8,
                longest_streak: summary.longest_streak as u32,
                best_month: summary.best_month_name.clone(),
            };

            // Check if we need to regenerate share images (skip if stats DB hasn't changed)
            let stats_db_time = self
                .statistics_db_path
                .as_ref()
                .and_then(|p| fs::metadata(p).ok())
                .and_then(|m| m.modified().ok())
                .unwrap_or(SystemTime::UNIX_EPOCH);

            // Generate all three formats in parallel using spawn_blocking (only if needed)
            info!("Generating share images for {}...", year);
            let assets_recap_dir = self.output_dir.join("assets").join("recap");
            fs::create_dir_all(&assets_recap_dir)?;

            let share_image_paths: Vec<std::path::PathBuf> = [
                crate::share_image::ShareFormat::Story,
                crate::share_image::ShareFormat::Square,
                crate::share_image::ShareFormat::Banner,
            ]
            .into_iter()
            .map(|format| assets_recap_dir.join(format!("{}_{}", year, format.filename())))
            .collect();

            let share_tasks: Vec<_> = [
                crate::share_image::ShareFormat::Story,
                crate::share_image::ShareFormat::Square,
                crate::share_image::ShareFormat::Banner,
            ]
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
                    Some(tokio::task::spawn_blocking(move || {
                        if let Err(e) = crate::share_image::generate_share_image(
                            &share_data,
                            format,
                            &output_path,
                        ) {
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
                    self.cache_manifest
                        .register_file(path, &self.output_dir, &content);
                }
            }
        }

        Ok(())
    }
}
