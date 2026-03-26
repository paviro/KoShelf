//! Page-activity domain logic — page-level reading heatmap data for a single item.

use std::collections::HashMap;

use anyhow::Result;

use crate::server::api::responses::library::{
    PageActivityAnnotation, PageActivityAnnotationKind, PageActivityChapter,
    PageActivityCompletion, PageActivityData, PageActivityEvent, PageActivityPage,
};
use crate::store::memory::ReadingData;
use crate::store::sqlite::repo::LibraryRepository;

/// Build the page-activity payload for a single library item.
///
/// Returns `None` when the item does not exist or has no linked reading data.
pub async fn page_activity(
    repo: &LibraryRepository,
    item_id: &str,
    reading_data: Option<&ReadingData>,
) -> Result<Option<PageActivityData>> {
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

    // Aggregate per-page totals.
    let mut page_map: HashMap<i64, (i64, i64)> = HashMap::new();
    for ev in &events {
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

    // Map completions.
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
                    reading_time_sec: c.reading_time,
                    pages_read: c.pages_read,
                })
                .collect()
        })
        .unwrap_or_default();

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

    Ok(Some(PageActivityData {
        total_pages,
        pages,
        annotations,
        completions,
        events,
        chapters,
    }))
}
