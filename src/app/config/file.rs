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
    #[serde(default, deserialize_with = "one_or_many_pathbuf")]
    pub statistics_db: Option<Vec<PathBuf>>,
    pub kobo_db: Option<PathBuf>,
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

/// Accept either a single path or an array of paths (backwards compatible).
fn one_or_many_pathbuf<'de, D>(deserializer: D) -> Result<Option<Vec<PathBuf>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(PathBuf),
        Many(Vec<PathBuf>),
    }

    Ok(match Option::<OneOrMany>::deserialize(deserializer)? {
        None => None,
        Some(OneOrMany::One(path)) => Some(vec![path]),
        Some(OneOrMany::Many(paths)) => Some(paths),
    })
}

impl FileConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let contents =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;
        toml::from_str(&contents).with_context(|| format!("Failed to parse {:?}", path))
    }
}

#[cfg(test)]
mod tests {
    use super::FileConfig;
    use std::path::PathBuf;

    #[test]
    fn statistics_db_accepts_single_path() {
        let config: FileConfig = toml::from_str("[library]\nstatistics_db = \"/a.sqlite3\"")
            .expect("single-path form should parse");
        assert_eq!(
            config.library.unwrap().statistics_db,
            Some(vec![PathBuf::from("/a.sqlite3")])
        );
    }

    #[test]
    fn statistics_db_accepts_array_of_paths() {
        let config: FileConfig =
            toml::from_str("[library]\nstatistics_db = [\"/a.sqlite3\", \"/b.sqlite3\"]")
                .expect("array form should parse");
        assert_eq!(
            config.library.unwrap().statistics_db,
            Some(vec![
                PathBuf::from("/a.sqlite3"),
                PathBuf::from("/b.sqlite3")
            ])
        );
    }
}
