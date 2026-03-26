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

use crate::pipeline::media;
use crate::server::api::responses::common::ContentTypeFilter;
use crate::server::api::responses::library::LibraryContentType;
use crate::server::api::responses::reading::ReadingAvailablePeriodsData;
use crate::server::api::responses::site::SiteCapabilities;
use crate::shelf::library::queries::IncludeSet;
use crate::shelf::library::{self, LibraryDetailQuery, LibraryListQuery};
use crate::shelf::statistics;
use crate::shelf::statistics::queries::{
    CompletionsGroupBy, CompletionsIncludeSet, CompletionsSelector, MetricsGroupBy, PeriodGroupBy,
    PeriodSource, ReadingAvailablePeriodsQuery, ReadingCalendarQuery, ReadingCompletionsQuery,
    ReadingMetric, ReadingMetricsQuery, ReadingSummaryQuery,
};
use crate::store::memory::ReadingData;
use crate::store::sqlite::repo::LibraryRepository;

// ── Export-specific serialization types ──────────────────────────────────

/// `data/site.json`
#[derive(Serialize)]
struct ExportSite {
    name: String,
    version: String,
    generated_at: String,
    default_language: String,
    capabilities: SiteCapabilities,
}

// Summary is exported directly as ReadingSummaryData per scope — no wrapper needed.

/// `data/reading/periods/{scope}.json` — all source/group_by combos for one scope.
#[derive(Serialize)]
struct ExportPeriods {
    reading_data: ExportPeriodsReadingData,
    completions: ExportPeriodsCompletions,
}

#[derive(Serialize)]
struct ExportPeriodsReadingData {
    week: ReadingAvailablePeriodsData,
    month: ReadingAvailablePeriodsData,
    year: ReadingAvailablePeriodsData,
}

#[derive(Serialize)]
struct ExportPeriodsCompletions {
    month: ReadingAvailablePeriodsData,
    year: ReadingAvailablePeriodsData,
}

/// `data/reading/metrics/{YYYY-MM}/{scope}.json`
#[derive(Serialize)]
struct ExportMonthMetrics {
    month: String,
    days: BTreeMap<String, ExportDayMetrics>,
}

#[derive(Default, Serialize)]
struct ExportDayMetrics {
    reading_time_sec: i64,
    pages_read: i64,
    sessions: i64,
    completions: i64,
    active_days: i64,
    longest_session_duration_sec: i64,
    average_session_duration_sec: i64,
}

const SCOPES: [ContentTypeFilter; 3] = [
    ContentTypeFilter::All,
    ContentTypeFilter::Books,
    ContentTypeFilter::Comics,
];

// ── Configuration ───────────────────────────────────────────────────────

/// Minimal configuration needed by the data exporter.
pub struct ExportConfig {
    pub site_title: String,
    pub language: String,
    pub include_files: bool,
}

// ── Public entry point ──────────────────────────────────────────────────

/// Export all static data files to `data_dir`.
///
/// Uses the same domain services as serve-mode handlers.
/// The `library_repo` must already be populated via the shared ingest pipeline.
pub async fn export_data_files(
    data_dir: &Path,
    output_dir: &Path,
    library_repo: &LibraryRepository,
    reading_data: Option<&ReadingData>,
    config: &ExportConfig,
) -> Result<()> {
    info!("Exporting static data files to {:?}", data_dir);
    fs::create_dir_all(data_dir)?;

    // ── Library domain ──────────────────────────────────────────────────
    let items_data = library::list(library_repo, LibraryListQuery::default()).await?;
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
            generated_at: chrono::Utc::now().to_rfc3339(),
            default_language: config.language.clone(),
            capabilities: SiteCapabilities {
                has_books,
                has_comics,
                has_reading_data,
                has_files: config.include_files,
                auth_enabled: false,
                has_writeback: false,
            },
        },
    )?;

    // items/index.json — all items, list projection (array only)
    let items_dir = data_dir.join("items");
    write_json(&items_dir.join("index.json"), items)?;

    // items/books.json + items/comics.json — scope-filtered subsets
    let books: Vec<_> = items
        .iter()
        .filter(|i| i.content_type == LibraryContentType::Book)
        .collect();
    let comics: Vec<_> = items
        .iter()
        .filter(|i| i.content_type == LibraryContentType::Comic)
        .collect();
    write_json(&items_dir.join("books.json"), &books)?;
    write_json(&items_dir.join("comics.json"), &comics)?;

    // items/{id}.json — per-item with all includes expanded
    export_item_details(data_dir, library_repo, reading_data, items).await?;

    // items/page-activity/{id}.json — per-item page-level reading heatmap data
    export_page_activity(data_dir, library_repo, reading_data, items).await?;

    info!(
        "Exported {} library items ({} detail files)",
        items.len(),
        items.len()
    );

    // ── Item files ─────────────────────────────────────────────────────
    if config.include_files {
        export_item_files(output_dir, library_repo).await?;
    } else {
        let files_dir = output_dir.join("assets").join("files");
        let keep: HashSet<String> = HashSet::new();
        cleanup_stale_item_files(&files_dir, &keep)?;
    }

    // ── Reading domain ──────────────────────────────────────────────────
    if let Some(rd) = reading_data
        && has_reading_data
    {
        export_reading_summary(data_dir, rd)?;
        export_reading_periods(data_dir, rd)?;
        export_reading_metrics(data_dir, rd)?;
        export_reading_calendar(data_dir, rd, library_repo).await?;
        export_reading_completions(data_dir, rd, library_repo).await?;
    }

    info!("Static data export complete");
    Ok(())
}

