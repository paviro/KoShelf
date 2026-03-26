//! Library item ingestion: parse, deduplicate, persist, and extract covers.
//!
//! Processes items concurrently via a worker pool — each worker owns its own
//! parser set and fully completes each item (parse → filter → dedup → upsert →
//! cover encode) before moving to the next.  No bulk `Vec<LibraryItem>` is
//! held in memory.
//!
//! This single function handles both full ingest (all paths from scanner)
//! and targeted updates (changed paths from watcher).  The caller decides
//! what paths to pass.

use anyhow::{Context, Result, bail};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use crate::app::config::SiteConfig;
use crate::pipeline::media;
use crate::shelf::library::upsert_single_item;
use crate::shelf::models::{BookInfo, KoReaderMetadata, LibraryItem, LibraryItemFormat};
use crate::source::koreader::merge::{normalize_partial_md5, resolve_canonical_partial_md5};
use crate::source::koreader::{LuaParser, calculate_partial_md5};
use crate::source::parsers::{ComicParser, EpubParser, Fb2Parser, MobiParser};
use crate::source::scanner::{MetadataLocation, collect_paths};
use crate::source::{
    FileFingerprint, ItemFingerprints, ReconcileAction, classify_reconcile_action,
};
use crate::store::sqlite::repo::LibraryRepository;

// ── Ingest stats ────────────────────────────────────────────────────────

/// Counters produced by `ingest_paths`.
#[derive(Debug, Clone, Copy, Default)]
pub struct IngestStats {
    pub processed: u64,
    pub upserted: u64,
    pub skipped_duplicates: u64,
    pub skipped_unread: u64,
    pub errors: u64,
    pub stats_invalidated: u64,
}

impl IngestStats {
    fn merge(&mut self, other: IngestStats) {
        self.processed += other.processed;
        self.upserted += other.upserted;
        self.skipped_duplicates += other.skipped_duplicates;
        self.skipped_unread += other.skipped_unread;
        self.errors += other.errors;
        self.stats_invalidated += other.stats_invalidated;
    }
}

// ── Metadata indices (immutable, built once) ────────────────────────────

/// Pre-built metadata location indices shared across all workers via `Arc`.
///
/// Building docsettings/hashdocsettings indices involves walking directories,
/// so we do it once and share the result.
struct MetadataIndices {
    metadata_location: MetadataLocation,
    docsettings_index: Option<HashMap<String, PathBuf>>,
    hashdocsettings_index: Option<HashMap<String, PathBuf>>,
}

impl MetadataIndices {
    fn new(metadata_location: &MetadataLocation) -> Result<Self> {
        let docsettings_index = match metadata_location {
            MetadataLocation::DocSettings(path) => Some(build_docsettings_index(path)?),
            _ => None,
        };

        let hashdocsettings_index = match metadata_location {
            MetadataLocation::HashDocSettings(path) => Some(build_hashdocsettings_index(path)?),
            _ => None,
        };

        Ok(Self {
            metadata_location: metadata_location.clone(),
            docsettings_index,
            hashdocsettings_index,
        })
    }
}

// ── Per-worker item processor (owns parsers including Lua VM) ────────────

/// Each worker owns its own `ItemProcessor` with fresh parser instances.
///
/// The `LuaParser` contains `mlua::Lua` which is Send but not Sync, so
/// processors cannot be shared via `Arc` — each worker creates its own.
struct ItemProcessor {
    config: Arc<MetadataIndices>,
    epub_parser: EpubParser,
    fb2_parser: Fb2Parser,
    comic_parser: ComicParser,
    mobi_parser: MobiParser,
    lua_parser: LuaParser,
}

