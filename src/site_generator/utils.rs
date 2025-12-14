//! Utility functions for site generation.

use chrono::Datelike;

use super::SiteGenerator;
use crate::templates::NavItem;
use anyhow::Result;
use minify_html::{Cfg, minify};
use std::fs;
use std::path::Path;

/// Describes what content exists in the generated site so the navbar can route correctly.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NavContext {
    pub has_books: bool,
    pub has_comics: bool,
    /// When true, the statistics page is rendered to `/` instead of `/statistics/`.
    pub stats_at_root: bool,
}

impl NavContext {
    /// Whether the UI should show a content-type filter (books vs comics).
    pub(crate) fn show_type_filter(self) -> bool {
        self.has_books && self.has_comics
    }
}

/// Shared UI routing context passed to page generators.
#[derive(Debug, Clone)]
pub(crate) struct UiContext {
    pub recap_latest_href: Option<String>,
    pub nav: NavContext,
}

/// Format a duration in seconds to a human-readable string (e.g., "2d 5h 30m")
pub(crate) fn format_duration(seconds: i64, translations: &crate::i18n::Translations) -> String {
    if seconds <= 0 {
        return format!("0{}", translations.get("units.m"));
    }
    let total_minutes = seconds / 60;
    let days = total_minutes / (24 * 60);
    let hours = (total_minutes % (24 * 60)) / 60;
    let mins = total_minutes % 60;
    let mut parts: Vec<String> = Vec::new();
    if days > 0 {
        parts.push(format!("{}{}", days, translations.get("units.d")));
    }
    if hours > 0 {
        parts.push(format!("{}{}", hours, translations.get("units.h")));
    }
    if mins > 0 || parts.is_empty() {
        parts.push(format!("{}{}", mins, translations.get("units.m")));
    }
    parts.join(" ")
}

/// Format an ISO date (YYYY-MM-DD) to a human-readable format (e.g., "7 Mar" or "7. März")
pub(crate) fn format_day_month(iso: &str, translations: &crate::i18n::Translations) -> String {
    if let Ok(date) = chrono::NaiveDate::parse_from_str(iso, "%Y-%m-%d") {
        let current_year = chrono::Utc::now().year();
        let locale = translations.locale();

        // Get appropriate format string from translations
        let format_key = if date.year() == current_year {
            "datetime.short-current-year"
        } else {
            "datetime.short-with-year"
        };
        let format_str = translations.get(format_key);

        date.format_localized(&format_str, locale).to_string()
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

    /// Minifies and writes HTML to disk, registering in cache manifest.
    pub(crate) fn write_minify_html<P: AsRef<Path>>(&self, path: P, html: &str) -> Result<()> {
        let cfg = Cfg {
            minify_js: true,
            minify_css: true,
            ..Default::default()
        };

        // Attempt minification; on failure fall back to original HTML
        let minified =
            String::from_utf8(minify(html.as_bytes(), &cfg)).unwrap_or_else(|_| html.to_string());

        // Register in cache manifest before writing
        self.cache_manifest
            .register_file(&path, &self.output_dir, minified.as_bytes());

        fs::write(path, minified)?;
        Ok(())
    }

    /// Writes bytes to disk and registers them in the cache manifest.
    pub(crate) fn write_registered_bytes<P: AsRef<Path>>(
        &self,
        path: P,
        bytes: &[u8],
    ) -> Result<()> {
        let path_ref = path.as_ref();
        self.cache_manifest
            .register_file(path_ref, &self.output_dir, bytes);
        fs::write(path_ref, bytes)?;
        Ok(())
    }

    /// Writes a UTF-8 string to disk and registers it in the cache manifest.
    pub(crate) fn write_registered_string<P: AsRef<Path>>(
        &self,
        path: P,
        content: &str,
    ) -> Result<()> {
        self.write_registered_bytes(path, content.as_bytes())
    }

    /// Writes pretty JSON to disk and registers it in the cache manifest.
    pub(crate) fn write_registered_json_pretty<P: AsRef<Path>, T: serde::Serialize>(
        &self,
        path: P,
        value: &T,
    ) -> Result<()> {
        let json = serde_json::to_string_pretty(value)?;
        self.write_registered_string(path, &json)
    }

    /// Create default navbar items
    pub(crate) fn create_navbar_items(&self, current_page: &str, nav: NavContext) -> Vec<NavItem> {
        let mut items = Vec::new();

        // Books list is only available when there are books.
        if nav.has_books {
            items.push(NavItem {
                label: self.translations.get("books"),
                href: "/".to_string(),
                icon_svg: "M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253".to_string(),
                is_active: current_page == "books",
                id: None,
            });
        }

        // Comics list is `/comics/` when books exist, otherwise it becomes the home page (`/`).
        if nav.has_comics {
            let href = if nav.has_books { "/comics/" } else { "/" };
            items.push(NavItem {
                label: self.translations.get("comics"),
                href: href.to_string(),
                // Comic book icon (speech bubble)
                icon_svg: "M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z".to_string(),
                is_active: current_page == "comics",
                id: None,
            });
        }

        // Add stats navigation item if we have a stats database path configured
        if self.statistics_db_path.is_some() {
            let stats_href = if nav.stats_at_root {
                "/"
            } else {
                "/statistics/"
            }
            .to_string();

            items.push(NavItem {
                label: self.translations.get("statistics"),
                href: stats_href,
                icon_svg: "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z".to_string(),
                is_active: current_page == "statistics",
                id: Some("nav-statistics".to_string()),
            });
        }

        // Add calendar navigation item if we have statistics data
        if self.statistics_db_path.is_some() {
            items.push(NavItem {
                label: self.translations.get("calendar"),
                href: "/calendar/".to_string(),  // Calendar always goes to /calendar/
                icon_svg: "M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z".to_string(),
                is_active: current_page == "calendar",
                id: None,
            });
        }

        items
    }

    pub(crate) fn create_navbar_items_with_recap(
        &self,
        current_page: &str,
        recap_latest_href: Option<&str>,
        nav: NavContext,
    ) -> Vec<NavItem> {
        let mut items = self.create_navbar_items(current_page, nav);
        if self.statistics_db_path.is_some() {
            let href = recap_latest_href.unwrap_or("/recap/");
            items.push(NavItem {
                label: self.translations.get("recap"),
                href: href.to_string(),
                icon_svg: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z".to_string(),
                is_active: current_page == "recap",
                id: Some("nav-recap".to_string()),
            });
        }
        items
    }
}
