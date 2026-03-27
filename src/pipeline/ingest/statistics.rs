//! Statistics loading: parse, filter, tag, and package reading data.
//!
//! Uses DB queries instead of in-memory item collections for content-type
//! tagging and library-item filtering.

use crate::app::config::SiteConfig;
use crate::shelf::statistics::{PageScaling, StatisticsCalculator};
use crate::source::koreader::StatisticsParser;
use crate::store::memory::ReadingData;
use crate::store::sqlite::repo::LibraryRepository;
use anyhow::Result;
use log::info;
use std::collections::HashSet;

/// Load and process reading statistics using DB queries for filtering and tagging.
///
/// Called after library ingest at startup, and during rebuild when stats DB changes.
pub async fn load_reading_data(
    config: &SiteConfig,
    repo: &LibraryRepository,
) -> Result<Option<ReadingData>> {
    let stats_path = match config.statistics_db_path.as_ref() {
        Some(p) if p.exists() => p,
        Some(p) => {
            info!("Statistics database not found: {:?}", p);
            return Ok(None);
        }
        None => return Ok(None),
    };

    let mut data = StatisticsParser::parse(stats_path).await?;
    let total_books = data.books.len();

    if config.min_pages_per_day.is_some() || config.min_time_per_day.is_some() {
        StatisticsCalculator::filter_stats(
            &mut data,
            &config.time_config,
            config.min_pages_per_day,
            config.min_time_per_day,
        );
    }

    if !config.include_all_stats {
        let item_ids = repo.load_all_item_ids().await?;
        if !item_ids.is_empty() {
            let md5s: HashSet<String> = item_ids.into_iter().collect();
            StatisticsCalculator::filter_to_library(&mut data, &md5s);
        }
    }

    let hidden_flow_pages = repo.load_hidden_flow_pages().await?;
    data.apply_hidden_flow_adjustments(&hidden_flow_pages);

    StatisticsCalculator::populate_completions(&mut data, &config.time_config);

    let content_type_map = repo.load_content_types_by_id().await?;
    data.tag_content_types(&content_type_map);

    let page_scaling = if config.use_stable_page_metadata {
        let scaling_inputs = repo.load_scaling_inputs().await?;
        PageScaling::from_db_inputs(&scaling_inputs, &data)
    } else {
        PageScaling::disabled()
    };

    let ignored = total_books - data.books.len();
    info!(
        "Statistics: {} books ({} ignored), {} with completions",
        data.books.len(),
        ignored,
        data.books
            .iter()
            .filter(|b| b.completions.is_some())
            .count()
    );

    Ok(Some(ReadingData {
        stats_data: data,
        time_config: config.time_config.clone(),
        heatmap_scale_max: config.heatmap_scale_max,
        page_scaling,
    }))
}