impl ItemProcessor {
    fn new(config: Arc<MetadataIndices>) -> Self {
        Self {
            config,
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
        locate_metadata_path(&self.config, path, format)
    }

    fn parse_koreader_metadata(&self, metadata_path: Option<PathBuf>) -> Option<KoReaderMetadata> {
        let metadata_path = metadata_path?;
        match self.lua_parser.parse(&metadata_path) {
            Ok(metadata) => {
                debug!("Found metadata at: {:?}", metadata_path);
                Some(metadata)
            }
            Err(e) => {
                log::warn!("Failed to parse metadata {:?}: {}", metadata_path, e);
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

// ── Index builders ──────────────────────────────────────────────────────

/// Build an index of metadata files in a docsettings folder.
/// Maps the book filename (e.g., "MyBook.epub") to the metadata file path.
/// Returns an error if duplicate filenames are found.
fn build_docsettings_index(docsettings_path: &PathBuf) -> Result<HashMap<String, PathBuf>> {
    let mut index: HashMap<String, PathBuf> = HashMap::new();
    let mut duplicates: Vec<String> = Vec::new();

    info!("Scanning docsettings folder: {:?}", docsettings_path);

    for entry in walkdir::WalkDir::new(docsettings_path) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read entry in docsettings: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Look for .sdr directories
        if path.is_dir()
            && let Some(dir_name) = path.file_name().and_then(|s| s.to_str())
            && let Some(book_stem) = dir_name.strip_suffix(".sdr")
        {
            let epub_metadata_path = path.join("metadata.epub.lua");
            let fb2_metadata_path = path.join("metadata.fb2.lua");

            if epub_metadata_path.exists() {
                let book_filename = format!("{}.epub", book_stem);

                match index.entry(book_filename.clone()) {
                    Entry::Occupied(_) => {
                        duplicates.push(book_filename);
                    }
                    Entry::Vacant(entry) => {
                        debug!("Found docsettings metadata for: {}", book_filename);
                        entry.insert(epub_metadata_path);
                    }
                }
            } else if fb2_metadata_path.exists() {
                let book_filename = format!("{}.fb2", book_stem);

                match index.entry(book_filename.clone()) {
                    Entry::Occupied(_) => {
                        duplicates.push(book_filename);
                    }
                    Entry::Vacant(entry) => {
                        debug!("Found docsettings metadata for: {}", book_filename);
                        entry.insert(fb2_metadata_path);
                    }
                }
            }
        }
    }

    if !duplicates.is_empty() {
        bail!(
            "Found duplicate book filenames in docsettings folder: {:?}\n\n\
            The --docsettings-path option matches books by filename only, not by their full path.\n\
            This is because the folder structure inside docsettings reflects the device path where \n\
            KOReader was used (e.g., /home/user/books/), which may differ from your local books path.\n\n\
            Unfortunately, KOShelf cannot distinguish between multiple books with the same filename \n\
            when using --docsettings-path. Consider using --hashdocsettings-path instead, which \n\
            matches books by their content hash and doesn't have this limitation.",
            duplicates
        );
    }

    info!("Found {} metadata files in docsettings folder", index.len());
    Ok(index)
}

/// Build an index of metadata files in a hashdocsettings folder.
/// Maps the partial MD5 hash to the metadata file path.
fn build_hashdocsettings_index(hashdocsettings_path: &PathBuf) -> Result<HashMap<String, PathBuf>> {
    let mut index: HashMap<String, PathBuf> = HashMap::new();

    info!(
        "Scanning hashdocsettings folder: {:?}",
        hashdocsettings_path
    );

    for entry in walkdir::WalkDir::new(hashdocsettings_path).max_depth(3) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read entry in hashdocsettings: {}", e);
                continue;
            }
        };

        let path = entry.path();

        if path.is_dir()
            && let Some(dir_name) = path.file_name().and_then(|s| s.to_str())
            && let Some(hash) = dir_name.strip_suffix(".sdr")
            && hash.len() == 32
            && hash.chars().all(|c| c.is_ascii_hexdigit())
        {
            let epub_metadata_path = path.join("metadata.epub.lua");
            if epub_metadata_path.exists() {
                debug!("Found hashdocsettings metadata for hash: {}", hash);
                index.insert(hash.to_lowercase(), epub_metadata_path);
            } else if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str()
                        && name.starts_with("metadata.")
                        && name.ends_with(".lua")
                    {
                        debug!(
                            "Found hashdocsettings metadata for hash: {} ({})",
                            hash, name
                        );
                        index.insert(hash.to_lowercase(), entry.path());
                        break;
                    }
                }
            }
        }
    }

    info!(
        "Found {} metadata files in hashdocsettings folder",
        index.len()
    );
    Ok(index)
}

