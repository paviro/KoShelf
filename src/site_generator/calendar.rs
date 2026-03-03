//! Calendar page generation.

use super::SiteGenerator;
use crate::contracts::mappers;
use crate::koreader::CalendarGenerator;
use crate::models::{LibraryItem, StatisticsData};
use crate::runtime::ContractSnapshot;
use crate::templates::CalendarTemplate;
use anyhow::Result;
use askama::Template;
use log::info;
use std::collections::HashMap;

use super::utils::UiContext;

impl SiteGenerator {
    pub(crate) async fn generate_calendar_page(
        &self,
        stats_data: &mut StatisticsData,
        books: &[LibraryItem],
        ui: &UiContext,
        snapshot: &mut ContractSnapshot,
    ) -> Result<()> {
        info!("Generating calendar page...");

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

        // Create the template
        let template = CalendarTemplate {
            site_title: self.site_title.clone(),
            show_type_filter: ui.nav.show_type_filter(),
            version: self.get_version(),
            last_updated: self.get_last_updated(),
            navbar_items: self.create_navbar_items_with_recap(
                "calendar",
                ui.recap_latest_href.as_deref(),
                ui.nav,
            ),
            translation: self.t(),
        };

        // Render and write the template
        let html = template.render()?;

        // Write to the calendar directory (already created in create_directories)
        self.write_minify_html(self.calendar_dir().join("index.html"), &html)?;

        Ok(())
    }
}
