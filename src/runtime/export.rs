//! Static data file exporter.
//!
//! Generates flat JSON data files for static hosting from the same domain
//! services used by serve-mode API handlers.
//!
//! See `rewamp/04_static_export_shim.md` for the target file layout.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use log::info;
use serde::Serialize;

use crate::contracts::common::ContentTypeFilter;
use crate::contracts::library::LibraryContentType;
use crate::contracts::reading::{ReadingAvailablePeriodsData, ReadingSummaryData};
use crate::domain::library::LibraryService;
use crate::domain::library::queries::{IncludeSet, LibraryDetailQuery, LibraryListQuery};
use crate::domain::reading::ReadingService;
use crate::domain::reading::queries::{
    CompletionsGroupBy, CompletionsIncludeSet, CompletionsSelector, MetricsGroupBy, PeriodGroupBy,
    PeriodSource, ReadingAvailablePeriodsQuery, ReadingCalendarQuery, ReadingCompletionsQuery,
    ReadingMetric, ReadingMetricsQuery, ReadingSummaryQuery,
};
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::runtime::ReadingData;

// ── Export-specific serialization types ──────────────────────────────────

/// `data/site.json`
#[derive(Serialize)]
struct ExportSite {
    name: String,
    version: String,
    default_language: String,
    capabilities: ExportSiteCapabilities,
}

#[derive(Serialize)]
struct ExportSiteCapabilities {
    has_books: bool,
    has_comics: bool,
    has_reading_data: bool,
}

/// `data/reading/summary.json` — keyed by scope.
#[derive(Serialize)]
struct ExportReadingSummary {
    all: ReadingSummaryData,
    books: ReadingSummaryData,
    comics: ReadingSummaryData,
}

/// `data/reading/periods.json` — all source/group_by/scope combos.
#[derive(Serialize)]
struct ExportReadingPeriods {
    reading_data: ExportPeriodsReadingData,
    completions: ExportPeriodsCompletions,
}

#[derive(Serialize)]
struct ExportPeriodsReadingData {
    week: ExportPeriodsByScope,
    month: ExportPeriodsByScope,
    year: ExportPeriodsByScope,
}

#[derive(Serialize)]
struct ExportPeriodsCompletions {
    month: ExportPeriodsByScope,
    year: ExportPeriodsByScope,
}

#[derive(Serialize)]
struct ExportPeriodsByScope {
    all: ReadingAvailablePeriodsData,
    books: ReadingAvailablePeriodsData,
    comics: ReadingAvailablePeriodsData,
}

/// `data/reading/metrics/{YYYY-MM}.json`
#[derive(Serialize)]
struct ExportMonthMetrics {
    month: String,
    days: BTreeMap<String, ExportDayMetricsByScope>,
}

#[derive(Serialize)]
struct ExportDayMetricsByScope {
    all: ExportDayMetrics,
    books: ExportDayMetrics,
    comics: ExportDayMetrics,
}

#[derive(Default, Serialize)]
struct ExportDayMetrics {
    reading_time_sec: i64,
    pages_read: i64,
    sessions: i64,
    completions: i64,
}

// ── Configuration ───────────────────────────────────────────────────────

/// Minimal configuration needed by the data exporter.
pub struct ExportConfig {
    pub site_title: String,
    pub language: String,
}

// ── Public entry point ──────────────────────────────────────────────────

/// Export all static data files to `data_dir`.
///
/// Uses the same domain services as serve-mode handlers.
/// The `library_repo` must already be populated via the shared ingest pipeline.
pub async fn export_data_files(
    data_dir: &Path,
    library_repo: &LibraryRepository,
    reading_data: Option<&ReadingData>,
    config: &ExportConfig,
) -> Result<()> {
    info!("Exporting static data files to {:?}", data_dir);
    fs::create_dir_all(data_dir)?;

    // ── Library domain ──────────────────────────────────────────────────
    let items_data = LibraryService::list(library_repo, LibraryListQuery::default()).await?;
    let items = &items_data.items;

    let has_books = items
        .iter()
        .any(|i| i.content_type == LibraryContentType::Book);
    let has_comics = items
        .iter()
        .any(|i| i.content_type == LibraryContentType::Comic);
    let has_reading_data = reading_data
        .map(|rd| !rd.stats_data.page_stats.is_empty())
        .unwrap_or(false);

    // site.json
    write_json(
        &data_dir.join("site.json"),
        &ExportSite {
            name: config.site_title.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            default_language: config.language.clone(),
            capabilities: ExportSiteCapabilities {
                has_books,
                has_comics,
                has_reading_data,
            },
        },
    )?;

    // items/index.json — all items, list projection (array only)
    write_json(&data_dir.join("items").join("index.json"), items)?;

    // items/{id}.json — per-item with all includes expanded
    export_item_details(data_dir, library_repo, reading_data, items).await?;

    info!(
        "Exported {} library items ({} detail files)",
        items.len(),
        items.len()
    );

    // ── Reading domain ──────────────────────────────────────────────────
    if let Some(rd) = reading_data
        && has_reading_data
    {
        export_reading_summary(data_dir, rd)?;
        export_reading_periods(data_dir, rd)?;
        export_reading_metrics(data_dir, rd)?;
        export_reading_calendar(data_dir, rd)?;
        export_reading_completions(data_dir, rd)?;
    }

    info!("Static data export complete");
    Ok(())
}

