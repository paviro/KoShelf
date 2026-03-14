use crate::shelf::models::{
    BookInfo, ContentType, KoReaderMetadata, LibraryItem, LibraryItemFormat,
};
use crate::source::koreader::types::{PageStat, StatBook, StatisticsData};
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) fn library_item(id: &str, metadata: Option<KoReaderMetadata>) -> LibraryItem {
    LibraryItem {
        id: id.to_string(),
        book_info: BookInfo {
            title: "Item".to_string(),
            authors: vec!["Author".to_string()],
            description: None,
            language: None,
            publisher: None,
            identifiers: Vec::new(),
            subjects: Vec::new(),
            series: None,
            series_number: None,
            pages: Some(123),
            cover_data: None,
            cover_mime_type: None,
        },
        koreader_metadata: metadata,
        file_path: PathBuf::from("/tmp/item.epub"),
        format: LibraryItemFormat::Epub,
    }
}

pub(crate) fn koreader_metadata_for_pages(
    md5: &str,
    use_labels: bool,
    synthetic: bool,
    stable_total: u32,
) -> KoReaderMetadata {
    KoReaderMetadata {
        annotations: Vec::new(),
        doc_pages: Some(200),
        doc_path: None,
        doc_props: None,
        pagemap_use_page_labels: Some(use_labels),
        pagemap_chars_per_synthetic_page: synthetic.then_some(1500),
        pagemap_doc_pages: Some(stable_total),
        pagemap_current_page_label: Some("12".to_string()),
        pagemap_last_page_label: Some(stable_total.to_string()),
        partial_md5_checksum: Some(md5.to_string()),
        percent_finished: None,
        stats: None,
        summary: None,
        text_lang: None,
    }
}

pub(crate) fn stat_book(id: i64, md5: &str, pages: i64, content_type: ContentType) -> StatBook {
    StatBook {
        id,
        title: "Stat Book".to_string(),
        authors: "Author".to_string(),
        notes: None,
        last_open: None,
        highlights: None,
        pages: Some(pages),
        md5: md5.to_string(),
        content_type: Some(content_type),
        total_read_time: Some(3600),
        total_read_pages: Some(10),
        completions: None,
    }
}

pub(crate) fn page_stat(id_book: i64, page: i64, start_time: i64, duration: i64) -> PageStat {
    PageStat {
        id_book,
        page,
        start_time,
        duration,
    }
}

pub(crate) fn statistics_data(books: Vec<StatBook>, page_stats: Vec<PageStat>) -> StatisticsData {
    let stats_by_md5 = books
        .iter()
        .cloned()
        .map(|book| (book.md5.clone(), book))
        .collect::<HashMap<_, _>>();

    StatisticsData {
        books,
        page_stats,
        stats_by_md5,
    }
}
