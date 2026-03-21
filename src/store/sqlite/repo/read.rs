//! Read operations for the library repository.

use std::collections::HashMap;

use anyhow::{Context, Result};

use crate::server::api::responses::library::{
    LibraryAnnotation, LibraryDetailItem, LibraryListItem,
};
use crate::shelf::library::queries::LibraryListQuery;
use crate::shelf::models::ContentType;

use super::LibraryRepository;
use super::rows::FingerprintRow;

impl LibraryRepository {
    /// List items matching the given query, sorted with a deterministic
    /// `id` tiebreaker.
    pub async fn list_items(&self, query: &LibraryListQuery) -> Result<Vec<LibraryListItem>> {
        let direction = query.order.unwrap_or_else(|| query.sort.default_order());

        let content_type = query.scope.sql_value();

        let sql = format!(
            "SELECT
                id, title, authors_json, series_json, status,
                progress_percentage, rating, annotation_count,
                cover_url, content_type
             FROM library_items
             WHERE (?1 IS NULL OR content_type = ?1)
             ORDER BY {} {} NULLS LAST, id ASC",
            query.sort.sql_column(),
            direction.sql_keyword(),
        );

        sqlx::query_as::<_, LibraryListItem>(&sql)
            .bind(content_type)
            .fetch_all(&self.pool)
            .await
            .context("Failed to list library items")
    }

    pub async fn get_item(&self, id: &str) -> Result<Option<LibraryDetailItem>> {
        let pages_expr = if self.use_stable_page_metadata {
            "COALESCE(pagemap_doc_pages, doc_pages, parser_pages)"
        } else {
            "COALESCE(doc_pages, parser_pages)"
        };

        let sql = format!(
            "SELECT
                id, title, authors_json, series_json, status,
                progress_percentage, rating, cover_url, content_type, format,
                language, publisher, description, review_note,
                {pages_expr} as pages,
                search_base_path, subjects_json, identifiers_json,
                partial_md5_checksum, reader_presentation
             FROM library_items WHERE id = ?1"
        );

        sqlx::query_as::<_, LibraryDetailItem>(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to get library item")
    }

    /// Load annotations for an item, optionally filtered by kind.
    ///
    /// `kind` should be `"highlight"` or `"bookmark"`, or `None` for both.
    pub async fn get_annotations(
        &self,
        item_id: &str,
        kind: Option<&str>,
    ) -> Result<Vec<LibraryAnnotation>> {
        sqlx::query_as::<_, LibraryAnnotation>(
            "SELECT chapter, datetime, pageno, text, note, pos0, pos1, color, drawer
             FROM library_annotations
             WHERE item_id = ?1 AND (?2 IS NULL OR annotation_kind = ?2)
             ORDER BY ordinal ASC",
        )
        .bind(item_id)
        .bind(kind)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get annotations")
    }

    /// Load all stored fingerprints (for reconciliation during incremental builds).
    pub async fn load_all_fingerprints(&self) -> Result<Vec<FingerprintRow>> {
        sqlx::query_as::<_, FingerprintRow>(
            "SELECT item_id, book_path, book_size_bytes, book_modified_unix_ms,
                    metadata_path, metadata_size_bytes, metadata_modified_unix_ms, updated_at
             FROM library_item_fingerprints",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to load fingerprints")
    }

    pub async fn item_exists(&self, id: &str) -> Result<bool> {
        let row = sqlx::query("SELECT 1 FROM library_items WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to check item existence")?;
        Ok(row.is_some())
    }

    /// Query whether the library contains books and/or comics.
    pub async fn query_content_type_flags(&self) -> Result<(bool, bool)> {
        let row: (i32, i32) = sqlx::query_as(
            "SELECT
                COALESCE(MAX(content_type = 'book'), 0),
                COALESCE(MAX(content_type = 'comic'), 0)
             FROM library_items",
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to query content type flags")?;

        Ok((row.0 != 0, row.1 != 0))
    }

    /// Find the fingerprint row for a given book file path.
    pub async fn find_fingerprint_by_book_path(
        &self,
        book_path: &str,
    ) -> Result<Option<FingerprintRow>> {
        sqlx::query_as::<_, FingerprintRow>(
            "SELECT item_id, book_path, book_size_bytes, book_modified_unix_ms,
                    metadata_path, metadata_size_bytes, metadata_modified_unix_ms, updated_at
             FROM library_item_fingerprints
             WHERE book_path = ?1",
        )
        .bind(book_path)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find fingerprint by book path")
    }

    /// Find the book_path whose metadata_path matches the given path.
    pub async fn find_book_path_by_metadata_path(
        &self,
        metadata_path: &str,
    ) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT book_path FROM library_item_fingerprints WHERE metadata_path = ?1",
        )
        .bind(metadata_path)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find book path by metadata path")?;

        Ok(row.map(|r| r.0))
    }

