//! Library domain service — list/detail queries backed by `library.sqlite`.

use anyhow::Result;

use super::queries::{IncludeToken, LibraryDetailQuery, LibraryListQuery};
use crate::server::api::responses::library::{
    LibraryCompletionEntry, LibraryCompletions, LibraryDetailData, LibraryDetailStatistics,
    LibraryItemStats, LibraryListData, LibrarySessionStats,
};
use crate::shelf::statistics::BookStatistics;
use crate::shelf::time_config::TimeConfig;
use crate::source::koreader::types::{BookSessionStats, StatBook};
use crate::store::memory::ReadingData;
use crate::store::sqlite::repo::LibraryRepository;

/// Fetch a filtered, sorted list of library items.
pub async fn list(repo: &LibraryRepository, query: LibraryListQuery) -> Result<LibraryListData> {
    let items = repo.list_items(&query).await?;
    Ok(LibraryListData { items })
}

/// Fetch a single library item with optional includes (highlights, bookmarks,
/// statistics, completions). Returns `None` if the item ID is not found.
pub async fn detail(
    repo: &LibraryRepository,
    query: &LibraryDetailQuery,
    reading_data: Option<&ReadingData>,
) -> Result<Option<LibraryDetailData>> {
    let Some(mut item) = repo.get_item(&query.id).await? else {
        return Ok(None);
    };

    let includes = &query.includes;

    let reader_presentation = if includes.has(IncludeToken::ReaderPresentation) {
        item.reader_presentation.take()
    } else {
        None
    };

    let highlights = if includes.has(IncludeToken::Highlights) {
        Some(repo.get_annotations(&query.id, Some("highlight")).await?)
    } else {
        None
    };

    let bookmarks = if includes.has(IncludeToken::Bookmarks) {
        Some(repo.get_annotations(&query.id, Some("bookmark")).await?)
    } else {
        None
    };

    // Resolve per-item statistics and completions via partial_md5_checksum
    // linkage into the in-memory reading data.
    let stat_book =
        if includes.has(IncludeToken::Statistics) || includes.has(IncludeToken::Completions) {
            reading_data
                .zip(item.partial_md5_checksum.as_deref())
                .and_then(|(rd, md5)| lookup_stat_book(&rd.stats_data, md5))
        } else {
            None
        };

    let statistics = if includes.has(IncludeToken::Statistics) {
        let annotation_counts = repo.get_annotation_counts(&query.id).await?;
        let session_and_reading = stat_book.as_ref().and_then(|sb| {
            let rd = reading_data?;
            let session_stats =
                sb.calculate_session_stats(&rd.stats_data.page_stats, &rd.time_config);
            Some(map_reading_stats(sb, &session_stats, &rd.time_config))
        });
        Some(map_detail_statistics(annotation_counts, session_and_reading))
    } else {
        None
    };

    let completions = if includes.has(IncludeToken::Completions) {
        stat_book
            .as_ref()
            .and_then(|sb| sb.completions.as_ref())
            .map(map_completions)
    } else {
        None
    };

    Ok(Some(LibraryDetailData {
        item,
        highlights,
        bookmarks,
        statistics,
        completions,
        reader_presentation,
    }))
}

/// Case-insensitive lookup into `stats_by_md5`.
fn lookup_stat_book<'a>(
    stats_data: &'a crate::source::koreader::types::StatisticsData,
    md5: &str,
) -> Option<&'a StatBook> {
    stats_data
        .stats_by_md5
        .get(md5)
        .or_else(|| stats_data.stats_by_md5.get(&md5.to_lowercase()))
        .or_else(|| stats_data.stats_by_md5.get(&md5.to_uppercase()))
}

// ── Statistics mapping ──────────────────────────────────────────────────

/// Reading-level stats extracted from KOReader's statistics database.
struct ReadingStats {
    last_open_at: Option<String>,
    pages: Option<i64>,
    total_reading_time_sec: Option<i64>,
    session_stats: LibrarySessionStats,
}

fn map_reading_stats(
    stat_book: &StatBook,
    session_stats: &BookSessionStats,
    time_config: &TimeConfig,
) -> ReadingStats {
    ReadingStats {
        last_open_at: stat_book
            .last_open
            .map(|ts| time_config.format_timestamp_rfc3339(ts)),
        pages: stat_book.pages,
        total_reading_time_sec: stat_book.total_read_time,
        session_stats: LibrarySessionStats {
            session_count: session_stats.session_count,
            average_session_duration_sec: session_stats.average_session_duration,
            longest_session_duration_sec: session_stats.longest_session_duration,
            last_read_date: session_stats.last_read_date.clone(),
            reading_speed: session_stats.reading_speed,
        },
    }
}

fn map_detail_statistics(
    annotation_counts: (i64, i64, i64),
    reading: Option<ReadingStats>,
) -> LibraryDetailStatistics {
    let (notes, highlights, bookmarks) = annotation_counts;
    LibraryDetailStatistics {
        item_stats: Some(LibraryItemStats {
            notes: Some(notes),
            highlights: Some(highlights),
            bookmarks: Some(bookmarks),
            last_open_at: reading.as_ref().and_then(|r| r.last_open_at.clone()),
            pages: reading.as_ref().and_then(|r| r.pages),
            total_reading_time_sec: reading.as_ref().and_then(|r| r.total_reading_time_sec),
        }),
        session_stats: reading.map(|r| r.session_stats),
    }
}

fn map_completions(
    completions: &crate::source::koreader::types::BookCompletions,
) -> LibraryCompletions {
    LibraryCompletions {
        entries: completions
            .entries
            .iter()
            .map(|c| LibraryCompletionEntry {
                start_date: c.start_date.clone(),
                end_date: c.end_date.clone(),
                reading_time_sec: c.reading_time,
                session_count: c.session_count,
                pages_read: c.pages_read,
            })
            .collect(),
        total_completions: completions.total_completions,
        last_completion_date: completions.last_completion_date.clone(),
    }
}
