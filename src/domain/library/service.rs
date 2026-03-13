//! Library domain service — list/detail queries backed by `library.sqlite`.

use anyhow::Result;

use crate::contracts::common::{ApiMeta, ContentTypeFilter};
use crate::contracts::library::{LibraryDetailResponse, LibraryListResponse};
use crate::domain::library::projections::{
    annotation_row_to_contract, book_completions_to_contract, row_to_detail_item, row_to_list_item,
    stat_book_to_detail_statistics,
};
use crate::domain::library::queries::{
    IncludeToken, ItemSort, LibraryDetailQuery, LibraryListQuery, SortOrder,
};
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::infra::sqlite::library_repo::queries::{LibraryListFilter, LibrarySort, SortDirection};
use crate::koreader::BookStatistics;
use crate::models::ReadingData;

#[derive(Debug, Default, Clone, Copy)]
pub struct LibraryService;

impl LibraryService {
    pub async fn list(
        repo: &LibraryRepository,
        query: LibraryListQuery,
        meta: ApiMeta,
    ) -> Result<LibraryListResponse> {
        let filter = to_list_filter(query);
        let rows = repo.list_items(&filter).await?;
        let items = rows.iter().map(row_to_list_item).collect();

        Ok(LibraryListResponse { meta, items })
    }

    pub async fn detail(
        repo: &LibraryRepository,
        query: &LibraryDetailQuery,
        meta: ApiMeta,
        reading_data: Option<&ReadingData>,
    ) -> Result<Option<LibraryDetailResponse>> {
        let row = repo.get_item(&query.id).await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let item = row_to_detail_item(&row);
        let includes = &query.includes;

        // Fetch annotations only when highlights or bookmarks are requested.
        let (highlights, bookmarks) =
            if includes.has(IncludeToken::Highlights) || includes.has(IncludeToken::Bookmarks) {
                let annotation_rows = repo.get_annotations(&query.id, None).await?;

                let hl = if includes.has(IncludeToken::Highlights) {
                    Some(
                        annotation_rows
                            .iter()
                            .filter(|a| a.annotation_kind == "highlight")
                            .map(annotation_row_to_contract)
                            .collect(),
                    )
                } else {
                    None
                };

                let bm = if includes.has(IncludeToken::Bookmarks) {
                    Some(
                        annotation_rows
                            .iter()
                            .filter(|a| a.annotation_kind == "bookmark")
                            .map(annotation_row_to_contract)
                            .collect(),
                    )
                } else {
                    None
                };

                (hl, bm)
            } else {
                (None, None)
            };

        // Resolve per-item statistics and completions via partial_md5_checksum
        // linkage into the in-memory reading data.
        let stat_book =
            if includes.has(IncludeToken::Statistics) || includes.has(IncludeToken::Completions) {
                reading_data
                    .zip(row.partial_md5_checksum.as_deref())
                    .and_then(|(rd, md5)| lookup_stat_book(&rd.stats_data, md5))
            } else {
                None
            };

        let statistics = if includes.has(IncludeToken::Statistics) {
            stat_book.as_ref().and_then(|sb| {
                let rd = reading_data?;
                let session_stats =
                    sb.calculate_session_stats(&rd.stats_data.page_stats, &rd.time_config);
                Some(stat_book_to_detail_statistics(
                    sb,
                    &session_stats,
                    &rd.time_config,
                ))
            })
        } else {
            None
        };

        let completions = if includes.has(IncludeToken::Completions) {
            stat_book
                .as_ref()
                .and_then(|sb| sb.completions.as_ref())
                .map(book_completions_to_contract)
        } else {
            None
        };

        Ok(Some(LibraryDetailResponse {
            meta,
            item,
            highlights,
            bookmarks,
            statistics,
            completions,
        }))
    }
}

/// Case-insensitive lookup into `stats_by_md5`.
fn lookup_stat_book<'a>(
    stats_data: &'a crate::models::StatisticsData,
    md5: &str,
) -> Option<&'a crate::models::StatBook> {
    stats_data
        .stats_by_md5
        .get(md5)
        .or_else(|| stats_data.stats_by_md5.get(&md5.to_lowercase()))
        .or_else(|| stats_data.stats_by_md5.get(&md5.to_uppercase()))
}

fn to_list_filter(query: LibraryListQuery) -> LibraryListFilter {
    let content_type = match query.scope {
        ContentTypeFilter::All => None,
        ContentTypeFilter::Books => Some("book".to_string()),
        ContentTypeFilter::Comics => Some("comic".to_string()),
    };

    let sort = match query.sort {
        ItemSort::Title => LibrarySort::Title,
        ItemSort::Author => LibrarySort::Author,
        ItemSort::Status => LibrarySort::Status,
        ItemSort::Progress => LibrarySort::Progress,
        ItemSort::Rating => LibrarySort::Rating,
        ItemSort::Annotations => LibrarySort::Annotations,
        ItemSort::LastOpenAt => LibrarySort::LastOpenAt,
    };

    let sort_direction = query.order.map(|o| match o {
        SortOrder::Asc => SortDirection::Asc,
        SortOrder::Desc => SortDirection::Desc,
    });

    LibraryListFilter {
        content_type,
        sort,
        sort_direction,
    }
}
