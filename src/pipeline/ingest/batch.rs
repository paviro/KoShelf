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

pub(crate) async fn ingest_items(
    items: &[CollectedItem],
    config: &SiteConfig,
    repo: &LibraryRepository,
    media_dirs: &MediaDirs,
) -> Result<IngestStats> {
    if items.is_empty() {
        return Ok(IngestStats::default());
    }

    let metadata_indices = Arc::new(MetadataIndices::new(&config.metadata_location)?);
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
    use super::ingest_items;
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
        std::fs::write(path, "return {}\n").expect("metadata file");
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
        let media_dirs = resolve_media_dirs(output_dir.path(), config.is_internal_server);
        std::fs::create_dir_all(&media_dirs.covers_dir).expect("covers dir");
        std::fs::create_dir_all(&media_dirs.files_dir).expect("files dir");

        let stats = ingest_items(
            &[CollectedItem {
                path: book_path,
                format: LibraryItemFormat::Epub,
                kobo_hints: Some(kobo_hints(false, false)),
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
        let media_dirs = resolve_media_dirs(output_dir.path(), config.is_internal_server);
        std::fs::create_dir_all(&media_dirs.covers_dir).expect("covers dir");
        std::fs::create_dir_all(&media_dirs.files_dir).expect("files dir");

        let stats = ingest_items(
            &[CollectedItem {
                path: book_path,
                format: LibraryItemFormat::Epub,
                kobo_hints: Some(kobo_hints(false, false)),
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
        let media_dirs = resolve_media_dirs(output_dir.path(), config.is_internal_server);
        std::fs::create_dir_all(&media_dirs.covers_dir).expect("covers dir");
        std::fs::create_dir_all(&media_dirs.files_dir).expect("files dir");

        let stats = ingest_items(
            &[CollectedItem {
                path: book_path,
                format: LibraryItemFormat::Fb2,
                kobo_hints: None,
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
}
