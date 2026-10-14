use anyhow::{Context, Result, bail};
use log::warn;
use std::path::Path;
use std::sync::Arc;

use crate::app::config::SiteConfig;
use crate::pipeline::ingest::batch::IngestStats;
use crate::pipeline::ingest::metadata::MetadataIndices;
use crate::pipeline::media::{self, MediaDirs};
use crate::shelf::library::upsert_captured_item;
use crate::shelf::models::{BookInfo, KoReaderMetadata, LibraryItem, LibraryItemFormat};
use crate::source::kobo::KoboFileHints;
use crate::source::koreader::calculate_partial_md5;
use crate::source::koreader::merge::{normalize_partial_md5, resolve_canonical_partial_md5};
use crate::source::parsers::{ComicParser, EpubParser, Fb2Parser, MobiParser};
use crate::source::scanner::CollectedItem;
use crate::store::sqlite::repo::LibraryRepository;

/// Each worker owns its ebook parsers and shares the batch's metadata index.
/// Metadata is parsed from stable captures in separate sandboxed Lua runtimes.
pub(super) struct ItemProcessor {
    metadata_indices: Arc<MetadataIndices>,
    epub_parser: EpubParser,
    fb2_parser: Fb2Parser,
    comic_parser: ComicParser,
    mobi_parser: MobiParser,
    #[cfg(test)]
    parse_gate: Option<(Arc<tokio::sync::Barrier>, Arc<tokio::sync::Barrier>)>,
}

impl ItemProcessor {
    pub(super) fn new(metadata_indices: Arc<MetadataIndices>) -> Self {
        Self {
            metadata_indices,
            epub_parser: EpubParser::new(),
            fb2_parser: Fb2Parser::new(),
            comic_parser: ComicParser::new(),
            mobi_parser: MobiParser::new(),
            #[cfg(test)]
            parse_gate: None,
        }
    }

    #[cfg(test)]
    pub(super) fn with_parse_gate(
        mut self,
        arrived: Arc<tokio::sync::Barrier>,
        release: Arc<tokio::sync::Barrier>,
    ) -> Self {
        self.parse_gate = Some((arrived, release));
        self
    }

    async fn parse_book_info(&self, format: LibraryItemFormat, path: &Path) -> Result<BookInfo> {
        #[cfg(test)]
        if let Some((arrived, release)) = &self.parse_gate {
            arrived.wait().await;
            release.wait().await;
        }
        match format {
            LibraryItemFormat::Epub => self.epub_parser.parse(path).await,
            LibraryItemFormat::Fb2 => self.fb2_parser.parse(path).await,
            LibraryItemFormat::Cbz | LibraryItemFormat::Cbr => self.comic_parser.parse(path).await,
            LibraryItemFormat::Mobi => self.mobi_parser.parse(path).await,
        }
    }

    fn canonical_item_id(
        &self,
        path: &Path,
        koreader_metadata: Option<&KoReaderMetadata>,
    ) -> Result<String> {
        let metadata_md5 =
            koreader_metadata.and_then(|metadata| metadata.partial_md5_checksum.as_deref());

        if let Some(resolved) = resolve_canonical_partial_md5(metadata_md5, None) {
            return Ok(resolved.value);
        }

        if let Some(metadata_md5) = metadata_md5
            && normalize_partial_md5(metadata_md5).is_none()
        {
            warn!(
                "Invalid KOReader partial_md5_checksum '{}' for {:?}; falling back to file-derived MD5",
                metadata_md5, path
            );
        }

        let derived_md5 = calculate_partial_md5(path)
            .with_context(|| format!("Failed to derive canonical md5 ID for {:?}", path))?;

        if let Some(resolved) = resolve_canonical_partial_md5(None, Some(derived_md5.as_str())) {
            return Ok(resolved.value);
        }

        bail!(
            "Derived canonical md5 ID '{}' for {:?} is invalid; expected 32 hex characters",
            derived_md5,
            path
        );
    }
}

pub(super) async fn process_single_item(
    item: &CollectedItem,
    processor: &ItemProcessor,
    config: &SiteConfig,
    repo: &LibraryRepository,
    media_dirs: &MediaDirs,
) -> IngestStats {
    process_single_item_attempt(item, processor, config, repo, media_dirs, true).await
}

