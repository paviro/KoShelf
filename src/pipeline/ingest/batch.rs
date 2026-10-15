use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use std::sync::Arc;
use std::time::Instant;

use crate::app::config::SiteConfig;
use crate::pipeline::ingest::metadata::MetadataIndices;
use crate::pipeline::ingest::processor::{ItemProcessor, process_single_item};
use crate::pipeline::media::MediaDirs;
use crate::source::scanner::CollectedItem;
use crate::store::sqlite::repo::LibraryRepository;

/// Counters produced by item ingestion.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct IngestStats {
    pub processed: u64,
    pub upserted: u64,
    pub skipped_duplicates: u64,
    pub skipped_unread: u64,
    pub errors: u64,
    pub stats_invalidated: u64,
}

impl IngestStats {
    pub(super) fn merge(&mut self, other: IngestStats) {
        self.processed += other.processed;
        self.upserted += other.upserted;
        self.skipped_duplicates += other.skipped_duplicates;
        self.skipped_unread += other.skipped_unread;
        self.errors += other.errors;
        self.stats_invalidated += other.stats_invalidated;
    }
}

const INGEST_CONCURRENCY: usize = 8;

#[cfg(test)]
pub(crate) async fn ingest_items(
    items: &[CollectedItem],
    config: &SiteConfig,
    repo: &LibraryRepository,
    media_dirs: &MediaDirs,
) -> Result<IngestStats> {
    if items.is_empty() {
        return Ok(IngestStats::default());
    }

    repo.configure_metadata(&config.metadata_location);
    *repo
        .runtime_config
        .write()
        .expect("runtime configuration lock") = Some(config.clone());
    let metadata_indices = Arc::new(repo.metadata_indices()?);
    ingest_items_with_metadata_indices(items, config, repo, media_dirs, metadata_indices).await
}

pub(super) async fn ingest_items_with_metadata_indices(
    items: &[CollectedItem],
    config: &SiteConfig,
    repo: &LibraryRepository,
    media_dirs: &MediaDirs,
    metadata_indices: Arc<MetadataIndices>,
) -> Result<IngestStats> {
    if items.is_empty() {
        return Ok(IngestStats::default());
    }

    info!("Ingesting {} items...", items.len());
    let start = Instant::now();

    let pb = ProgressBar::new(items.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} {bar:30.cyan/blue} {pos}/{len}")
            .unwrap()
            .progress_chars("━╸─"),
    );
    pb.set_message("Ingesting library:");

    let (tx, rx) = tokio::sync::mpsc::channel::<CollectedItem>(INGEST_CONCURRENCY * 2);
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    let mut workers = tokio::task::JoinSet::new();
    for _ in 0..INGEST_CONCURRENCY {
        let rx = rx.clone();
        let repo = repo.clone();
        let config = config.clone();
        let media_dirs = media_dirs.clone();
        let metadata_indices = metadata_indices.clone();
        let pb = pb.clone();

        workers.spawn(async move {
            let processor = ItemProcessor::new(metadata_indices);
            let mut stats = IngestStats::default();

            loop {
                let item = {
                    let mut rx = rx.lock().await;
                    match rx.recv().await {
                        Some(p) => p,
                        None => break,
                    }
                };

                let item_stats =
                    process_single_item(&item, &processor, &config, &repo, &media_dirs).await;
                stats.merge(item_stats);
                pb.inc(1);
            }

            stats
        });
    }

    for item in items {
        if tx.send(item.clone()).await.is_err() {
            break;
        }
    }
    drop(tx);

    let mut total_stats = IngestStats::default();
    while let Some(result) = workers.join_next().await {
        match result {
            Ok(worker_stats) => total_stats.merge(worker_stats),
            Err(e) => warn!("Ingest worker panicked: {}", e),
        }
    }

    pb.finish_and_clear();
    total_stats.processed = items.len() as u64;

    let elapsed = start.elapsed();
    info!(
        "Ingest complete in {:.1}s: {} processed, {} upserted, {} duplicates, {} unread, {} errors",
        elapsed.as_secs_f64(),
        total_stats.processed,
        total_stats.upserted,
        total_stats.skipped_duplicates,
        total_stats.skipped_unread,
        total_stats.errors,
    );

    Ok(total_stats)
}

#[cfg(test)]
mod tests {
    use super::{ItemProcessor, MetadataIndices, ingest_items, process_single_item};
    use crate::app::config::SiteConfig;
    use crate::pipeline::media::resolve_media_dirs;
    use crate::shelf::library::queries::LibraryListQuery;
    use crate::shelf::models::LibraryItemFormat;
    use crate::shelf::time_config::TimeConfig;
    use crate::source::kobo::KoboFileHints;
    use crate::source::scanner::{CollectedItem, MetadataLocation};
    use crate::store::lifecycle::{RuntimeDataPathOptions, resolve_runtime_data_policy};
    use crate::store::sqlite::repo::tests::test_repo;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::sync::Arc;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    fn test_config(library_path: &Path, output_dir: &Path) -> SiteConfig {
        let mut runtime_data_policy =
            resolve_runtime_data_policy(&RuntimeDataPathOptions::default());
        runtime_data_policy.set_resolved_data_dir(output_dir.join("runtime"));

        SiteConfig {
            output_dir: output_dir.to_path_buf(),
            site_title: "KoShelf".to_string(),
            include_unread: true,
            library_paths: vec![library_path.to_path_buf()],
            metadata_location: MetadataLocation::InBookFolder,
            document_metadata_folder: None,
            statistics_db_paths: vec![],
            kobo_db_path: None,
            heatmap_scale_max: None,
            time_config: TimeConfig::from_cli(&None, &None).expect("time config"),
            min_pages_per_day: None,
            min_time_per_day: None,
            include_all_stats: false,
            is_internal_server: false,
            language: "en_US".to_string(),
            use_stable_page_metadata: true,
            auth_enabled: false,
            writeback_enabled: false,
            library_management_enabled: false,
            archive_enabled: true,
            include_files: false,
            runtime_data_policy,
        }
    }

    fn write_minimal_epub(path: &Path) {
        let file = File::create(path).expect("epub file");
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        zip.start_file("META-INF/container.xml", options)
            .expect("container start");
        zip.write_all(
            br#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
        )
        .expect("container write");

        zip.start_file("OPS/content.opf", options)
            .expect("opf start");
        zip.write_all(
            br#"<?xml version="1.0"?>
<package version="3.0" xmlns="http://www.idpf.org/2007/opf">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Extensionless Kobo EPUB</dc:title>
    <dc:creator>Fixture Author</dc:creator>
    <dc:language>en</dc:language>
  </metadata>
  <manifest/>
  <spine/>
</package>"#,
        )
        .expect("opf write");

        zip.finish().expect("zip finish");
    }

    fn write_minimal_fb2_zip(path: &Path) {
        let file = File::create(path).expect("fb2 zip file");
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        zip.start_file("book.fb2", options).expect("fb2 start");
        zip.write_all(
            br#"<?xml version="1.0" encoding="utf-8"?>
<FictionBook>
  <description>
    <title-info>
      <book-title>Compressed FB2</book-title>
      <author><first-name>Fixture</first-name><last-name>Author</last-name></author>
      <lang>en</lang>
    </title-info>
  </description>
  <body><section><title><p>Chapter 1</p></title><p>Text.</p></section></body>
</FictionBook>"#,
        )
        .expect("fb2 write");

        zip.finish().expect("zip finish");
    }

    fn write_metadata(path: &Path) {
        std::fs::create_dir_all(path.parent().expect("metadata parent"))
            .expect("metadata parent dir");
        std::fs::write(path, "return { annotations = {} }\n").expect("metadata file");
    }

    fn kobo_hints(is_encrypted: bool, has_content_keys: bool) -> KoboFileHints {
        KoboFileHints {
            content_id: "matched-book".to_string(),
            title: Some("Kobo Title".to_string()),
            author: Some("Kobo Author".to_string()),
            is_encrypted,
            has_content_keys,
        }
    }

    #[tokio::test]
    async fn ingests_valid_epub_from_extensionless_kobo_match() {
        let library_dir = tempfile::tempdir().expect("library dir");
        let output_dir = tempfile::tempdir().expect("output dir");

        let book_path = library_dir
            .path()
            .join(".kobo")
            .join("kepub")
            .join("matched-book");
        std::fs::create_dir_all(book_path.parent().expect("book parent")).expect("kepub dir");
        write_minimal_epub(&book_path);

        let repo = test_repo().await;
        let config = test_config(library_dir.path(), output_dir.path());
        let media_dirs = resolve_media_dirs(output_dir.path());
        std::fs::create_dir_all(&media_dirs.covers_dir).expect("covers dir");
        std::fs::create_dir_all(&media_dirs.files_dir).expect("files dir");

        let stats = ingest_items(
            &[CollectedItem {
                path: book_path,
                format: LibraryItemFormat::Epub,
                kobo_hints: Some(kobo_hints(false, false)),
                sidecar_checksum: None,
            }],
            &config,
            &repo,
            &media_dirs,
        )
        .await
        .expect("ingest paths");

        assert_eq!(stats.processed, 1);
        assert_eq!(stats.upserted, 1);
        assert_eq!(stats.errors, 0);

        let items = repo
            .list_items(&LibraryListQuery::default())
            .await
            .expect("list items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Extensionless Kobo EPUB");
    }

