//! Statistics page generation and JSON export.

use super::SiteGenerator;
use crate::models::{ContentType, ReadingStats, StatisticsData};
use crate::koreader::StatisticsParser;
use crate::templates::{StatsEmptyTemplate, StatsTemplate};
use anyhow::Result;
use askama::Template;
use log::info;
use std::fs;
use std::path::Path;

use super::utils::UiContext;

impl SiteGenerator {
    pub(crate) async fn generate_statistics_page(
        &self,
        stats_data: &mut StatisticsData,
        render_to_root: bool,
        ui: &UiContext,
    ) -> Result<()> {
        let show_type_filter = ui.nav.show_type_filter();
        if render_to_root {
            info!("Generating statistics page at root index...");
        } else {
            info!("Generating statistics page (all)...");
        }

        // Calculate reading stats for ALL content
        let reading_stats_all = StatisticsParser::calculate_stats(stats_data, &self.time_config);

        // Export JSON for ALL scope into a dedicated folder to keep `/assets/json/statistics/` tidy.
        let all_dir = self.statistics_json_dir().join("all");
        fs::create_dir_all(&all_dir)?;
        let available_years = self
            .export_daily_activity_by_year_to_dir(&reading_stats_all.daily_activity, &all_dir)
            .await?;
        self.export_week_stats_to_dir(&reading_stats_all.weeks, &all_dir)?;

        // Only export/render per-type views when we actually have both types in the site.
        // If only one type exists, the "all" view is the only meaningful one.
        let (reading_stats_books, available_years_books, reading_stats_comics, available_years_comics) =
            if show_type_filter {
                // Also export separate JSON outputs for books and comics
                let books_data = stats_data.filtered_by_content_type(ContentType::Book);
                let comics_data = stats_data.filtered_by_content_type(ContentType::Comic);

                let (reading_stats_books, available_years_books) = self
                    .export_stats_bundle(&books_data, ContentType::Book)
                    .await?;

                let (reading_stats_comics, available_years_comics) = self
                    .export_stats_bundle(&comics_data, ContentType::Comic)
                    .await?;

                (
                    Some(reading_stats_books),
                    Some(available_years_books),
                    Some(reading_stats_comics),
                    Some(available_years_comics),
                )
            } else {
                (None, None, None, None)
            };

        if render_to_root {
            let template = StatsTemplate {
                site_title: self.site_title.clone(),
                stats_scope: "all".to_string(),
                show_type_filter,
                stats_json_base_path: "/assets/json/statistics/all".to_string(),
                reading_stats: reading_stats_all.clone(),
                available_years,
                version: self.get_version(),
                last_updated: self.get_last_updated(),
                navbar_items: self.create_navbar_items_with_recap(
                    "statistics",
                    ui.recap_latest_href.as_deref(),
                    ui.nav,
                ),
                translation: self.t(),
            };
            let html = template.render()?;
            self.write_minify_html(self.output_dir.join("index.html"), &html)?;
        } else {
            let stats_dir = self.statistics_dir();
            fs::create_dir_all(&stats_dir)?;
            let template_all = StatsTemplate {
                site_title: self.site_title.clone(),
                stats_scope: "all".to_string(),
                show_type_filter,
                stats_json_base_path: "/assets/json/statistics/all".to_string(),
                reading_stats: reading_stats_all.clone(),
                available_years,
                version: self.get_version(),
                last_updated: self.get_last_updated(),
                navbar_items: self.create_navbar_items_with_recap(
                    "statistics",
                    ui.recap_latest_href.as_deref(),
                    ui.nav,
                ),
                translation: self.t(),
            };
            let html_all = template_all.render()?;
            self.write_minify_html(stats_dir.join("index.html"), &html_all)?;

            if show_type_filter {
                // /statistics/books/
                info!("Generating statistics page (books)...");
                let stats_books_dir = stats_dir.join("books");
                fs::create_dir_all(&stats_books_dir)?;
                let years_books = available_years_books.expect("books years must exist");
                let html_books = if years_books.is_empty() {
                    let template = StatsEmptyTemplate {
                        site_title: self.site_title.clone(),
                        stats_scope: "books".to_string(),
                        show_type_filter,
                        version: self.get_version(),
                        last_updated: self.get_last_updated(),
                        navbar_items: self.create_navbar_items_with_recap(
                            "statistics",
                            ui.recap_latest_href.as_deref(),
                            ui.nav,
                        ),
                        translation: self.t(),
                    };
                    template.render()?
                } else {
                    let template = StatsTemplate {
                        site_title: self.site_title.clone(),
                        stats_scope: "books".to_string(),
                        show_type_filter,
                        stats_json_base_path: "/assets/json/statistics/books".to_string(),
                        reading_stats: reading_stats_books.expect("books stats must exist"),
                        available_years: years_books,
                        version: self.get_version(),
                        last_updated: self.get_last_updated(),
                        navbar_items: self.create_navbar_items_with_recap(
                            "statistics",
                            ui.recap_latest_href.as_deref(),
                            ui.nav,
                        ),
                        translation: self.t(),
                    };
                    template.render()?
                };
                self.write_minify_html(stats_books_dir.join("index.html"), &html_books)?;

                // /statistics/comics/
                info!("Generating statistics page (comics)...");
                let stats_comics_dir = stats_dir.join("comics");
                fs::create_dir_all(&stats_comics_dir)?;
                let years_comics = available_years_comics.expect("comics years must exist");
                let html_comics = if years_comics.is_empty() {
                    let template = StatsEmptyTemplate {
                        site_title: self.site_title.clone(),
                        stats_scope: "comics".to_string(),
                        show_type_filter,
                        version: self.get_version(),
                        last_updated: self.get_last_updated(),
                        navbar_items: self.create_navbar_items_with_recap(
                            "statistics",
                            ui.recap_latest_href.as_deref(),
                            ui.nav,
                        ),
                        translation: self.t(),
                    };
                    template.render()?
                } else {
                    let template = StatsTemplate {
                        site_title: self.site_title.clone(),
                        stats_scope: "comics".to_string(),
                        show_type_filter,
                        stats_json_base_path: "/assets/json/statistics/comics".to_string(),
                        reading_stats: reading_stats_comics.expect("comics stats must exist"),
                        available_years: years_comics,
                        version: self.get_version(),
                        last_updated: self.get_last_updated(),
                        navbar_items: self.create_navbar_items_with_recap(
                            "statistics",
                            ui.recap_latest_href.as_deref(),
                            ui.nav,
                        ),
                        translation: self.t(),
                    };
                    template.render()?
                };
                self.write_minify_html(stats_comics_dir.join("index.html"), &html_comics)?;
            }
        }

        Ok(())
    }

