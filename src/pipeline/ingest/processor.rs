use anyhow::{Context, Result, bail};
use log::warn;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::app::config::SiteConfig;
use crate::pipeline::ingest::batch::IngestStats;
use crate::pipeline::ingest::metadata::MetadataIndices;
use crate::pipeline::media::{self, MediaDirs};
use crate::shelf::library::upsert_captured_item;
use crate::shelf::models::{
    Availability, BookInfo, DocumentState, KoReaderMetadata, LibraryItem, LibraryItemFormat,
};
use crate::source::custom;
use crate::source::kobo::KoboFileHints;
use crate::source::koreader::merge::{normalize_partial_md5, resolve_canonical_partial_md5};
use crate::source::koreader::{calculate_partial_md5, custom_metadata};
use crate::source::metadata::CapturedMetadata;
use crate::source::parsers::{ComicParser, EpubParser, Fb2Parser, MobiParser};
use crate::source::scanner::CollectedItem;
use crate::source::sidecars;
use crate::store::sqlite::repo::LibraryRepository;

/// Each worker owns its document parsers and shares the batch's metadata index.
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
            LibraryItemFormat::Other => bail!("no parser for {}", path.display()),
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

    let sidecar_only = item.sidecar_checksum.is_some();
    // Without the document, cover and metadata come from KOReader's files only.
    let parsed = !sidecar_only && format.is_parsed();

    let guard = repo.metadata_guard().await;
    let checksum = match item
        .sidecar_checksum
        .clone()
        .map_or_else(|| calculate_partial_md5(path), Ok)
    {
        Ok(value) => value,
        Err(e) => {
            warn!("Failed to calculate checksum: {}", e);
            stats.errors = 1;
            return stats;
        }
    };
    let existing = match repo.find_import_record(path, &checksum).await {
        Ok(value) => value,
        Err(e) => {
            warn!("Failed to resolve existing book {:?}: {}", path, e);
            stats.errors = 1;
            return stats;
        }
    };
    // A sidecar never competes with a present document of the same identity.
    if sidecar_only {
        let id = existing
            .as_ref()
            .map_or(checksum.as_str(), |fp| fp.item_id.as_str());
        let duplicate = async {
            let current = repo.get_item(id).await?;
            let current_path = repo.find_book_path_by_id(id).await?;
            Ok::<_, anyhow::Error>(
                current.is_some_and(|i| i.availability == Availability::Available)
                    && current_path
                        .as_deref()
                        .is_some_and(|p| Some(p) != path.to_str() && Path::new(p).is_file()),
            )
        }
        .await;
        match duplicate {
            Ok(true) => {
                stats.skipped_duplicates = 1;
                return stats;
            }
            Ok(false) => {}
            Err(e) => {
                warn!("Failed to check for duplicate ID '{id}': {e:#}");
                stats.errors = 1;
                return stats;
            }
        }
    }
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
    let custom = custom::resolve(
        path,
        &checksum,
        resolution
            .selected
            .as_ref()
            .map(|s| s.primary_path.as_path()),
        &processor.metadata_indices,
        config.document_metadata_folder,
    );
    let previous_custom = match &existing {
        Some(fp) => repo
            .get_item(&fp.item_id)
            .await
            .ok()
            .flatten()
            .and_then(|i| i.custom_metadata),
        None => None,
    };
    let custom_unchanged = |m: &custom_metadata::Metadata| {
        m.error.is_none()
            && m.metadata_digest.as_deref() == custom.metadata_digest()
            && m.cover_digest.as_deref() == custom.cover_digest()
    };
    let writable = resolution.selected.is_some();
    let saved_revision = saved.as_ref().map(|s| s.revision.clone());
    let captured = resolution.selected;
    if sidecar_only && captured.is_none() {
        stats.skipped_unread = 1;
        return stats;
    }
    let durable_current = repo
        .bundles
        .as_ref()
        .and_then(|store| {
            existing
                .as_ref()
                .and_then(|fp| store.get(&fp.item_id).ok().flatten())
        })
        .is_some_and(|b| {
            b.record.state == crate::store::bundles::State::Available
                && (b.record.document_state == DocumentState::Present) != sidecar_only
                && b.record.book_path == path
                && b.record.checksum == checksum
                && b.record
                    .book_info
                    .custom
                    .as_ref()
                    .is_some_and(custom_unchanged)
                && (captured.as_ref().map(|c| &c.revision)
                    == b.record.source.as_ref().map(|s| &s.revision)
                    || captured.is_none())
        });
    if let Some(fp) = &existing
        && repo
            .get_item(&fp.item_id)
            .await
            .ok()
            .flatten()
            .is_some_and(|i| {
                !i.is_archived() && (i.availability == Availability::Available) != sidecar_only
            })
        && previous_custom.as_ref().is_some_and(custom_unchanged)
        && (repo.bundles.is_none() || durable_current)
    {
        let current_book = crate::source::FileFingerprint::capture(path)
            .ok()
            .map(|b| (b.size_bytes as i64, b.modified_unix_ms as i64));
        // Missing metadata can skip parsing only after the durable path and
        // lifecycle are current. Moves still need to publish their new path,
        // using the saved capture without recreating an original sidecar.
        if (!writable && saved.is_some())
            || (captured.as_ref().map(|s| &s.revision) == saved_revision.as_ref()
                && current_book == fp.book_size_bytes.zip(fp.book_modified_unix_ms))
        {
            if let Err(e) = repo.set_metadata_writable(&fp.item_id, writable).await {
                warn!("Failed to update metadata availability: {}", e);
                stats.errors = 1;
            }
            drop(guard);
            let cover_path = media_dirs.covers_dir.join(format!("{}.webp", fp.item_id));
            let cover_changed = cover_outdated(parsed, custom.cover.is_some(), path, &cover_path);
            if cover_changed {
                let ebook_cover = match &custom.cover {
                    Some(_) => None,
                    None if !parsed => None,
                    None => match processor.parse_book_info(format, path).await {
                        Ok(info) => info.cover_data,
                        Err(e) => {
                            warn_parse_failure(format, path, &e, item.kobo_hints.as_ref());
                            None
                        }
                    },
                };
                let custom_cover = custom.cover.as_ref().map(|c| c.bytes.clone());
                publish_cover(custom_cover, ebook_cover, cover_path.clone()).await;
            }
            if !sidecar_only {
                sync_item_media(&fp.item_id, path, format, config, media_dirs).await;
            }
            // Keep the archived cover copy current; the served cover stays in the media cache.
            if let Some(store) = &repo.bundles
                && cover_path.is_file()
            {
                let _guard = repo.metadata_guard().await;
                let result = async {
                    if let Some(bundle) = store.get(&fp.item_id)?
                        && (cover_changed || bundle.cover().is_none())
                    {
                        let saved = store.capture(
                            &bundle.item()?,
                            &bundle.record.checksum,
                            None,
                            Some(&cover_path),
                            bundle.record.state,
                        )?;
                        repo.note_bundle(&saved).await?;
                    }
                    Ok::<_, anyhow::Error>(())
                }
                .await;
                if let Err(e) = result {
                    warn!("Cover capture pending for {}: {e:#}", fp.item_id);
                    stats.errors += 1;
                }
            }
            return stats;
        }
    }
    let mut koreader_metadata = match captured.as_ref().map(|s| s.parse()).transpose() {
        Ok(value) => value,
        Err(e) => {
            warn!("Failed to parse captured metadata: {}", e);
            stats.errors = 1;
            return stats;
        }
    };
    if koreader_metadata.is_none()
        && let (Some(store), Some(fp)) = (&repo.bundles, &existing)
    {
        match store
            .get(&fp.item_id)
            .and_then(|b| b.map(|b| b.item()).transpose())
        {
            Ok(Some(item)) => koreader_metadata = item.koreader_metadata,
            Ok(None) => {}
            Err(e) => {
                warn!("Cannot recover saved metadata: {e:#}");
                stats.errors = 1;
                return stats;
            }
        }
    }
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
    // Sidecar-only entries keep the metadata of an document seen before.
    let saved_base = if sidecar_only {
        match &existing {
            Some(fp) => match saved_document_base(repo, &fp.item_id).await {
                Ok(base) => base,
                Err(e) => {
                    warn!("Cannot recover document metadata: {e:#}");
                    stats.errors = 1;
                    return stats;
                }
            },
            None => None,
        }
    } else {
        None
    };
    let document_state = if !sidecar_only {
        DocumentState::Present
    } else {
        saved_base
            .as_ref()
            .map_or(DocumentState::Never, |(state, _)| state.without_file())
    };
    drop(guard);
    let mut book_info = match saved_base {
        Some((_, mut info))
            if (document_state == DocumentState::Missing && format.is_parsed())
                || koreader_metadata
                    .as_ref()
                    .is_none_or(|m| m.doc_props.is_none()) =>
        {
            custom_metadata::restore_base(&mut info);
            info
        }
        _ if !parsed => sidecars::book_info(koreader_metadata.as_ref(), path),
        _ => match processor.parse_book_info(format, path).await {
            Ok(info) => info,
            Err(e) => {
                warn_parse_failure(format, path, &e, item.kobo_hints.as_ref());
                stats.errors = 1;
                return stats;
            }
        },
    };

    // Directory traversal is also preparation work: do not hold the metadata
    // gate or block an async worker while walking the configured source roots.
    let location = config.metadata_location.clone();
    let excluded: Vec<_> = repo.bundles.iter().map(|s| s.root.clone()).collect();
    let fresh_indices =
        tokio::task::spawn_blocking(move || MetadataIndices::new_excluding(&location, &excluded))
            .await;
    let guard = repo.metadata_guard().await;
    // Parsing can overlap other workers and writeback. Re-resolve all mutable
    // inputs under the commit gate before publishing this prepared item.
    let unchanged = async {
        let current_existing = repo.find_import_record(path, &checksum).await?;
        if current_existing.as_ref().map(|fp| &fp.item_id)
            != existing.as_ref().map(|fp| &fp.item_id)
        {
            return Ok::<_, anyhow::Error>(false);
        }
        let current_saved = match &current_existing {
            Some(fp) => repo.metadata_provenance(&fp.item_id).await?,
            None => None,
        };
        let current_checksum = match &item.sidecar_checksum {
            Some(checksum) => checksum.clone(),
            None => calculate_partial_md5(path)?,
        };
        let indices = fresh_indices??;
        let current = indices.resolve(
            path,
            format,
            &current_checksum,
            current_existing.as_ref().map(|fp| fp.item_id.as_str()),
        );
        let current_custom = custom::resolve(
            path,
            &current_checksum,
            current.selected.as_ref().map(|s| s.primary_path.as_path()),
            &indices,
            config.document_metadata_folder,
        );
        Ok(current_checksum == checksum
            && !current.blocked
            && current.selected.as_ref().map(|s| &s.revision)
                == captured.as_ref().map(|s| &s.revision)
            && current_saved.as_ref().map(|s| &s.revision) == saved_revision.as_ref()
            && current_custom.metadata_digest() == custom.metadata_digest()
            && current_custom.cover_digest() == custom.cover_digest()
            && (sidecar_only || book_before.is_some())
            && crate::source::FileFingerprint::capture(path).ok() == book_before)
    }
    .await;
    if !matches!(unchanged, Ok(true)) {
        drop(guard);
        if retry {
            // Refresh the batch index as migration may have created a new path.
            if let Ok(indices) = repo.metadata_indices() {
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
        Ok(Some(ref existing_path))
            if existing_path != path_str.as_ref()
                && config
                    .library_paths
                    .iter()
                    .filter_map(|p| std::path::absolute(p).ok())
                    .any(|root| std::path::Path::new(existing_path).starts_with(root))
                && std::fs::File::open(existing_path).is_ok()
                && repo.bundles.as_ref().is_none_or(|store| {
                    store.get(&item_id).ok().flatten().is_some_and(|bundle| {
                        calculate_partial_md5(Path::new(existing_path))
                            .ok()
                            .as_deref()
                            == Some(bundle.record.checksum.as_str())
                            && !processor
                                .metadata_indices
                                .resolve(
                                    Path::new(existing_path),
                                    bundle.record.format,
                                    &bundle.record.checksum,
                                    Some(&item_id),
                                )
                                .blocked
                    })
                })
                && !repo
                    .get_item(&item_id)
                    .await
                    .ok()
                    .flatten()
                    .is_some_and(|i| i.is_archived()) =>
        {
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

    custom_metadata::refresh(&mut book_info, &custom, previous_custom.as_ref());
    let ebook_cover = book_info.cover_data.take();
    let cover_path = media_dirs.covers_dir.join(format!("{item_id}.webp"));
    let cover_changed = previous_custom
        .as_ref()
        .and_then(|m| m.cover_digest.as_deref())
        != custom.cover_digest()
        || cover_outdated(parsed, custom.cover.is_some(), path, &cover_path);
    let item = LibraryItem {
        id: item_id.clone(),
        book_info,
        koreader_metadata,
        file_path: path.to_path_buf(),
        format,
        document_state,
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

    stats.upserted = 1;
    if stats_fields_changed {
        stats.stats_invalidated = 1;
    }

    // Media generation stays outside the metadata gate.
    drop(guard);
    if cover_changed {
        let custom_cover = custom.cover.as_ref().map(|c| c.bytes.clone());
        publish_cover(custom_cover, ebook_cover, cover_path.clone()).await;
    }
    if sidecar_only {
        if let Err(e) = media::remove_item_files_by_id(&item_id, &media_dirs.files_dir) {
            warn!("Failed to remove file link for {item_id}: {e:#}");
        }
    } else {
        sync_item_media(&item_id, path, format, config, media_dirs).await;
    }
    let _guard = repo.metadata_guard().await;
    if let Some(store) = &repo.bundles {
        // A writeback between the cache upsert and here captures newer bytes itself.
        let result = match captured.as_ref().map(|c| c.verify()).transpose() {
            Ok(_) => store.capture(
                &item,
                &checksum,
                captured.as_ref(),
                cover_path.is_file().then_some(cover_path.as_path()),
                crate::store::bundles::State::Available,
            ),
            Err(e) => Err(e),
        };
        match result {
            Ok(bundle) => {
                if let Err(e) = repo.note_bundle(&bundle).await {
                    warn!("Bundle projection pending for {item_id}: {e:#}");
                    stats.errors += 1;
                }
            }
            Err(e) => {
                warn!("Capture pending for {item_id}: {e:#}");
                stats.errors += 1;
            }
        }
    }
    if let Err(e) = repo.set_archived(&item_id, false).await {
        warn!("Cannot restore availability: {e:#}");
        stats.errors += 1;
    }
    stats
}

/// Whether the served cover is older than its source. Documents without a
/// parsed cover only have the custom one.
fn cover_outdated(parsed: bool, has_custom: bool, path: &Path, cover_path: &Path) -> bool {
    if parsed {
        media::cover_needs_generation(path, cover_path)
    } else {
        has_custom && !cover_path.exists()
    }
}

/// Write the served cover from the custom cover, falling back to the document's
/// own. Without either, a stale cover is removed.
async fn publish_cover(
    custom: Option<Vec<u8>>,
    document_cover: Option<Vec<u8>>,
    cover_path: PathBuf,
) {
    let result =
        tokio::task::spawn_blocking(move || write_cover(custom, document_cover, &cover_path)).await;
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => warn!("Cover encode failed: {e:#}"),
        Err(e) => warn!("Cover encode task panicked: {e}"),
    }
}

fn write_cover(
    custom: Option<Vec<u8>>,
    document_cover: Option<Vec<u8>>,
    cover_path: &Path,
) -> Result<()> {
    if let Some(data) = &custom {
        match media::encode_cover_to_disk(data, cover_path) {
            Ok(()) => return Ok(()),
            Err(e) => warn!("Invalid custom cover, using the document cover: {e:#}"),
        }
    }
    match document_cover {
        Some(data) => media::encode_cover_to_disk(&data, cover_path),
        None => match std::fs::remove_file(cover_path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
            _ => Ok(()),
        },
    }
}

async fn sync_item_media(
    item_id: &str,
    path: &Path,
    format: LibraryItemFormat,
    config: &SiteConfig,
    media_dirs: &MediaDirs,
) {
    if config.is_internal_server
        && let Err(e) = media::sync_item_file_symlink(
            item_id,
            &format.extension_for_path(path),
            path,
            &media_dirs.files_dir,
        )
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

/// Recover the document-derived base from durable storage or the current cache.
/// Archive-disabled libraries still retain that base while a sidecar survives.
async fn saved_document_base(
    repo: &LibraryRepository,
    id: &str,
) -> Result<Option<(DocumentState, BookInfo)>> {
    if let Some(store) = &repo.bundles
        && let Some(bundle) = store.get(id)?
    {
        return Ok(Some((
            bundle.record.document_state,
            bundle.record.book_info,
        )));
    }
    let Some(source) = repo.item_source_row(id).await? else {
        return Ok(None);
    };
    let state = source.document_state;
    let detail = repo.get_item(id).await?.context("saved item missing")?;
    let info = BookInfo {
        title: detail.title,
        authors: detail.authors.0,
        description: detail.description,
        language: detail.language,
        publisher: detail.publisher,
        identifiers: detail
            .identifiers
            .iter()
            .map(|i| crate::shelf::models::Identifier::new(i.scheme.clone(), i.value.clone()))
            .collect(),
        subjects: detail.subjects.0,
        series: detail.series.as_ref().map(|s| s.name.clone()),
        series_number: detail.series.and_then(|s| s.0.index),
        pages: detail.pages.and_then(|p| p.try_into().ok()),
        chapters: vec![],
        cover_data: None,
        cover_mime_type: None,
        custom: detail.custom_metadata,
    };
    Ok(Some((state, info)))
}

/// Caller holds the repository metadata guard through capture and persistence.
/// Refresh one book after a writeback while the caller holds metadata_guard.
/// `captured` is the reading sidecar just written, if the book has one.
pub(crate) async fn refresh_capture(
    repo: &LibraryRepository,
    id: &str,
    captured: Option<&CapturedMetadata>,
    time: &crate::shelf::time_config::TimeConfig,
) -> Result<()> {
    let config = repo
        .runtime_config
        .read()
        .expect("runtime configuration lock")
        .clone()
        .context("runtime configuration unavailable")?;
    let source = repo.item_source(id).await?;
    let path = PathBuf::from(&source.file_path);
    let format = source.format();
    let document_state = source.document_state;
    let checksum = source.checksum()?.to_string();
    let detail = repo.get_item(id).await?.context("book not found")?;
    let book_before = crate::source::FileFingerprint::capture_optional(&path)?;
    let indices = Arc::new(repo.metadata_indices()?);
    let processor = ItemProcessor::new(indices.clone());
    let saved = repo
        .bundles
        .as_ref()
        .map(|store| store.get(id))
        .transpose()?
        .flatten();
    let koreader_metadata = match captured {
        Some(captured) => Some(captured.parse()?),
        None => saved
            .as_ref()
            .map(|b| b.item())
            .transpose()?
            .and_then(|i| i.koreader_metadata),
    };
    let mut book_info = if document_state == DocumentState::Present && format.is_parsed() {
        processor.parse_book_info(format, &path).await?
    } else if (document_state == DocumentState::Missing && format.is_parsed())
        || koreader_metadata
            .as_ref()
            .is_none_or(|m| m.doc_props.is_none())
    {
        let (_, mut info) = saved_document_base(repo, id)
            .await?
            .context("saved document metadata missing")?;
        custom_metadata::restore_base(&mut info);
        info
    } else {
        sidecars::book_info(koreader_metadata.as_ref(), &path)
    };
    let custom = custom::resolve(
        &path,
        &checksum,
        captured.map(|c| c.primary_path.as_path()),
        &indices,
        config.document_metadata_folder,
    );
    let previous = detail.custom_metadata.as_ref();
    custom_metadata::refresh(&mut book_info, &custom, previous);
    let ebook_cover = book_info.cover_data.take();
    let item = LibraryItem {
        id: id.to_string(),
        book_info,
        koreader_metadata,
        file_path: path,
        format,
        document_state,
    };
    // Document parsing awaits I/O. External writers are not covered by our gate,
    // so recheck both the bytes and source selection before publishing the cache.
    if let Some(captured) = captured {
        captured.verify()?;
    }
    let current = repo
        .metadata_indices()?
        .resolve(&item.file_path, format, &checksum, Some(id));
    if current.selected.as_ref().map(|s| &s.revision) != captured.map(|c| &c.revision)
        || crate::source::FileFingerprint::capture_optional(&item.file_path)? != book_before
    {
        return Err(crate::source::metadata::MetadataChanged.into());
    }
    let cover_path = media::resolve_media_dirs(&config.output_dir)
        .covers_dir
        .join(format!("{id}.webp"));
    if previous.and_then(|m| m.cover_digest.as_deref()) != custom.cover_digest()
        || !cover_path.exists()
    {
        let custom_cover = custom.cover.map(|c| c.bytes);
        let target = cover_path.clone();
        tokio::task::spawn_blocking(move || write_cover(custom_cover, ebook_cover, &target))
            .await??;
    }
    upsert_captured_item(repo, &item, captured, &checksum, captured.is_some(), time).await?;
    if let Some(store) = &repo.bundles {
        let bundle = store.capture(
            &item,
            &checksum,
            captured,
            cover_path.is_file().then_some(cover_path.as_path()),
            crate::store::bundles::State::Available,
        )?;
        repo.note_bundle(&bundle).await?;
    }
    Ok(())
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
