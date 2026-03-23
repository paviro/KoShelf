//! TOML configuration file support for KoShelf.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct FileConfig {
    pub library: Option<LibrarySection>,
    pub koshelf: Option<KoshelfSection>,
    pub server: Option<ServerSection>,
    pub output: Option<OutputSection>,
    pub statistics: Option<StatisticsSection>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct LibrarySection {
    pub paths: Option<Vec<PathBuf>>,
    pub docsettings_path: Option<PathBuf>,
    pub hashdocsettings_path: Option<PathBuf>,
    pub statistics_db: Option<PathBuf>,
    pub include_unread: Option<bool>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct KoshelfSection {
    pub title: Option<String>,
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub data_path: Option<PathBuf>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct ServerSection {
    pub port: Option<u16>,
    pub enable_auth: Option<bool>,
    pub enable_writeback: Option<bool>,
    pub trusted_proxies: Option<Vec<String>>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct OutputSection {
    pub path: Option<PathBuf>,
    pub include_files: Option<bool>,
    pub watch: Option<bool>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct StatisticsSection {
    pub heatmap_scale_max: Option<String>,
    pub day_start_time: Option<String>,
    pub min_pages_per_day: Option<u32>,
    pub min_time_per_day: Option<String>,
    pub include_all_stats: Option<bool>,
    pub ignore_stable_page_metadata: Option<bool>,
}

impl FileConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let contents =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;
        toml::from_str(&contents).with_context(|| format!("Failed to parse {:?}", path))
    }
}
