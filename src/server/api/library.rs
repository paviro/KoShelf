use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::contracts::common::{ApiMeta, ContentTypeFilter};
use crate::contracts::library::{
    LibraryContentType, LibraryDetailItem, LibraryDetailResponse, LibraryDetailStatistics,
    LibraryListResponse, LibraryStatus,
};
use crate::server::ServerState;

use super::shared::{ContentTypeQuery, fallback_meta, parse_content_type, runtime_snapshot};

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
            .filter(|item| item_matches_content_type(content_type, item.content_type))
            .cloned()
            .collect(),
    }
}

pub async fn items(
    State(state): State<ServerState>,
    Query(query): Query<ContentTypeQuery>,
) -> Response {
    let content_type = match parse_content_type(query) {
        Ok(content_type) => content_type,
        Err(error) => return error.into_response(),
    };

    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let base_payload = snapshot
        .items
        .as_ref()
        .cloned()
        .unwrap_or_else(|| empty_library_list_response(fallback_meta(&snapshot)));
    let payload = filter_library_items(&base_payload, content_type);

    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn item_detail(State(state): State<ServerState>, Path(id): Path<String>) -> Response {
    let snapshot = match runtime_snapshot(&state) {
        Ok(snapshot) => snapshot,
        Err(error) => return error.into_response(),
    };

    let payload = snapshot
        .item_details
        .get(&id)
        .cloned()
        .unwrap_or_else(|| empty_library_detail_response(fallback_meta(&snapshot), id.clone()));

    (StatusCode::OK, Json(payload)).into_response()
}