// ── Metadata path resolution (free function) ────────────────────────────

/// Locate the KOReader metadata file for a given book path using the
/// pre-built metadata indices.  Callable without a full `ItemProcessor`.
fn locate_metadata_path(
    indices: &MetadataIndices,
    path: &Path,
    format: LibraryItemFormat,
) -> Option<PathBuf> {
    match &indices.metadata_location {
        MetadataLocation::InBookFolder => {
            let book_stem = path.file_stem().and_then(|s| s.to_str())?;
            let sdr_path = path.parent()?.join(format!("{}.sdr", book_stem));
            let metadata_file = sdr_path.join(format.metadata_filename());
            metadata_file.exists().then_some(metadata_file)
        }
        MetadataLocation::DocSettings(_) => {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                indices
                    .docsettings_index
                    .as_ref()
                    .and_then(|idx| idx.get(filename).cloned())
            } else {
                None
            }
        }
        MetadataLocation::HashDocSettings(_) => match calculate_partial_md5(path) {
            Ok(hash) => {
                debug!("Calculated partial MD5 for {:?}: {}", path, hash);
                indices
                    .hashdocsettings_index
                    .as_ref()
                    .and_then(|idx| idx.get(&hash.to_lowercase()).cloned())
            }
            Err(e) => {
                warn!("Failed to calculate partial MD5 for {:?}: {}", path, e);
                None
            }
        },
    }
}

// ── Single-item processing ──────────────────────────────────────────────

