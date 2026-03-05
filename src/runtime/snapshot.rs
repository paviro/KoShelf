//! Typed in-memory representation of the generated transport contracts.
//!
//! This snapshot is the long-term runtime source of truth for `/api` responses.

use anyhow::{Context, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::contracts::calendar::{ActivityMonthResponse, ActivityMonthsResponse};
use crate::contracts::library::{LibraryDetailResponse, LibraryListResponse};
use crate::contracts::locales::LocalesResponse;
use crate::contracts::recap::{CompletionYearResponse, CompletionYearsResponse};
use crate::contracts::site::SiteResponse;
use crate::contracts::statistics::{
    ActivityWeekResponse, ActivityWeeksResponse, ActivityYearDailyResponse,
    ActivityYearSummaryResponse,
};

const FILTER_KEYS: [&str; 3] = ["all", "books", "comics"];

#[derive(Debug, Clone, Default)]
pub struct ContractSnapshot {
    pub site: Option<SiteResponse>,
    pub locales: Option<LocalesResponse>,
    pub items: Option<LibraryListResponse>,
    pub item_details: HashMap<String, LibraryDetailResponse>,
    pub activity_weeks: HashMap<String, ActivityWeeksResponse>,
    pub activity_weeks_by_key: HashMap<String, HashMap<String, ActivityWeekResponse>>,
    pub activity_year_daily: HashMap<String, HashMap<String, ActivityYearDailyResponse>>,
    pub activity_year_summary: HashMap<String, HashMap<String, ActivityYearSummaryResponse>>,
    pub activity_months: HashMap<String, ActivityMonthsResponse>,
    pub activity_months_by_key: HashMap<String, HashMap<String, ActivityMonthResponse>>,
    pub completion_years: HashMap<String, CompletionYearsResponse>,
    pub completion_years_by_key: HashMap<String, HashMap<String, CompletionYearResponse>>,
}

impl ContractSnapshot {
    pub fn load_from_data_dir(data_dir: &Path) -> Result<Self> {
        let activity_dir = data_dir.join("activity");
        let completions_dir = data_dir.join("completions");

        Ok(Self {
            site: Self::read_optional_json(&data_dir.join("site.json"))?,
            locales: Self::read_optional_json(&data_dir.join("locales.json"))?,
            items: Self::read_optional_json(&data_dir.join("items").join("index.json"))?,
            item_details: Self::read_json_map_from_dir(&data_dir.join("items").join("by_id"))?,
            activity_weeks: Self::read_filtered_index_json(&activity_dir.join("weeks"))?,
            activity_weeks_by_key: Self::read_filtered_json_maps(
                &activity_dir.join("weeks"),
                "by_key",
            )?,
            activity_year_daily: Self::read_filtered_json_maps(
                &activity_dir.join("years"),
                "daily",
            )?,
            activity_year_summary: Self::read_filtered_json_maps(
                &activity_dir.join("years"),
                "summary",
            )?,
            activity_months: Self::read_filtered_index_json(&activity_dir.join("months"))?,
            activity_months_by_key: Self::read_filtered_json_maps(
                &activity_dir.join("months"),
                "by_key",
            )?,
            completion_years: Self::read_filtered_index_json(&completions_dir.join("years"))?,
            completion_years_by_key: Self::read_filtered_json_maps(
                &completions_dir.join("years"),
                "by_key",
            )?,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.site.is_none()
            && self.locales.is_none()
            && self.items.is_none()
            && self.item_details.is_empty()
            && self.activity_weeks.is_empty()
            && self.activity_weeks_by_key.is_empty()
            && self.activity_year_daily.is_empty()
            && self.activity_year_summary.is_empty()
            && self.activity_months.is_empty()
            && self.activity_months_by_key.is_empty()
            && self.completion_years.is_empty()
            && self.completion_years_by_key.is_empty()
    }

    pub fn generated_at(&self) -> Option<&str> {
        self.site
            .as_ref()
            .map(|site| site.meta.generated_at.as_str())
    }

    pub fn write_to_data_dir(&self, data_dir: &Path) -> Result<()> {
        fs::create_dir_all(data_dir)?;

        // Ensure data dir contains only current contract outputs.
        for legacy_file in ["books.json", "comics.json"] {
            if let Err(error) = fs::remove_file(data_dir.join(legacy_file))
                && error.kind() != std::io::ErrorKind::NotFound
            {
                return Err(error.into());
            }
        }
        for legacy_dir in ["books", "comics", "statistics", "calendar", "recap"] {
            if let Err(error) = fs::remove_dir_all(data_dir.join(legacy_dir))
                && error.kind() != std::io::ErrorKind::NotFound
            {
                return Err(error.into());
            }
        }

        Self::write_optional_json(&data_dir.join("site.json"), self.site.as_ref())?;
        Self::write_optional_json(&data_dir.join("locales.json"), self.locales.as_ref())?;

        let items_dir = data_dir.join("items");
        Self::write_optional_json(&items_dir.join("index.json"), self.items.as_ref())?;
        Self::write_json_map(&items_dir.join("by_id"), &self.item_details)?;

        let activity_dir = data_dir.join("activity");
        Self::write_filtered_index_json(&activity_dir.join("weeks"), &self.activity_weeks)?;
        Self::write_filtered_json_maps(
            &activity_dir.join("weeks"),
            "by_key",
            &self.activity_weeks_by_key,
        )?;
        Self::write_filtered_json_maps(
            &activity_dir.join("years"),
            "daily",
            &self.activity_year_daily,
        )?;
        Self::write_filtered_json_maps(
            &activity_dir.join("years"),
            "summary",
            &self.activity_year_summary,
        )?;
        Self::write_filtered_index_json(&activity_dir.join("months"), &self.activity_months)?;
        Self::write_filtered_json_maps(
            &activity_dir.join("months"),
            "by_key",
            &self.activity_months_by_key,
        )?;

        let completions_dir = data_dir.join("completions");
        Self::write_filtered_index_json(
            &completions_dir.join("years"),
            &self.completion_years,
        )?;
        Self::write_filtered_json_maps(
            &completions_dir.join("years"),
            "by_key",
            &self.completion_years_by_key,
        )?;

        Ok(())
    }

    fn read_filtered_index_json<T: DeserializeOwned>(
        directory: &Path,
    ) -> Result<HashMap<String, T>> {
        let mut values = HashMap::new();

        for filter in FILTER_KEYS {
            let path = directory.join(filter).join("index.json");
            if let Some(value) = Self::read_optional_json::<T>(&path)? {
                values.insert(filter.to_string(), value);
            }
        }

        Ok(values)
    }

    fn write_filtered_index_json<T: Serialize>(
        directory: &Path,
        values: &HashMap<String, T>,
    ) -> Result<()> {
        for filter in FILTER_KEYS {
            let path = directory.join(filter).join("index.json");
            Self::write_optional_json(&path, values.get(filter))?;
        }

        Ok(())
    }

    fn read_filtered_json_maps<T: DeserializeOwned>(
        directory: &Path,
        nested_subdir: &str,
    ) -> Result<HashMap<String, HashMap<String, T>>> {
        let mut values = HashMap::new();

        for filter in FILTER_KEYS {
            let filter_values =
                Self::read_json_map_from_dir(&directory.join(filter).join(nested_subdir))?;
            values.insert(filter.to_string(), filter_values);
        }

        Ok(values)
    }

    fn write_filtered_json_maps<T: Serialize>(
        directory: &Path,
        nested_subdir: &str,
        values: &HashMap<String, HashMap<String, T>>,
    ) -> Result<()> {
        for filter in FILTER_KEYS {
            let empty = HashMap::new();
            let filter_values = values.get(filter).unwrap_or(&empty);
            Self::write_json_map(&directory.join(filter).join(nested_subdir), filter_values)?;
        }

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

#[cfg(test)]
mod tests {
    use super::ContractSnapshot;
    use crate::contracts::calendar::{ActivityMonthResponse, ActivityMonthsResponse, CalendarMonthlyStats};
    use crate::contracts::common::{ApiMeta, ContentTypeFilter};
    use crate::contracts::library::{
        LibraryContentType, LibraryListItem, LibraryListResponse, LibraryStatus,
    };
    use crate::contracts::recap::{
        CompletionYearResponse, CompletionYearsResponse, RecapMonthResponse, RecapSummaryResponse,
    };
    use crate::contracts::site::{SiteCapabilities, SiteResponse};
    use crate::contracts::statistics::{
        ActivityHeatmapConfig, ActivityOverview, ActivityStreaks, ActivityWeekResponse,
        ActivityWeeksResponse, ActivityYearDailyResponse, ActivityYearSummaryResponse,
        AvailableWeek, YearlySummary,
    };
    use crate::models::{DailyStats, StreakInfo, WeeklyStats};
    use std::collections::{BTreeMap, HashMap};

    fn sample_meta() -> ApiMeta {
        ApiMeta {
            version: "test".to_string(),
            generated_at: "2026-03-05T00:00:00+00:00".to_string(),
        }
    }

    #[test]
    fn snapshot_roundtrip_persists_model_centric_layout() {
        let mut snapshot = ContractSnapshot::default();

        snapshot.site = Some(SiteResponse {
            meta: sample_meta(),
            title: "Test".to_string(),
            capabilities: SiteCapabilities {
                has_books: true,
                has_comics: true,
                has_activity: true,
                has_completions: true,
            },
        });

        snapshot.items = Some(LibraryListResponse {
            meta: sample_meta(),
            items: vec![LibraryListItem {
                id: "item-1".to_string(),
                title: "Item One".to_string(),
                authors: vec!["Author".to_string()],
                series: None,
                status: LibraryStatus::Reading,
                progress_percentage: Some(0.5),
                rating: None,
                annotation_count: 0,
                cover_url: "/assets/covers/item-1.webp".to_string(),
                content_type: LibraryContentType::Book,
            }],
        });

        snapshot.activity_weeks.insert(
            "all".to_string(),
            ActivityWeeksResponse {
                meta: sample_meta(),
                content_type: ContentTypeFilter::All,
                available_years: vec![2026],
                available_weeks: vec![AvailableWeek {
                    week_key: "2026-03-02".to_string(),
                    start_date: "2026-03-02".to_string(),
                    end_date: "2026-03-08".to_string(),
                }],
                overview: ActivityOverview {
                    total_read_time: 10,
                    total_page_reads: 20,
                    longest_read_time_in_day: 5,
                    most_pages_in_day: 8,
                    average_session_duration: Some(3),
                    longest_session_duration: Some(4),
                    total_completions: 1,
                    items_completed: 1,
                    most_completions: 1,
                },
                streaks: ActivityStreaks {
                    longest: StreakInfo::new(
                        2,
                        Some("2026-03-01".to_string()),
                        Some("2026-03-02".to_string()),
                    ),
                    current: StreakInfo::new(
                        1,
                        Some("2026-03-05".to_string()),
                        Some("2026-03-05".to_string()),
                    ),
                },
                heatmap_config: ActivityHeatmapConfig {
                    max_scale_seconds: Some(3600),
                },
            },
        );

        snapshot.activity_weeks_by_key.insert(
            "all".to_string(),
            HashMap::from([(
                "2026-03-02".to_string(),
                ActivityWeekResponse {
                    meta: sample_meta(),
                    content_type: ContentTypeFilter::All,
                    week_key: "2026-03-02".to_string(),
                    stats: WeeklyStats {
                        start_date: "2026-03-02".to_string(),
                        end_date: "2026-03-08".to_string(),
                        read_time: 10,
                        pages_read: 20,
                        avg_pages_per_day: 2.0,
                        avg_read_time_per_day: 1.0,
                        longest_session_duration: Some(4),
                        average_session_duration: Some(3),
                    },
                },
            )]),
        );

        snapshot.activity_year_daily.insert(
            "all".to_string(),
            HashMap::from([(
                "2026".to_string(),
                ActivityYearDailyResponse {
                    meta: sample_meta(),
                    content_type: ContentTypeFilter::All,
                    year: 2026,
                    daily_activity: vec![DailyStats {
                        date: "2026-03-05".to_string(),
                        read_time: 10,
                        pages_read: 20,
                    }],
                    config: Some(ActivityHeatmapConfig {
                        max_scale_seconds: Some(3600),
                    }),
                },
            )]),
        );

        snapshot.activity_year_summary.insert(
            "all".to_string(),
            HashMap::from([(
                "2026".to_string(),
                ActivityYearSummaryResponse {
                    meta: sample_meta(),
                    content_type: ContentTypeFilter::All,
                    year: 2026,
                    summary: YearlySummary { completed_count: 1 },
                    monthly_aggregates: vec![],
                    config: Some(ActivityHeatmapConfig {
                        max_scale_seconds: Some(3600),
                    }),
                },
            )]),
        );

        snapshot.activity_months.insert(
            "all".to_string(),
            ActivityMonthsResponse {
                meta: sample_meta(),
                content_type: ContentTypeFilter::All,
                months: vec!["2026-03".to_string()],
            },
        );
        snapshot.activity_months_by_key.insert(
            "all".to_string(),
            HashMap::from([(
                "2026-03".to_string(),
                ActivityMonthResponse {
                    meta: sample_meta(),
                    content_type: ContentTypeFilter::All,
                    events: vec![],
                    items: BTreeMap::new(),
                    stats: CalendarMonthlyStats {
                        items_read: 1,
                        pages_read: 20,
                        time_read: 10,
                        days_read_pct: 5,
                    },
                },
            )]),
        );

        snapshot.completion_years.insert(
            "all".to_string(),
            CompletionYearsResponse {
                meta: sample_meta(),
                content_type: ContentTypeFilter::All,
                available_years: vec![2026],
                latest_year: Some(2026),
            },
        );
        snapshot.completion_years_by_key.insert(
            "all".to_string(),
            HashMap::from([(
                "2026".to_string(),
                CompletionYearResponse {
                    meta: sample_meta(),
                    content_type: ContentTypeFilter::All,
                    year: 2026,
                    summary: RecapSummaryResponse {
                        total_items: 1,
                        total_time_seconds: 10,
                        total_time_days: 0,
                        total_time_hours: 0,
                        longest_session_hours: 0,
                        longest_session_minutes: 1,
                        average_session_hours: 0,
                        average_session_minutes: 1,
                        active_days: 1,
                        active_days_percentage: 1.0,
                        longest_streak: 1,
                        best_month_name: Some("March".to_string()),
                        best_month_time_display: Some("10m".to_string()),
                    },
                    months: vec![RecapMonthResponse {
                        month_key: "2026-03".to_string(),
                        month_label: "March".to_string(),
                        items_finished: 1,
                        read_time: 10,
                        items: vec![],
                    }],
                    items: vec![],
                    share_assets: None,
                },
            )]),
        );

        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        snapshot
            .write_to_data_dir(temp_dir.path())
            .expect("snapshot should serialize");

        let loaded = ContractSnapshot::load_from_data_dir(temp_dir.path())
            .expect("snapshot should deserialize");

        assert_eq!(
            loaded
                .items
                .as_ref()
                .expect("items should exist")
                .items
                .len(),
            1
        );
        assert!(loaded.activity_weeks.contains_key("all"));
        assert!(
            loaded
                .activity_weeks_by_key
                .get("all")
                .and_then(|weeks| weeks.get("2026-03-02"))
                .is_some()
        );
        assert!(loaded.activity_months.contains_key("all"));
        assert!(loaded.completion_years.contains_key("all"));
        assert!(
            loaded
                .completion_years_by_key
                .get("all")
                .and_then(|years| years.get("2026"))
                .is_some()
        );
    }
}