async fn process_single_item_attempt(
    item: &CollectedItem,
    processor: &ItemProcessor,
    config: &SiteConfig,
    repo: &LibraryRepository,
    media_dirs: &MediaDirs,
    retry: bool,
) -> IngestStats {
    let mut stats = IngestStats {
        processed: 1,
        ..Default::default()
    };

    let path = item.path.as_path();
    let format = item.format;

    let guard = repo.metadata_guard().await;
    let existing = match repo
        .find_fingerprint_by_book_path(&path.to_string_lossy())
        .await
    {
        Ok(value) => value,
        Err(e) => {
            warn!("Failed to resolve existing book {:?}: {}", path, e);
            stats.errors = 1;
            return stats;
        }
    };
    let saved = match existing.as_ref() {
        Some(fp) => match repo.metadata_provenance(&fp.item_id).await {
            Ok(value) => value,
            Err(e) => {
                warn!("Failed to load metadata cache: {}", e);
                stats.errors = 1;
                return stats;
            }
        },
        None => None,
    };
    let checksum = match calculate_partial_md5(path) {
        Ok(value) => value,
        Err(e) => {
            warn!("Failed to calculate checksum: {}", e);
            stats.errors = 1;
            return stats;
        }
    };
    let resolution = processor.metadata_indices.resolve(
        path,
        format,
        &checksum,
        existing.as_ref().map(|fp| fp.item_id.as_str()),
    );
    for issue in &resolution.issues {
        warn!("{}: {}", path.display(), issue);
    }
    if resolution.blocked {
        if let Some(fp) = existing.as_ref()
            && let Err(e) = repo.set_metadata_writable(&fp.item_id, false).await
        {
            warn!("Failed to block writeback: {}", e);
        }
        stats.errors = 1;
        return stats;
    }
    let writable = resolution.selected.is_some();
    let saved_revision = saved.as_ref().map(|s| s.revision.clone());
    let captured = resolution.selected;
    if let Some(fp) = &existing {
        let current_book = crate::source::FileFingerprint::capture(path).ok();
        // Without an accepted current original, leave the parsed reading data and
        // annotations intact. Never reconstitute a sidecar from the cache.
        if (!writable && saved.is_some())
            || (captured.as_ref().map(|s| &s.revision) == saved_revision.as_ref()
                && current_book.is_some_and(|b| {
                    b.size_bytes as i64 == fp.book_size_bytes
                        && b.modified_unix_ms as i64 == fp.book_modified_unix_ms
                }))
        {
            if let Err(e) = repo.set_metadata_writable(&fp.item_id, writable).await {
                warn!("Failed to update metadata availability: {}", e);
                stats.errors = 1;
            }
            drop(guard);
            let cover_path = media_dirs.covers_dir.join(format!("{}.webp", fp.item_id));
            let cover_data = if media::cover_needs_generation(path, &cover_path) {
                match processor.parse_book_info(format, path).await {
                    Ok(info) => info.cover_data,
                    Err(e) => {
                        warn_parse_failure(format, path, &e, item.kobo_hints.as_ref());
                        None
                    }
                }
            } else {
                None
            };
            sync_item_media(&fp.item_id, path, format, cover_data, config, media_dirs).await;
            return stats;
        }
    }
    let koreader_metadata = match captured.as_ref().map(|s| s.parse()).transpose() {
        Ok(value) => value,
        Err(e) => {
            warn!("Failed to parse captured metadata: {}", e);
            stats.errors = 1;
            return stats;
        }
    };
    if existing.is_none() && koreader_metadata.is_none() && !config.include_unread {
        stats.skipped_unread = 1;
        return stats;
    }
    let item_id = match existing.as_ref() {
        Some(fp) => fp.item_id.clone(),
        None => match processor.canonical_item_id(path, koreader_metadata.as_ref()) {
            Ok(id) => id,
            Err(e) => {
                warn!("Failed to derive item ID: {}", e);
                stats.errors = 1;
                return stats;
            }
        },
    };

    let book_before = crate::source::FileFingerprint::capture(path).ok();
    drop(guard);
    let mut book_info = match processor.parse_book_info(format, path).await {
        Ok(info) => info,
        Err(e) => {
            warn_parse_failure(format, path, &e, item.kobo_hints.as_ref());
            stats.errors = 1;
            return stats;
        }
    };

    // Directory traversal is also preparation work: do not hold the metadata
    // gate or block an async worker while walking the configured source roots.
    let location = config.metadata_location.clone();
    let fresh_indices = tokio::task::spawn_blocking(move || MetadataIndices::new(&location)).await;
    let guard = repo.metadata_guard().await;
    // Parsing can overlap other workers and writeback. Re-resolve all mutable
    // inputs under the commit gate before publishing this prepared item.
    let unchanged = async {
        let current_existing = repo
            .find_fingerprint_by_book_path(&path.to_string_lossy())
            .await?;
        if current_existing.as_ref().map(|fp| &fp.item_id)
            != existing.as_ref().map(|fp| &fp.item_id)
        {
            return Ok::<_, anyhow::Error>(false);
        }
        let current_saved = match &current_existing {
            Some(fp) => repo.metadata_provenance(&fp.item_id).await?,
            None => None,
        };
        let current_checksum = calculate_partial_md5(path)?;
        let current = fresh_indices??.resolve(
            path,
            format,
            &current_checksum,
            current_existing.as_ref().map(|fp| fp.item_id.as_str()),
        );
        Ok(current_checksum == checksum
            && !current.blocked
            && current.selected.as_ref().map(|s| &s.revision)
                == captured.as_ref().map(|s| &s.revision)
            && current_saved.as_ref().map(|s| &s.revision) == saved_revision.as_ref()
            && book_before.is_some()
            && crate::source::FileFingerprint::capture(path).ok() == book_before)
    }
    .await;
    if !matches!(unchanged, Ok(true)) {
        drop(guard);
        if retry {
            // Refresh the batch index as migration may have created a new path.
            if let Ok(indices) = MetadataIndices::new(&config.metadata_location) {
                let processor = ItemProcessor::new(Arc::new(indices));
                return Box::pin(process_single_item_attempt(
                    item, &processor, config, repo, media_dirs, false,
                ))
                .await;
            }
        }
        warn!("Deferring changed book/source during ingestion: {:?}", path);
        stats.errors = 1;
        return stats;
    }

    let path_str = path.to_string_lossy();
    match repo.find_book_path_by_id(&item_id).await {
        Ok(Some(ref existing_path)) if existing_path != path_str.as_ref() => {
            warn!(
                "Duplicate canonical ID '{}': already indexed at {}, skipping {:?}",
                item_id, existing_path, path
            );
            stats.skipped_duplicates = 1;
            return stats;
        }
        Err(e) => {
            warn!("Failed to check for duplicate ID '{}': {}", item_id, e);
            stats.errors = 1;
            return stats;
        }
        _ => {}
    }

    let cover_data = book_info.cover_data.take();
    let item = LibraryItem {
        id: item_id.clone(),
        book_info,
        koreader_metadata,
        file_path: path.to_path_buf(),
        format,
    };

    let stats_fields_changed = needs_stats_reload(&item, repo).await;

    if let Err(e) = upsert_captured_item(
        repo,
        &item,
        captured.as_ref(),
        &checksum,
        writable,
        &config.time_config,
    )
    .await
    {
        warn!("Failed to upsert item {:?}: {}", path, e);
        stats.errors = 1;
        return stats;
    }

    drop(guard);
    stats.upserted = 1;
    if stats_fields_changed {
        stats.stats_invalidated = 1;
    }

    sync_item_media(&item_id, path, format, cover_data, config, media_dirs).await;

    stats
}

