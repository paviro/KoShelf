//! Library domain service — list/detail queries backed by `library.sqlite`.

use anyhow::Result;

use crate::contracts::common::{ApiMeta, ContentTypeFilter};
use crate::contracts::library::{
    LibraryDetailResponse, LibraryDetailStatistics, LibraryListResponse,
};
use crate::domain::library::projections::{
    annotation_row_to_contract, row_to_detail_item, row_to_list_item,
};
use crate::domain::library::queries::{ItemSort, LibraryDetailQuery, LibraryListQuery, SortOrder};
use crate::infra::sqlite::library_repo::LibraryRepository;
use crate::infra::sqlite::library_repo::queries::{LibraryListFilter, LibrarySort, SortDirection};

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
    ) -> Result<Option<LibraryDetailResponse>> {
        let row = repo.get_item(&query.id).await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let item = row_to_detail_item(&row);

        let annotation_rows = repo.get_annotations(&query.id, None).await?;
        let highlights = annotation_rows
            .iter()
            .filter(|a| a.annotation_kind == "highlight")
            .map(annotation_row_to_contract)
            .collect();
        let bookmarks = annotation_rows
            .iter()
            .filter(|a| a.annotation_kind == "bookmark")
            .map(annotation_row_to_contract)
            .collect();

        Ok(Some(LibraryDetailResponse {
            meta,
            item,
            highlights,
            bookmarks,
            statistics: LibraryDetailStatistics {
                item_stats: None,
                session_stats: None,
                completions: None,
            },
        }))
    }
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
