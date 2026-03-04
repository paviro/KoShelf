//! Calendar payload computation.

use super::SnapshotBuilder;
use crate::contracts::common::ContentTypeFilter;
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
        items: &[LibraryItem],
        snapshot: &mut ContractSnapshot,
    ) -> Result<()> {
        info!("Computing calendar data...");

        // Generate per-month calendar payloads.
        let calendar_months =
            CalendarGenerator::generate_calendar_months(stats_data, items, &self.time_config);

        // Available months (newest first).
        let mut available_months: Vec<String> = calendar_months.keys().cloned().collect();
        available_months.sort_by(|a, b| b.cmp(a));

        let meta = mappers::build_meta(self.get_version(), self.get_last_updated());

        snapshot.activity_months.clear();
        snapshot.activity_months_by_key.clear();

        for filter in [
            ContentTypeFilter::All,
            ContentTypeFilter::Books,
            ContentTypeFilter::Comics,
        ] {
            let filter_key = filter.as_str().to_string();
            let months_response = mappers::map_activity_months_response(
                meta.clone(),
                filter,
                available_months.clone(),
            );
            snapshot
                .activity_months
                .insert(filter_key.clone(), months_response);

            let mut month_payloads = HashMap::new();
            for (month_key, month_data) in &calendar_months {
                let contract_month =
                    mappers::map_activity_month_response(meta.clone(), filter, month_data);
                month_payloads.insert(month_key.clone(), contract_month);
            }
            snapshot
                .activity_months_by_key
                .insert(filter_key, month_payloads);
        }

        Ok(())
    }
}