async fn sync_item_media(
    item_id: &str,
    path: &Path,
    format: LibraryItemFormat,
    cover_data: Option<Vec<u8>>,
    config: &SiteConfig,
    media_dirs: &MediaDirs,
) {
    if let Some(cover_data) = cover_data {
        let cover_path = media_dirs.covers_dir.join(format!("{}.webp", item_id));

        if media::cover_needs_generation(path, &cover_path) {
            match tokio::task::spawn_blocking(move || {
                media::encode_cover_to_disk(&cover_data, &cover_path)
            })
            .await
            {
                Ok(Ok(())) => {}
                Ok(Err(e)) => warn!("Cover encode failed for {:?}: {}", path, e),
                Err(e) => warn!("Cover encode task panicked for {:?}: {}", path, e),
            }
        }
    }

    if config.is_internal_server
        && let Err(e) =
            media::sync_item_file_symlink(item_id, format.extension(), path, &media_dirs.files_dir)
    {
        warn!("Failed to create file symlink for {:?}: {}", path, e);
    }
}

pub(super) fn should_use_kobo_encryption_warning(kobo_hints: Option<&KoboFileHints>) -> bool {
    kobo_hints.is_some_and(KoboFileHints::suggests_encryption)
}

pub(super) fn kobo_hint_label(hints: &KoboFileHints) -> String {
    match (hints.title.as_deref(), hints.author.as_deref()) {
        (Some(title), Some(author)) => format!("{} by {}", title, author),
        (Some(title), None) => title.to_string(),
        (None, Some(author)) => format!("author {}", author),
        (None, None) => "unknown title".to_string(),
    }
}