    /// Load all item IDs (canonical MD5s, used as library_md5s for stats filtering).
    pub async fn load_all_item_ids(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT id FROM library_items")
            .fetch_all(&self.pool)
            .await
            .context("Failed to load all item IDs")?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Load page scaling inputs keyed by item ID (MD5).
    ///
    /// Returns `(pagemap_doc_pages, doc_pages)` pairs. `doc_pages` is the rendered
    /// page count used as fallback denominator when `stat_book.pages` is unavailable.
    pub async fn load_scaling_inputs(&self) -> Result<HashMap<String, (i32, Option<i32>)>> {
        let rows: Vec<(String, i32, Option<i32>)> = sqlx::query_as(
            "SELECT id, pagemap_doc_pages, doc_pages
             FROM library_items
             WHERE pagemap_doc_pages IS NOT NULL AND has_synthetic_pagination = 1",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to load page scaling inputs")?;
        Ok(rows
            .into_iter()
            .map(|(id, pagemap, doc)| (id, (pagemap, doc)))
            .collect())
    }

    /// Load hidden flow page counts keyed by item ID (MD5).
    ///
    /// Only returns entries where `hidden_flow_pages` is set (i.e., the user has
    /// enabled handmade flows in KOReader with hidden sections).
    pub async fn load_hidden_flow_pages(&self) -> Result<HashMap<String, i32>> {
        let rows: Vec<(String, i32)> = sqlx::query_as(
            "SELECT id, hidden_flow_pages
             FROM library_items
             WHERE hidden_flow_pages IS NOT NULL",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to load hidden flow pages")?;
        Ok(rows.into_iter().collect())
    }

    /// Load `(id, file_path, format)` for every library item.
    ///
    /// Used by the static export pipeline to copy item files into the output.
    pub async fn load_all_item_file_info(&self) -> Result<Vec<(String, String, String)>> {
        let rows: Vec<(String, String, String)> =
            sqlx::query_as("SELECT id, file_path, format FROM library_items")
                .fetch_all(&self.pool)
                .await
                .context("Failed to load item file info")?;
        Ok(rows)
    }

    pub async fn count_items(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM library_items")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count items")?;
        Ok(row.0)
    }

    /// Find the file_path for an item by its canonical ID.
    ///
    /// Used by ingest for duplicate detection (same MD5, different path).
    pub async fn find_book_path_by_id(&self, item_id: &str) -> Result<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT file_path FROM library_items WHERE id = ?1")
                .bind(item_id)
                .fetch_optional(&self.pool)
                .await
                .context("Failed to find book path by id")?;

