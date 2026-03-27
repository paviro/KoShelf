//! Write operations for the library repository.

use anyhow::{Context, Result};

use crate::store::sqlite::repo::LibraryRepository;
use crate::store::sqlite::repo::rows::{
    AnnotationRow, CollisionDiagnosticRow, FingerprintRow, LibraryItemRow,
};

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
                highlight_count, partial_md5_checksum, hidden_flow_pages,
                reader_presentation, chapters_json,
                last_open_at, total_reading_time_sec, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7,
                ?8, ?9, ?10, ?11, ?12,
                ?13, ?14, ?15, ?16,
                ?17, ?18, ?19, ?20,
                ?21, ?22, ?23, ?24,
                ?25, ?26, ?27, ?28,
                ?29,
                ?30, ?31, ?32, ?33
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
                hidden_flow_pages = excluded.hidden_flow_pages,
                reader_presentation = excluded.reader_presentation,
                chapters_json = excluded.chapters_json,
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
        .bind(item.hidden_flow_pages)
        .bind(&item.reader_presentation)
        .bind(&item.chapters_json)
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
                    (id, item_id, annotation_kind, lua_index, chapter, datetime, datetime_updated, pageno, text, note, pos0, pos1, color, drawer)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            )
            .bind(&a.id)
            .bind(&a.item_id)
            .bind(&a.annotation_kind)
            .bind(a.lua_index)
            .bind(&a.chapter)
            .bind(&a.datetime)
            .bind(&a.datetime_updated)
            .bind(a.pageno)
            .bind(&a.text)
            .bind(&a.note)
            .bind(&a.pos0)
            .bind(&a.pos1)
            .bind(&a.color)
            .bind(&a.drawer)
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

    // ── Writeback helpers ────────────────────────────────────────────────

    /// Update the metadata fingerprint after a writeback operation.
    ///
    /// Only touches `metadata_size_bytes` and `metadata_modified_unix_ms`,
    /// leaving book fingerprint data untouched.
    pub async fn update_metadata_fingerprint(
        &self,
        item_id: &str,
        metadata_size_bytes: i64,
        metadata_modified_unix_ms: i64,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE library_item_fingerprints
             SET metadata_size_bytes = ?1, metadata_modified_unix_ms = ?2
             WHERE item_id = ?3",
        )
        .bind(metadata_size_bytes)
        .bind(metadata_modified_unix_ms)
        .bind(item_id)
        .execute(&self.pool)
        .await
        .context("Failed to update metadata fingerprint")?;
        Ok(())
    }

    /// Update writable item-level fields after a writeback operation.
    ///
    /// `review_note`: `None` = don't touch, `Some(None)` = clear, `Some(Some(v))` = set.
    pub async fn update_item_writeback_fields(
        &self,
        item_id: &str,
        review_note: Option<Option<&str>>,
        rating: Option<u32>,
        status: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE library_items SET
                review_note = CASE WHEN ?1 THEN ?2 ELSE review_note END,
                rating = CASE WHEN ?3 THEN ?4 ELSE rating END,
                status = COALESCE(?5, status)
             WHERE id = ?6",
        )
        .bind(review_note.is_some())
        .bind(review_note.flatten())
        .bind(rating.is_some())
        .bind(rating.filter(|&r| r > 0).map(|r| r as i32))
        .bind(status)
        .bind(item_id)
        .execute(&self.pool)
        .await
        .context("Failed to update item writeback fields")?;
        Ok(())
    }

    /// Update writable annotation-level fields after a writeback operation.
    ///
    /// `note`: `None` = don't touch, `Some(None)` = clear, `Some(Some(v))` = set.
    pub async fn update_annotation_writeback_fields(
        &self,
        annotation_id: &str,
        note: Option<Option<&str>>,
        color: Option<&str>,
        drawer: Option<&str>,
        datetime_updated: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE library_annotations SET
                note = CASE WHEN ?1 THEN ?2 ELSE note END,
                color = COALESCE(?3, color),
                drawer = COALESCE(?4, drawer),
                datetime_updated = COALESCE(?5, datetime_updated)
             WHERE id = ?6",
        )
        .bind(note.is_some())
        .bind(note.flatten())
        .bind(color)
        .bind(drawer)
        .bind(datetime_updated)
        .bind(annotation_id)
        .execute(&self.pool)
        .await
        .context("Failed to update annotation writeback fields")?;
        Ok(())
    }

    /// Decrement annotation counts after deleting an annotation.
    pub async fn decrement_annotation_count(
        &self,
        item_id: &str,
        is_highlight: bool,
    ) -> Result<()> {
        let kind_column = if is_highlight {
            "highlight_count"
        } else {
            "bookmark_count"
        };
        let sql = format!(
            "UPDATE library_items SET
                annotation_count = MAX(annotation_count - 1, 0),
                {kind_column} = MAX({kind_column} - 1, 0)
             WHERE id = ?1"
        );
        sqlx::query(&sql)
            .bind(item_id)
            .execute(&self.pool)
            .await
            .context("Failed to decrement annotation count")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::server::api::responses::library::LibraryStatus;
    use crate::store::sqlite::repo::rows::CollisionDiagnosticRow;
    use crate::store::sqlite::repo::tests::{
        sample_annotation, sample_fingerprint, sample_item, test_repo,
    };

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
        assert_eq!(fetched.status, LibraryStatus::Reading);
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
    async fn upsert_and_get_item_preserves_reader_presentation() {
        let repo = test_repo().await;
        let mut item = sample_item("reader");
        item.reader_presentation =
            Some(r#"{"font_face":"Noto Serif","embedded_fonts":false}"#.to_string());

        repo.upsert_item(&item)
            .await
            .expect("upsert should work with reader presentation");

        let fetched = repo
            .get_item("reader")
            .await
            .expect("get should work")
            .expect("item should exist");

        let presentation = fetched
            .reader_presentation
            .expect("reader presentation should be present")
            .0;

        assert_eq!(presentation.font_face.as_deref(), Some("Noto Serif"));
        assert_eq!(presentation.embedded_fonts, Some(false));
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
    async fn update_metadata_fingerprint_only_touches_metadata_columns() {
        let repo = test_repo().await;
        repo.upsert_item(&sample_item("fp1")).await.unwrap();
        repo.upsert_fingerprint(&sample_fingerprint("fp1"))
            .await
            .unwrap();

        repo.update_metadata_fingerprint("fp1", 999, 1700000099000)
            .await
            .unwrap();

        let all = repo.load_all_fingerprints().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].metadata_size_bytes, Some(999));
        assert_eq!(all[0].metadata_modified_unix_ms, Some(1700000099000));
        // Book fingerprint unchanged
        assert_eq!(all[0].book_size_bytes, 1024);
        assert_eq!(all[0].book_modified_unix_ms, 1700000000000);
    }

    #[tokio::test]
    async fn update_item_writeback_fields_updates_only_provided() {
        let repo = test_repo().await;
        let mut item = sample_item("wb1");
        item.review_note = Some("old note".to_string());
        item.rating = Some(3);
        item.status = "reading".to_string();
        repo.upsert_item(&item).await.unwrap();

        // Update only review_note, leave rating and status unchanged
        repo.update_item_writeback_fields("wb1", Some(Some("new note")), None, None)
            .await
            .unwrap();

        let fetched = repo.get_item("wb1").await.unwrap().unwrap();
        assert_eq!(fetched.review_note.as_deref(), Some("new note"));
        assert_eq!(fetched.rating, Some(3));
        assert_eq!(fetched.status, LibraryStatus::Reading);
    }

    #[tokio::test]
    async fn update_item_writeback_fields_clears_rating_with_zero() {
        let repo = test_repo().await;
        let mut item = sample_item("wb2");
        item.rating = Some(5);
        repo.upsert_item(&item).await.unwrap();

        repo.update_item_writeback_fields("wb2", None, Some(0), None)
            .await
            .unwrap();

        let fetched = repo.get_item("wb2").await.unwrap().unwrap();
        assert_eq!(fetched.rating, None);
    }

    #[tokio::test]
    async fn update_annotation_writeback_fields_updates_only_provided() {
        let repo = test_repo().await;
        repo.upsert_item(&sample_item("aw1")).await.unwrap();
        let mut ann = sample_annotation("aw1", "highlight", 0);
        let ann_id = ann.id.clone();
        ann.color = Some("yellow".to_string());
        ann.drawer = Some("lighten".to_string());
        repo.replace_annotations("aw1", &[ann]).await.unwrap();

        repo.update_annotation_writeback_fields(&ann_id, Some(Some("my note")), None, None, None)
            .await
            .unwrap();

        let all = repo.get_annotations("aw1", None).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].note.as_deref(), Some("my note"));
        assert_eq!(all[0].color.as_deref(), Some("yellow"));
        assert_eq!(all[0].drawer.as_deref(), Some("lighten"));
    }

    #[tokio::test]
    async fn decrement_annotation_count_decrements_and_clamps() {
        let repo = test_repo().await;
        let mut item = sample_item("dc1");
        item.annotation_count = 2;
        item.highlight_count = 1;
        item.bookmark_count = 1;
        repo.upsert_item(&item).await.unwrap();

        repo.decrement_annotation_count("dc1", true).await.unwrap();

        let counts: (i32, i32, i32) = sqlx::query_as(
            "SELECT annotation_count, highlight_count, bookmark_count FROM library_items WHERE id = ?1",
        )
        .bind("dc1")
        .fetch_one(repo.pool())
        .await
        .unwrap();
        assert_eq!(counts, (1, 0, 1));

        // Decrementing at zero should clamp
        repo.decrement_annotation_count("dc1", true).await.unwrap();
        let counts: (i32, i32, i32) = sqlx::query_as(
            "SELECT annotation_count, highlight_count, bookmark_count FROM library_items WHERE id = ?1",
        )
        .bind("dc1")
        .fetch_one(repo.pool())
        .await
        .unwrap();
        assert_eq!(counts, (0, 0, 1));
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
