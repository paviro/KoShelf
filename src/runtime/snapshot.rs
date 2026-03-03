//! Typed in-memory representation of the generated transport contracts.
//!
//! This snapshot is the long-term runtime source of truth for `/api` responses.

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
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

    fn read_optional_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
        if !path.exists() {
            return Ok(None);
        }

        Ok(Some(Self::read_required_json(path)?))
    }

    fn read_required_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
        let bytes = fs::read(path).with_context(|| format!("failed to read {:?}", path))?;
        serde_json::from_slice::<T>(&bytes).with_context(|| format!("failed to parse {:?}", path))
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
