use crate::contracts::common::{ApiMeta, ContentTypeFilter};
use crate::contracts::library::{
    LibraryContentType, LibraryDetailItem, LibraryDetailResponse, LibraryDetailStatistics,
    LibraryListResponse, LibraryStatus,
};
use crate::domain::library::{LibraryDetailQuery, LibraryListQuery};
use crate::domain::meta::fallback_meta;
use crate::runtime::ContractSnapshot;

#[derive(Debug, Default, Clone, Copy)]
pub struct LibraryService;

impl LibraryService {
    pub fn list(snapshot: &ContractSnapshot, query: LibraryListQuery) -> LibraryListResponse {
        let base_payload = snapshot
            .items
            .as_ref()
            .cloned()
            .unwrap_or_else(|| Self::empty_library_list_response(fallback_meta(snapshot)));

        Self::filter_library_items(&base_payload, query.scope)
    }

    pub fn detail(
        snapshot: &ContractSnapshot,
        query: &LibraryDetailQuery,
    ) -> LibraryDetailResponse {
        snapshot
            .item_details
            .get(&query.id)
            .cloned()
            .unwrap_or_else(|| {
                Self::empty_library_detail_response(fallback_meta(snapshot), query.id.clone())
            })
    }

    fn empty_library_list_response(meta: ApiMeta) -> LibraryListResponse {
        LibraryListResponse {
            meta,
            items: vec![],
        }
    }

    fn empty_library_detail_response(meta: ApiMeta, id: String) -> LibraryDetailResponse {
        LibraryDetailResponse {
            meta,
            item: LibraryDetailItem {
                id,
                title: String::new(),
                authors: vec![],
                series: None,
                status: LibraryStatus::Unknown,
                progress_percentage: None,
                rating: None,
                cover_url: String::new(),
                content_type: LibraryContentType::Book,
                language: None,
                publisher: None,
                description: None,
                review_note: None,
                pages: None,
                search_base_path: String::new(),
                subjects: vec![],
                identifiers: vec![],
            },
            highlights: vec![],
            bookmarks: vec![],
            statistics: LibraryDetailStatistics {
                item_stats: None,
                session_stats: None,
                completions: None,
            },
        }
    }

    fn filter_library_items(
        response: &LibraryListResponse,
        content_type: ContentTypeFilter,
    ) -> LibraryListResponse {
        if content_type == ContentTypeFilter::All {
            return response.clone();
        }

        LibraryListResponse {
            meta: response.meta.clone(),
            items: response
                .items
                .iter()
                .filter(|item| Self::item_matches_content_type(content_type, item.content_type))
                .cloned()
                .collect(),
        }
    }

    fn item_matches_content_type(
        content_type: ContentTypeFilter,
        item_content_type: LibraryContentType,
    ) -> bool {
        match content_type {
            ContentTypeFilter::All => true,
            ContentTypeFilter::Books => item_content_type == LibraryContentType::Book,
            ContentTypeFilter::Comics => item_content_type == LibraryContentType::Comic,
        }
    }
}
