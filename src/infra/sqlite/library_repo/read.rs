//! Read operations for the library repository.

use anyhow::{Context, Result};
use sqlx::Row;

use super::LibraryRepository;
use super::queries::LibraryListFilter;
use super::rows::{AnnotationRow, FingerprintRow, LibraryItemRow};

impl LibraryRepository {
    /// List items matching the given filter, sorted with a deterministic
    /// `id` tiebreaker.
    pub async fn list_items(&self, filter: &LibraryListFilter) -> Result<Vec<LibraryItemRow>> {
        let direction = filter
            .sort_direction
            .unwrap_or_else(|| filter.sort.default_direction());

        let sql = format!(
            "SELECT
                id, file_path, format, content_type, title, title_sort,
                primary_author_sort, authors_json, series_name, series_index,
                description, language, publisher, subjects_json, identifiers_json,
                status, progress_percentage, rating, review_note, pages,
                cover_url, search_base_path, annotation_count, bookmark_count,
                highlight_count, partial_md5_checksum, last_open_at,
                total_reading_time_sec, created_at, updated_at
             FROM library_items
             WHERE (?1 IS NULL OR content_type = ?1)
             ORDER BY {} {} NULLS LAST, id ASC",
            filter.sort.sql_column(),
            direction.sql_keyword(),
        );

        let rows = sqlx::query(&sql)
            .bind(filter.content_type.as_deref())
            .fetch_all(&self.pool)
            .await
            .context("Failed to list library items")?;

        Ok(rows.into_iter().map(|r| row_to_item(&r)).collect())
    }

    pub async fn get_item(&self, id: &str) -> Result<Option<LibraryItemRow>> {
        let row = sqlx::query(
            "SELECT
                id, file_path, format, content_type, title, title_sort,
                primary_author_sort, authors_json, series_name, series_index,
                description, language, publisher, subjects_json, identifiers_json,
                status, progress_percentage, rating, review_note, pages,
                cover_url, search_base_path, annotation_count, bookmark_count,
                highlight_count, partial_md5_checksum, last_open_at,
                total_reading_time_sec, created_at, updated_at
             FROM library_items WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get library item")?;

        Ok(row.as_ref().map(row_to_item))
    }

    /// Load annotations for an item, optionally filtered by kind.
    ///
    /// `kind` should be `"highlight"` or `"bookmark"`, or `None` for both.
    pub async fn get_annotations(
        &self,
        item_id: &str,
        kind: Option<&str>,
    ) -> Result<Vec<AnnotationRow>> {
        let rows = sqlx::query(
            "SELECT item_id, annotation_kind, ordinal, chapter, datetime, pageno, text, note
             FROM library_annotations
             WHERE item_id = ?1 AND (?2 IS NULL OR annotation_kind = ?2)
             ORDER BY ordinal ASC",
        )
        .bind(item_id)
        .bind(kind)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get annotations")?;

        Ok(rows
            .into_iter()
            .map(|r| AnnotationRow {
                item_id: r.get("item_id"),
                annotation_kind: r.get("annotation_kind"),
                ordinal: r.get("ordinal"),
                chapter: r.get("chapter"),
                datetime: r.get("datetime"),
                pageno: r.get("pageno"),
                text: r.get("text"),
                note: r.get("note"),
            })
            .collect())
    }

    /// Load all stored fingerprints (for reconciliation during incremental builds).
    pub async fn load_all_fingerprints(&self) -> Result<Vec<FingerprintRow>> {
        let rows = sqlx::query(
            "SELECT item_id, book_path, book_size_bytes, book_modified_unix_ms,
                    metadata_path, metadata_size_bytes, metadata_modified_unix_ms, updated_at
             FROM library_item_fingerprints",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to load fingerprints")?;

        Ok(rows
            .into_iter()
            .map(|r| FingerprintRow {
                item_id: r.get("item_id"),
                book_path: r.get("book_path"),
                book_size_bytes: r.get("book_size_bytes"),
                book_modified_unix_ms: r.get("book_modified_unix_ms"),
                metadata_path: r.get("metadata_path"),
                metadata_size_bytes: r.get("metadata_size_bytes"),
                metadata_modified_unix_ms: r.get("metadata_modified_unix_ms"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    pub async fn item_exists(&self, id: &str) -> Result<bool> {
        let row = sqlx::query("SELECT 1 FROM library_items WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to check item existence")?;
        Ok(row.is_some())
    }

    pub async fn count_items(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as cnt FROM library_items")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count items")?;
        Ok(row.get("cnt"))
    }
}

fn row_to_item(r: &sqlx::sqlite::SqliteRow) -> LibraryItemRow {
    LibraryItemRow {
        id: r.get("id"),
        file_path: r.get("file_path"),
        format: r.get("format"),
        content_type: r.get("content_type"),
        title: r.get("title"),
        title_sort: r.get("title_sort"),
        primary_author_sort: r.get("primary_author_sort"),
        authors_json: r.get("authors_json"),
        series_name: r.get("series_name"),
        series_index: r.get("series_index"),
        description: r.get("description"),
        language: r.get("language"),
        publisher: r.get("publisher"),
        subjects_json: r.get("subjects_json"),
        identifiers_json: r.get("identifiers_json"),
        status: r.get("status"),
        progress_percentage: r.get("progress_percentage"),
        rating: r.get("rating"),
        review_note: r.get("review_note"),
        pages: r.get("pages"),
        cover_url: r.get("cover_url"),
        search_base_path: r.get("search_base_path"),
        annotation_count: r.get("annotation_count"),
        bookmark_count: r.get("bookmark_count"),
        highlight_count: r.get("highlight_count"),
        partial_md5_checksum: r.get("partial_md5_checksum"),
        last_open_at: r.get("last_open_at"),
        total_reading_time_sec: r.get("total_reading_time_sec"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::super::queries::{LibraryListFilter, LibrarySort};
    use super::super::tests::{sample_annotation, sample_item, test_repo};

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

        let all = repo
            .list_items(&LibraryListFilter::default())
            .await
            .unwrap();
        assert_eq!(all.len(), 2);

        let books = repo
            .list_items(&LibraryListFilter {
                content_type: Some("book".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].id, "book1");

        let comics = repo
            .list_items(&LibraryListFilter {
                content_type: Some("comic".to_string()),
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
        b.title_sort = "zebra".to_string();
        b.file_path = "/books/z.epub".to_string();
        repo.upsert_item(&b).await.unwrap();

        let mut a = sample_item("id-a");
        a.title_sort = "alpha".to_string();
        a.file_path = "/books/a.epub".to_string();
        repo.upsert_item(&a).await.unwrap();

        let items = repo
            .list_items(&LibraryListFilter::default())
            .await
            .unwrap();
        assert_eq!(items[0].title_sort, "alpha");
        assert_eq!(items[1].title_sort, "zebra");
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
            .list_items(&LibraryListFilter {
                sort: LibrarySort::Rating,
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
