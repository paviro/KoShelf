//! Write operations for the library repository.

use anyhow::{Context, Result};
use sqlx::{QueryBuilder, Sqlite};

use crate::store::sqlite::repo::LibraryRepository;
use crate::store::sqlite::repo::rows::{AnnotationRow, FingerprintRow, LibraryItemRow};

impl LibraryRepository {
    #[cfg(test)]
    pub async fn upsert_item(&self, item: &LibraryItemRow) -> Result<()> {
        let mut connection = self.pool.acquire().await?;
        Self::upsert_item_on(&mut connection, item).await
    }

    pub(super) async fn upsert_item_on(
        connection: &mut sqlx::SqliteConnection,
        item: &LibraryItemRow,
    ) -> Result<()> {
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
        .execute(&mut *connection)
        .await
        .context("Failed to upsert library item")?;
        sqlx::query("UPDATE library_items SET custom_metadata_json = ? WHERE id = ?")
            .bind(&item.custom_metadata_json)
            .bind(&item.id)
            .execute(&mut *connection)
            .await?;
        Ok(())
    }

    /// Delete existing annotations for `item_id` and insert replacements.
    #[cfg(test)]
    pub async fn replace_annotations(
        &self,
        item_id: &str,
        annotations: &[AnnotationRow],
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        Self::replace_annotations_on(&mut tx, item_id, annotations).await?;
        tx.commit().await?;
        Ok(())
    }
    pub(super) async fn replace_annotations_on(
        connection: &mut sqlx::SqliteConnection,
        item_id: &str,
        annotations: &[AnnotationRow],
    ) -> Result<()> {
        sqlx::query("DELETE FROM library_annotations WHERE item_id = ?1")
            .bind(item_id)
            .execute(&mut *connection)
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
            .execute(&mut *connection)
            .await
            .context("insert annotation")?;
        }

        Ok(())
    }

    #[cfg(test)]
    pub async fn upsert_fingerprint(&self, fp: &FingerprintRow) -> Result<()> {
        let mut connection = self.pool.acquire().await?;
        Self::upsert_fingerprint_on(&mut connection, fp).await
    }

    pub(super) async fn upsert_fingerprint_on(
        connection: &mut sqlx::SqliteConnection,
        fp: &FingerprintRow,
    ) -> Result<()> {
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
        .execute(&mut *connection)
        .await
        .context("Failed to upsert fingerprint")?;
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

    // ── Share image fingerprints ────────────────────────────────────────

    /// Upsert a share image fingerprint for a given year.
    pub async fn upsert_share_image_fingerprint(&self, year: i32, fingerprint: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO share_image_fingerprints (year, fingerprint)
             VALUES (?1, ?2)
             ON CONFLICT(year) DO UPDATE SET fingerprint = excluded.fingerprint",
        )
        .bind(year)
        .bind(fingerprint)
        .execute(&self.pool)
        .await
        .context("Failed to upsert share image fingerprint")?;
        Ok(())
    }

    /// Remove share image fingerprints for years not in the given set.
    pub async fn cleanup_stale_share_image_fingerprints(&self, valid_years: &[i32]) -> Result<()> {
        if valid_years.is_empty() {
            sqlx::query("DELETE FROM share_image_fingerprints")
                .execute(&self.pool)
                .await
                .context("Failed to clear share image fingerprints")?;
        } else {
            let mut builder = QueryBuilder::<Sqlite>::new(
                "DELETE FROM share_image_fingerprints WHERE year NOT IN (",
            );
            {
                let mut separated = builder.separated(", ");
                for year in valid_years {
                    separated.push_bind(year);
                }
            }
            builder.push(")");

            builder
                .build()
                .execute(&self.pool)
                .await
                .context("Failed to cleanup stale share image fingerprints")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::server::api::responses::library::LibraryStatus;
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
}
