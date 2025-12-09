//! Statistics page generation and JSON export.

use super::SiteGenerator;
use crate::models::StatisticsData;
use crate::statistics_parser::StatisticsParser;
use crate::templates::StatsTemplate;
use anyhow::Result;
use askama::Template;
use log::info;
use std::collections::HashMap;
use std::fs;

impl SiteGenerator {
    pub(crate) async fn generate_statistics_page(&self, stats_data: &mut StatisticsData, render_to_root: bool, recap_latest_href: Option<String>) -> Result<()> {
        if render_to_root {
            info!("Generating statistics page at root index...");
        } else {
            info!("Generating statistics page...");
        }
        
        // Calculate reading stats from the parsed data and populate completions
        let reading_stats = StatisticsParser::calculate_stats(stats_data, &self.time_config);
        
        // Export daily activity data grouped by year as separate JSON files and get available years
        let available_years = self.export_daily_activity_by_year(&reading_stats.daily_activity).await?;

        // Export individual week data as separate JSON files
        for (index, week) in reading_stats.weeks.iter().enumerate() {
            let week_json = serde_json::to_string_pretty(&week)?;
            let file_path = self.statistics_json_dir().join(format!("week_{}.json", index));
            self.cache_manifest.register_file(&file_path, &self.output_dir, week_json.as_bytes());
            fs::write(file_path, week_json)?;
        }
        
        // Create the template with appropriate navbar
        let template = StatsTemplate {
            site_title: self.site_title.clone(),
            reading_stats: reading_stats.clone(),
            available_years,
            version: self.get_version(),
            last_updated: self.get_last_updated(),
            navbar_items: self.create_navbar_items_with_recap("statistics", recap_latest_href.as_deref()),
        };
        
        // Render and write the template
        let html = template.render()?;

        if render_to_root {
            // Write directly to index.html
            self.write_minify_html(self.output_dir.join("index.html"), &html)?;
        } else {
            // Create stats directory and write the index file
            let stats_dir = self.output_dir.join("statistics");
            fs::create_dir_all(&stats_dir)?;
            self.write_minify_html(stats_dir.join("index.html"), &html)?;
        }
        
        Ok(())
    }

    /// Export daily activity data grouped by year as separate JSON files and return available years
    pub(crate) async fn export_daily_activity_by_year(&self, daily_activity: &[crate::models::DailyStats]) -> Result<Vec<i32>> {
        // Group daily stats by year
        let mut activity_by_year: HashMap<i32, Vec<&crate::models::DailyStats>> = HashMap::new();
        
        for day_stat in daily_activity {
            // Extract year from date (format: yyyy-mm-dd)
            if let Some(year_str) = day_stat.date.get(0..4)
                && let Ok(year) = year_str.parse::<i32>() {
                activity_by_year.entry(year).or_default().push(day_stat);
            }
        }
        
        // Collect available years before consuming the map
        let mut available_years: Vec<i32> = activity_by_year.keys().cloned().collect();
        available_years.sort_by(|a, b| b.cmp(a)); // Sort descending (newest first)
        
        // Export each year's data to a separate file
        for (year, year_data) in activity_by_year {
            let filename = format!("daily_activity_{}.json", year);
            let file_path = self.statistics_json_dir().join(filename);
            
            // Wrap the data with configuration information
            let json_data = serde_json::json!({
                "data": year_data,
                "config": {
                    "max_scale_seconds": self.heatmap_scale_max
                }
            });
            
            let json = serde_json::to_string_pretty(&json_data)?;
            self.cache_manifest.register_file(&file_path, &self.output_dir, json.as_bytes());
            fs::write(file_path, json)?;
        }
        
        Ok(available_years)
    }
}