    #[tokio::test]
    async fn ingests_extensionless_kobo_docsettings_metadata_when_unread_excluded() {
        let library_dir = tempfile::tempdir().expect("library dir");
        let output_dir = tempfile::tempdir().expect("output dir");
        let docsettings_dir = tempfile::tempdir().expect("docsettings dir");

        let kobo_id = "428047c7-8b1e-4d15-80d1-4d12829cd185";
        let book_path = library_dir.path().join("books").join(kobo_id);
        std::fs::create_dir_all(book_path.parent().expect("book parent")).expect("books dir");
        write_minimal_epub(&book_path);
        write_metadata(
            &docsettings_dir
                .path()
                .join("mnt")
                .join("onboard")
                .join(".kobo")
                .join("kepub")
                .join(format!("{}.sdr", kobo_id))
                .join("metadata.epub.lua"),
        );

        let repo = test_repo().await;
        let mut config = test_config(library_dir.path(), output_dir.path());
        config.include_unread = false;
        config.metadata_location =
            MetadataLocation::DocSettings(docsettings_dir.path().to_path_buf());
        let media_dirs = resolve_media_dirs(output_dir.path());
        std::fs::create_dir_all(&media_dirs.covers_dir).expect("covers dir");
        std::fs::create_dir_all(&media_dirs.files_dir).expect("files dir");

        let stats = ingest_items(
            &[CollectedItem {
                path: book_path,
                format: LibraryItemFormat::Epub,
                kobo_hints: Some(kobo_hints(false, false)),
                sidecar_checksum: None,
            }],
            &config,
            &repo,
            &media_dirs,
        )
        .await
        .expect("ingest paths");

        assert_eq!(stats.processed, 1);
        assert_eq!(stats.upserted, 1);
        assert_eq!(stats.skipped_unread, 0);
        assert_eq!(stats.errors, 0);

        let items = repo
            .list_items(&LibraryListQuery::default())
            .await
            .expect("list items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Extensionless Kobo EPUB");
    }