// ── Item detail export ──────────────────────────────────────────────────

async fn export_item_details(
    data_dir: &Path,
    library_repo: &LibraryRepository,
    reading_data: Option<&ReadingData>,
    items: &[crate::server::api::responses::library::LibraryListItem],
) -> Result<()> {
    let items_dir = data_dir.join("items");

    let mut exported_ids = HashSet::new();

    for item in items {
        if !media::is_canonical_item_id(&item.id) {
            log::warn!(
                "Skipping detail export for non-canonical item id: {}",
                item.id
            );
            continue;
        }

        let query = LibraryDetailQuery::new(&item.id, IncludeSet::all());
        if let Some(detail) = library::detail(library_repo, &query, reading_data).await? {
            write_json(&items_dir.join(format!("{}.json", item.id)), &detail)?;
            exported_ids.insert(item.id.clone());
        }
    }

    cleanup_stale_json(&items_dir, &exported_ids, &["index", "books", "comics"])?;

    Ok(())
}

// ── Page activity export ────────────────────────────────────────────────

/// Pre-computed page-activity data with per-completion aggregated pages.
#[derive(Serialize)]
struct ExportPageActivityData {
    #[serde(flatten)]
    data: crate::server::api::responses::library::PageActivityData,
    /// Aggregated pages per completion index (key = completion index as string).
    /// Includes an "all" key for the unfiltered view.
    by_completion: BTreeMap<String, Vec<crate::server::api::responses::library::PageActivityPage>>,
}

async fn export_page_activity(
    data_dir: &Path,
    library_repo: &LibraryRepository,
    reading_data: Option<&ReadingData>,
    items: &[crate::server::api::responses::library::LibraryListItem],
) -> Result<()> {
    let page_activity_dir = data_dir.join("items").join("page-activity");
    let mut exported_ids = HashSet::new();

    for item in items {
        if !media::is_canonical_item_id(&item.id) {
            continue;
        }

        // Fetch the "all completions" view (no filter).
        if let Some(result) =
            library::page_activity(library_repo, &item.id, reading_data, None).await?
            && result.data.total_pages > 0
            && !result.data.pages.is_empty()
        {
            // Pre-compute pages for each individual completion.
            let mut by_completion = BTreeMap::new();
            for comp in &result.data.completions {
                if let Some(comp_result) =
                    library::page_activity(library_repo, &item.id, reading_data, Some(comp.index))
                        .await?
                {
                    by_completion.insert(comp.index.to_string(), comp_result.data.pages);
                }
            }

            let export = ExportPageActivityData {
                data: result.data,
                by_completion,
            };
            write_json(
                &page_activity_dir.join(format!("{}.json", item.id)),
                &export,
            )?;
            exported_ids.insert(item.id.clone());
        }
    }

    cleanup_stale_json(&page_activity_dir, &exported_ids, &[])?;

    info!("Exported page activity for {} items", exported_ids.len());
    Ok(())
}

// ── Reading summary export ──────────────────────────────────────────────

fn export_reading_summary(data_dir: &Path, reading_data: &ReadingData) -> Result<()> {
    let summary_dir = data_dir.join("reading").join("summary");
    for scope in SCOPES {
        let data = statistics::summary(
            reading_data,
            ReadingSummaryQuery {
                scope,
                range: None,
                tz: None,
            },
        );
        write_json(&summary_dir.join(format!("{}.json", scope.as_str())), &data)?;
    }
    Ok(())
}

// ── Reading periods export ──────────────────────────────────────────────

