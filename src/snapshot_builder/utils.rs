//! Utility functions for snapshot building.

use super::SnapshotBuilder;

/// Parse completion end date into `(year, year_month)` where `year_month` is `YYYY-MM`.
pub(crate) fn completion_year_and_month(end_date: &str) -> Option<(i32, String)> {
    let year_str = end_date.get(0..4)?;
    let year_month = end_date.get(0..7)?.to_string();
    let year = year_str.parse::<i32>().ok()?;
    Some((year, year_month))
}

impl SnapshotBuilder {
    /// Get current version from Cargo.toml
    pub(crate) fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Get current snapshot timestamp as an RFC3339 instant.
    pub(crate) fn get_last_updated(&self) -> String {
        self.time_config.now_rfc3339()
    }
}