fn warn_parse_failure(
    format: LibraryItemFormat,
    path: &Path,
    error: &anyhow::Error,
    kobo_hints: Option<&KoboFileHints>,
) {
    if should_use_kobo_encryption_warning(kobo_hints) {
        let hints = kobo_hints.expect("checked above");
        warn!(
            "Failed to parse Kobo {:?} {:?} ({}): {}. Kobo database marks this file as possibly encrypted (ContentID: {}, IsEncrypted: {}, content_keys: {}). Only plaintext EPUB metadata may be readable.",
            format,
            path,
            kobo_hint_label(hints),
            error,
            hints.content_id,
            hints.is_encrypted,
            hints.has_content_keys
        );
        return;
    }

    warn!("Failed to parse {:?} {:?}: {}", format, path, error);
}

async fn needs_stats_reload(item: &LibraryItem, repo: &LibraryRepository) -> bool {
    let old_fields = match repo.load_stats_influencing_fields(&item.id).await {
        Ok(Some(fields)) => fields,
        Ok(None) | Err(_) => return true,
    };

    let new_fields = (
        item.koreader_metadata
            .as_ref()
            .and_then(|m| m.hidden_flow_pages())
            .map(|p| p as i32),
        item.stable_display_page_total().map(|p| p as i32),
        item.synthetic_scaling_page_total().is_some(),
    );

    old_fields != new_fields
}

/// Caller holds the repository metadata guard through capture and persistence.
pub(crate) async fn refresh_capture(
    repo: &LibraryRepository,
    id: &str,
    captured: &crate::source::metadata::CapturedMetadata,
    time: &crate::shelf::time_config::TimeConfig,
) -> Result<()> {
    let path = repo
        .find_book_path_by_id(id)
        .await?
        .context("book not found")?;
    let path = std::path::PathBuf::from(path);
    let detail = repo.get_item(id).await?.context("book not found")?;
    let format = LibraryItemFormat::from_path(Path::new(&format!("book.{}", detail.format)))
        .context("unsupported book format")?;
    let book_before = crate::source::FileFingerprint::capture(&path)?;
    let processor = ItemProcessor::new(Arc::new(MetadataIndices::new(&repo.metadata_location())?));
    let book_info = processor.parse_book_info(format, &path).await?;
    let checksum = calculate_partial_md5(&path)?;
    let item = LibraryItem {
        id: id.to_string(),
        book_info,
        koreader_metadata: Some(captured.parse()?),
        file_path: path,
        format,
    };
    // Ebook parsing awaits I/O. External writers are not covered by our gate,
    // so recheck both the bytes and source selection before publishing the cache.
    captured.verify()?;
    let current = MetadataIndices::new(&repo.metadata_location())?.resolve(
        &item.file_path,
        format,
        &checksum,
        Some(id),
    );
    if current
        .selected
        .is_none_or(|source| source.revision != captured.revision)
        || crate::source::FileFingerprint::capture(&item.file_path)? != book_before
    {
        return Err(crate::source::metadata::MetadataChanged.into());
    }
    upsert_captured_item(repo, &item, Some(captured), &checksum, true, time).await
}

#[cfg(test)]
mod tests {
    use super::{kobo_hint_label, should_use_kobo_encryption_warning};
    use crate::source::kobo::KoboFileHints;

    fn kobo_hints(is_encrypted: bool, has_content_keys: bool) -> KoboFileHints {
        KoboFileHints {
            content_id: "matched-book".to_string(),
            title: Some("Kobo Title".to_string()),
            author: Some("Kobo Author".to_string()),
            is_encrypted,
            has_content_keys,
        }
    }

    #[test]
    fn parse_failure_classification_uses_kobo_warning_only_for_encryption_hints() {
        let encrypted = kobo_hints(true, false);
        let keyed = kobo_hints(false, true);
        let plain = kobo_hints(false, false);

        assert!(should_use_kobo_encryption_warning(Some(&encrypted)));
        assert!(should_use_kobo_encryption_warning(Some(&keyed)));
        assert!(!should_use_kobo_encryption_warning(Some(&plain)));
        assert!(!should_use_kobo_encryption_warning(None));
    }

    #[test]
    fn kobo_warning_label_uses_title_and_author_when_available() {
        let hints = kobo_hints(true, false);
        assert_eq!(kobo_hint_label(&hints), "Kobo Title by Kobo Author");

        let title_only = KoboFileHints {
            author: None,
            ..hints.clone()
        };
        assert_eq!(kobo_hint_label(&title_only), "Kobo Title");

        let author_only = KoboFileHints {
            title: None,
            ..hints
        };
        assert_eq!(kobo_hint_label(&author_only), "author Kobo Author");
    }
}
