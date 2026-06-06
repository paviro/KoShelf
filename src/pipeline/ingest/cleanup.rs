use log::{debug, info, warn};
use std::fs;

use crate::pipeline::media::{self, MediaDirs};
use crate::store::sqlite::repo::LibraryRepository;

pub(crate) async fn delete_item_for_book_path(
    repo: &LibraryRepository,
    book_path: &str,
    media_dirs: &MediaDirs,
    is_internal_server: bool,
) -> bool {
    match repo.find_fingerprint_by_book_path(book_path).await {
        Ok(Some(fp)) => {
            delete_item_and_media(
                repo,
                &fp.item_id,
                media_dirs,
                is_internal_server,
                &format!("book removed: {book_path}"),
            )
            .await
        }
        Ok(None) => {
            debug!("No DB fingerprint for removed path: {}", book_path);
            false
        }
        Err(e) => {
            warn!("Failed to look up fingerprint for {}: {}", book_path, e);
            false
        }
    }
}

pub(crate) async fn delete_item_and_media(
    repo: &LibraryRepository,
    item_id: &str,
    media_dirs: &MediaDirs,
    is_internal_server: bool,
    reason: &str,
) -> bool {
    if let Err(e) = repo.delete_item(item_id).await {
        warn!("Failed to delete removed item {}: {}", item_id, e);
        return false;
    }

    if media::is_canonical_item_id(item_id) {
        let cover_path = media_dirs.covers_dir.join(format!("{}.webp", item_id));
        let _ = fs::remove_file(&cover_path);
    } else {
        warn!(
            "Skipping cover cleanup for non-canonical item id: {}",
            item_id
        );
    }

    if is_internal_server
        && let Err(e) = media::remove_item_files_by_id(item_id, &media_dirs.files_dir)
    {
        warn!("Failed to clean file symlinks for {}: {}", item_id, e);
    }

    info!("Deleted item {} ({})", item_id, reason);
    true
}

#[cfg(test)]
mod tests {
    use super::delete_item_and_media;
    use crate::pipeline::media::{self, resolve_media_dirs};
    use crate::store::sqlite::repo::rows::FingerprintRow;
    use crate::store::sqlite::repo::tests::{sample_item, test_repo};

    const CANONICAL_ID: &str = "0123456789abcdef0123456789abcdef";

    #[tokio::test]
    async fn delete_item_and_media_removes_db_item_and_canonical_cover() {
        let repo = test_repo().await;
        repo.upsert_item(&sample_item(CANONICAL_ID))
            .await
            .expect("item upsert");

        let output_dir = tempfile::Builder::new()
            .prefix("koshelf-cleanup-")
            .tempdir_in(std::env::current_dir().expect("cwd"))
            .expect("output dir");
        let media_dirs = resolve_media_dirs(output_dir.path(), false);
        std::fs::create_dir_all(&media_dirs.covers_dir).expect("covers dir");
        let cover_path = media_dirs.covers_dir.join(format!("{CANONICAL_ID}.webp"));
        std::fs::write(&cover_path, b"cover").expect("cover write");

        assert!(
            delete_item_and_media(&repo, CANONICAL_ID, &media_dirs, false, "test removal").await
        );

        assert!(
            repo.get_item(CANONICAL_ID)
                .await
                .expect("get item")
                .is_none()
        );
        assert!(!cover_path.exists());
    }

    #[tokio::test]
    async fn delete_item_and_media_removes_internal_server_file_link() {
        let repo = test_repo().await;
        repo.upsert_item(&sample_item(CANONICAL_ID))
            .await
            .expect("item upsert");

        let output_dir = tempfile::Builder::new()
            .prefix("koshelf-cleanup-")
            .tempdir_in(std::env::current_dir().expect("cwd"))
            .expect("output dir");
        let media_dirs = resolve_media_dirs(output_dir.path(), true);
        std::fs::create_dir_all(&media_dirs.files_dir).expect("files dir");
        let source_path = output_dir.path().join("book.epub");
        std::fs::write(&source_path, b"book").expect("book write");
        media::sync_item_file_symlink(CANONICAL_ID, "epub", &source_path, &media_dirs.files_dir)
            .expect("file link");
        let link_path = media_dirs.files_dir.join(format!("{CANONICAL_ID}.epub"));

        assert!(link_path.exists());
        assert!(
            delete_item_and_media(&repo, CANONICAL_ID, &media_dirs, true, "test removal").await
        );
        assert!(!link_path.exists());
    }

    #[tokio::test]
    async fn delete_item_and_media_skips_noncanonical_cover_cleanup() {
        let repo = test_repo().await;
        let item_id = "not-canonical";
        repo.upsert_item(&sample_item(item_id))
            .await
            .expect("item upsert");
        repo.upsert_fingerprint(&FingerprintRow {
            item_id: item_id.to_string(),
            book_path: "/books/not-canonical.epub".to_string(),
            book_size_bytes: 1,
            book_modified_unix_ms: 1,
            metadata_path: None,
            metadata_size_bytes: None,
            metadata_modified_unix_ms: None,
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        })
        .await
        .expect("fingerprint upsert");

        let output_dir = tempfile::tempdir().expect("output dir");
        let media_dirs = resolve_media_dirs(output_dir.path(), false);
        std::fs::create_dir_all(&media_dirs.covers_dir).expect("covers dir");
        let cover_path = media_dirs.covers_dir.join(format!("{item_id}.webp"));
        std::fs::write(&cover_path, b"cover").expect("cover write");

        assert!(delete_item_and_media(&repo, item_id, &media_dirs, false, "test removal").await);

        assert!(repo.get_item(item_id).await.expect("get item").is_none());
        assert!(cover_path.exists());
    }
}