    /// Export daily activity data grouped by year as separate JSON files and return available years
    pub(crate) async fn export_daily_activity_by_year_to_dir(
        &self,
        daily_activity: &[crate::models::DailyStats],
        output_dir: &Path,
    ) -> Result<Vec<i32>> {
        // Group daily stats by year
        let mut activity_by_year: std::collections::HashMap<i32, Vec<&crate::models::DailyStats>> =
            std::collections::HashMap::new();

        for day_stat in daily_activity {
            // Extract year from date (format: yyyy-mm-dd)
            if let Some(year_str) = day_stat.date.get(0..4)
                && let Ok(year) = year_str.parse::<i32>()
            {
                activity_by_year.entry(year).or_default().push(day_stat);
            }
        }

        // Collect available years before consuming the map
        let mut available_years: Vec<i32> = activity_by_year.keys().cloned().collect();
        available_years.sort_by(|a, b| b.cmp(a)); // Sort descending (newest first)

        // Export each year's data to a separate file
        for (year, year_data) in activity_by_year {
            let filename = format!("daily_activity_{}.json", year);
            let file_path = output_dir.join(filename);

            // Wrap the data with configuration information
            let json_data = serde_json::json!({
                "data": year_data,
                "config": {
                    "max_scale_seconds": self.heatmap_scale_max
                }
            });

            self.write_registered_json_pretty(file_path, &json_data)?;
        }

        Ok(available_years)
    }

    fn export_week_stats_to_dir(
        &self,
        weeks: &[crate::models::WeeklyStats],
        output_dir: &Path,
    ) -> Result<()> {
        for (index, week) in weeks.iter().enumerate() {
            let file_path = output_dir.join(format!("week_{}.json", index));
            self.write_registered_json_pretty(file_path, week)?;
        }
        Ok(())
    }

    async fn export_stats_bundle(
        &self,
        data: &StatisticsData,
        content_type: ContentType,
    ) -> Result<(ReadingStats, Vec<i32>)> {
        let mut data = data.clone();
        let reading_stats = StatisticsParser::calculate_stats(&mut data, &self.time_config);

        let subdir = match content_type {
            ContentType::Book => self.statistics_json_dir().join("books"),
            ContentType::Comic => self.statistics_json_dir().join("comics"),
        };
        fs::create_dir_all(&subdir)?;

        let years = self
            .export_daily_activity_by_year_to_dir(&reading_stats.daily_activity, &subdir)
            .await?;
        self.export_week_stats_to_dir(&reading_stats.weeks, &subdir)?;

        Ok((reading_stats, years))
    }
}
