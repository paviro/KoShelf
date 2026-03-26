//! Page-activity domain logic — page-level reading heatmap data for a single item.

use std::collections::HashMap;

use anyhow::Result;
use chrono::NaiveDate;

use crate::server::api::responses::library::{
    PageActivityAnnotation, PageActivityAnnotationKind, PageActivityChapter,
    PageActivityCompletion, PageActivityData, PageActivityEvent, PageActivityPage,
};
use crate::store::memory::ReadingData;
use crate::store::sqlite::repo::LibraryRepository;

/// The full result of building page-activity data.
///
/// `data` is the API response (no raw events).  `events` are the raw page
/// visits, included only so the static export can write them for the
/// in-memory-filtering static client.
pub struct PageActivityResult {
    pub data: PageActivityData,
    pub events: Vec<PageActivityEvent>,
}

/// Build the page-activity payload for a single library item.
///
/// When `completion_filter` is `None`, all events are aggregated.  When
/// `Some(index)`, only events within that completion's date range are included.
///
/// Returns `None` when the item does not exist or has no linked reading data.
pub async fn page_activity(
    repo: &LibraryRepository,
    item_id: &str,
    reading_data: Option<&ReadingData>,
    completion_filter: Option<usize>,
) -> Result<Option<PageActivityResult>> {
    let Some(item) = repo.get_item(item_id).await? else {
        return Ok(None);
    };

    let Some(md5) = item.partial_md5_checksum.as_deref() else {
        return Ok(None);
    };

    let Some(rd) = reading_data else {
        return Ok(None);
    };

    let Some(stat_book) = super::lookup_stat_book(&rd.stats_data, md5) else {
        return Ok(None);
    };

    let total_pages = stat_book.pages.unwrap_or(0);
    let book_id = stat_book.id;

    // Collect raw page events for this book.
    let events: Vec<PageActivityEvent> = rd
        .stats_data
        .page_stats
        .iter()
        .filter(|ps| ps.id_book == book_id && ps.duration > 0)
        .map(|ps| PageActivityEvent {
            page: ps.page,
            start_time: ps.start_time,
            duration: ps.duration,
        })
        .collect();

    // Map completions (needed before aggregation so we can filter by date range).
    let completions: Vec<PageActivityCompletion> = stat_book
        .completions
        .as_ref()
        .map(|bc| {
            bc.entries
                .iter()
                .enumerate()
                .map(|(i, c)| PageActivityCompletion {
                    index: i,
                    start_date: c.start_date.clone(),
                    end_date: c.end_date.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    // Determine the time range to aggregate.
    let time_range = completion_filter.and_then(|idx| {
        let c = completions.get(idx)?;
        let start = NaiveDate::parse_from_str(&c.start_date, "%Y-%m-%d")
            .ok()?
            .and_hms_opt(0, 0, 0)?
            .and_utc()
            .timestamp();
        let end = NaiveDate::parse_from_str(&c.end_date, "%Y-%m-%d")
            .ok()?
            .and_hms_opt(23, 59, 59)?
            .and_utc()
            .timestamp();
        Some((start, end))
    });

    // Aggregate per-page totals, optionally filtered by completion date range.
    let mut page_map: HashMap<i64, (i64, i64)> = HashMap::new();
    for ev in &events {
        if let Some((start, end)) = time_range
            && (ev.start_time < start || ev.start_time > end)
        {
            continue;
        }
        let entry = page_map.entry(ev.page).or_insert((0, 0));
        entry.0 += ev.duration;
        entry.1 += 1;
    }

    let mut pages: Vec<PageActivityPage> = page_map
        .into_iter()
        .map(|(page, (total_duration, read_count))| PageActivityPage {
            page,
            total_duration,
            read_count,
        })
        .collect();
    pages.sort_by_key(|p| p.page);

    // Collect annotations that have a page number.
    let all_annotations = repo.get_annotations(item_id, None).await?;
    let annotations: Vec<PageActivityAnnotation> = all_annotations
        .into_iter()
        .filter_map(|a| {
            let page = i64::from(a.pageno?);
            let kind = if a.note.is_some() {
                PageActivityAnnotationKind::Note
            } else if a.text.is_some() {
                PageActivityAnnotationKind::Highlight
            } else {
                PageActivityAnnotationKind::Bookmark
            };
            Some(PageActivityAnnotation { page, kind })
        })
        .collect();

    // Map chapter fractional positions to page numbers.
    let chapter_entries = repo.get_item_chapters(item_id).await?;
    let chapters: Vec<PageActivityChapter> = chapter_entries
        .into_iter()
        .map(|c| PageActivityChapter {
            title: c.title,
            page: (c.position.clamp(0.0, 1.0) * total_pages as f64).round() as i64,
        })
        .collect();

    Ok(Some(PageActivityResult {
        data: PageActivityData {
            total_pages,
            pages,
            annotations,
            completions,
            chapters,
        },
        events,
    }))
}
