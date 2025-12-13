//! Calendar page generation.

use super::SiteGenerator;
use crate::calendar::CalendarGenerator;
use crate::models::{LibraryItem, StatisticsData};
use crate::templates::CalendarTemplate;
use anyhow::Result;
use askama::Template;
use log::info;
use std::fs;

use super::utils::NavContext;

impl SiteGenerator {
    pub(crate) async fn generate_calendar_page(
        &self,
        stats_data: &mut StatisticsData,
        books: &[LibraryItem],
        recap_latest_href: Option<String>,
        nav: NavContext,
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
        let months_json = serde_json::to_string_pretty(&available_months)?;
        let months_path = self.calendar_json_dir().join("available_months.json");
        self.cache_manifest
            .register_file(&months_path, &self.output_dir, months_json.as_bytes());
        fs::write(months_path, months_json)?;

        // Individual month files
        for (ym, month_data) in &calendar_months {
            let filename = format!("{}.json", ym);
            let file_path = self.calendar_json_dir().join(filename);
            let month_json = serde_json::to_string_pretty(&month_data)?;
            self.cache_manifest
                .register_file(&file_path, &self.output_dir, month_json.as_bytes());
            fs::write(file_path, month_json)?;
        }

        // Create the template
        let template = CalendarTemplate {
            site_title: self.site_title.clone(),
            show_type_filter: nav.has_books && nav.has_comics,
            version: self.get_version(),
            last_updated: self.get_last_updated(),
            navbar_items: self.create_navbar_items_with_recap(
                "calendar",
                recap_latest_href.as_deref(),
                nav,
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
