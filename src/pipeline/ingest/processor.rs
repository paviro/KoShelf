use anyhow::{Context, Result, bail};
use log::{debug, warn};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::app::config::SiteConfig;
use crate::pipeline::ingest::batch::IngestStats;
use crate::pipeline::ingest::metadata::{MetadataIndices, locate_metadata_path};
use crate::pipeline::media::{self, MediaDirs};
use crate::shelf::library::upsert_single_item;
use crate::shelf::models::{BookInfo, KoReaderMetadata, LibraryItem, LibraryItemFormat};
use crate::source::kobo::KoboFileHints;
use crate::source::koreader::merge::{normalize_partial_md5, resolve_canonical_partial_md5};
use crate::source::koreader::{LuaParser, calculate_partial_md5};
use crate::source::parsers::{ComicParser, EpubParser, Fb2Parser, MobiParser};
use crate::source::scanner::CollectedItem;
use crate::store::sqlite::repo::LibraryRepository;

/// Each worker owns its own parser set. `LuaParser` contains an `mlua::Lua`
/// runtime, so processors are created per worker instead of shared.
pub(super) struct ItemProcessor {
    metadata_indices: Arc<MetadataIndices>,
    epub_parser: EpubParser,
    fb2_parser: Fb2Parser,
    comic_parser: ComicParser,
    mobi_parser: MobiParser,
    lua_parser: LuaParser,
}

impl ItemProcessor {
    pub(super) fn new(metadata_indices: Arc<MetadataIndices>) -> Self {
        Self {
            metadata_indices,
            epub_parser: EpubParser::new(),
            fb2_parser: Fb2Parser::new(),
            comic_parser: ComicParser::new(),
            mobi_parser: MobiParser::new(),
            lua_parser: LuaParser::new(),
        }
    }

    async fn parse_book_info(&self, format: LibraryItemFormat, path: &Path) -> Result<BookInfo> {
        match format {
            LibraryItemFormat::Epub => self.epub_parser.parse(path).await,
            LibraryItemFormat::Fb2 => self.fb2_parser.parse(path).await,
            LibraryItemFormat::Cbz | LibraryItemFormat::Cbr => self.comic_parser.parse(path).await,
            LibraryItemFormat::Mobi => self.mobi_parser.parse(path).await,
        }
    }

    fn locate_metadata_path(&self, path: &Path, format: LibraryItemFormat) -> Option<PathBuf> {
        locate_metadata_path(&self.metadata_indices, path, format)
    }

    fn parse_koreader_metadata(&self, metadata_path: Option<PathBuf>) -> Option<KoReaderMetadata> {
        let metadata_path = metadata_path?;
        match self.lua_parser.parse(&metadata_path) {
            Ok(metadata) => {
                debug!("Found metadata at: {:?}", metadata_path);
                Some(metadata)
            }
            Err(e) => {
                warn!("Failed to parse metadata {:?}: {}", metadata_path, e);
                None
            }
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
    let mut stats = IngestStats {
        processed: 1,
        ..Default::default()
    };

    let path = item.path.as_path();
    let format = item.format;

    let mut book_info = match processor.parse_book_info(format, path).await {
        Ok(info) => info,
        Err(e) => {
            warn_parse_failure(format, path, &e, item.kobo_hints.as_ref());
            stats.errors = 1;
            return stats;
        }
    };

    let metadata_path = processor.locate_metadata_path(path, format);
    let koreader_metadata = processor.parse_koreader_metadata(metadata_path.clone());

    if koreader_metadata.is_none() && !config.include_unread {
        stats.skipped_unread = 1;
        return stats;
    }

    let item_id = match processor.canonical_item_id(path, koreader_metadata.as_ref()) {
        Ok(id) => id,
        Err(e) => {
            warn!("Failed to derive item ID for {:?}: {}", path, e);
            stats.errors = 1;
            return stats;
        }
    };

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

    if let Err(e) =
        upsert_single_item(repo, &item, metadata_path.as_deref(), &config.time_config).await
    {
        warn!("Failed to upsert item {:?}: {}", path, e);
        stats.errors = 1;
        return stats;
    }

    stats.upserted = 1;
    if stats_fields_changed {
        stats.stats_invalidated = 1;
    }

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
            media::sync_item_file_symlink(&item_id, format.extension(), path, &media_dirs.files_dir)
    {
        warn!("Failed to create file symlink for {:?}: {}", path, e);
    }

    stats
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
