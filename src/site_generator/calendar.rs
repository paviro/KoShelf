//! Calendar page generation.

use super::SiteGenerator;
use crate::calendar::CalendarGenerator;
use crate::models::{Book, StatisticsData};
use crate::templates::CalendarTemplate;
use anyhow::Result;
use askama::Template;
use log::info;
use std::fs;

impl SiteGenerator {
    pub(crate) async fn generate_calendar_page(&self, stats_data: &mut StatisticsData, books: &[Book], recap_latest_href: Option<String>) -> Result<()> {
        info!("Generating calendar page...");
        
        // Generate per-month calendar payloads (events + books + stats)
        let calendar_months = CalendarGenerator::generate_calendar_months(stats_data, books, &self.time_config);

        // ------------------------------------------------------------------
        // Write JSON files --------------------------------------------------
        // ------------------------------------------------------------------
        fs::create_dir_all(&self.calendar_json_dir())?;

        // Available months (newest first)
        let mut available_months: Vec<String> = calendar_months.keys().cloned().collect();
        available_months.sort_by(|a, b| b.cmp(a));
        fs::write(
            self.calendar_json_dir().join("available_months.json"),
            serde_json::to_string_pretty(&available_months)?,
        )?;

        // Individual month files
        for (ym, month_data) in &calendar_months {
            let filename = format!("{}.json", ym);
            fs::write(
                self.calendar_json_dir().join(filename),
                serde_json::to_string_pretty(&month_data)?,
            )?;
        }

        // Create the template
        let template = CalendarTemplate {
            site_title: self.site_title.clone(),
            version: self.get_version(),
            last_updated: self.get_last_updated(),
            navbar_items: self.create_navbar_items_with_recap("calendar", recap_latest_href.as_deref()),
        };
        
        // Render and write the template
        let html = template.render()?;

        // Write to the calendar directory (already created in create_directories)
        self.write_minify_html(self.calendar_dir().join("index.html"), &html)?;
        
        Ok(())
    }
}