/// Fully process one path: parse → filter → dedup → upsert → cover encode.
///
/// Returns the outcome as an `IngestStats` with exactly one counter incremented.
async fn process_single_item(
    path: &Path,
    processor: &ItemProcessor,
    config: &SiteConfig,
    repo: &LibraryRepository,
    covers_dir: &Path,
    files_dir: &Path,
) -> IngestStats {
    let mut stats = IngestStats {
        processed: 1,
        ..Default::default()
    };

    let format = match LibraryItemFormat::from_path(path) {
        Some(f) => f,
        None => {
            warn!("Unsupported file format: {:?}", path);
            stats.errors = 1;
            return stats;
        }
    };

    let mut book_info = match processor.parse_book_info(format, path).await {
        Ok(info) => info,
        Err(e) => {
            warn!("Failed to parse {:?} {:?}: {}", format, path, e);
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
        let cover_path = covers_dir.join(format!("{}.webp", item_id));

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
        && let Err(e) = media::sync_item_file_symlink(&item_id, format.extension(), path, files_dir)
    {
        warn!("Failed to create file symlink for {:?}: {}", path, e);
    }

    stats
}

// ── Ingest entry point ──────────────────────────────────────────────────

/// Number of concurrent worker tasks processing items in parallel.
const INGEST_CONCURRENCY: usize = 8;

/// Process book file paths concurrently: each worker fully handles its item
/// (parse → filter → dedup → upsert → cover encode) before taking the next.
///
/// This single function handles both full ingest (all paths from scanner)
/// and targeted updates (changed paths from watcher).
pub async fn ingest_paths(
    paths: &[PathBuf],
    config: &SiteConfig,
    repo: &LibraryRepository,
    covers_dir: &Path,
    files_dir: &Path,
) -> Result<IngestStats> {
    if paths.is_empty() {
        return Ok(IngestStats::default());
    }

    info!("Ingesting {} items...", paths.len());
    let start = Instant::now();

    let metadata_indices = Arc::new(MetadataIndices::new(&config.metadata_location)?);

    let pb = ProgressBar::new(paths.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} {bar:30.cyan/blue} {pos}/{len}")
            .unwrap()
            .progress_chars("━╸─"),
    );
    pb.set_message("Ingesting library:");

    let (tx, rx) = tokio::sync::mpsc::channel::<PathBuf>(INGEST_CONCURRENCY * 2);
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    let mut workers = tokio::task::JoinSet::new();
    for _ in 0..INGEST_CONCURRENCY {
        let rx = rx.clone();
        let repo = repo.clone();
        let config = config.clone();
        let covers_dir = covers_dir.to_path_buf();
        let files_dir = files_dir.to_path_buf();
        let metadata_indices = metadata_indices.clone();
        let pb = pb.clone();

        workers.spawn(async move {
            let processor = ItemProcessor::new(metadata_indices);
            let mut stats = IngestStats::default();

            loop {
                let path = {
                    let mut rx = rx.lock().await;
                    match rx.recv().await {
                        Some(p) => p,
                        None => break,
                    }
                };

                let item_stats =
                    process_single_item(&path, &processor, &config, &repo, &covers_dir, &files_dir)
                        .await;
                stats.merge(item_stats);
                pb.inc(1);
            }

            stats
        });
    }

    for path in paths {
        if tx.send(path.clone()).await.is_err() {
            break;
        }
    }
    drop(tx); // Signal no more work

    let mut total_stats = IngestStats::default();
    while let Some(result) = workers.join_next().await {
        match result {
            Ok(worker_stats) => total_stats.merge(worker_stats),
            Err(e) => warn!("Ingest worker panicked: {}", e),
        }
    }

    pb.finish_and_clear();
    total_stats.processed = paths.len() as u64;

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

// ── Library update ──────────────────────────────────────────────────────

/// Summary of a library update: what changed since the last run.
#[derive(Debug, Default)]
pub struct UpdateResult {
    pub unchanged: u64,
    pub changed: u64,
    pub added: u64,
    pub removed: u64,
    pub ingest_stats: Option<IngestStats>,
}

/// Update the library DB to match the current filesystem state.
///
/// Handles both first run (empty DB → everything is new) and subsequent
/// runs (detect added/removed/modified books via fingerprint comparison).
pub async fn update_library(
    config: &SiteConfig,
    repo: &LibraryRepository,
    covers_dir: &Path,
    files_dir: &Path,
) -> Result<UpdateResult> {
    let start = Instant::now();
    let fs_paths = collect_paths(&config.library_paths);
    let stored_fingerprints = repo.load_all_fingerprints().await?;

    let stored_by_book_path: HashMap<&str, _> = stored_fingerprints
        .iter()
        .map(|fp| (fp.book_path.as_str(), fp))
        .collect();

    let fs_path_set: HashSet<String> = fs_paths
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    let metadata_indices = MetadataIndices::new(&config.metadata_location)?;

    let mut result = UpdateResult::default();
    let mut ingest_paths_list: Vec<PathBuf> = Vec::new();
    let mut remove_ids: Vec<String> = Vec::new();

    // ── Phase 1: Check stored items against filesystem ───────────────
    for fp in &stored_fingerprints {
        if !fs_path_set.contains(&fp.book_path) {
            // Book file gone from disk
            remove_ids.push(fp.item_id.clone());
            result.removed += 1;
            continue;
        }

        let book_path = Path::new(&fp.book_path);

        let current_book_fp = match FileFingerprint::capture(book_path) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to capture fingerprint for {:?}: {}", book_path, e);
                ingest_paths_list.push(book_path.to_path_buf());
                result.changed += 1;
                continue;
            }
        };

        let current_metadata_path = if let Some(ref stored_meta) = fp.metadata_path {
            Some(PathBuf::from(stored_meta))
        } else {
            // Check if metadata has appeared since last run.
            LibraryItemFormat::from_path(book_path)
                .and_then(|fmt| locate_metadata_path(&metadata_indices, book_path, fmt))
        };

        let current_metadata_fp = match &current_metadata_path {
            Some(meta_path) => match FileFingerprint::capture_optional(meta_path) {
                Ok(fp) => fp,
                Err(e) => {
                    warn!(
                        "Failed to capture metadata fingerprint {:?}: {}",
                        meta_path, e
                    );
                    None
                }
            },
            None => None,
        };

        let current = ItemFingerprints {
            book_file: current_book_fp,
            metadata_file: current_metadata_fp,
        };

        let stored_book_fp = FileFingerprint {
            path: book_path.to_path_buf(),
            size_bytes: fp.book_size_bytes as u64,
            modified_unix_ms: fp.book_modified_unix_ms as u64,
        };
        let stored_metadata_fp = match (
            &fp.metadata_path,
            fp.metadata_size_bytes,
            fp.metadata_modified_unix_ms,
        ) {
            (Some(meta_path), Some(size), Some(modified)) => Some(FileFingerprint {
                path: PathBuf::from(meta_path),
                size_bytes: size as u64,
                modified_unix_ms: modified as u64,
            }),
            _ => None,
        };
        let stored = ItemFingerprints {
            book_file: stored_book_fp,
            metadata_file: stored_metadata_fp,
        };

        // Edge case: stored had no metadata, but we now found one
        let metadata_appeared = fp.metadata_path.is_none() && current.metadata_file.is_some();

        match classify_reconcile_action(Some(&stored), Some(&current)) {
            ReconcileAction::Unchanged if !metadata_appeared => {
                let cover_path = covers_dir.join(format!("{}.webp", fp.item_id));
                if media::cover_needs_generation(book_path, &cover_path) {
                    ingest_paths_list.push(book_path.to_path_buf());
                    result.changed += 1;
                } else {
                    // Ensure file symlink exists for unchanged items
                    if config.is_internal_server
                        && let Some(ext) = book_path.extension().and_then(|e| e.to_str())
                        && let Err(e) =
                            media::sync_item_file_symlink(&fp.item_id, ext, book_path, files_dir)
                    {
                        warn!("Failed to create file symlink for {}: {}", fp.item_id, e);
                    }
                    result.unchanged += 1;
                }
            }
            ReconcileAction::Unchanged => {
                // metadata_appeared — need to re-ingest to pick up the new metadata
                ingest_paths_list.push(book_path.to_path_buf());
                result.changed += 1;
            }
            ReconcileAction::Reparse(_) => {
                ingest_paths_list.push(book_path.to_path_buf());
                result.changed += 1;
            }
            _ => {
                // Added/Removed shouldn't happen here (both Some), but handle gracefully
                result.unchanged += 1;
            }
        }
    }

    // ── Phase 2: Detect new paths ────────────────────────────────────
    for path in &fs_paths {
        let path_str = path.to_string_lossy();
        if !stored_by_book_path.contains_key(path_str.as_ref()) {
            ingest_paths_list.push(path.clone());
            result.added += 1;
        }
    }

    // ── Phase 3: Execute ─────────────────────────────────────────────

    for item_id in &remove_ids {
        if let Err(e) = repo.delete_item(item_id).await {
            warn!("Failed to delete removed item {}: {}", item_id, e);
        } else {
            if media::is_canonical_item_id(item_id) {
                let cover_path = covers_dir.join(format!("{}.webp", item_id));
                let _ = fs::remove_file(&cover_path);
            } else {
                warn!(
                    "Skipping cover cleanup for non-canonical item id: {}",
                    item_id
                );
            }
            if config.is_internal_server
                && let Err(e) = media::remove_item_files_by_id(item_id, files_dir)
            {
                warn!("Failed to clean file symlinks for {}: {}", item_id, e);
            }
            info!("Deleted item {} (book removed from disk)", item_id);
        }
    }

    if !ingest_paths_list.is_empty() {
        let stats = ingest_paths(&ingest_paths_list, config, repo, covers_dir, files_dir).await?;
        result.ingest_stats = Some(stats);
    }

    if ingest_paths_list.is_empty() && remove_ids.is_empty() {
        info!(
            "Library unchanged ({} items, checked in {} ms)",
            result.unchanged,
            start.elapsed().as_millis(),
        );
    }

    Ok(result)
}

/// Check whether an ingested item requires a statistics reload.
///
/// A reload is needed when:
/// - The item is new (its MD5 changes the `filter_to_library` set)
/// - Any stats-influencing field changed (`hidden_flow_pages`,
///   `pagemap_doc_pages`, or `has_synthetic_pagination`)
async fn needs_stats_reload(item: &LibraryItem, repo: &LibraryRepository) -> bool {
    let old_fields = match repo.load_stats_influencing_fields(&item.id).await {
        Ok(Some(fields)) => fields,
        // New item (changes the library filter set) or DB error.
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