fn export_reading_periods(data_dir: &Path, reading_data: &ReadingData) -> Result<()> {
    let periods_dir = data_dir.join("reading").join("periods");
    for scope in SCOPES {
        let query = |source, group_by| ReadingAvailablePeriodsQuery {
            scope,
            source,
            group_by,
            range: None,
            tz: None,
        };

        let data = ExportPeriods {
            reading_data: ExportPeriodsReadingData {
                week: statistics::available_periods(
                    reading_data,
                    query(PeriodSource::ReadingData, PeriodGroupBy::Week),
                ),
                month: statistics::available_periods(
                    reading_data,
                    query(PeriodSource::ReadingData, PeriodGroupBy::Month),
                ),
                year: statistics::available_periods(
                    reading_data,
                    query(PeriodSource::ReadingData, PeriodGroupBy::Year),
                ),
            },
            completions: ExportPeriodsCompletions {
                month: statistics::available_periods(
                    reading_data,
                    query(PeriodSource::Completions, PeriodGroupBy::Month),
                ),
                year: statistics::available_periods(
                    reading_data,
                    query(PeriodSource::Completions, PeriodGroupBy::Year),
                ),
            },
        };
        write_json(&periods_dir.join(format!("{}.json", scope.as_str())), &data)?;
    }
    Ok(())
}

// ── Reading metrics export ──────────────────────────────────────────────

fn export_reading_metrics(data_dir: &Path, reading_data: &ReadingData) -> Result<()> {
    // Determine which months have reading data.
    let month_periods = statistics::available_periods(
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
    let metrics = [
        ReadingMetric::ReadingTimeSec,
        ReadingMetric::PagesRead,
        ReadingMetric::Sessions,
        ReadingMetric::Completions,
        ReadingMetric::ActiveDays,
        ReadingMetric::LongestSessionDurationSec,
        ReadingMetric::AverageSessionDurationSec,
    ];

    // For each scope, get all daily points for all metrics and partition into months.
    let mut exported_month_dirs = HashSet::new();

    for scope in SCOPES {
        let mut months: BTreeMap<String, BTreeMap<String, ExportDayMetrics>> = BTreeMap::new();

        for &metric in &metrics {
            let result = statistics::metrics(
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
                let day_metrics = day_map.entry(point.key.clone()).or_default();

                let value = point.values.get(metric_name).copied().unwrap_or(0);
                match metric {
                    ReadingMetric::ReadingTimeSec => day_metrics.reading_time_sec = value,
                    ReadingMetric::PagesRead => day_metrics.pages_read = value,
                    ReadingMetric::Sessions => day_metrics.sessions = value,
                    ReadingMetric::Completions => day_metrics.completions = value,
                    ReadingMetric::ActiveDays => day_metrics.active_days = value,
                    ReadingMetric::LongestSessionDurationSec => {
                        day_metrics.longest_session_duration_sec = value;
                    }
                    ReadingMetric::AverageSessionDurationSec => {
                        day_metrics.average_session_duration_sec = value;
                    }
                }
            }
        }

        let scope_name = scope.as_str();

        for (month_key, days) in months {
            let file = ExportMonthMetrics {
                month: month_key.clone(),
                days,
            };
            let month_dir = metrics_dir.join(&month_key);
            write_json(&month_dir.join(format!("{scope_name}.json")), &file)?;
            exported_month_dirs.insert(month_key);
        }
    }

    cleanup_stale_dirs(&metrics_dir, &exported_month_dirs)?;

    info!(
        "Exported reading metrics for {} months",
        exported_month_dirs.len()
    );

    Ok(())
}

// ── Reading calendar export ─────────────────────────────────────────────

async fn export_reading_calendar(
    data_dir: &Path,
    reading_data: &ReadingData,
    repo: &LibraryRepository,
) -> Result<()> {
    // Determine which months have reading data.
    let month_periods = statistics::available_periods(
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
    let mut exported_month_dirs = HashSet::new();

    for period in &month_periods.periods {
        for scope in SCOPES {
            let data = statistics::calendar(
                reading_data,
                repo,
                ReadingCalendarQuery {
                    month: period.key.clone(),
                    scope,
                    tz: None,
                },
            )
            .await;
            let month_dir = calendar_dir.join(&period.key);
            write_json(&month_dir.join(format!("{}.json", scope.as_str())), &data)?;
        }
        exported_month_dirs.insert(period.key.clone());
    }

    cleanup_stale_dirs(&calendar_dir, &exported_month_dirs)?;

    info!(
        "Exported reading calendar for {} months",
        exported_month_dirs.len()
    );

    Ok(())
}

// ── Reading completions export ──────────────────────────────────────────

async fn export_reading_completions(
    data_dir: &Path,
    reading_data: &ReadingData,
    repo: &LibraryRepository,
) -> Result<()> {
    // Determine which years have completion data.
    let year_periods = statistics::available_periods(
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
    let mut exported_year_dirs = HashSet::new();

    for period in &year_periods.periods {
        let year: i32 = match period.key.parse() {
            Ok(y) => y,
            Err(_) => continue,
        };

        for scope in SCOPES {
            let data = statistics::completions(
                reading_data,
                repo,
                ReadingCompletionsQuery {
                    scope,
                    selector: CompletionsSelector::Year(year),
                    group_by: CompletionsGroupBy::Month,
                    includes: CompletionsIncludeSet::parse(Some("summary,share_assets"))
                        .expect("known-valid include tokens"),
                    tz: None,
                },
            )
            .await;
            let year_dir = completions_dir.join(&period.key);
            write_json(&year_dir.join(format!("{}.json", scope.as_str())), &data)?;
        }
        exported_year_dirs.insert(period.key.clone());
    }

    cleanup_stale_dirs(&completions_dir, &exported_year_dirs)?;

    info!(
        "Exported reading completions for {} years",
        exported_year_dirs.len()
    );

    Ok(())
}

// ── Item file export ─────────────────────────────────────────────────

/// Copy item files to `output_dir/assets/files/{id}.{ext}` for static hosting.
async fn export_item_files(output_dir: &Path, library_repo: &LibraryRepository) -> Result<()> {
    let files_dir = output_dir.join("assets").join("files");
    fs::create_dir_all(&files_dir)?;
    media::ensure_plain_directory(&files_dir)?;

    let file_infos = library_repo.load_all_item_file_info().await?;
    let mut expected_file_names = HashSet::new();
    let mut copied_count = 0usize;

    for (id, file_path, format) in &file_infos {
        let Some(file_name) = media::item_file_basename(id, format) else {
            log::warn!(
                "Skipping invalid item file export target for id='{}', format='{}'",
                id,
                format
            );
            continue;
        };
        expected_file_names.insert(file_name.clone());

        let source = Path::new(file_path);
        if !source.is_file() {
            log::warn!("Item file missing, skipping export: {:?}", source);
            continue;
        }
        let dest = files_dir.join(file_name);

        if dest.exists() {
            if !dest.is_file() && !dest.is_symlink() {
                log::warn!("Invalid export destination (not a file): {:?}", dest);
                continue;
            }
            if let Err(e) = fs::remove_file(&dest) {
                log::warn!("Failed to reset export destination {:?}: {}", dest, e);
                continue;
            }
        }

        if let Err(e) = fs::copy(source, &dest) {
            log::warn!("Failed to copy item file {:?} → {:?}: {}", source, dest, e);
        } else {
            copied_count += 1;
        }
    }

    // Clean up stale files
    cleanup_stale_item_files(&files_dir, &expected_file_names)?;

    info!("Exported {} item files", copied_count);
    Ok(())
}

/// Remove item files whose basename is not in the given set.
fn cleanup_stale_item_files(files_dir: &Path, valid_file_names: &HashSet<String>) -> Result<()> {
    if !files_dir.exists() {
        return Ok(());
    }

    media::ensure_plain_directory(files_dir)?;

    for entry in fs::read_dir(files_dir)?.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(file_name) = path.file_name().and_then(|s| s.to_str())
            && !valid_file_names.contains(file_name)
        {
            fs::remove_file(&path)?;
        }
    }

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

    media::ensure_plain_directory(directory)?;

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

/// Remove subdirectories from `directory` whose name is not in `valid_names`.
fn cleanup_stale_dirs(directory: &Path, valid_names: &HashSet<String>) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }

    media::ensure_plain_directory(directory)?;

    for entry in fs::read_dir(directory)?.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|s| s.to_str())
            && !valid_names.contains(name)
        {
            fs::remove_dir_all(&path)?;
        }
    }

    Ok(())
}

