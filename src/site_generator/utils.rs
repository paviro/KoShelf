//! Utility functions for site generation.

use super::SiteGenerator;
use crate::templates::NavItem;
use anyhow::Result;
use minify_html::{Cfg, minify};
use std::fs;
use std::path::Path;

/// Format a duration in seconds to a human-readable string (e.g., "2d 5h 30m")
pub(crate) fn format_duration(seconds: i64) -> String {
    if seconds <= 0 { return "0m".to_string(); }
    let total_minutes = seconds / 60;
    let days = total_minutes / (24 * 60);
    let hours = (total_minutes % (24 * 60)) / 60;
    let mins = total_minutes % 60;
    let mut parts: Vec<String> = Vec::new();
    if days > 0 { parts.push(format!("{}d", days)); }
    if hours > 0 { parts.push(format!("{}h", hours)); }
    if mins > 0 || parts.is_empty() { parts.push(format!("{}m", mins)); }
    parts.join(" ")
}

/// Format an ISO date (YYYY-MM-DD) to a human-readable format (e.g., "7 Mar")
pub(crate) fn format_day_month(iso: &str) -> String {
    if let Ok(date) = chrono::NaiveDate::parse_from_str(iso, "%Y-%m-%d") {
        // "%e %b" -> day (space-padded) and abbreviated month name; trim for clean day
        let s = date.format("%e %b").to_string();
        s.trim_start().to_string()
    } else {
        iso.to_string()
    }
}

impl SiteGenerator {
    /// Get current version from Cargo.toml
    pub(crate) fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
    
    /// Get current datetime as formatted string
    pub(crate) fn get_last_updated(&self) -> String { 
        self.time_config.now_formatted() 
    }

    /// Minifies and writes HTML to disk.
    pub(crate) fn write_minify_html<P: AsRef<Path>>(&self, path: P, html: &str) -> Result<()> {
        let cfg = Cfg {
            minify_js: true,
            minify_css: true,
            ..Default::default()
        };

        // Attempt minification; on failure fall back to original HTML
        let minified = String::from_utf8(minify(html.as_bytes(), &cfg)).unwrap_or_else(|_| html.to_string());
        fs::write(path, minified)?;
        Ok(())
    }

    /// Create default navbar items
    pub(crate) fn create_navbar_items(&self, current_page: &str) -> Vec<NavItem> {
        let mut items = Vec::new();
        
        // Add books navigation item only if we have a books path configured
        if self.books_path.is_some() {
            items.push(NavItem {
                label: "Books".to_string(),
                href: "/".to_string(),
                icon_svg: "M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253".to_string(),
                is_active: current_page == "books",
            });
        }
        
        // Add stats navigation item if we have a stats database path configured
        if self.statistics_db_path.is_some() {
            let stats_href = if self.books_path.is_some() {
                "/statistics/".to_string()  // Books exist, stats go to subfolder
            } else {
                "/".to_string()  // No books, stats are at root
            };
            
            items.push(NavItem {
                label: "Statistics".to_string(),
                href: stats_href,
                icon_svg: "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z".to_string(),
                is_active: current_page == "statistics",
            });
        }
        
        // Add calendar navigation item if we have statistics data
        if self.statistics_db_path.is_some() {
            items.push(NavItem {
                label: "Calendar".to_string(),
                href: "/calendar/".to_string(),  // Calendar always goes to /calendar/
                icon_svg: "M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z".to_string(),
                is_active: current_page == "calendar",
            });
        }
        
        items
    }

    pub(crate) fn create_navbar_items_with_recap(&self, current_page: &str, recap_latest_href: Option<&str>) -> Vec<NavItem> {
        let mut items = self.create_navbar_items(current_page);
        if self.statistics_db_path.is_some() {
            if let Some(href) = recap_latest_href {
                items.push(NavItem {
                    label: "Recap".to_string(),
                    href: href.to_string(),
                    icon_svg: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z".to_string(),
                    is_active: current_page == "recap",
                });
            }
        }
        items
    }
}
