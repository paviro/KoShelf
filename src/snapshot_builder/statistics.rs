//! Statistics payload computation.

use super::SnapshotBuilder;
use super::scaling::PageScaling;
use super::utils::completion_counts_by_year;
use crate::contracts::common::ContentTypeFilter;
use crate::contracts::mappers;
use crate::koreader::StatisticsCalculator;
use crate::models::{ContentType, StatisticsData};
use crate::runtime::ContractSnapshot;
use anyhow::Result;
use log::info;
use std::collections::HashMap;

impl SnapshotBuilder {
    pub(crate) fn compute_statistics_data(
        &self,
        stats_data: &mut StatisticsData,
        page_scaling: &PageScaling,
        snapshot: &mut ContractSnapshot,
    ) -> Result<()> {
        info!("Computing statistics data...");

        // Calculate reading stats for ALL content.
        let mut reading_stats_all =
            StatisticsCalculator::calculate_stats(stats_data, &self.time_config);
        page_scaling.apply_to_reading_stats(stats_data, &mut reading_stats_all, &self.time_config);
        let completion_counts_all = completion_counts_by_year(stats_data);

        // Build per-type stats for contract exports.
        let books_data = stats_data.filtered_by_content_type(ContentType::Book);
        let comics_data = stats_data.filtered_by_content_type(ContentType::Comic);

        let mut books_stats_data = books_data.clone();
        let mut reading_stats_books =
            StatisticsCalculator::calculate_stats(&mut books_stats_data, &self.time_config);
        page_scaling.apply_to_reading_stats(
            &books_stats_data,
            &mut reading_stats_books,
            &self.time_config,
        );
        let completion_counts_books = completion_counts_by_year(&books_stats_data);

        let mut comics_stats_data = comics_data.clone();
        let mut reading_stats_comics =
            StatisticsCalculator::calculate_stats(&mut comics_stats_data, &self.time_config);
        page_scaling.apply_to_reading_stats(
            &comics_stats_data,
            &mut reading_stats_comics,
            &self.time_config,
        );
        let completion_counts_comics = completion_counts_by_year(&comics_stats_data);

        let max_scale_seconds = self.heatmap_scale_max.map(i64::from).unwrap_or(0);
        let meta = mappers::build_meta(self.get_version(), self.get_last_updated());

        let filters = [
            (
                ContentTypeFilter::All,
                &reading_stats_all,
                &completion_counts_all,
            ),
            (
                ContentTypeFilter::Books,
                &reading_stats_books,
                &completion_counts_books,
            ),
            (
                ContentTypeFilter::Comics,
                &reading_stats_comics,
                &completion_counts_comics,
            ),
        ];

        snapshot.activity_weeks.clear();
        snapshot.activity_weeks_by_key.clear();
        snapshot.activity_year_daily.clear();
        snapshot.activity_year_summary.clear();

        for (filter, reading_stats, completion_counts) in filters {
            let filter_key = filter.as_str().to_string();
            let available_years =
                mappers::available_years_from_stats(reading_stats, completion_counts);

            let weeks_response = mappers::map_activity_weeks_response(
                meta.clone(),
                filter,
                available_years.clone(),
                reading_stats,
                max_scale_seconds,
            );
            snapshot
                .activity_weeks
                .insert(filter_key.clone(), weeks_response.clone());

            let week_stats = Self::week_stats_by_key(&reading_stats.weeks);
            let mut weeks_by_key = HashMap::new();
            for week in &weeks_response.available_weeks {
                let week_response = mappers::map_activity_week_response(
                    meta.clone(),
                    filter,
                    week.week_key.clone(),
                    week_stats.get(&week.week_key),
                    &reading_stats.daily_activity,
                );
                weeks_by_key.insert(week.week_key.clone(), week_response);
            }
            snapshot
                .activity_weeks_by_key
                .insert(filter_key.clone(), weeks_by_key);

            let mut year_daily = HashMap::new();
            let mut year_summary = HashMap::new();
            for year in available_years {
                year_daily.insert(
                    year.to_string(),
                    mappers::map_activity_year_daily_response(
                        meta.clone(),
                        filter,
                        year,
                        reading_stats,
                        max_scale_seconds,
                    ),
                );
                year_summary.insert(
                    year.to_string(),
                    mappers::map_activity_year_summary_response(
                        meta.clone(),
                        filter,
                        year,
                        reading_stats,
                        completion_counts,
                        max_scale_seconds,
                    ),
                );
            }

            snapshot
                .activity_year_daily
                .insert(filter_key.clone(), year_daily);
            snapshot
                .activity_year_summary
                .insert(filter_key, year_summary);
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
}