// ── Export ↔ API sync guard ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::server::api::route_paths;

    /// API routes that have static export implementations in this module.
    const EXPORTED: &[&str] = &[
        "/api/site",
        "/api/items",
        "/api/items/{id}",
        "/api/items/{id}/page-activity",
        "/api/reading/summary",
        "/api/reading/metrics",
        "/api/reading/available-periods",
        "/api/reading/calendar",
        "/api/reading/completions",
    ];

    /// API routes that intentionally have no static export equivalent.
    const NON_EXPORTED: &[&str] = &[
        "/api/events/stream", // SSE — live-only, not exportable
    ];

    /// Verifies every API route is accounted for in either `EXPORTED` or
    /// `NON_EXPORTED`.
    ///
    /// If this test fails you added a new API route without an export.
    /// Either implement its export in this module and add it to `EXPORTED`,
    /// or add it to `NON_EXPORTED` with a comment explaining why.
    #[test]
    fn export_covers_all_api_routes() {
        for route in route_paths() {
            assert!(
                EXPORTED.contains(route) || NON_EXPORTED.contains(route),
                "API route `{route}` has no export coverage. \
                 Implement its export in pipeline/export.rs and add it to EXPORTED, \
                 or add it to NON_EXPORTED if it cannot be statically exported."
            );
        }
    }
}