        Ok(row.map(|r| r.0))
    }

    /// Load a mapping of item ID → content type for all items.
    ///
    /// Used by statistics loading to tag stats entries by content type
    /// without needing in-memory items.
    pub async fn load_content_types_by_id(&self) -> Result<HashMap<String, ContentType>> {
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT id, content_type FROM library_items")
                .fetch_all(&self.pool)
                .await
                .context("Failed to load content types")?;

        Ok(rows
            .into_iter()
            .filter_map(|(id, ct)| {
                let content_type = match ct.as_str() {
                    "book" => ContentType::Book,
                    "comic" => ContentType::Comic,
                    _ => return None,
                };
                Some((id, content_type))
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{sample_annotation, sample_item, test_repo};
    use crate::server::api::responses::common::ContentTypeFilter;
    use crate::shelf::library::queries::{ItemSort, LibraryListQuery};

    #[tokio::test]
    async fn get_item_returns_none_for_missing_id() {
        let repo = test_repo().await;
        let result = repo.get_item("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_items_filters_by_content_type() {
        let repo = test_repo().await;

        let mut book = sample_item("book1");
        book.content_type = "book".to_string();
        let mut comic = sample_item("comic1");
        comic.content_type = "comic".to_string();
        comic.format = "cbz".to_string();
        comic.file_path = "/comics/comic1.cbz".to_string();

        repo.upsert_item(&book).await.unwrap();
        repo.upsert_item(&comic).await.unwrap();

        let all = repo.list_items(&LibraryListQuery::default()).await.unwrap();
        assert_eq!(all.len(), 2);

        let books = repo
            .list_items(&LibraryListQuery {
                scope: ContentTypeFilter::Books,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].id, "book1");

        let comics = repo
            .list_items(&LibraryListQuery {
                scope: ContentTypeFilter::Comics,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(comics.len(), 1);
        assert_eq!(comics[0].id, "comic1");
    }

    #[tokio::test]
    async fn list_items_sorts_by_title_asc_by_default() {
        let repo = test_repo().await;

        let mut b = sample_item("id-z");
        b.title = "Zebra".to_string();
        b.file_path = "/books/z.epub".to_string();
        repo.upsert_item(&b).await.unwrap();

        let mut a = sample_item("id-a");
        a.title = "Alpha".to_string();
        a.file_path = "/books/a.epub".to_string();
        repo.upsert_item(&a).await.unwrap();

        let items = repo.list_items(&LibraryListQuery::default()).await.unwrap();
        assert_eq!(items[0].title, "Alpha");
        assert_eq!(items[1].title, "Zebra");
    }

    #[tokio::test]
    async fn list_items_sorts_by_rating_desc() {
        let repo = test_repo().await;

        let mut low = sample_item("low");
        low.rating = Some(1);
        low.file_path = "/books/low.epub".to_string();
        repo.upsert_item(&low).await.unwrap();

        let mut high = sample_item("high");
        high.rating = Some(5);
        high.file_path = "/books/high.epub".to_string();
        repo.upsert_item(&high).await.unwrap();

        let items = repo
            .list_items(&LibraryListQuery {
                sort: ItemSort::Rating,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(items[0].rating, Some(5));
        assert_eq!(items[1].rating, Some(1));
    }

    #[tokio::test]
    async fn get_annotations_filters_by_kind() {
        let repo = test_repo().await;
        repo.upsert_item(&sample_item("eee")).await.unwrap();
        repo.replace_annotations(
            "eee",
            &[
                sample_annotation("eee", "highlight", 0),
                sample_annotation("eee", "bookmark", 1),
            ],
        )
        .await
        .unwrap();

        let highlights = repo
            .get_annotations("eee", Some("highlight"))
            .await
            .unwrap();
        assert_eq!(highlights.len(), 1);

        let bookmarks = repo.get_annotations("eee", Some("bookmark")).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
    }

    #[tokio::test]
    async fn count_and_exists() {
        let repo = test_repo().await;
        assert_eq!(repo.count_items().await.unwrap(), 0);
        assert!(!repo.item_exists("hhh").await.unwrap());

        repo.upsert_item(&sample_item("hhh")).await.unwrap();

        assert_eq!(repo.count_items().await.unwrap(), 1);
        assert!(repo.item_exists("hhh").await.unwrap());
    }
}
