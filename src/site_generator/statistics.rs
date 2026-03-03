//! Statistics page generation and JSON export.

use super::SiteGenerator;
use crate::contracts::mappers;
use crate::koreader::StatisticsParser;
use crate::models::{ContentType, ReadingStats, StatisticsData};
use crate::runtime::ContractSnapshot;
use crate::templates::{StatsEmptyTemplate, StatsTemplate};
use anyhow::Result;
use askama::Template;
use log::info;
use std::collections::HashMap;
use std::fs;

use super::utils::{UiContext, completion_counts_by_year};

impl SiteGenerator {
    pub(crate) async fn generate_statistics_page(
        &self,
        stats_data: &mut StatisticsData,
        render_to_root: bool,
        ui: &UiContext,
        snapshot: &mut ContractSnapshot,
    ) -> Result<()> {
        let show_type_filter = ui.nav.show_type_filter();
        if render_to_root {
            info!("Generating statistics page at root index...");
        } else {
            info!("Generating statistics page (all)...");
        }

        // Calculate reading stats for ALL content
        let reading_stats_all = StatisticsParser::calculate_stats(stats_data, &self.time_config);
        let completion_counts_all = completion_counts_by_year(stats_data);

        // Build per-type stats for contract exports. We always compute these so `/data` can
        // expose stable scoped payloads even when one content type is empty.
        let books_data = stats_data.filtered_by_content_type(ContentType::Book);
        let comics_data = stats_data.filtered_by_content_type(ContentType::Comic);

        let mut books_stats_data_for_contract = books_data.clone();
        let reading_stats_books_for_contract = StatisticsParser::calculate_stats(
            &mut books_stats_data_for_contract,
            &self.time_config,
        );
        let completion_counts_books = completion_counts_by_year(&books_stats_data_for_contract);

        let mut comics_stats_data_for_contract = comics_data.clone();
        let reading_stats_comics_for_contract = StatisticsParser::calculate_stats(
            &mut comics_stats_data_for_contract,
            &self.time_config,
        );
        let completion_counts_comics = completion_counts_by_year(&comics_stats_data_for_contract);

        let available_years_all =
            mappers::available_years_from_stats(&reading_stats_all, &completion_counts_all);
        self.populate_statistics_snapshot(
            snapshot,
            &available_years_all,
            &reading_stats_all,
            &completion_counts_all,
            &reading_stats_books_for_contract,
            &completion_counts_books,
            &reading_stats_comics_for_contract,
            &completion_counts_comics,
        );

        // Render per-type views when both types exist in the site.
        let (
            reading_stats_books,
            available_years_books,
            reading_stats_comics,
            available_years_comics,
        ) = if show_type_filter {
            let available_years_books = mappers::available_years_from_stats(
                &reading_stats_books_for_contract,
                &completion_counts_books,
            );
            let available_years_comics = mappers::available_years_from_stats(
                &reading_stats_comics_for_contract,
                &completion_counts_comics,
            );

            (
                Some(reading_stats_books_for_contract.clone()),
                Some(available_years_books),
                Some(reading_stats_comics_for_contract.clone()),
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
                reading_stats: reading_stats_all.clone(),
                available_years: available_years_all.clone(),
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
                reading_stats: reading_stats_all.clone(),
                available_years: available_years_all.clone(),
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

    fn week_stats_by_key(
        weeks: &[crate::models::WeeklyStats],
    ) -> HashMap<String, crate::models::WeeklyStats> {
        let mut by_key = HashMap::new();
        for week in weeks {
            by_key
                .entry(week.start_date.clone())
                .or_insert_with(|| week.clone());
        }
        by_key
    }

    #[allow(clippy::too_many_arguments)]
    fn populate_statistics_snapshot(
        &self,
        snapshot: &mut ContractSnapshot,
        available_years: &[i32],
        reading_stats_all: &ReadingStats,
        completion_counts_all: &HashMap<i32, i64>,
        reading_stats_books: &ReadingStats,
        completion_counts_books: &HashMap<i32, i64>,
        reading_stats_comics: &ReadingStats,
        completion_counts_comics: &HashMap<i32, i64>,
    ) {
        let max_scale_seconds = self.heatmap_scale_max.map(i64::from).unwrap_or(0);
        let meta = mappers::build_meta(self.get_version(), self.get_last_updated());

        let index_response = mappers::map_statistics_index_response(
            meta.clone(),
            available_years.to_vec(),
            reading_stats_all,
            reading_stats_books,
            reading_stats_comics,
            max_scale_seconds,
        );
        snapshot.statistics_index = Some(index_response.clone());

        let all_weeks = Self::week_stats_by_key(&reading_stats_all.weeks);
        let books_weeks = Self::week_stats_by_key(&reading_stats_books.weeks);
        let comics_weeks = Self::week_stats_by_key(&reading_stats_comics.weeks);
        let mut week_payloads = HashMap::new();

        for week in &index_response.available_weeks {
            let week_response = mappers::map_statistics_week_response(
                meta.clone(),
                week.week_key.clone(),
                all_weeks.get(&week.week_key),
                books_weeks.get(&week.week_key),
                comics_weeks.get(&week.week_key),
            );
            week_payloads.insert(week.week_key.clone(), week_response);
        }
        snapshot.statistics_weeks = week_payloads;

        let mut year_payloads = HashMap::new();

        for year in available_years {
            let year_response = mappers::map_statistics_year_response(
                meta.clone(),
                *year,
                reading_stats_all,
                completion_counts_all,
                reading_stats_books,
                completion_counts_books,
                reading_stats_comics,
                completion_counts_comics,
                max_scale_seconds,
            );
            year_payloads.insert(year.to_string(), year_response);
        }
        snapshot.statistics_years = year_payloads;
    }
}
