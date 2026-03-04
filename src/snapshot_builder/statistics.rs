//! Statistics payload computation.

use super::SnapshotBuilder;
use super::utils::completion_counts_by_year;
use crate::contracts::mappers;
use crate::koreader::StatisticsCalculator;
use crate::models::{ContentType, ReadingStats, StatisticsData};
use crate::runtime::ContractSnapshot;
use anyhow::Result;
use log::info;
use std::collections::HashMap;

impl SnapshotBuilder {
    pub(crate) fn compute_statistics_data(
        &self,
        stats_data: &mut StatisticsData,
        snapshot: &mut ContractSnapshot,
    ) -> Result<()> {
        info!("Computing statistics data...");

        // Calculate reading stats for ALL content
        let reading_stats_all = StatisticsCalculator::calculate_stats(stats_data, &self.time_config);
        let completion_counts_all = completion_counts_by_year(stats_data);

        // Build per-type stats for contract exports. We always compute these so `/data` can
        // expose stable scoped payloads even when one content type is empty.
        let books_data = stats_data.filtered_by_content_type(ContentType::Book);
        let comics_data = stats_data.filtered_by_content_type(ContentType::Comic);

        let mut books_stats_data_for_contract = books_data.clone();
        let reading_stats_books_for_contract = StatisticsCalculator::calculate_stats(
            &mut books_stats_data_for_contract,
            &self.time_config,
        );
        let completion_counts_books = completion_counts_by_year(&books_stats_data_for_contract);

        let mut comics_stats_data_for_contract = comics_data.clone();
        let reading_stats_comics_for_contract = StatisticsCalculator::calculate_stats(
            &mut comics_stats_data_for_contract,
            &self.time_config,
        );
        let completion_counts_comics = completion_counts_by_year(&comics_stats_data_for_contract);

        let available_years_all =
            mappers::available_years_from_stats(&reading_stats_all, &completion_counts_all);
        self.store_statistics_snapshot_data(
            snapshot,
            &available_years_all,
            &reading_stats_all,
            &completion_counts_all,
            &reading_stats_books_for_contract,
            &completion_counts_books,
            &reading_stats_comics_for_contract,
            &completion_counts_comics,
        );

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
    fn store_statistics_snapshot_data(
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
