//! Calendar page generation.

use super::SiteGenerator;
use crate::koreader::CalendarGenerator;
use crate::models::{LibraryItem, StatisticsData};
use crate::templates::CalendarTemplate;
use anyhow::Result;
use askama::Template;
use log::info;
use std::fs;

use super::utils::UiContext;

impl SiteGenerator {
    pub(crate) async fn generate_calendar_page(
        &self,
        stats_data: &mut StatisticsData,
        books: &[LibraryItem],
        ui: &UiContext,
    ) -> Result<()> {
        info!("Generating calendar page...");

        // Generate per-month calendar payloads (events + books + stats)
        let calendar_months =
            CalendarGenerator::generate_calendar_months(stats_data, books, &self.time_config);

        // ------------------------------------------------------------------
        // Write JSON files --------------------------------------------------
        // ------------------------------------------------------------------
        fs::create_dir_all(self.calendar_json_dir())?;

        // Available months (newest first)
        let mut available_months: Vec<String> = calendar_months.keys().cloned().collect();
        available_months.sort_by(|a, b| b.cmp(a));
        let months_path = self.calendar_json_dir().join("available_months.json");
        self.write_registered_json_pretty(months_path, &available_months)?;

        // Individual month files
        for (ym, month_data) in &calendar_months {
            let filename = format!("{}.json", ym);
            let file_path = self.calendar_json_dir().join(filename);
            self.write_registered_json_pretty(file_path, month_data)?;
        }

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