    #[tokio::test]
    async fn ingests_fb2_zip_with_koreader_zip_metadata_when_unread_excluded() {
        let library_dir = tempfile::tempdir().expect("library dir");
        let output_dir = tempfile::tempdir().expect("output dir");

        let book_path = library_dir.path().join("Book.fb2.zip");
        write_minimal_fb2_zip(&book_path);
        write_metadata(
            &library_dir
                .path()
                .join("Book.fb2.sdr")
                .join("metadata.zip.lua"),
        );

        let repo = test_repo().await;
        let mut config = test_config(library_dir.path(), output_dir.path());
        config.include_unread = false;
        let media_dirs = resolve_media_dirs(output_dir.path());
        std::fs::create_dir_all(&media_dirs.covers_dir).expect("covers dir");
        std::fs::create_dir_all(&media_dirs.files_dir).expect("files dir");

        let stats = ingest_items(
            &[CollectedItem {
                path: book_path,
                format: LibraryItemFormat::Fb2,
                kobo_hints: None,
                sidecar_checksum: None,
            }],
            &config,
            &repo,
            &media_dirs,
        )
        .await
        .expect("ingest paths");

        assert_eq!(stats.processed, 1);
        assert_eq!(stats.upserted, 1);
        assert_eq!(stats.skipped_unread, 0);
        assert_eq!(stats.errors, 0);

        let items = repo
            .list_items(&LibraryListQuery::default())
            .await
            .expect("list items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Compressed FB2");
    }
    #[tokio::test]
    async fn migration_preserves_identity_with_older_fallback_and_annotation_removal() {
        use std::time::{Duration, UNIX_EPOCH};
        let library = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let book = library.path().join("Book.epub");
        write_minimal_epub(&book);
        let checksum = crate::source::koreader::calculate_partial_md5(&book).unwrap();
        let id = "11111111111111111111111111111111";
        let doc = library.path().join("doc");
        let hash = library.path().join("hash");
        std::fs::create_dir_all(&hash).unwrap();
        let old_path = doc.join("Book.sdr/metadata.epub.lua");
        let new_path = hash.join(format!(
            "{}/{checksum}.sdr/metadata.epub.lua",
            &checksum[..2]
        ));
        let put = |path: &Path, contents: &str, seconds| {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, contents).unwrap();
            File::options()
                .write(true)
                .open(path)
                .unwrap()
                .set_times(
                    std::fs::FileTimes::new()
                        .set_modified(UNIX_EPOCH + Duration::from_secs(seconds)),
                )
                .unwrap();
        };
        let with_note = format!(
            "return {{partial_md5_checksum='{id}', percent_finished=0.7, summary={{rating=4}}, annotations={{{{text='hello', drawer='lighten', pageno=1}}}}}}"
        );
        put(&old_path, &with_note, 100);
        let repo = test_repo().await;
        let output_path = std::fs::canonicalize(output.path()).unwrap();
        let mut config = test_config(library.path(), &output_path);
        config.include_unread = false;
        config.is_internal_server = true;
        config.metadata_location = MetadataLocation::Central {
            docsettings: doc,
            hashdocsettings: hash,
        };
        let media = resolve_media_dirs(&output_path);
        std::fs::create_dir_all(&media.covers_dir).unwrap();
        std::fs::create_dir_all(&media.files_dir).unwrap();
        let items = [CollectedItem {
            path: book,
            format: LibraryItemFormat::Epub,
            kobo_hints: None,
            sidecar_checksum: None,
        }];
        ingest_items(&items, &config, &repo, &media).await.unwrap();
        let initial = repo.get_item(id).await.unwrap().unwrap();
        let initial_annotations = repo.get_annotations(id, None).await.unwrap();
        assert_eq!(initial_annotations.len(), 1);
        // A no-op refresh repairs missing media links without changing annotation IDs.
        let link = media.files_dir.join(format!("{id}.epub"));
        assert!(link.exists());
        std::fs::remove_file(&link).unwrap();
        ingest_items(&items, &config, &repo, &media).await.unwrap();
        assert!(link.exists());
        assert_eq!(
            repo.get_annotations(id, None).await.unwrap()[0].id,
            initial_annotations[0].id
        );
        put(&new_path, &with_note, 200);
        ingest_items(&items, &config, &repo, &media).await.unwrap();
        let migrated = repo.get_item(id).await.unwrap().unwrap();
        assert_ne!(initial.metadata_revision, migrated.metadata_revision);
        std::fs::remove_file(&new_path).unwrap();
        // A changed ebook fingerprint still permits fallback to an older valid source.
        File::options()
            .write(true)
            .open(&items[0].path)
            .unwrap()
            .set_times(
                std::fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(500)),
            )
            .unwrap();
        ingest_items(&items, &config, &repo, &media).await.unwrap();
        let retained = repo.get_item(id).await.unwrap().unwrap();
        assert!(retained.has_metadata);
        assert_eq!(retained.metadata_revision, initial.metadata_revision);
        assert_eq!(retained.progress_percentage, migrated.progress_percentage);
        assert_eq!(retained.rating, migrated.rating);
        assert_eq!(repo.get_annotations(id, None).await.unwrap().len(), 1);
        // Invalid and empty primaries fall back to KOReader's `.old` backup.
        let backup = crate::source::koreader::lua_writer::backup_path(&old_path);
        put(&backup, &with_note, 250);
        for invalid in ["return {}", "not lua"] {
            put(&old_path, invalid, 300);
            ingest_items(&items, &config, &repo, &media).await.unwrap();
            assert_eq!(repo.get_annotations(id, None).await.unwrap().len(), 1);
            assert!(repo.get_item(id).await.unwrap().unwrap().has_metadata);
        }
        std::fs::remove_file(&backup).unwrap();
        put(
            &old_path,
            &format!("return {{partial_md5_checksum='{id}', annotations={{}}}}"),
            400,
        );
        ingest_items(&items, &config, &repo, &media).await.unwrap();
        assert!(repo.get_annotations(id, None).await.unwrap().is_empty());
        assert!(repo.get_item(id).await.unwrap().unwrap().has_metadata);
        std::fs::remove_file(&old_path).unwrap();
        ingest_items(&items, &config, &repo, &media).await.unwrap();
        let rows = repo.list_items(&LibraryListQuery::default()).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, id);
    }
    #[tokio::test]
    async fn parsing_overlaps_without_blocking_metadata_and_revalidates_before_commit() {
        let library = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let repo = test_repo().await;
        let config = test_config(library.path(), output.path());
        let media = resolve_media_dirs(output.path());
        std::fs::create_dir_all(&media.covers_dir).unwrap();
        let ids = [
            "11111111111111111111111111111111",
            "22222222222222222222222222222222",
        ];
        let items: Vec<_> = ids
            .iter()
            .enumerate()
            .map(|(n, id)| {
                let book = library.path().join(format!("Book{n}.epub"));
                write_minimal_epub(&book);
                let metadata = library
                    .path()
                    .join(format!("Book{n}.sdr/metadata.epub.lua"));
                std::fs::create_dir_all(metadata.parent().unwrap()).unwrap();
                std::fs::write(
                    metadata,
                    format!("return {{partial_md5_checksum='{id}',percent_finished=0.2}}"),
                )
                .unwrap();
                CollectedItem {
                    path: book,
                    format: LibraryItemFormat::Epub,
                    kobo_hints: None,
                    sidecar_checksum: None,
                }
            })
            .collect();
        let arrived = Arc::new(tokio::sync::Barrier::new(3));
        let release = Arc::new(tokio::sync::Barrier::new(3));
        let processor = ItemProcessor::new(Arc::new(
            MetadataIndices::new(&config.metadata_location).unwrap(),
        ))
        .with_parse_gate(arrived.clone(), release.clone());
        let controller = async {
            arrived.wait().await;
            // Both workers are parsing, and item detail/writeback can still acquire the gate.
            let guard = repo.metadata_guard().await;
            std::fs::write(
                library.path().join("Book0.sdr/metadata.epub.lua"),
                format!(
                    "return {{partial_md5_checksum='{}',percent_finished=0.8}}",
                    ids[0]
                ),
            )
            .unwrap();
            drop(guard);
            release.wait().await;
        };
        let (a, b, ()) = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            tokio::join!(
                process_single_item(&items[0], &processor, &config, &repo, &media),
                process_single_item(&items[1], &processor, &config, &repo, &media),
                controller,
            )
        })
        .await
        .expect("ebook parsing must not serialize under the metadata gate");
        assert_eq!(a.errors + b.errors, 0);
        assert_eq!(a.upserted + b.upserted, 2);
        assert_eq!(
            repo.get_item(ids[0])
                .await
                .unwrap()
                .unwrap()
                .progress_percentage,
            Some(0.8)
        );
    }
    async fn persistent_fixture() -> (
        tempfile::TempDir,
        SiteConfig,
        crate::store::sqlite::repo::LibraryRepository,
        crate::pipeline::media::MediaDirs,
        std::path::PathBuf,
        std::path::PathBuf,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let library = dir.path().join("books");
        std::fs::create_dir(&library).unwrap();
        let output = dir.path().join("output");
        std::fs::create_dir(&output).unwrap();
        let book = library.join("Book.epub");
        write_minimal_epub(&book);
        let sidecar = library.join("Book.sdr/metadata.epub.lua");
        std::fs::create_dir(sidecar.parent().unwrap()).unwrap();
        std::fs::write(&sidecar, "-- exact\nreturn {partial_md5_checksum='0123456789abcdef0123456789abcdef', annotations={{text='preserve me', drawer='lighten'}}}").unwrap();
        let output = std::fs::canonicalize(&output).unwrap();
        let config = test_config(&library, &output);
        let media = resolve_media_dirs(&output);
        crate::pipeline::media::create_media_directories(&media).unwrap();
        let mut repo = test_repo().await;
        repo.bundles = Some(Arc::new(
            crate::store::bundles::BundleStore::open(
                &dir.path().join("data"),
                crate::store::bundles::Retention::Forever,
            )
            .unwrap(),
        ));
        repo.preservation_config = Some(Arc::new(config.clone()));
        (dir, config, repo, media, book, sidecar)
    }
    async fn rebuild_paths(
        config: &SiteConfig,
        repo: &crate::store::sqlite::repo::LibraryRepository,
        paths: Vec<std::path::PathBuf>,
    ) {
        crate::pipeline::rebuild::rebuild(
            paths.into_iter().collect(),
            config,
            repo,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn disabling_archives_hides_saved_books_and_reenabling_restores_them() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        let (_dir, mut config, mut repo, media, book, sidecar) = persistent_fixture().await;
        sync_library(&config, &repo, &media).await.unwrap();
        std::fs::remove_file(&sidecar).unwrap();
        std::fs::remove_file(&book).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(repo.get_item(ID).await.unwrap().unwrap().is_archived());
        assert!(
            repo.find_fingerprint_by_book_path(&book.to_string_lossy())
                .await
                .unwrap()
                .is_none()
        );
        let store = repo.bundles.take().unwrap();
        let revision = store.get(ID).unwrap().unwrap().revision().unwrap();
        repo.preservation_config = None;
        config.archive_enabled = false;
        repo.remove_archived_projections().await.unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(repo.get_item(ID).await.unwrap().is_none());
        std::fs::write(&sidecar, format!("return {{partial_md5_checksum='{ID}'}}")).unwrap();
        write_minimal_epub(&book);
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(!repo.get_item(ID).await.unwrap().unwrap().is_archived());
        std::fs::remove_file(&sidecar).unwrap();
        std::fs::remove_file(&book).unwrap();
        rebuild_paths(&config, &repo, vec![book.clone()]).await;
        assert!(repo.get_item(ID).await.unwrap().is_none());
        assert_eq!(
            store.get(ID).unwrap().unwrap().revision().unwrap(),
            revision
        );
        config.archive_enabled = true;
        repo.bundles = Some(store);
        repo.preservation_config = Some(Arc::new(config.clone()));
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(repo.get_item(ID).await.unwrap().unwrap().is_archived());
        std::fs::write(&sidecar, format!("return {{partial_md5_checksum='{ID}'}}")).unwrap();
        write_minimal_epub(&book);
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(!repo.get_item(ID).await.unwrap().unwrap().is_archived());
    }

    #[tokio::test]
    async fn incremental_replacement_unlinks_old_identity_even_when_invalid() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        for valid in [false, true] {
            let (_dir, mut config, repo, _media, book, sidecar) = persistent_fixture().await;
            config.is_internal_server = true;
            let media = resolve_media_dirs(&config.output_dir);
            crate::pipeline::media::create_media_directories(&media).unwrap();
            sync_library(&config, &repo, &media).await.unwrap();
            let link = media.files_dir.join(format!("{ID}.epub"));
            assert!(link.exists());
            std::fs::remove_file(sidecar).unwrap();
            if valid {
                // A valid EPUB with different bytes/checksum.
                write_minimal_epub(&book);
                use std::io::Write;
                std::fs::OpenOptions::new()
                    .append(true)
                    .open(&book)
                    .unwrap()
                    .write_all(b"replacement")
                    .unwrap();
            } else {
                std::fs::write(&book, b"invalid replacement").unwrap();
            }
            rebuild_paths(&config, &repo, vec![book.clone()]).await;
            assert!(repo.get_item(ID).await.unwrap().unwrap().is_archived());
            assert!(std::fs::symlink_metadata(&link).is_err());
            assert_eq!(
                repo.get_annotations(ID, None).await.unwrap()[0]
                    .text
                    .as_deref(),
                Some("preserve me")
            );
            let replacement = crate::source::koreader::calculate_partial_md5(&book).unwrap();
            assert_ne!(replacement, ID);
            assert_eq!(repo.get_item(&replacement).await.unwrap().is_some(), valid);
            // Duplicate removal notifications are harmless.
            std::fs::remove_file(&book).unwrap();
            rebuild_paths(&config, &repo, vec![book.clone()]).await;
            rebuild_paths(&config, &repo, vec![book]).await;
            assert!(repo.get_item(ID).await.unwrap().unwrap().is_archived());
        }
    }

    #[tokio::test]
    async fn incremental_duplicates_survive_reconstruction_and_readable_replacement() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        for reconstruct in [false, true] {
            for replace in [false, true] {
                let (_dir, mut config, mut repo, _media, book, sidecar) =
                    persistent_fixture().await;
                config.is_internal_server = true;
                let media = resolve_media_dirs(&config.output_dir);
                crate::pipeline::media::create_media_directories(&media).unwrap();
                sync_library(&config, &repo, &media).await.unwrap();
                let stale = book.with_file_name("A-stale.epub");
                let alternative = book.with_file_name("Z-copy.epub");
                std::fs::copy(&book, &stale).unwrap();
                std::fs::copy(&book, &alternative).unwrap();
                sync_library(&config, &repo, &media).await.unwrap();
                if reconstruct {
                    let mut fresh = test_repo().await;
                    fresh.bundles = repo.bundles.clone();
                    sync_library(&config, &fresh, &media).await.unwrap();
                    repo = fresh;
                }
                std::fs::remove_file(stale).unwrap();
                std::fs::remove_file(sidecar).unwrap();
                if replace {
                    std::fs::write(&book, b"replacement").unwrap();
                } else {
                    std::fs::remove_file(&book).unwrap();
                }
                rebuild_paths(&config, &repo, vec![book]).await;
                assert!(!repo.get_item(ID).await.unwrap().unwrap().is_archived());
                assert_eq!(
                    repo.find_book_path_by_id(ID).await.unwrap().as_deref(),
                    alternative.to_str()
                );
                assert_eq!(
                    repo.bundles
                        .as_ref()
                        .unwrap()
                        .get(ID)
                        .unwrap()
                        .unwrap()
                        .record
                        .book_path,
                    alternative
                );
                assert_eq!(
                    std::fs::read_link(media.files_dir.join(format!("{ID}.epub"))).unwrap(),
                    std::fs::canonicalize(&alternative).unwrap()
                );
                assert_eq!(
                    repo.get_annotations(ID, None).await.unwrap()[0]
                        .text
                        .as_deref(),
                    Some("preserve me")
                );
            }
        }
    }

    #[tokio::test]
    async fn full_sync_replacement_explicitly_releases_old_fingerprint() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        for alternative in [false, true] {
            let (_dir, config, repo, media, book, sidecar) = persistent_fixture().await;
            sync_library(&config, &repo, &media).await.unwrap();
            if alternative {
                std::fs::copy(&book, book.with_file_name("Copy.epub")).unwrap();
            }
            std::fs::remove_file(sidecar).unwrap();
            std::fs::OpenOptions::new()
                .append(true)
                .open(&book)
                .unwrap()
                .write_all(b"replacement")
                .unwrap();
            let checksum = crate::source::koreader::calculate_partial_md5(&book).unwrap();
            sync_library(&config, &repo, &media).await.unwrap();
            assert!(repo.get_item(&checksum).await.unwrap().is_some());
            assert_eq!(
                repo.get_item(ID).await.unwrap().unwrap().is_archived(),
                !alternative
            );
            assert_eq!(
                repo.find_fingerprint_by_book_path(&book.to_string_lossy())
                    .await
                    .unwrap()
                    .unwrap()
                    .item_id,
                checksum
            );
        }
    }

    #[tokio::test]
    async fn incremental_fallback_skips_unparseable_candidate() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        let (_dir, config, repo, media, book, _) = persistent_fixture().await;
        sync_library(&config, &repo, &media).await.unwrap();
        let invalid = book.with_file_name("A-invalid.epub");
        let valid = book.with_file_name("Z-valid.epub");
        std::fs::write(&invalid, b"invalid EPUB").unwrap();
        let sidecar = invalid.with_extension("sdr").join("metadata.epub.lua");
        std::fs::create_dir_all(sidecar.parent().unwrap()).unwrap();
        std::fs::write(sidecar, format!("return {{partial_md5_checksum='{ID}'}}")).unwrap();
        std::fs::copy(&book, &valid).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        std::fs::remove_file(&book).unwrap();
        rebuild_paths(&config, &repo, vec![book]).await;
        assert!(!repo.get_item(ID).await.unwrap().unwrap().is_archived());
        assert_eq!(
            repo.find_book_path_by_id(ID).await.unwrap().as_deref(),
            valid.to_str()
        );
        assert_eq!(
            repo.get_annotations(ID, None).await.unwrap()[0]
                .text
                .as_deref(),
            Some("preserve me")
        );
    }

    #[tokio::test]
    async fn incremental_rename_registers_returning_source_before_archiving() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        let (_dir, config, repo, media, book, _) = persistent_fixture().await;
        sync_library(&config, &repo, &media).await.unwrap();
        let moved = book.with_file_name("Moved.epub");
        std::fs::rename(&book, &moved).unwrap();
        rebuild_paths(&config, &repo, vec![book, moved.clone()]).await;
        assert!(!repo.get_item(ID).await.unwrap().unwrap().is_archived());
        assert_eq!(
            repo.find_book_path_by_id(ID).await.unwrap().as_deref(),
            moved.to_str()
        );
        assert_eq!(
            repo.get_annotations(ID, None).await.unwrap()[0]
                .text
                .as_deref(),
            Some("preserve me")
        );
    }

    #[tokio::test]
    async fn incremental_sidecar_conflict_cannot_select_indexed_alternative() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        let (_dir, config, repo, media, book, _) = persistent_fixture().await;
        sync_library(&config, &repo, &media).await.unwrap();
        let copy = book.with_file_name("Copy.epub");
        std::fs::copy(&book, &copy).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        let sidecar = copy.with_extension("sdr").join("metadata.epub.lua");
        std::fs::create_dir_all(sidecar.parent().unwrap()).unwrap();
        std::fs::write(
            &sidecar,
            "return {partial_md5_checksum='aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'}",
        )
        .unwrap();
        rebuild_paths(&config, &repo, vec![sidecar]).await;
        std::fs::remove_file(&book).unwrap();
        rebuild_paths(&config, &repo, vec![book]).await;
        assert_eq!(
            repo.get_item(ID).await.unwrap().unwrap().availability,
            crate::shelf::models::Availability::Metadata
        );
        assert!(
            repo.get_item("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn incremental_displacement_unlinks_before_failed_archive_publication() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        let (_dir, mut config, repo, _media, book, sidecar) = persistent_fixture().await;
        config.is_internal_server = true;
        let media = resolve_media_dirs(&config.output_dir);
        crate::pipeline::media::create_media_directories(&media).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        std::fs::remove_file(sidecar).unwrap();
        std::fs::write(&book, b"replacement").unwrap();
        repo.bundles
            .as_ref()
            .unwrap()
            .fail_transition
            .store(true, std::sync::atomic::Ordering::SeqCst);
        let result = crate::pipeline::rebuild::rebuild(
            [book].into_iter().collect(),
            &config,
            &repo,
            None,
            None,
            None,
        )
        .await;
        assert!(result.is_err());
        assert!(std::fs::symlink_metadata(media.files_dir.join(format!("{ID}.epub"))).is_err());
    }

    #[tokio::test]
    async fn preservation_deletion_order_and_cache_rebuild_keep_identical_reading_data() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        for sidecar_first in [true, false] {
            let (_dir, config, repo, media, book, sidecar) = persistent_fixture().await;
            sync_library(&config, &repo, &media).await.unwrap();
            assert_eq!(repo.bundles.as_ref().unwrap().records().unwrap().len(), 1);
            if sidecar_first {
                std::fs::remove_file(&sidecar).unwrap();
            } else {
                std::fs::remove_file(&book).unwrap();
            }
            sync_library(&config, &repo, &media).await.unwrap();
            assert_eq!(
                repo.get_item(ID).await.unwrap().unwrap().availability,
                if sidecar_first {
                    crate::shelf::models::Availability::Available
                } else {
                    crate::shelf::models::Availability::Metadata
                }
            );
            if sidecar_first {
                std::fs::remove_file(&book).unwrap();
            } else {
                std::fs::remove_file(&sidecar).unwrap();
            }
            sync_library(&config, &repo, &media).await.unwrap();
            let mut rebuilt = test_repo().await;
            rebuilt.bundles = repo.bundles.clone();
            sync_library(&config, &rebuilt, &media).await.unwrap();
            assert!(rebuilt.get_item(ID).await.unwrap().unwrap().is_archived());
            assert_eq!(
                rebuilt.get_annotations(ID, None).await.unwrap()[0]
                    .text
                    .as_deref(),
                Some("preserve me")
            );
            assert!(rebuilt.load_all_fingerprints().await.unwrap().is_empty());
        }
    }
    #[tokio::test]
    async fn preservation_follows_archived_sources_returns_and_never_imports_orphans() {
        use crate::pipeline::ingest::sync_library;
        use crate::store::bundles::State;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        let (dir, mut config, repo, media, book, sidecar) = persistent_fixture().await;
        sync_library(&config, &repo, &media).await.unwrap();
        let bytes = std::fs::read(&book).unwrap();
        std::fs::remove_file(&book).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        std::fs::write(
            &sidecar,
            format!(
                "return {{partial_md5_checksum='{ID}', annotations={{}}, summary={{rating=5}}}}"
            ),
        )
        .unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(repo.get_annotations(ID, None).await.unwrap().is_empty());
        assert_eq!(repo.get_item(ID).await.unwrap().unwrap().rating, Some(5));
        std::fs::write(&book, &bytes).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(!repo.get_item(ID).await.unwrap().unwrap().is_archived());
        assert!(
            !super::super::preservation::confirm_absent(&config, &repo, ID)
                .await
                .unwrap()
        );
        std::fs::remove_file(&book).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        std::fs::remove_file(&sidecar).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(
            super::super::preservation::confirm_absent(&config, &repo, ID)
                .await
                .unwrap()
        );
        repo.bundles
            .as_ref()
            .unwrap()
            .transition(ID, State::Retired)
            .unwrap();
        repo.bundles.as_ref().unwrap().expire().unwrap();
        // A stale cache must stay removed even when an unrelated root is unavailable.
        config.library_paths.push(dir.path().join("unavailable"));
        // Simulate a crash before the retired projection was removed.
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(repo.get_item(ID).await.unwrap().is_none());
        assert!(!sidecar.exists());
        let mut rebuilt = test_repo().await;
        rebuilt.bundles = repo.bundles.clone();
        sync_library(&config, &rebuilt, &media).await.unwrap();
        assert!(rebuilt.load_all_item_ids().await.unwrap().is_empty());
        std::fs::write(&sidecar, format!("return {{partial_md5_checksum='{ID}'}}")).unwrap();
        std::fs::write(&book, &bytes).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(!repo.get_item(ID).await.unwrap().unwrap().is_archived());
    }
    #[tokio::test]
    async fn incremental_archive_captures_surviving_sidecar_in_same_batch() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        for source in ["updated", "invalid", "missing"] {
            let (_dir, config, repo, media, book, sidecar) = persistent_fixture().await;
            sync_library(&config, &repo, &media).await.unwrap();
            match source {
                "updated" => std::fs::write(&sidecar, format!(
                    "return {{partial_md5_checksum='{ID}', annotations={{}}, summary={{rating=5}}}}"
                )).unwrap(),
                "invalid" => std::fs::write(&sidecar, "not lua").unwrap(),
                _ => std::fs::remove_file(&sidecar).unwrap(),
            }
            std::fs::remove_file(&book).unwrap();
            rebuild_paths(&config, &repo, vec![book, sidecar.clone()]).await;
            let updated = source == "updated";
            let expected_rating = updated.then_some(5);
            let expected_annotations = usize::from(!updated);
            let item = repo.get_item(ID).await.unwrap().unwrap();
            assert_eq!(
                item.availability,
                match source {
                    "updated" => crate::shelf::models::Availability::Metadata,
                    "invalid" => crate::shelf::models::Availability::Available,
                    _ => crate::shelf::models::Availability::Archived,
                }
            );
            assert_eq!(item.rating, expected_rating, "{source}");
            assert_eq!(
                repo.get_annotations(ID, None).await.unwrap().len(),
                expected_annotations
            );
            let saved = repo
                .bundles
                .as_ref()
                .unwrap()
                .get(ID)
                .unwrap()
                .unwrap()
                .item()
                .unwrap();
            assert_eq!(saved.rating(), updated.then_some(5));
            assert_eq!(saved.annotations().len(), expected_annotations);
            if sidecar.exists() {
                std::fs::remove_file(&sidecar).unwrap();
            }
            let mut rebuilt = test_repo().await;
            rebuilt.bundles = repo.bundles.clone();
            sync_library(&config, &rebuilt, &media).await.unwrap();
            let item = rebuilt.get_item(ID).await.unwrap().unwrap();
            assert!(item.is_archived());
            assert_eq!(item.rating, expected_rating);
            assert_eq!(
                rebuilt.get_annotations(ID, None).await.unwrap().len(),
                expected_annotations
            );
        }
    }

    #[tokio::test]
    async fn full_sync_tries_duplicate_fallback_before_archiving() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        for has_valid_copy in [true, false] {
            let (_dir, config, repo, media, book, _) = persistent_fixture().await;
            sync_library(&config, &repo, &media).await.unwrap();
            let invalid = book.with_file_name("A-invalid.epub");
            std::fs::write(&invalid, b"invalid EPUB").unwrap();
            let sidecar = invalid.with_extension("sdr").join("metadata.epub.lua");
            std::fs::create_dir_all(sidecar.parent().unwrap()).unwrap();
            std::fs::write(sidecar, format!("return {{partial_md5_checksum='{ID}'}}")).unwrap();
            let alternative = book.with_file_name("Z-alternative.epub");
            if has_valid_copy {
                std::fs::copy(&book, &alternative).unwrap();
            } else {
                std::fs::copy(&invalid, &alternative).unwrap();
                let sidecar = alternative.with_extension("sdr").join("metadata.epub.lua");
                std::fs::create_dir_all(sidecar.parent().unwrap()).unwrap();
                std::fs::write(sidecar, format!("return {{partial_md5_checksum='{ID}'}}")).unwrap();
            }
            sync_library(&config, &repo, &media).await.unwrap();
            assert_eq!(
                repo.find_book_path_by_id(ID).await.unwrap().as_deref(),
                book.to_str()
            );
            std::fs::remove_file(&book).unwrap();
            sync_library(&config, &repo, &media).await.unwrap();
            assert_eq!(
                repo.get_item(ID).await.unwrap().unwrap().availability,
                if has_valid_copy {
                    crate::shelf::models::Availability::Available
                } else {
                    crate::shelf::models::Availability::Metadata
                }
            );
            if has_valid_copy {
                assert_eq!(
                    repo.find_book_path_by_id(ID).await.unwrap().as_deref(),
                    alternative.to_str()
                );
            }
            assert_eq!(repo.get_annotations(ID, None).await.unwrap().len(), 1);
        }
    }

    #[tokio::test]
    async fn rebuild_reconciles_surviving_sidecars_and_document_returns() {
        use crate::pipeline::{ingest::sync_library, rebuild::rebuild};
        use std::collections::HashSet;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        let (_dir, config, repo, media, book, sidecar) = persistent_fixture().await;
        sync_library(&config, &repo, &media).await.unwrap();
        // A targeted update must not discover unrelated, unannounced documents.
        let sentinel = book.with_file_name("Unannounced.fb2");
        std::fs::write(&sentinel, r#"<FictionBook><description><title-info><book-title>Sentinel</book-title></title-info></description><body><section><p>Text</p></section></body></FictionBook>"#).unwrap();
        let bytes = std::fs::read(&book).unwrap();
        std::fs::remove_file(&book).unwrap();
        rebuild(
            HashSet::from([book.clone()]),
            &config,
            &repo,
            None,
            None,
            None,
        )
        .await
        .unwrap();
        assert_eq!(repo.load_all_item_ids().await.unwrap(), vec![ID]);
        assert_eq!(
            repo.get_item(ID).await.unwrap().unwrap().availability,
            crate::shelf::models::Availability::Metadata
        );
        // A sidecar change for a Metadata entry refreshes its saved record.
        std::fs::write(
            &sidecar,
            format!(
                "return {{partial_md5_checksum='{ID}', annotations={{}}, summary={{rating=5}}}}"
            ),
        )
        .unwrap();
        rebuild(
            HashSet::from([sidecar.clone()]),
            &config,
            &repo,
            None,
            None,
            None,
        )
        .await
        .unwrap();
        let item = repo.get_item(ID).await.unwrap().unwrap();
        assert_eq!(
            item.availability,
            crate::shelf::models::Availability::Metadata
        );
        assert_eq!(item.rating, Some(5));
        std::fs::write(&book, &bytes).unwrap();
        rebuild(
            HashSet::from([book.clone()]),
            &config,
            &repo,
            None,
            None,
            None,
        )
        .await
        .unwrap();
        assert_eq!(repo.load_all_item_ids().await.unwrap(), vec![ID]);
        let item = repo.get_item(ID).await.unwrap().unwrap();
        assert!(!item.is_archived());
        assert_eq!(item.rating, Some(5));
        assert_eq!(
            repo.bundles
                .as_ref()
                .unwrap()
                .get(ID)
                .unwrap()
                .unwrap()
                .record
                .state,
            crate::store::bundles::State::Available
        );
    }
    #[tokio::test]
    async fn removal_survives_expiry_and_stale_cache_then_readable_return() {
        use crate::pipeline::ingest::sync_library;
        use crate::store::bundles::{Retention, State};
        const ID: &str = "0123456789abcdef0123456789abcdef";
        let (dir, config, mut repo, media, book, _) = persistent_fixture().await;
        Arc::get_mut(repo.bundles.as_mut().unwrap())
            .unwrap()
            .retention = Retention::Immediate;
        sync_library(&config, &repo, &media).await.unwrap();
        let offline = dir.path().join("offline");
        std::fs::rename(book.parent().unwrap(), &offline).unwrap();
        std::fs::create_dir(book.parent().unwrap()).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(repo.get_item(ID).await.unwrap().unwrap().is_archived());
        assert!(
            super::super::preservation::confirm_absent(&config, &repo, ID)
                .await
                .unwrap()
        );
        let store = repo.bundles.as_ref().unwrap();
        store.transition(ID, State::Retired).unwrap();
        store.expire().unwrap();
        assert_eq!(
            std::fs::read_dir(store.root.join("retired"))
                .unwrap()
                .count(),
            0
        );
        // Simulate an older disposable database restored after removal, including
        // one predating its managed revision. Absence needs no deletion marker.
        sqlx::query("UPDATE library_items SET managed_revision = NULL WHERE id = ?")
            .bind(ID)
            .execute(repo.pool())
            .await
            .unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(repo.get_item(ID).await.unwrap().is_none());
        std::fs::remove_dir(book.parent().unwrap()).unwrap();
        std::fs::rename(&offline, book.parent().unwrap()).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(!repo.get_item(ID).await.unwrap().unwrap().is_archived());
        assert_eq!(repo.get_annotations(ID, None).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn corrupt_capture_and_failed_storage_enumeration_do_not_delete_cached_record() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        let (_dir, config, repo, media, book, _) = persistent_fixture().await;
        sync_library(&config, &repo, &media).await.unwrap();
        std::fs::remove_file(book).unwrap();
        let store = repo.bundles.as_ref().unwrap();
        let saved = store.get(ID).unwrap().unwrap();
        std::fs::write(saved.directory.join("book.json"), b"invalid").unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(repo.get_item(ID).await.unwrap().is_some());
        std::fs::rename(store.root.join("snapshots"), store.root.join("unavailable")).unwrap();
        assert!(sync_library(&config, &repo, &media).await.is_err());
        assert!(repo.get_item(ID).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn preservation_inaccessible_roots_keep_availability_and_identity() {
        use crate::pipeline::ingest::sync_library;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        let (dir, mut config, repo, media, book, sidecar) = persistent_fixture().await;
        sync_library(&config, &repo, &media).await.unwrap();
        std::fs::remove_file(&sidecar).unwrap();
        let moved = book.with_file_name("Moved.epub");
        std::fs::rename(&book, &moved).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert_eq!(repo.load_all_item_ids().await.unwrap(), vec![ID]);
        assert!(!repo.get_item(ID).await.unwrap().unwrap().is_archived());
        assert_eq!(repo.get_annotations(ID, None).await.unwrap().len(), 1);
        assert_eq!(
            repo.find_book_path_by_id(ID).await.unwrap().unwrap(),
            moved.to_string_lossy()
        );
        assert_eq!(
            repo.bundles
                .as_ref()
                .unwrap()
                .get(ID)
                .unwrap()
                .unwrap()
                .record
                .book_path,
            moved
        );
        let offline = dir.path().join("offline-library");
        let library_root = moved.parent().unwrap();
        std::fs::rename(library_root, &offline).unwrap();
        config.library_paths.push(dir.path().join("unmounted"));
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(!repo.get_item(ID).await.unwrap().unwrap().is_archived());
        assert!(
            !super::super::preservation::confirm_absent(&config, &repo, ID)
                .await
                .unwrap()
        );
        std::fs::rename(&offline, library_root).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(!repo.get_item(ID).await.unwrap().unwrap().is_archived());
        assert_eq!(repo.get_annotations(ID, None).await.unwrap().len(), 1);
    }
    #[tokio::test]
    async fn preservation_moves_without_valid_metadata_restore_paths_files_and_saved_data() {
        use crate::pipeline::{
            export::{ExportConfig, export_data_files},
            ingest::sync_library,
        };
        const ID: &str = "0123456789abcdef0123456789abcdef";
        for (invalid, archived) in [(false, false), (true, false), (false, true)] {
            let (dir, mut config, repo, _, book, sidecar) = persistent_fixture().await;
            // Server mode serves ebooks through links in the media cache.
            config.is_internal_server = true;
            let media = resolve_media_dirs(&config.output_dir);
            crate::pipeline::media::create_media_directories(&media).unwrap();
            let served = media.files_dir.join(format!("{ID}.epub"));
            sync_library(&config, &repo, &media).await.unwrap();
            let store = repo.bundles.as_ref().unwrap();
            let saved_bytes = std::fs::read(&sidecar).unwrap();
            let ebook_bytes = std::fs::read(&book).unwrap();
            std::fs::remove_file(&sidecar).unwrap();
            // An unchanged ebook must keep its last accepted metadata too.
            sync_library(&config, &repo, &media).await.unwrap();
            assert_eq!(repo.get_annotations(ID, None).await.unwrap().len(), 1);
            let moved = book.with_file_name("Moved.epub");
            std::fs::remove_file(&book).unwrap();
            if archived {
                sync_library(&config, &repo, &media).await.unwrap();
                assert!(repo.get_item(ID).await.unwrap().unwrap().is_archived());
                assert!(std::fs::symlink_metadata(&served).is_err());
            }
            std::fs::write(&moved, &ebook_bytes).unwrap();
            let moved_sidecar = moved.with_extension("sdr").join("metadata.epub.lua");
            if invalid {
                std::fs::create_dir_all(moved_sidecar.parent().unwrap()).unwrap();
                std::fs::write(&moved_sidecar, "invalid lua").unwrap();
            }
            for _ in 0..2 {
                sync_library(&config, &repo, &media).await.unwrap();
                assert_eq!(repo.load_all_item_ids().await.unwrap(), vec![ID]);
                let item = repo.get_item(ID).await.unwrap().unwrap();
                assert!(!item.is_archived());
                assert!(!item.has_metadata);
                assert_eq!(
                    repo.find_book_path_by_id(ID).await.unwrap().unwrap(),
                    moved.to_string_lossy()
                );
                assert_eq!(
                    repo.load_all_fingerprints().await.unwrap()[0].book_path,
                    moved.to_string_lossy()
                );
                let bundle = store.get(ID).unwrap().unwrap();
                assert_eq!(bundle.record.book_path, moved);
                assert_eq!(
                    std::fs::read(bundle.directory.join("metadata.epub.lua")).unwrap(),
                    saved_bytes
                );
                assert_eq!(
                    repo.get_annotations(ID, None).await.unwrap()[0]
                        .text
                        .as_deref(),
                    Some("preserve me")
                );
            }
            let mut rebuilt = test_repo().await;
            rebuilt.bundles = repo.bundles.clone();
            sync_library(&config, &rebuilt, &media).await.unwrap();
            assert_eq!(
                rebuilt.find_book_path_by_id(ID).await.unwrap().unwrap(),
                moved.to_string_lossy()
            );
            assert_eq!(
                rebuilt.get_annotations(ID, None).await.unwrap()[0]
                    .text
                    .as_deref(),
                Some("preserve me")
            );
            assert_eq!(std::fs::read(&served).unwrap(), ebook_bytes);
            let output = std::fs::canonicalize(dir.path())
                .unwrap()
                .join("fresh-export");
            export_data_files(
                &output.join("data"),
                &output,
                &rebuilt,
                None,
                &ExportConfig {
                    site_title: "Test".into(),
                    language: "en".into(),
                    include_files: true,
                },
            )
            .await
            .unwrap();
            assert_eq!(
                std::fs::read(output.join(format!("assets/files/{ID}.epub"))).unwrap(),
                ebook_bytes
            );
            assert!(!sidecar.exists());
            if invalid {
                assert_eq!(
                    std::fs::read_to_string(moved_sidecar).unwrap(),
                    "invalid lua"
                );
            } else {
                assert!(!moved_sidecar.exists());
            }
        }
    }
    #[tokio::test]
    async fn preservation_archives_switch_central_layouts_and_accept_older_fallback_after_cache_reset()
     {
        use crate::pipeline::ingest::sync_library;
        use std::time::{Duration, UNIX_EPOCH};
        const ID: &str = "0123456789abcdef0123456789abcdef";
        let (dir, mut config, repo, media, book, sidecar) = persistent_fixture().await;
        sync_library(&config, &repo, &media).await.unwrap();
        let checksum = repo
            .bundles
            .as_ref()
            .unwrap()
            .get(ID)
            .unwrap()
            .unwrap()
            .record
            .checksum;
        let doc = dir.path().join("doc");
        let hash = dir.path().join("hash");
        let a = doc.join("Book.sdr/metadata.epub.lua");
        let b = hash.join(format!(
            "{}/{}.sdr/metadata.epub.lua",
            &checksum[..2],
            checksum
        ));
        config.metadata_location = MetadataLocation::Central {
            docsettings: doc,
            hashdocsettings: hash,
        };
        let put = |path: &Path, rating: u8, seconds: u64| {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(
                path,
                format!("return {{partial_md5_checksum='{ID}',summary={{rating={rating}}}}}"),
            )
            .unwrap();
            std::fs::File::options()
                .write(true)
                .open(path)
                .unwrap()
                .set_times(
                    std::fs::FileTimes::new()
                        .set_modified(UNIX_EPOCH + Duration::from_secs(seconds)),
                )
                .unwrap();
        };
        put(&a, 1, 100);
        put(&b, 5, 200);
        std::fs::remove_file(&book).unwrap();
        std::fs::remove_file(&sidecar).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert_eq!(
            repo.get_item(ID).await.unwrap().unwrap().availability,
            crate::shelf::models::Availability::Metadata
        );
        assert_eq!(repo.get_item(ID).await.unwrap().unwrap().rating, Some(5));
        std::fs::remove_file(&b).unwrap();
        let mut rebuilt = test_repo().await;
        rebuilt.bundles = repo.bundles.clone();
        sync_library(&config, &rebuilt, &media).await.unwrap();
        assert_eq!(rebuilt.get_item(ID).await.unwrap().unwrap().rating, Some(1));
        put(&b, 4, 300);
        put(&a, 3, 300);
        sync_library(&config, &rebuilt, &media).await.unwrap();
        assert_eq!(rebuilt.get_item(ID).await.unwrap().unwrap().rating, Some(3));
        std::fs::write(&a, "invalid lua").unwrap();
        sync_library(&config, &rebuilt, &media).await.unwrap();
        assert_eq!(rebuilt.get_item(ID).await.unwrap().unwrap().rating, Some(4));
        std::fs::remove_file(&b).unwrap();
        sync_library(&config, &rebuilt, &media).await.unwrap();
        assert_eq!(rebuilt.get_item(ID).await.unwrap().unwrap().rating, Some(4));
        assert_eq!(rebuilt.load_all_item_ids().await.unwrap(), vec![ID]);
    }
    #[tokio::test]
    async fn preservation_backfills_unchanged_imports_and_metadata_only_books() {
        use crate::pipeline::ingest::sync_library;
        let (_dir, config, mut repo, media, book, sidecar) = persistent_fixture().await;
        let store = repo.bundles.take().unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert!(store.records().unwrap().is_empty());
        repo.bundles = Some(store.clone());
        sync_library(&config, &repo, &media).await.unwrap();
        assert_eq!(store.records().unwrap().len(), 1);
        let b = store.records().unwrap().remove(0);
        let annotations = repo.get_annotations(&b.record.id, None).await.unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert_eq!(
            repo.get_annotations(&b.record.id, None).await.unwrap()[0].id,
            annotations[0].id
        );
        assert_eq!(
            std::fs::read(b.directory.join("metadata.epub.lua")).unwrap(),
            std::fs::read(&sidecar).unwrap()
        );
        // A different ebook with no sidecar is included when include_unread is enabled.
        let plain = book.with_file_name("Unread.fb2");
        std::fs::write(&plain, r#"<FictionBook><description><title-info><book-title>Unread</book-title></title-info></description><body><section><p>Unread text</p></section></body></FictionBook>"#).unwrap();
        sync_library(&config, &repo, &media).await.unwrap();
        assert_eq!(store.records().unwrap().len(), 2);
        assert!(
            store
                .records()
                .unwrap()
                .iter()
                .any(|b| b.record.source.is_none())
        );
    }
    #[tokio::test]
    async fn preservation_recovers_committed_bundle_when_cache_publication_was_interrupted() {
        use crate::{
            pipeline::ingest::sync_library,
            source::metadata::{CapturedMetadata, SourceKind},
            store::bundles::State,
        };
        const ID: &str = "0123456789abcdef0123456789abcdef";
        let (_dir, config, repo, media, book, sidecar) = persistent_fixture().await;
        sync_library(&config, &repo, &media).await.unwrap();
        let store = repo.bundles.as_ref().unwrap();
        std::fs::remove_file(&book).unwrap();
        let archived = store.transition(ID, State::Archived).unwrap();
        // The cache has not received the lifecycle transition or new captured data.
        assert!(!repo.get_item(ID).await.unwrap().unwrap().is_archived());
        std::fs::write(
            &sidecar,
            format!("return {{partial_md5_checksum='{ID}', summary={{rating=5}}}}"),
        )
        .unwrap();
        let capture = CapturedMetadata::capture(&sidecar, SourceKind::Adjacent).unwrap();
        store
            .capture(
                &archived.item().unwrap(),
                &archived.record.checksum,
                Some(&capture),
                None,
                State::Archived,
            )
            .unwrap();
        std::fs::remove_file(&sidecar).unwrap();
        let update = sync_library(&config, &repo, &media).await.unwrap();
        assert!(update.ingest_stats.unwrap().stats_invalidated > 0);
        let item = repo.get_item(ID).await.unwrap().unwrap();
        assert!(item.is_archived());
        assert_eq!(item.rating, Some(5));
    }
    #[tokio::test]
    async fn sidecar_only_pdf_lifecycle_editing_and_export() {
        use crate::pipeline::ingest::{refresh_capture, sync_library};
        use crate::shelf::models::{Availability, DocumentState};
        const ID: &str = "0123456789abcdef0123456789abcdef";
        for persistent in [false, true] {
            let (_dir, mut config, mut repo, media, book, sidecar) = persistent_fixture().await;
            std::fs::remove_file(&book).unwrap();
            std::fs::remove_file(&sidecar).unwrap();
            if !persistent {
                repo.bundles = None;
                config.archive_enabled = false;
            }
            config.is_internal_server = true;
            let pdf = book.with_extension("pdf");
            let sidecar = sidecar.with_file_name("metadata.pdf.lua");
            let contents = format!(
                "return {{partial_md5_checksum='{ID}', doc_props={{title='PDF title',authors='One\\nTwo',series='Series',series_index=2}}, doc_pages=42, annotations={{{{text='PDF text',page=4,pos0={{page=4,x=0.1}},pos1={{page=5,x=0.2}},note='Note'}}}}}}"
            );
            std::fs::write(&sidecar, &contents).unwrap();
            let result = sync_library(&config, &repo, &media).await.unwrap();
            assert_eq!(result.ingest_stats.unwrap().errors, 0);
            let item = repo.get_item(ID).await.unwrap().unwrap();
            assert_eq!(item.availability, Availability::Metadata);
            assert_eq!(item.title, "PDF title");
            assert_eq!(item.format, "pdf");
            assert!(repo.load_all_item_file_info().await.unwrap().is_empty());
            if persistent {
                let mut rebuilt = test_repo().await;
                rebuilt.bundles = repo.bundles.clone();
                sync_library(&config, &rebuilt, &media).await.unwrap();
                assert_eq!(
                    rebuilt.get_item(ID).await.unwrap().unwrap().availability,
                    Availability::Metadata
                );
            }
            let export = crate::pipeline::export::ExportConfig {
                site_title: "Test".into(),
                language: "en_US".into(),
                include_files: true,
            };
            crate::pipeline::export::export_data_files(
                &config.output_dir.join("data"),
                &config.output_dir,
                &repo,
                None,
                &export,
            )
            .await
            .unwrap();
            assert!(std::fs::symlink_metadata(media.files_dir.join(format!("{ID}.pdf"))).is_err());
            let annotations = repo.get_annotations(ID, None).await.unwrap();
            assert_eq!(annotations.len(), 1);
            assert_eq!(annotations[0].pageno, Some(4));
            assert!(annotations[0].pos0.is_none());
            // Capture after a metadata edit works without trying to hash a PDF.
            let indices = repo.metadata_indices().unwrap();
            let capture = indices
                .resolve(&pdf, LibraryItemFormat::Other, ID, Some(ID))
                .selected
                .unwrap();
            let guard = repo.metadata_guard().await;
            refresh_capture(&repo, ID, Some(&capture), &config.time_config)
                .await
                .unwrap();
            drop(guard);
            assert_eq!(
                repo.get_item(ID).await.unwrap().unwrap().availability,
                Availability::Metadata
            );
            std::fs::write(&pdf, b"synthetic PDF document").unwrap();
            sync_library(&config, &repo, &media).await.unwrap();
            assert_eq!(
                repo.get_item(ID).await.unwrap().unwrap().availability,
                Availability::Available
            );
            assert_eq!(repo.load_all_item_ids().await.unwrap(), vec![ID]);
            assert!(media.files_dir.join(format!("{ID}.pdf")).exists());
            std::fs::remove_file(&pdf).unwrap();
            sync_library(&config, &repo, &media).await.unwrap();
            assert_eq!(
                repo.get_item(ID).await.unwrap().unwrap().availability,
                Availability::Metadata
            );
            assert!(std::fs::symlink_metadata(media.files_dir.join(format!("{ID}.pdf"))).is_err());
            if persistent {
                assert_eq!(
                    repo.bundles
                        .as_ref()
                        .unwrap()
                        .get(ID)
                        .unwrap()
                        .unwrap()
                        .record
                        .document_state,
                    DocumentState::Missing
                );
            }
            std::fs::remove_file(&sidecar).unwrap();
            sync_library(&config, &repo, &media).await.unwrap();
            if persistent {
                assert_eq!(
                    repo.get_item(ID).await.unwrap().unwrap().availability,
                    Availability::Archived
                );
                std::fs::write(&sidecar, &contents).unwrap();
                sync_library(&config, &repo, &media).await.unwrap();
                assert_eq!(
                    repo.get_item(ID).await.unwrap().unwrap().availability,
                    Availability::Metadata
                );
                assert!(
                    !super::super::preservation::confirm_absent(&config, &repo, ID)
                        .await
                        .unwrap()
                );
            } else {
                assert!(repo.get_item(ID).await.unwrap().is_none());
            }
        }
    }

    #[tokio::test]
    async fn targeted_arbitrary_documents_preserve_static_exports_and_match_full_sync() {
        use crate::pipeline::export::{ExportConfig, export_data_files};
        use crate::pipeline::ingest::{reconcile_paths, sync_library};
        use crate::shelf::models::Availability;
        use std::collections::HashSet;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        for persistent in [false, true] {
            for suffix in ["txt", ""] {
                let (_dir, mut config, mut repo, media, original, old_sidecar) =
                    persistent_fixture().await;
                std::fs::remove_file(&original).unwrap();
                std::fs::remove_file(&old_sidecar).unwrap();
                if !persistent {
                    repo.bundles = None;
                    config.archive_enabled = false;
                }
                let document = original.with_file_name(if suffix.is_empty() {
                    "Book"
                } else {
                    "Book.txt"
                });
                let sidecar = old_sidecar.with_file_name(format!(
                    "metadata.{}.lua",
                    if suffix.is_empty() { "_" } else { suffix }
                ));
                let contents = format!(
                    "return {{partial_md5_checksum='{ID}',doc_props={{title='Text document'}},annotations={{{{text='quote',pageno=3}}}}}}"
                );
                std::fs::write(&sidecar, &contents).unwrap();
                sync_library(&config, &repo, &media).await.unwrap();
                let export = ExportConfig {
                    site_title: "Test".into(),
                    language: "en_US".into(),
                    include_files: true,
                };
                let filename = crate::pipeline::media::item_file_basename(ID, suffix).unwrap();
                let file = media.files_dir.join(filename);
                let data = config.output_dir.join("data");
                for (present, expected) in [
                    (true, Availability::Available),
                    (false, Availability::Metadata),
                    (true, Availability::Available),
                ] {
                    if present {
                        std::fs::write(&document, b"original download bytes").unwrap();
                    } else {
                        std::fs::remove_file(&document).unwrap();
                    }
                    let result =
                        reconcile_paths(&config, &repo, &media, &HashSet::from([document.clone()]))
                            .await
                            .unwrap();
                    assert_eq!(result.ingest_stats.unwrap().errors, 0);
                    let item = repo.get_item(ID).await.unwrap().unwrap();
                    assert_eq!(
                        item.availability, expected,
                        "persistent={persistent} suffix={suffix}"
                    );
                    assert_eq!(repo.get_annotations(ID, None).await.unwrap().len(), 1);
                    export_data_files(&data, &config.output_dir, &repo, None, &export)
                        .await
                        .unwrap();
                    let list: serde_json::Value = serde_json::from_slice(
                        &std::fs::read(data.join("items/index.json")).unwrap(),
                    )
                    .unwrap();
                    assert_eq!(
                        list[0]["availability"],
                        if present { "available" } else { "metadata" }
                    );
                    assert!(list[0].get("is_archived").is_none());
                    if present {
                        assert_eq!(std::fs::read(&file).unwrap(), b"original download bytes");
                    } else {
                        assert!(!file.exists());
                    }
                    sync_library(&config, &repo, &media).await.unwrap();
                    let full = repo.get_item(ID).await.unwrap().unwrap();
                    assert_eq!(full.availability, item.availability);
                    assert_eq!(full.title, item.title);
                    assert_eq!(full.format, item.format);
                }
                let without_files = ExportConfig {
                    include_files: false,
                    ..export
                };
                export_data_files(&data, &config.output_dir, &repo, None, &without_files)
                    .await
                    .unwrap();
                assert!(!file.exists());
                std::fs::remove_file(&document).unwrap();
                std::fs::remove_file(&sidecar).unwrap();
                reconcile_paths(
                    &config,
                    &repo,
                    &media,
                    &HashSet::from([document.clone(), sidecar.clone()]),
                )
                .await
                .unwrap();
                if persistent {
                    assert_eq!(
                        repo.get_item(ID).await.unwrap().unwrap().availability,
                        Availability::Archived
                    );
                } else {
                    assert!(repo.get_item(ID).await.unwrap().is_none());
                }
                std::fs::write(&sidecar, contents).unwrap();
                reconcile_paths(&config, &repo, &media, &HashSet::from([sidecar]))
                    .await
                    .unwrap();
                assert_eq!(
                    repo.get_item(ID).await.unwrap().unwrap().availability,
                    Availability::Metadata
                );
            }
        }
    }

    #[tokio::test]
    async fn targeted_central_sidecar_backup_and_custom_updates_leave_other_books_untouched() {
        use crate::pipeline::ingest::{reconcile_paths, sync_library};
        use crate::source::scanner::MetadataLocation;
        use std::collections::HashSet;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        for persistent in [false, true] {
            let (dir, mut config, mut repo, media, book, adjacent) = persistent_fixture().await;
            std::fs::remove_file(&book).unwrap();
            std::fs::remove_file(&adjacent).unwrap();
            if !persistent {
                repo.bundles = None;
                config.archive_enabled = false;
            }
            let central = dir.path().join("central");
            let sidecar = central.join("device/Book.sdr/metadata.pdf.lua");
            std::fs::create_dir_all(sidecar.parent().unwrap()).unwrap();
            std::fs::write(
                &sidecar,
                format!("return {{partial_md5_checksum='{ID}',doc_props={{title='PDF'}}}}"),
            )
            .unwrap();
            config.metadata_location = MetadataLocation::DocSettings(central);
            sync_library(&config, &repo, &media).await.unwrap();
            let sentinel = book.with_file_name("Unannounced.epub");
            write_minimal_epub(&sentinel);
            let backup = sidecar.with_file_name("metadata.pdf.lua.old");
            std::fs::rename(&sidecar, &backup).unwrap();
            std::fs::write(&backup, format!("return {{partial_md5_checksum='{ID}',doc_props={{title='Updated'}},summary={{rating=5}}}}" )).unwrap();
            reconcile_paths(
                &config,
                &repo,
                &media,
                &HashSet::from([sidecar.clone(), backup.clone()]),
            )
            .await
            .unwrap();
            assert_eq!(repo.load_all_item_ids().await.unwrap(), vec![ID]);
            assert_eq!(repo.get_item(ID).await.unwrap().unwrap().title, "Updated");
            assert_eq!(repo.get_item(ID).await.unwrap().unwrap().rating, Some(5));
            let custom = sidecar.with_file_name("custom_metadata.lua");
            std::fs::write(&custom, "return {custom_props={title='Custom'}}").unwrap();
            reconcile_paths(&config, &repo, &media, &HashSet::from([custom.clone()]))
                .await
                .unwrap();
            assert_eq!(repo.get_item(ID).await.unwrap().unwrap().title, "Custom");
            std::fs::remove_file(&custom).unwrap();
            reconcile_paths(&config, &repo, &media, &HashSet::from([custom]))
                .await
                .unwrap();
            assert_eq!(repo.get_item(ID).await.unwrap().unwrap().title, "Updated");
            assert_eq!(repo.load_all_item_ids().await.unwrap(), vec![ID]);
        }
    }

    #[tokio::test]
    async fn targeted_duplicate_fallback_and_uncertain_absence_work_in_both_storage_modes() {
        use crate::pipeline::ingest::{reconcile_paths, sync_library};
        use crate::shelf::models::Availability;
        use std::collections::HashSet;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        for persistent in [false, true] {
            let (dir, mut config, mut repo, media, book, sidecar) = persistent_fixture().await;
            if !persistent {
                repo.bundles = None;
                config.archive_enabled = false;
            }
            let copy = book.with_file_name("Copy.epub");
            let copy_sidecar = book.parent().unwrap().join("Copy.sdr/metadata.epub.lua");
            std::fs::copy(&book, &copy).unwrap();
            std::fs::create_dir(copy_sidecar.parent().unwrap()).unwrap();
            std::fs::copy(&sidecar, &copy_sidecar).unwrap();
            sync_library(&config, &repo, &media).await.unwrap();
            std::fs::remove_file(&book).unwrap();
            std::fs::remove_file(&sidecar).unwrap();
            reconcile_paths(&config, &repo, &media, &HashSet::from([book, sidecar]))
                .await
                .unwrap();
            assert_eq!(
                repo.get_item(ID).await.unwrap().unwrap().availability,
                Availability::Available
            );
            assert_eq!(
                repo.find_book_path_by_id(ID).await.unwrap().as_deref(),
                copy.to_str()
            );
            let title = repo.get_item(ID).await.unwrap().unwrap().title;
            std::fs::remove_file(&copy).unwrap();
            let missing_root = dir.path().join("offline");
            config.library_paths.push(missing_root.clone());
            reconcile_paths(&config, &repo, &media, &HashSet::from([copy.clone()]))
                .await
                .unwrap();
            assert_eq!(
                repo.get_item(ID).await.unwrap().unwrap().availability,
                Availability::Available
            );
            std::fs::create_dir(missing_root).unwrap();
            reconcile_paths(&config, &repo, &media, &HashSet::from([copy]))
                .await
                .unwrap();
            let item = repo.get_item(ID).await.unwrap().unwrap();
            assert_eq!(item.availability, Availability::Metadata);
            assert_eq!(item.title, title);
            sync_library(&config, &repo, &media).await.unwrap();
            assert_eq!(
                repo.get_item(ID).await.unwrap().unwrap().availability,
                item.availability
            );
        }
    }

    #[tokio::test]
    async fn missing_extensionless_epub_keeps_detected_format_and_base() {
        use crate::pipeline::ingest::{reconcile_paths, sync_library};
        use crate::shelf::models::Availability;
        use std::collections::HashSet;
        const ID: &str = "0123456789abcdef0123456789abcdef";
        for persistent in [false, true] {
            let (_dir, mut config, mut repo, media, book, _) = persistent_fixture().await;
            if !persistent {
                repo.bundles = None;
                config.archive_enabled = false;
            }
            let extensionless = book.with_extension("");
            std::fs::rename(book, &extensionless).unwrap();
            let items = [crate::source::scanner::CollectedItem {
                path: extensionless.clone(),
                format: LibraryItemFormat::Epub,
                kobo_hints: None,
                sidecar_checksum: None,
            }];
            ingest_items(&items, &config, &repo, &media).await.unwrap();
            let title = repo.get_item(ID).await.unwrap().unwrap().title;
            std::fs::remove_file(&extensionless).unwrap();
            reconcile_paths(&config, &repo, &media, &HashSet::from([extensionless]))
                .await
                .unwrap();
            for _ in 0..2 {
                let item = repo.get_item(ID).await.unwrap().unwrap();
                assert_eq!(item.availability, Availability::Metadata);
                assert_eq!(item.format, "epub");
                assert_eq!(item.title, title);
                sync_library(&config, &repo, &media).await.unwrap();
            }
        }
    }
}
