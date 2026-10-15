use super::{LibraryRepository, rows::FingerprintRow};
use crate::{
    shelf::{
        library::item_mapping::{map_annotations_to_rows, map_item_to_row},
        time_config::TimeConfig,
    },
    store::bundles::{Bundle, State},
};
use anyhow::{Result, ensure};
use std::path::Path;

impl LibraryRepository {
    pub(crate) async fn remove_archived_projections(&self) -> Result<()> {
        sqlx::query("DELETE FROM library_items WHERE is_archived = 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub(crate) async fn managed_revision(&self, id: &str) -> Result<Option<String>> {
        Ok(sqlx::query_scalar::<_, Option<String>>(
            "SELECT managed_revision FROM library_items WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .flatten())
    }
    pub(crate) async fn note_bundle(&self, bundle: &Bundle) -> Result<()> {
        sqlx::query("UPDATE library_items SET managed_revision = ?, is_archived = ? WHERE id = ?")
            .bind(bundle.revision()?)
            .bind(bundle.record.state == State::Archived)
            .bind(&bundle.record.id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub(crate) async fn set_archived(&self, id: &str, archived: bool) -> Result<()> {
        sqlx::query("UPDATE library_items SET is_archived = ? WHERE id = ?")
            .bind(archived)
            .bind(id)
            .execute(&self.pool)
            .await?;
        if archived {
            sqlx::query("DELETE FROM library_item_fingerprints WHERE item_id = ?")
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }
    pub(crate) async fn project_bundle(&self, bundle: &Bundle, time: &TimeConfig) -> Result<()> {
        let item = bundle.item()?;
        let row = map_item_to_row(&item, time);
        let annotations = map_annotations_to_rows(&item.id, &item, time);
        let mut tx = self.pool.begin().await?;
        Self::upsert_item_on(&mut tx, &row).await?;
        Self::replace_annotations_on(&mut tx, &item.id, &annotations).await?;
        sqlx::query("UPDATE library_items SET is_archived = ? WHERE id = ?")
            .bind(bundle.record.state == State::Archived)
            .bind(&item.id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM library_item_fingerprints WHERE item_id = ?")
            .bind(&item.id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE library_items SET calculated_checksum = ? WHERE id = ?")
            .bind(&bundle.record.checksum)
            .bind(&item.id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE library_items SET managed_revision = ? WHERE id = ?")
            .bind(bundle.revision()?)
            .bind(&item.id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
    pub(crate) async fn item_source_row(&self, id: &str) -> Result<Option<ItemSource>> {
        Ok(sqlx::query_as("SELECT id, file_path, format, document_state, calculated_checksum, is_archived AS archived FROM library_items WHERE id = ?")
            .bind(id).fetch_optional(&self.pool).await?)
    }

    pub(crate) async fn item_sources(&self) -> Result<Vec<ItemSource>> {
        Ok(sqlx::query_as(
            "SELECT id, file_path, format, document_state, calculated_checksum, is_archived AS archived FROM library_items",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    /// Read identity from the source when present, or from its saved capture.
    pub(crate) async fn item_source(&self, id: &str) -> Result<ItemSource> {
        use anyhow::Context;
        let mut source = self.item_source_row(id).await?.context("item not found")?;
        if source.document_state == crate::shelf::models::DocumentState::Present {
            source.calculated_checksum = Some(crate::source::koreader::calculate_partial_md5(
                Path::new(&source.file_path),
            )?);
        }
        source.checksum()?;
        Ok(source)
    }
    pub(crate) async fn find_import_record(
        &self,
        path: &Path,
        checksum: &str,
    ) -> Result<Option<FingerprintRow>> {
        if self.bundles.is_some() {
            let matches: Vec<(String, String)> = sqlx::query_as(
                "SELECT id, file_path FROM library_items WHERE calculated_checksum = ? ORDER BY id",
            )
            .bind(checksum)
            .fetch_all(&self.pool)
            .await?;
            ensure!(
                matches.len() <= 1,
                "multiple managed identities match document checksum {checksum}"
            );
            if let Some((id, _)) = matches.first() {
                if let Some(fp) = self
                    .find_fingerprint_by_book_path(&path.to_string_lossy())
                    .await?
                    && fp.item_id == *id
                {
                    return Ok(Some(fp));
                }
                return Ok(Some(FingerprintRow {
                    custom_state: None,
                    item_id: id.clone(),
                    book_path: path.to_string_lossy().into_owned(),
                    book_size_bytes: Some(-1),
                    book_modified_unix_ms: Some(-1),
                    metadata_path: None,
                    metadata_size_bytes: None,
                    metadata_modified_unix_ms: None,
                    updated_at: String::new(),
                }));
            }
            if let Some(fp) = self
                .find_fingerprint_by_book_path(&path.to_string_lossy())
                .await?
            {
                let old: Option<String> = sqlx::query_scalar(
                    "SELECT calculated_checksum FROM library_items WHERE id = ?",
                )
                .bind(&fp.item_id)
                .fetch_one(&self.pool)
                .await?;
                if old.as_deref().is_some_and(|old| old != checksum) {
                    return Ok(None);
                }
                return Ok(Some(fp));
            }
            return Ok(None);
        }
        if let Some(existing) = self
            .find_fingerprint_by_book_path(&path.to_string_lossy())
            .await?
        {
            return Ok(Some(existing));
        }
        // Alternate sidecars carry the canonical identity. Do not merge ordinary
        // documents merely because their bytes match: their sidecars may name
        // deliberately different identities.
        let path: Option<String> =
            sqlx::query_scalar("SELECT file_path FROM library_items WHERE id = ?")
                .bind(checksum)
                .fetch_optional(&self.pool)
                .await?;
        match path {
            Some(path) => self.find_fingerprint_by_book_path(&path).await,
            None => Ok(None),
        }
    }
}

/// Source identity stored independently of document availability.
#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct ItemSource {
    pub id: String,
    pub file_path: String,
    pub format: String,
    pub document_state: crate::shelf::models::DocumentState,
    pub calculated_checksum: Option<String>,
    pub archived: bool,
}

impl ItemSource {
    pub fn format(&self) -> crate::shelf::models::LibraryItemFormat {
        crate::shelf::models::LibraryItemFormat::from_document_path(Path::new(&format!(
            "book.{}",
            self.format
        )))
    }

    pub fn checksum(&self) -> Result<&str> {
        use anyhow::Context;
        self.calculated_checksum
            .as_deref()
            .context("missing saved checksum")
    }
}
