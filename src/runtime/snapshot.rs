//! Typed in-memory representation of the generated transport contracts.
//!
//! This snapshot is the long-term runtime source of truth for `/api` responses.

use anyhow::{Context, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::contracts::calendar::{CalendarMonthResponse, CalendarMonthsResponse};
use crate::contracts::library::{LibraryDetailResponse, LibraryListResponse};
use crate::contracts::locales::LocalesResponse;
use crate::contracts::recap::{RecapIndexResponse, RecapYearResponse};
use crate::contracts::site::SiteResponse;
use crate::contracts::statistics::{
    StatisticsIndexResponse, StatisticsWeekResponse, StatisticsYearResponse,
};

#[derive(Debug, Clone, Default)]
pub struct ContractSnapshot {
    pub site: Option<SiteResponse>,
    pub locales: Option<LocalesResponse>,
    pub books: Option<LibraryListResponse>,
    pub comics: Option<LibraryListResponse>,
    pub book_details: HashMap<String, LibraryDetailResponse>,
    pub comic_details: HashMap<String, LibraryDetailResponse>,
    pub statistics_index: Option<StatisticsIndexResponse>,
    pub statistics_weeks: HashMap<String, StatisticsWeekResponse>,
    pub statistics_years: HashMap<String, StatisticsYearResponse>,
    pub calendar_months: Option<CalendarMonthsResponse>,
    pub calendar_by_month: HashMap<String, CalendarMonthResponse>,
    pub recap_index: Option<RecapIndexResponse>,
    pub recap_years: HashMap<String, RecapYearResponse>,
}

impl ContractSnapshot {
    pub fn load_from_data_dir(data_dir: &Path) -> Result<Self> {
        let mut snapshot = Self::default();

        snapshot.site = Self::read_optional_json(&data_dir.join("site.json"))?;
        snapshot.locales = Self::read_optional_json(&data_dir.join("locales.json"))?;
        snapshot.books = Self::read_optional_json(&data_dir.join("books.json"))?;
        snapshot.comics = Self::read_optional_json(&data_dir.join("comics.json"))?;
        snapshot.book_details = Self::read_json_map_from_dir(&data_dir.join("books"))?;
        snapshot.comic_details = Self::read_json_map_from_dir(&data_dir.join("comics"))?;

        snapshot.statistics_index =
            Self::read_optional_json(&data_dir.join("statistics").join("index.json"))?;
        snapshot.statistics_weeks =
            Self::read_json_map_from_dir(&data_dir.join("statistics").join("weeks"))?;
        snapshot.statistics_years =
            Self::read_json_map_from_dir(&data_dir.join("statistics").join("years"))?;

        snapshot.calendar_months =
            Self::read_optional_json(&data_dir.join("calendar").join("months.json"))?;
        snapshot.calendar_by_month =
            Self::read_json_map_from_dir(&data_dir.join("calendar").join("months"))?;

        snapshot.recap_index =
            Self::read_optional_json(&data_dir.join("recap").join("index.json"))?;
        snapshot.recap_years = Self::read_json_map_from_dir(&data_dir.join("recap").join("years"))?;

        Ok(snapshot)
    }

    pub fn is_empty(&self) -> bool {
        self.site.is_none()
            && self.locales.is_none()
            && self.books.is_none()
            && self.comics.is_none()
            && self.book_details.is_empty()
            && self.comic_details.is_empty()
            && self.statistics_index.is_none()
            && self.statistics_weeks.is_empty()
            && self.statistics_years.is_empty()
            && self.calendar_months.is_none()
            && self.calendar_by_month.is_empty()
            && self.recap_index.is_none()
            && self.recap_years.is_empty()
    }

    pub fn generated_at(&self) -> Option<&str> {
        self.site
            .as_ref()
            .map(|site| site.meta.generated_at.as_str())
    }

    pub fn write_to_data_dir(&self, data_dir: &Path) -> Result<()> {
        fs::create_dir_all(data_dir)?;

        Self::write_optional_json(&data_dir.join("site.json"), self.site.as_ref())?;
        Self::write_optional_json(&data_dir.join("locales.json"), self.locales.as_ref())?;
        Self::write_optional_json(&data_dir.join("books.json"), self.books.as_ref())?;
        Self::write_optional_json(&data_dir.join("comics.json"), self.comics.as_ref())?;

        Self::write_json_map(&data_dir.join("books"), &self.book_details)?;
        Self::write_json_map(&data_dir.join("comics"), &self.comic_details)?;

        let statistics_dir = data_dir.join("statistics");
        Self::write_optional_json(
            &statistics_dir.join("index.json"),
            self.statistics_index.as_ref(),
        )?;
        Self::write_json_map(&statistics_dir.join("weeks"), &self.statistics_weeks)?;
        Self::write_json_map(&statistics_dir.join("years"), &self.statistics_years)?;

        let calendar_dir = data_dir.join("calendar");
        Self::write_optional_json(&calendar_dir.join("months.json"), self.calendar_months.as_ref())?;
        Self::write_json_map(&calendar_dir.join("months"), &self.calendar_by_month)?;

        let recap_dir = data_dir.join("recap");
        Self::write_optional_json(&recap_dir.join("index.json"), self.recap_index.as_ref())?;
        Self::write_json_map(&recap_dir.join("years"), &self.recap_years)?;

        Ok(())
    }

    fn read_optional_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
        if !path.exists() {
            return Ok(None);
        }

        Ok(Some(Self::read_required_json(path)?))
    }

    fn write_optional_json<T: Serialize>(path: &Path, value: Option<&T>) -> Result<()> {
        if let Some(value) = value {
            Self::write_json_pretty(path, value)?;
        } else if let Err(error) = fs::remove_file(path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            return Err(error.into());
        }

        Ok(())
    }

    fn read_required_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
        let bytes = fs::read(path).with_context(|| format!("failed to read {:?}", path))?;
        serde_json::from_slice::<T>(&bytes).with_context(|| format!("failed to parse {:?}", path))
    }

    fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(value)?;
        fs::write(path, json)?;
        Ok(())
    }

    fn write_json_map<T: Serialize>(directory: &Path, values: &HashMap<String, T>) -> Result<()> {
        fs::create_dir_all(directory)?;

        for (key, value) in values {
            Self::write_json_pretty(&directory.join(format!("{key}.json")), value)?;
        }

        let valid_stems: HashSet<&str> = values.keys().map(String::as_str).collect();
        Self::cleanup_stale_json_files(directory, &valid_stems)
    }

    fn cleanup_stale_json_files(directory: &Path, valid_stems: &HashSet<&str>) -> Result<()> {
        if !directory.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(directory)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            if let Some(stem) = path.file_stem().and_then(|name| name.to_str())
                && !valid_stems.contains(stem)
            {
                fs::remove_file(path)?;
            }
        }

        Ok(())
    }

    fn read_json_map_from_dir<T: DeserializeOwned>(directory: &Path) -> Result<HashMap<String, T>> {
        let mut values = HashMap::new();

        if !directory.exists() {
            return Ok(values);
        }

        let entries = fs::read_dir(directory)
            .with_context(|| format!("failed to list directory {:?}", directory))?;

        for entry in entries {
            let entry =
                entry.with_context(|| format!("failed to read entry in {:?}", directory))?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
                continue;
            };

            let value = Self::read_required_json(&path)?;
            values.insert(stem.to_string(), value);
        }

        Ok(values)
    }
}
