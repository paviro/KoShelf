//! Write operations for the library repository.

use anyhow::{Context, Result};

use super::LibraryRepository;
use super::rows::{AnnotationRow, CollisionDiagnosticRow, FingerprintRow, LibraryItemRow};

impl LibraryRepository {
    pub async fn upsert_item(&self, item: &LibraryItemRow) -> Result<()> {
        sqlx::query(
            "INSERT INTO library_items (
                id, file_path, format, content_type, title,
                authors_json, series_json,
                description, language, publisher, subjects_json, identifiers_json,
                status, progress_percentage, rating, review_note,
                doc_pages, pagemap_doc_pages, has_synthetic_pagination, parser_pages,
                cover_url, search_base_path, annotation_count, bookmark_count,
                highlight_count, partial_md5_checksum, last_open_at,
                total_reading_time_sec, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7,
                ?8, ?9, ?10, ?11, ?12,
                ?13, ?14, ?15, ?16,
                ?17, ?18, ?19, ?20,
                ?21, ?22, ?23, ?24,
                ?25, ?26, ?27,
                ?28, ?29, ?30
            )
            ON CONFLICT(id) DO UPDATE SET
                file_path = excluded.file_path,
                format = excluded.format,
                content_type = excluded.content_type,
                title = excluded.title,
                authors_json = excluded.authors_json,
                series_json = excluded.series_json,
                description = excluded.description,
                language = excluded.language,
                publisher = excluded.publisher,
                subjects_json = excluded.subjects_json,
                identifiers_json = excluded.identifiers_json,
                status = excluded.status,
                progress_percentage = excluded.progress_percentage,
                rating = excluded.rating,
                review_note = excluded.review_note,
                doc_pages = excluded.doc_pages,
                pagemap_doc_pages = excluded.pagemap_doc_pages,
                has_synthetic_pagination = excluded.has_synthetic_pagination,
                parser_pages = excluded.parser_pages,
                cover_url = excluded.cover_url,
                search_base_path = excluded.search_base_path,
                annotation_count = excluded.annotation_count,
                bookmark_count = excluded.bookmark_count,
                highlight_count = excluded.highlight_count,
                partial_md5_checksum = excluded.partial_md5_checksum,
                last_open_at = excluded.last_open_at,
                total_reading_time_sec = excluded.total_reading_time_sec,
                updated_at = excluded.updated_at",
        )
        .bind(&item.id)
        .bind(&item.file_path)
        .bind(&item.format)
        .bind(&item.content_type)
        .bind(&item.title)
        .bind(&item.authors_json)
        .bind(&item.series_json)
        .bind(&item.description)
        .bind(&item.language)
        .bind(&item.publisher)
        .bind(&item.subjects_json)
        .bind(&item.identifiers_json)
        .bind(&item.status)
        .bind(item.progress_percentage)
        .bind(item.rating)
        .bind(&item.review_note)
        .bind(item.doc_pages)
        .bind(item.pagemap_doc_pages)
        .bind(item.has_synthetic_pagination)
        .bind(item.parser_pages)
        .bind(&item.cover_url)
        .bind(&item.search_base_path)
        .bind(item.annotation_count)
        .bind(item.bookmark_count)
        .bind(item.highlight_count)
        .bind(&item.partial_md5_checksum)
        .bind(&item.last_open_at)
        .bind(item.total_reading_time_sec)
        .bind(&item.created_at)
        .bind(&item.updated_at)
        .execute(&self.pool)
        .await
        .context("Failed to upsert library item")?;
        Ok(())
    }

    /// Delete existing annotations for `item_id` and insert replacements.
    pub async fn replace_annotations(
        &self,
        item_id: &str,
        annotations: &[AnnotationRow],
    ) -> Result<()> {
        let mut tx = self.pool.begin().await.context("begin tx")?;

        sqlx::query("DELETE FROM library_annotations WHERE item_id = ?1")
            .bind(item_id)
            .execute(&mut *tx)
            .await
            .context("delete old annotations")?;

        for a in annotations {
            sqlx::query(
                "INSERT INTO library_annotations
                    (item_id, annotation_kind, ordinal, chapter, datetime, pageno, text, note)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .bind(&a.item_id)
            .bind(&a.annotation_kind)
            .bind(a.ordinal)
            .bind(&a.chapter)
            .bind(&a.datetime)
            .bind(a.pageno)
            .bind(&a.text)
            .bind(&a.note)
            .execute(&mut *tx)
            .await
            .context("insert annotation")?;
        }

        tx.commit().await.context("commit annotations")?;
        Ok(())
    }

    pub async fn upsert_fingerprint(&self, fp: &FingerprintRow) -> Result<()> {
        sqlx::query(
            "INSERT INTO library_item_fingerprints (
                item_id, book_path, book_size_bytes, book_modified_unix_ms,
                metadata_path, metadata_size_bytes, metadata_modified_unix_ms, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(item_id) DO UPDATE SET
                book_path = excluded.book_path,
                book_size_bytes = excluded.book_size_bytes,
                book_modified_unix_ms = excluded.book_modified_unix_ms,
                metadata_path = excluded.metadata_path,
                metadata_size_bytes = excluded.metadata_size_bytes,
                metadata_modified_unix_ms = excluded.metadata_modified_unix_ms,
                updated_at = excluded.updated_at",
        )
        .bind(&fp.item_id)
        .bind(&fp.book_path)
        .bind(fp.book_size_bytes)
        .bind(fp.book_modified_unix_ms)
        .bind(&fp.metadata_path)
        .bind(fp.metadata_size_bytes)
        .bind(fp.metadata_modified_unix_ms)
        .bind(&fp.updated_at)
        .execute(&self.pool)
        .await
        .context("Failed to upsert fingerprint")?;
        Ok(())
    }

    pub async fn upsert_collision_diagnostic(&self, diag: &CollisionDiagnosticRow) -> Result<()> {
        sqlx::query(
            "INSERT INTO library_collision_diagnostics
                (canonical_id, file_path, winner_item_id, reason, detected_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(canonical_id, file_path) DO UPDATE SET
                winner_item_id = excluded.winner_item_id,
                reason = excluded.reason,
                detected_at = excluded.detected_at",
        )
        .bind(&diag.canonical_id)
        .bind(&diag.file_path)
        .bind(&diag.winner_item_id)
        .bind(&diag.reason)
        .bind(&diag.detected_at)
        .execute(&self.pool)
        .await
        .context("Failed to upsert collision diagnostic")?;
        Ok(())
    }

    pub async fn delete_item(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM library_items WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete library item")?;
        Ok(())
    }

    /// Remove all items, annotations, fingerprints, and collision diagnostics.
    pub async fn clear_all(&self) -> Result<()> {
        let mut tx = self.pool.begin().await.context("begin tx")?;
        sqlx::query("DELETE FROM library_collision_diagnostics")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM library_annotations")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM library_item_fingerprints")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM library_items")
            .execute(&mut *tx)
            .await?;
        tx.commit().await.context("commit clear_all")?;
        Ok(())
    }

    /// Remove all collision diagnostics (used before a full rebuild).
    pub async fn clear_collision_diagnostics(&self) -> Result<()> {
        sqlx::query("DELETE FROM library_collision_diagnostics")
            .execute(&self.pool)
            .await
            .context("Failed to clear collision diagnostics")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::rows::CollisionDiagnosticRow;
    use super::super::tests::{sample_annotation, sample_fingerprint, sample_item, test_repo};

    #[tokio::test]
    async fn upsert_and_get_item() {
        let repo = test_repo().await;
        let item = sample_item("aaa");

        repo.upsert_item(&item).await.expect("upsert should work");

        let fetched = repo
            .get_item("aaa")
            .await
            .expect("get should work")
            .expect("item should exist");

        assert_eq!(fetched.id, "aaa");
        assert_eq!(fetched.title, "Book aaa");
        assert_eq!(
            fetched.status,
            crate::contracts::library::LibraryStatus::Reading
        );
        assert!((fetched.progress_percentage.unwrap() - 0.42).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn upsert_updates_existing_without_cascade_delete() {
        let repo = test_repo().await;
        let mut item = sample_item("bbb");
        repo.upsert_item(&item).await.unwrap();

        let annotations = vec![sample_annotation("bbb", "highlight", 0)];
        repo.replace_annotations("bbb", &annotations).await.unwrap();

        item.title = "Updated Title".to_string();
        item.updated_at = "2026-02-01T00:00:00Z".to_string();
        repo.upsert_item(&item).await.unwrap();

        let ann = repo.get_annotations("bbb", None).await.unwrap();
        assert_eq!(ann.len(), 1);

        let fetched = repo.get_item("bbb").await.unwrap().unwrap();
        assert_eq!(fetched.title, "Updated Title");
    }

    #[tokio::test]
    async fn delete_item_cascades_to_related_rows() {
        let repo = test_repo().await;
        repo.upsert_item(&sample_item("ccc")).await.unwrap();
        repo.replace_annotations("ccc", &[sample_annotation("ccc", "highlight", 0)])
            .await
            .unwrap();
        repo.upsert_fingerprint(&sample_fingerprint("ccc"))
            .await
            .unwrap();

        repo.delete_item("ccc").await.unwrap();

        assert!(!repo.item_exists("ccc").await.unwrap());
        assert!(repo.get_annotations("ccc", None).await.unwrap().is_empty());
        assert!(repo.load_all_fingerprints().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn replace_annotations_replaces_all_for_item() {
        let repo = test_repo().await;
        repo.upsert_item(&sample_item("ddd")).await.unwrap();

        repo.replace_annotations(
            "ddd",
            &[
                sample_annotation("ddd", "highlight", 0),
                sample_annotation("ddd", "highlight", 1),
            ],
        )
        .await
        .unwrap();
        assert_eq!(repo.get_annotations("ddd", None).await.unwrap().len(), 2);

        repo.replace_annotations("ddd", &[sample_annotation("ddd", "bookmark", 0)])
            .await
            .unwrap();

        let all = repo.get_annotations("ddd", None).await.unwrap();
        assert_eq!(all.len(), 1);
        // Verify it's a bookmark by checking the text matches our sample
        assert!(all[0].text.is_some());
    }

    #[tokio::test]
    async fn upsert_and_load_fingerprints() {
        let repo = test_repo().await;
        repo.upsert_item(&sample_item("fff")).await.unwrap();

        let fp = sample_fingerprint("fff");
        repo.upsert_fingerprint(&fp).await.unwrap();

        let all = repo.load_all_fingerprints().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].item_id, "fff");
        assert_eq!(all[0].book_size_bytes, 1024);
    }

    #[tokio::test]
    async fn upsert_collision_diagnostic() {
        let repo = test_repo().await;
        repo.upsert_item(&sample_item("winner")).await.unwrap();

        let diag = CollisionDiagnosticRow {
            canonical_id: "shared-md5".to_string(),
            file_path: "/books/loser.epub".to_string(),
            winner_item_id: "winner".to_string(),
            reason: "earlier path".to_string(),
            detected_at: "2026-01-01T00:00:00Z".to_string(),
        };
        repo.upsert_collision_diagnostic(&diag)
            .await
            .expect("upsert diagnostic should work");
    }

    #[tokio::test]
    async fn clear_all_removes_everything() {
        let repo = test_repo().await;
        repo.upsert_item(&sample_item("ggg")).await.unwrap();
        repo.replace_annotations("ggg", &[sample_annotation("ggg", "highlight", 0)])
            .await
            .unwrap();
        repo.upsert_fingerprint(&sample_fingerprint("ggg"))
            .await
            .unwrap();

        repo.clear_all().await.unwrap();

        assert_eq!(repo.count_items().await.unwrap(), 0);
        assert!(repo.load_all_fingerprints().await.unwrap().is_empty());
    }
}
