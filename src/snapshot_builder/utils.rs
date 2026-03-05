//! Utility functions for snapshot building.

use super::SnapshotBuilder;
use crate::models::StatisticsData;
use std::collections::HashMap;

/// Parse completion end date into `(year, year_month)` where `year_month` is `YYYY-MM`.
pub(crate) fn completion_year_and_month(end_date: &str) -> Option<(i32, String)> {
    let year_str = end_date.get(0..4)?;
    let year_month = end_date.get(0..7)?.to_string();
    let year = year_str.parse::<i32>().ok()?;
    Some((year, year_month))
}

/// Count completion entries grouped by completion year.
pub(crate) fn completion_counts_by_year(stats_data: &StatisticsData) -> HashMap<i32, i64> {
    let mut completion_counts: HashMap<i32, i64> = HashMap::new();

    for book in &stats_data.books {
        let Some(completions) = &book.completions else {
            continue;
        };

        for completion in &completions.entries {
            if let Some((year, _)) = completion_year_and_month(&completion.end_date) {
                *completion_counts.entry(year).or_insert(0) += 1;
            }
        }
    }

    completion_counts
}

impl SnapshotBuilder {
    /// Get current version from Cargo.toml
    pub(crate) fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Get current datetime as formatted string
    pub(crate) fn get_last_updated(&self) -> String {
        self.time_config.now_formatted()
    }
}
