use crate::{
    source::{metadata::CapturedMetadata, scanner::MetadataLocation},
    store::sqlite::repo::{
        LibraryRepository,
        rows::{AnnotationRow, FingerprintRow, LibraryItemRow},
    },
};
use anyhow::Result;

/// Identifies the primary or backup that produced the cached reading data; contains no file bytes.
pub(crate) struct MetadataProvenance {
    pub revision: String,
}

impl LibraryRepository {
    pub(crate) fn metadata_indices(&self) -> Result<crate::source::metadata::MetadataIndices> {
        let excluded: Vec<_> = self.bundles.iter().map(|s| s.root.clone()).collect();
        crate::source::metadata::MetadataIndices::new_excluding(
            &self.metadata_location(),
            &excluded,
        )
    }

    /// Serializes accepted metadata and writeback across all cloned repositories.
    /// A shared gate also protects competing book IDs during duplicate checks.
    /// Media generation remains outside the critical section.
    pub(crate) async fn metadata_guard(&self) -> tokio::sync::OwnedMutexGuard<()> {
        self.metadata_gate.clone().lock_owned().await
    }
    pub(crate) fn configure_metadata(&self, location: &MetadataLocation) {
        *self
            .metadata_location
            .write()
            .expect("metadata configuration lock poisoned") = location.clone();
    }
    pub(crate) fn metadata_location(&self) -> MetadataLocation {
        self.metadata_location
            .read()
            .expect("metadata configuration lock poisoned")
            .clone()
    }
    pub(crate) async fn metadata_provenance(&self, id: &str) -> Result<Option<MetadataProvenance>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT metadata_revision FROM library_item_fingerprints WHERE item_id = ? AND metadata_revision IS NOT NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(revision,)| MetadataProvenance { revision }))
    }
    pub(crate) async fn set_metadata_writable(&self, id: &str, writable: bool) -> Result<()> {
        sqlx::query("UPDATE library_item_fingerprints SET metadata_writable = ? WHERE item_id = ?")
            .bind(writable)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    pub(crate) async fn persist_captured_item(
        &self,
        row: &LibraryItemRow,
        annotations: &[AnnotationRow],
        fp: &FingerprintRow,
        captured: Option<&CapturedMetadata>,
        checksum: &str,
        writable: bool,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        Self::upsert_item_on(&mut tx, row).await?;
        Self::replace_annotations_on(&mut tx, &row.id, annotations).await?;
        Self::upsert_fingerprint_on(&mut tx, fp).await?;
        sqlx::query(
            "UPDATE library_item_fingerprints
             SET metadata_modified_ns = ?, metadata_revision = COALESCE(?, metadata_revision), metadata_source = COALESCE(?, metadata_source),
                 calculated_checksum = ?, metadata_writable = ? WHERE item_id = ?",
        )
        .bind(captured.map(|c| c.modified_ns.to_string()))
        .bind(captured.map(|s| &s.revision))
        .bind(captured.map(|s| if s.is_backup { "backup" } else { "primary" }))
        .bind(checksum)
        .bind(writable)
        .bind(&row.id)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE library_items SET calculated_checksum = ? WHERE id = ?")
            .bind(checksum)
            .bind(&row.id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::store::sqlite::repo::tests::{
        sample_annotation, sample_fingerprint, sample_item, test_repo,
    };
    #[tokio::test]
    async fn failed_annotation_insert_rolls_back_item_fingerprint_and_provenance() {
        let repo = test_repo().await;
        let original = sample_item("aaa");
        let fp = sample_fingerprint("aaa");
        repo.persist_captured_item(&original, &[], &fp, None, "hash", false)
            .await
            .unwrap();
        let mut changed = original;
        changed.title = "must roll back".into();
        let mut annotation = sample_annotation("missing", "highlight", 0);
        annotation.id = "invalid-foreign-key".into();
        assert!(
            repo.persist_captured_item(&changed, &[annotation], &fp, None, "changed", true)
                .await
                .is_err()
        );
        let item = repo.get_item("aaa").await.unwrap().unwrap();
        assert_eq!(item.title, "Book aaa");
        assert!(!item.has_metadata);
        let checksum: String = sqlx::query_scalar(
            "SELECT calculated_checksum FROM library_item_fingerprints WHERE item_id='aaa'",
        )
        .fetch_one(repo.pool())
        .await
        .unwrap();
        assert_eq!(checksum, "hash");
    }
}
