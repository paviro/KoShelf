//! Library-domain boundaries for list/detail queries and item persistence.

pub mod build;
pub mod item_mapping;
pub mod page_activity;
pub mod queries;
pub mod service;

pub use build::upsert_single_item;
pub use page_activity::page_activity;
pub use queries::{LibraryDetailQuery, LibraryListQuery};
pub use service::{detail, list};

/// Case-insensitive lookup into `stats_by_md5`.
pub(crate) fn lookup_stat_book<'a>(
    stats_data: &'a crate::source::koreader::types::StatisticsData,
    md5: &str,
) -> Option<&'a crate::source::koreader::types::StatBook> {
    stats_data
        .stats_by_md5
        .get(md5)
        .or_else(|| stats_data.stats_by_md5.get(&md5.to_lowercase()))
        .or_else(|| stats_data.stats_by_md5.get(&md5.to_uppercase()))
}
