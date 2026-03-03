//! Calendar page generation.

use super::SiteGenerator;
use crate::contracts::mappers;
use crate::koreader::CalendarGenerator;
use crate::models::{LibraryItem, StatisticsData};
use crate::templates::CalendarTemplate;
use anyhow::Result;
use askama::Template;
use log::{info, warn};
use std::collections::HashSet;
use std::fs;

use super::utils::UiContext;

impl SiteGenerator {
    fn cleanup_stale_calendar_month_data(&self, valid_month_keys: &HashSet<String>) -> Result<()> {
        let months_dir = self.data_calendar_dir().join("months");
        if !months_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&months_dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            if let Some(stem) = path.file_stem().and_then(|name| name.to_str())
                && !valid_month_keys.contains(stem)
            {
                info!("Removing stale calendar month data file: {:?}", path);
                if let Err(error) = fs::remove_file(&path) {
                    warn!(
                        "Failed to remove stale calendar month data file {:?}: {}",
                        path, error
                    );
                }
            }
        }

        Ok(())
    }

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

        // Available months (newest first)
        let mut available_months: Vec<String> = calendar_months.keys().cloned().collect();
        available_months.sort_by(|a, b| b.cmp(a));

        let valid_month_keys: HashSet<String> = available_months.iter().cloned().collect();
        self.cleanup_stale_calendar_month_data(&valid_month_keys)?;

        let meta = mappers::build_meta(self.get_version(), self.get_last_updated());
        let data_months = mappers::map_calendar_months_response(meta.clone(), available_months);
        self.write_registered_json_pretty(
            self.data_calendar_dir().join("months.json"),
            &data_months,
        )?;

        // Individual month files
        for (ym, month_data) in &calendar_months {
            let contract_month = mappers::map_calendar_month_response(meta.clone(), month_data);
            let data_month_path = self
                .data_calendar_dir()
                .join("months")
                .join(format!("{}.json", ym));
            self.write_registered_json_pretty(data_month_path, &contract_month)?;
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