// ── Item detail export ──────────────────────────────────────────────────

async fn export_item_details(
    data_dir: &Path,
    library_repo: &LibraryRepository,
    reading_data: Option<&ReadingData>,
    items: &[crate::contracts::library::LibraryListItem],
) -> Result<()> {
    let items_dir = data_dir.join("items");

    let mut exported_ids = HashSet::new();

    for item in items {
        let query = LibraryDetailQuery::new(&item.id, IncludeSet::all());
        if let Some(detail) = LibraryService::detail(library_repo, &query, reading_data).await? {
            write_json(&items_dir.join(format!("{}.json", item.id)), &detail)?;
            exported_ids.insert(item.id.clone());
        }
    }

    cleanup_stale_json(&items_dir, &exported_ids, &["index"])?;

    Ok(())
}

// ── Reading summary export ──────────────────────────────────────────────

fn export_reading_summary(data_dir: &Path, reading_data: &ReadingData) -> Result<()> {
    let summary = ExportReadingSummary {
        all: ReadingService::summary(
            reading_data,
            ReadingSummaryQuery {
                scope: ContentTypeFilter::All,
                range: None,
                tz: None,
            },
        ),
        books: ReadingService::summary(
            reading_data,
            ReadingSummaryQuery {
                scope: ContentTypeFilter::Books,
                range: None,
                tz: None,
            },
        ),
        comics: ReadingService::summary(
            reading_data,
            ReadingSummaryQuery {
                scope: ContentTypeFilter::Comics,
                range: None,
                tz: None,
            },
        ),
    };

    write_json(&data_dir.join("reading").join("summary.json"), &summary)
}

// ── Reading periods export ──────────────────────────────────────────────

fn export_reading_periods(data_dir: &Path, reading_data: &ReadingData) -> Result<()> {
    let periods = ExportReadingPeriods {
        reading_data: ExportPeriodsReadingData {
            week: periods_by_scope(reading_data, PeriodSource::ReadingData, PeriodGroupBy::Week),
            month: periods_by_scope(
                reading_data,
                PeriodSource::ReadingData,
                PeriodGroupBy::Month,
            ),
            year: periods_by_scope(reading_data, PeriodSource::ReadingData, PeriodGroupBy::Year),
        },
        completions: ExportPeriodsCompletions {
            month: periods_by_scope(
                reading_data,
                PeriodSource::Completions,
                PeriodGroupBy::Month,
            ),
            year: periods_by_scope(reading_data, PeriodSource::Completions, PeriodGroupBy::Year),
        },
    };

    write_json(&data_dir.join("reading").join("periods.json"), &periods)
}

fn periods_by_scope(
    reading_data: &ReadingData,
    source: PeriodSource,
    group_by: PeriodGroupBy,
) -> ExportPeriodsByScope {
    let query = |scope| ReadingAvailablePeriodsQuery {
        scope,
        source,
        group_by,
        range: None,
        tz: None,
    };

    ExportPeriodsByScope {
        all: ReadingService::available_periods(reading_data, query(ContentTypeFilter::All)),
        books: ReadingService::available_periods(reading_data, query(ContentTypeFilter::Books)),
        comics: ReadingService::available_periods(reading_data, query(ContentTypeFilter::Comics)),
    }
}

// ── Reading metrics export ──────────────────────────────────────────────

