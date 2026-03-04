//! Calendar payload computation.

use super::SnapshotBuilder;
use crate::contracts::mappers;
use crate::koreader::CalendarGenerator;
use crate::models::{LibraryItem, StatisticsData};
use crate::runtime::ContractSnapshot;
use anyhow::Result;
use log::info;
use std::collections::HashMap;

impl SnapshotBuilder {
    pub(crate) fn compute_calendar_data(
        &self,
        stats_data: &mut StatisticsData,
        books: &[LibraryItem],
        snapshot: &mut ContractSnapshot,
    ) -> Result<()> {
        info!("Computing calendar data...");

        // Generate per-month calendar payloads (events + books + stats)
        let calendar_months =
            CalendarGenerator::generate_calendar_months(stats_data, books, &self.time_config);

        // Available months (newest first)
        let mut available_months: Vec<String> = calendar_months.keys().cloned().collect();
        available_months.sort_by(|a, b| b.cmp(a));

        let meta = mappers::build_meta(self.get_version(), self.get_last_updated());
        let data_months = mappers::map_calendar_months_response(meta.clone(), available_months);
        snapshot.calendar_months = Some(data_months);

        // Individual month files
        let mut month_payloads = HashMap::new();
        for (ym, month_data) in &calendar_months {
            let contract_month = mappers::map_calendar_month_response(meta.clone(), month_data);
            month_payloads.insert(ym.clone(), contract_month);
        }
        snapshot.calendar_by_month = month_payloads;

        Ok(())
    }
}
