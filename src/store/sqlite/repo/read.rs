//! Read operations for the library repository.

use std::collections::HashMap;

use anyhow::{Context, Result};
use log::warn;

use crate::server::api::responses::library::{
    LibraryAnnotation, LibraryDetailItem, LibraryListItem,
};
use crate::shelf::library::queries::LibraryListQuery;
use crate::shelf::models::ChapterEntry;
use crate::shelf::models::ContentType;

use crate::store::sqlite::repo::LibraryRepository;
use crate::store::sqlite::repo::rows::FingerprintRow;

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
            "COALESCE(i.pagemap_doc_pages, i.doc_pages, i.parser_pages)"
        } else {
            "COALESCE(i.doc_pages, i.parser_pages)"
        };

        let sql = format!(
            "SELECT
                i.id, i.title, i.authors_json, i.series_json, i.status,
                i.progress_percentage, i.rating, i.cover_url, i.content_type, i.format,
                i.language, i.publisher, i.description, i.review_note,
                {pages_expr} as pages,
                i.search_base_path, i.subjects_json, i.identifiers_json,
                (f.metadata_path IS NOT NULL) AS has_metadata,
                i.partial_md5_checksum, i.reader_presentation
             FROM library_items i
             LEFT JOIN library_item_fingerprints f ON f.item_id = i.id
             WHERE i.id = ?1"
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
            "SELECT id, chapter, datetime, datetime_updated, pageno, text, note, pos0, pos1, color, drawer
             FROM library_annotations
             WHERE item_id = ?1 AND (?2 IS NULL OR annotation_kind = ?2)
             ORDER BY lua_index ASC",
        )
        .bind(item_id)
        .bind(kind)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get annotations")
    }

    /// Load chapter entries (fractional positions) for an item.
    pub async fn get_item_chapters(&self, item_id: &str) -> Result<Vec<ChapterEntry>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT chapters_json FROM library_items WHERE id = ?1")
                .bind(item_id)
                .fetch_optional(&self.pool)
                .await
                .context("Failed to get item chapters")?;
        match row {
            Some((json,)) => match serde_json::from_str(&json) {
                Ok(chapters) => Ok(chapters),
                Err(e) => {
                    warn!("Failed to parse chapters_json for {}: {}", item_id, e);
                    Ok(Vec::new())
                }
            },
            None => Ok(Vec::new()),
        }
    }

    /// Count annotations grouped by KoShelf type for a given item.
    ///
    /// Returns `(notes, highlights, bookmarks)` where:
    /// - `notes`: highlights that contain a note
    /// - `highlights`: all highlights (including those with notes)
    /// - `bookmarks`: bookmark-type annotations
    pub async fn get_annotation_counts(&self, item_id: &str) -> Result<(i64, i64, i64)> {
        let row: (i64, i64, i64) = sqlx::query_as(
            "SELECT
                COUNT(CASE WHEN annotation_kind = 'highlight' AND note IS NOT NULL THEN 1 END),
                COUNT(CASE WHEN annotation_kind = 'highlight' THEN 1 END),
                COUNT(CASE WHEN annotation_kind = 'bookmark' THEN 1 END)
             FROM library_annotations
             WHERE item_id = ?1",
        )
        .bind(item_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to get annotation counts")?;
        Ok(row)
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

    /// Load stats-relevant fields for a single item.
    ///
    /// Returns `(hidden_flow_pages, pagemap_doc_pages, has_synthetic_pagination)`,
    /// or `None` if the item doesn't exist.  Used by ingest to detect whether
    /// a re-parsed sidecar changed any field that feeds into statistics.
    pub async fn load_stats_influencing_fields(
        &self,
        item_id: &str,
    ) -> Result<Option<(Option<i32>, Option<i32>, bool)>> {
        let row: Option<(Option<i32>, Option<i32>, bool)> = sqlx::query_as(
            "SELECT hidden_flow_pages, pagemap_doc_pages, has_synthetic_pagination
             FROM library_items WHERE id = ?1",
        )
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to load stats fields")?;
        Ok(row)
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

    /// Load all share image fingerprints keyed by year.
    pub async fn load_share_image_fingerprints(&self) -> Result<HashMap<i32, String>> {
        let rows: Vec<(i32, String)> =
            sqlx::query_as("SELECT year, fingerprint FROM share_image_fingerprints")
                .fetch_all(&self.pool)
                .await
                .context("Failed to load share image fingerprints")?;
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

    /// Find the lua_index and note-on-highlight state for an annotation.
    ///
    /// Returns `(lua_index, had_note)` where `had_note` is:
    /// - `Some(true)` — highlight with an existing note
    /// - `Some(false)` — highlight without a note
    /// - `None` — bookmark (no drawer, not tracked in stats)
    ///
    /// Used by both update and delete write handlers:
    /// - Update uses `had_note` to detect highlight↔note type transitions
    ///   (for setting `datetime_updated` to match KOReader).
    /// - Delete uses `had_note.is_some()` to determine whether the annotation
    ///   is a highlight (decrement `highlight_count`) or bookmark.
    pub async fn find_annotation_write_info(
        &self,
        item_id: &str,
        annotation_id: &str,
    ) -> Result<Option<(i32, Option<bool>)>> {
        let row: Option<(i32, Option<bool>)> = sqlx::query_as(
            "SELECT lua_index, (CASE WHEN drawer IS NOT NULL THEN note IS NOT NULL END)
             FROM library_annotations WHERE id = ?1 AND item_id = ?2",
        )
        .bind(annotation_id)
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find annotation write info")?;
        Ok(row)
    }

    /// Delete an annotation row and shift lua_index for remaining annotations.
    pub async fn delete_annotation_and_shift(
        &self,
        item_id: &str,
        annotation_id: &str,
        lua_index: i32,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await.context("begin tx")?;

        sqlx::query("DELETE FROM library_annotations WHERE id = ?1 AND item_id = ?2")
            .bind(annotation_id)
            .bind(item_id)
            .execute(&mut *tx)
            .await
            .context("delete annotation")?;

        sqlx::query(
            "UPDATE library_annotations SET lua_index = lua_index - 1 WHERE item_id = ?1 AND lua_index > ?2",
        )
        .bind(item_id)
        .bind(lua_index)
        .execute(&mut *tx)
        .await
        .context("shift lua_index after deletion")?;

        tx.commit().await.context("commit annotation deletion")?;
        Ok(())
    }

    /// Find the metadata sidecar path and fingerprint for an item.
    ///
    /// Returns `(metadata_path, metadata_size_bytes, metadata_modified_unix_ms)`.
    /// Returns `None` if the item has no metadata sidecar or fingerprint data.
    pub async fn find_metadata_fingerprint_by_item_id(
        &self,
        item_id: &str,
    ) -> Result<Option<(String, i64, i64)>> {
        let row: Option<(Option<String>, Option<i64>, Option<i64>)> = sqlx::query_as(
            "SELECT metadata_path, metadata_size_bytes, metadata_modified_unix_ms
             FROM library_item_fingerprints WHERE item_id = ?1",
        )
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find metadata fingerprint by item id")?;

        Ok(row.and_then(|(path, size, modified)| Some((path?, size?, modified?))))
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
    use crate::server::api::responses::common::ContentTypeFilter;
    use crate::shelf::library::queries::{ItemSort, LibraryListQuery};
    use crate::store::sqlite::repo::tests::{sample_annotation, sample_item, test_repo};

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