fn export_reading_metrics(data_dir: &Path, reading_data: &ReadingData) -> Result<()> {
    // Determine which months have reading data.
    let month_periods = ReadingService::available_periods(
        reading_data,
        ReadingAvailablePeriodsQuery {
            scope: ContentTypeFilter::All,
            source: PeriodSource::ReadingData,
            group_by: PeriodGroupBy::Month,
            range: None,
            tz: None,
        },
    );

    if month_periods.periods.is_empty() {
        return Ok(());
    }

    let metrics_dir = data_dir.join("reading").join("metrics");
    let scopes = [
        ContentTypeFilter::All,
        ContentTypeFilter::Books,
        ContentTypeFilter::Comics,
    ];
    let metrics = [
        ReadingMetric::ReadingTimeSec,
        ReadingMetric::PagesRead,
        ReadingMetric::Sessions,
        ReadingMetric::Completions,
    ];

    // For each (scope, metric), get all daily points and partition into months.
    let mut months: BTreeMap<String, BTreeMap<String, ExportDayMetricsByScope>> = BTreeMap::new();

    for &scope in &scopes {
        let scope_name = scope.as_str();

        for &metric in &metrics {
            let result = ReadingService::metrics(
                reading_data,
                ReadingMetricsQuery {
                    scope,
                    metrics: vec![metric],
                    group_by: MetricsGroupBy::Day,
                    range: None,
                    tz: None,
                },
            );

            let metric_name = metric.as_str();

            for point in &result.points {
                let month_key = point.key[..7].to_string();
                let day_map = months.entry(month_key).or_default();
                let by_scope =
                    day_map
                        .entry(point.key.clone())
                        .or_insert_with(|| ExportDayMetricsByScope {
                            all: ExportDayMetrics::default(),
                            books: ExportDayMetrics::default(),
                            comics: ExportDayMetrics::default(),
                        });

                let day_metrics = match scope_name {
                    "all" => &mut by_scope.all,
                    "books" => &mut by_scope.books,
                    "comics" => &mut by_scope.comics,
                    _ => unreachable!(),
                };

                let value = point.values.get(metric_name).copied().unwrap_or(0);
                match metric {
                    ReadingMetric::ReadingTimeSec => day_metrics.reading_time_sec = value,
                    ReadingMetric::PagesRead => day_metrics.pages_read = value,
                    ReadingMetric::Sessions => day_metrics.sessions = value,
                    ReadingMetric::Completions => day_metrics.completions = value,
                    _ => {}
                }
            }
        }
    }

    let mut exported_stems = HashSet::new();

    for (month_key, days) in months {
        let file = ExportMonthMetrics {
            month: month_key.clone(),
            days,
        };
        write_json(&metrics_dir.join(format!("{month_key}.json")), &file)?;
        exported_stems.insert(month_key);
    }

    cleanup_stale_json(&metrics_dir, &exported_stems, &[])?;

    info!(
        "Exported reading metrics for {} months",
        exported_stems.len()
    );

    Ok(())
}

// ── Reading calendar export ─────────────────────────────────────────────

fn export_reading_calendar(data_dir: &Path, reading_data: &ReadingData) -> Result<()> {
    // Determine which months have reading data.
    let month_periods = ReadingService::available_periods(
        reading_data,
        ReadingAvailablePeriodsQuery {
            scope: ContentTypeFilter::All,
            source: PeriodSource::ReadingData,
            group_by: PeriodGroupBy::Month,
            range: None,
            tz: None,
        },
    );

    let calendar_dir = data_dir.join("reading").join("calendar");
    let mut exported_stems = HashSet::new();

    for period in &month_periods.periods {
        // Calendar uses scope=All; stats_by_scope is always included for all three scopes.
        let data = ReadingService::calendar(
            reading_data,
            ReadingCalendarQuery {
                month: period.key.clone(),
                scope: ContentTypeFilter::All,
                tz: None,
            },
        );
        write_json(&calendar_dir.join(format!("{}.json", period.key)), &data)?;
        exported_stems.insert(period.key.clone());
    }

    cleanup_stale_json(&calendar_dir, &exported_stems, &[])?;

    info!(
        "Exported reading calendar for {} months",
        exported_stems.len()
    );

    Ok(())
}

// ── Reading completions export ──────────────────────────────────────────

fn export_reading_completions(data_dir: &Path, reading_data: &ReadingData) -> Result<()> {
    // Determine which years have completion data.
    let year_periods = ReadingService::available_periods(
        reading_data,
        ReadingAvailablePeriodsQuery {
            scope: ContentTypeFilter::All,
            source: PeriodSource::Completions,
            group_by: PeriodGroupBy::Year,
            range: None,
            tz: None,
        },
    );

    let completions_dir = data_dir.join("reading").join("completions");
    let mut exported_stems = HashSet::new();

    for period in &year_periods.periods {
        let year: i32 = match period.key.parse() {
            Ok(y) => y,
            Err(_) => continue,
        };

        // Export with scope=all, group_by=month, all includes.
        // The StaticApiClient filters by scope and trims includes client-side.
        let data = ReadingService::completions(
            reading_data,
            ReadingCompletionsQuery {
                scope: ContentTypeFilter::All,
                selector: CompletionsSelector::Year(year),
                group_by: CompletionsGroupBy::Month,
                includes: CompletionsIncludeSet::parse(Some("summary,share_assets"))
                    .expect("known-valid include tokens"),
                tz: None,
            },
        );
        write_json(&completions_dir.join(format!("{}.json", period.key)), &data)?;
        exported_stems.insert(period.key.clone());
    }

    cleanup_stale_json(&completions_dir, &exported_stems, &[])?;

    info!(
        "Exported reading completions for {} years",
        exported_stems.len()
    );

    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, json).with_context(|| format!("failed to write {:?}", path))
}

/// Remove `.json` files from `directory` whose stem is not in `valid_stems`
/// and not in `protected` (e.g. "index" which is managed separately).
fn cleanup_stale_json(
    directory: &Path,
    valid_stems: &HashSet<String>,
    protected: &[&str],
) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(directory)?.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            && !valid_stems.contains(stem)
            && !protected.contains(&stem)
        {
            fs::remove_file(&path)?;
        }
    }

    Ok(())
}
