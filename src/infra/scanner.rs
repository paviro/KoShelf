use anyhow::{Context, Result, bail};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use crate::koreader::merge_precedence::{normalize_partial_md5, resolve_canonical_partial_md5};
use crate::koreader::{LuaParser, calculate_partial_md5};
use crate::models::{BookInfo, KoReaderMetadata, LibraryItem, LibraryItemFormat};
use crate::parsers::{ComicParser, EpubParser, Fb2Parser, MobiParser};

/// Configuration for where to find KOReader metadata
#[derive(Clone, Debug, Default)]
pub enum MetadataLocation {
    /// Default: metadata stored in .sdr folder next to each book
    #[default]
    InBookFolder,
    /// Metadata stored in docsettings folder with full path structure
    DocSettings(PathBuf),
    /// Metadata stored in hashdocsettings folder organized by partial MD5 hash
    HashDocSettings(PathBuf),
}

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
            // Extract the book filename from the sdr directory name
            // e.g., "MyBook.sdr" -> "MyBook.epub"

            // Check for metadata files inside the .sdr folder (try both epub and fb2)
            let epub_metadata_path = path.join("metadata.epub.lua");
            let fb2_metadata_path = path.join("metadata.fb2.lua");

            if epub_metadata_path.exists() {
                let book_filename = format!("{}.epub", book_stem);

                match index.entry(book_filename.clone()) {
                    std::collections::hash_map::Entry::Occupied(_) => {
                        duplicates.push(book_filename);
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        debug!("Found docsettings metadata for: {}", book_filename);
                        entry.insert(epub_metadata_path);
                    }
                }
            } else if fb2_metadata_path.exists() {
                let book_filename = format!("{}.fb2", book_stem);

                match index.entry(book_filename.clone()) {
                    std::collections::hash_map::Entry::Occupied(_) => {
                        duplicates.push(book_filename);
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
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

    // hashdocsettings structure: {hash_prefix}/{full_hash}.sdr/metadata.epub.lua
    // e.g., 57/570615f811d504e628db1ef262bea270.sdr/metadata.epub.lua
    for entry in walkdir::WalkDir::new(hashdocsettings_path).max_depth(3) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read entry in hashdocsettings: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Look for .sdr directories with hash names
        if path.is_dir()
            && let Some(dir_name) = path.file_name().and_then(|s| s.to_str())
            && let Some(hash) = dir_name.strip_suffix(".sdr")
        {
            // Extract the hash from the directory name
            // e.g., "570615f811d504e628db1ef262bea270.sdr" -> "570615f811d504e628db1ef262bea270"

            // Validate it looks like an MD5 hash (32 hex characters)
            if hash.len() == 32 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
                // Check for metadata file (could be .epub.lua or other extensions)
                let epub_metadata_path = path.join("metadata.epub.lua");
                if epub_metadata_path.exists() {
                    debug!("Found hashdocsettings metadata for hash: {}", hash);
                    index.insert(hash.to_lowercase(), epub_metadata_path);
                } else {
                    // Check for any metadata.*.lua file
                    if let Ok(entries) = std::fs::read_dir(path) {
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
        }
    }

    info!(
        "Found {} metadata files in hashdocsettings folder",
        index.len()
    );
    Ok(index)
}

struct LibraryScanner {
    metadata_location: MetadataLocation,
    docsettings_index: Option<HashMap<String, PathBuf>>,
    hashdocsettings_index: Option<HashMap<String, PathBuf>>,
    epub_parser: EpubParser,
    fb2_parser: Fb2Parser,
    comic_parser: ComicParser,
    mobi_parser: MobiParser,
    lua_parser: LuaParser,
}

impl LibraryScanner {
    fn new(metadata_location: &MetadataLocation) -> Result<Self> {
        // Pre-build metadata indices for external storage modes
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
            epub_parser: EpubParser::new(),
            fb2_parser: Fb2Parser::new(),
            comic_parser: ComicParser::new(),
            mobi_parser: MobiParser::new(),
            lua_parser: LuaParser::new(),
        })
    }

    async fn parse_book_info(
        &self,
        format: LibraryItemFormat,
        path: &std::path::Path,
    ) -> Result<BookInfo> {
        match format {
            LibraryItemFormat::Epub => self.epub_parser.parse(path).await,
            LibraryItemFormat::Fb2 => self.fb2_parser.parse(path).await,
            LibraryItemFormat::Cbz | LibraryItemFormat::Cbr => self.comic_parser.parse(path).await,
            LibraryItemFormat::Mobi => self.mobi_parser.parse(path).await,
        }
    }

    fn locate_metadata_path(
        &self,
        path: &std::path::Path,
        format: LibraryItemFormat,
    ) -> Option<PathBuf> {
        match &self.metadata_location {
            MetadataLocation::InBookFolder => {
                // Default: look for .sdr directory next to the book
                let book_stem = path.file_stem().unwrap().to_str().unwrap();
                let sdr_path = path.parent().unwrap().join(format!("{}.sdr", book_stem));
                // KOReader creates format-specific metadata files based on the book extension
                let metadata_file = sdr_path.join(format.metadata_filename());
                metadata_file.exists().then_some(metadata_file)
            }
            MetadataLocation::DocSettings(_) => {
                // Look up by filename in the pre-built index
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    self.docsettings_index
                        .as_ref()
                        .and_then(|idx| idx.get(filename).cloned())
                } else {
                    None
                }
            }
            MetadataLocation::HashDocSettings(_) => {
                // Calculate partial MD5 and look up in the pre-built index
                match calculate_partial_md5(path) {
                    Ok(hash) => {
                        debug!("Calculated partial MD5 for {:?}: {}", path, hash);
                        self.hashdocsettings_index
                            .as_ref()
                            .and_then(|idx| idx.get(&hash.to_lowercase()).cloned())
                    }
                    Err(e) => {
                        warn!("Failed to calculate partial MD5 for {:?}: {}", path, e);
                        None
                    }
                }
            }
        }
    }

    async fn parse_koreader_metadata(
        &self,
        metadata_path: Option<PathBuf>,
    ) -> Option<KoReaderMetadata> {
        let metadata_path = metadata_path?;
        match self.lua_parser.parse(&metadata_path).await {
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

    fn collect_md5_for_item(
        &self,
        library_md5s: &mut HashSet<String>,
        path: &std::path::Path,
        koreader_metadata: &Option<KoReaderMetadata>,
    ) {
        let metadata_md5 = koreader_metadata
            .as_ref()
            .and_then(|metadata| metadata.partial_md5_checksum.as_deref());

        // Collect MD5 for statistics filtering:
        // 1. Prefer MD5 from metadata (stable even if file contents vary)
        // 2. Fall back to deriving MD5 from file content
        if let Some(resolved) = resolve_canonical_partial_md5(metadata_md5, None) {
            library_md5s.insert(resolved.value);
            return;
        }

        if let Some(metadata_md5) = metadata_md5
            && normalize_partial_md5(metadata_md5).is_none()
        {
            warn!(
                "Ignoring invalid KOReader partial_md5_checksum '{}' for {:?}; expected 32 hex chars",
                metadata_md5, path
            );
        }

        if let Ok(derived_md5) = calculate_partial_md5(path) {
            if let Some(resolved) = resolve_canonical_partial_md5(None, Some(derived_md5.as_str()))
            {
                library_md5s.insert(resolved.value);
            } else {
                warn!(
                    "Ignoring invalid derived partial MD5 '{}' for {:?}; expected 32 hex chars",
                    derived_md5, path
                );
            }
        }
    }

    fn canonical_item_id(
        &self,
        path: &std::path::Path,
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

/// A single item produced by targeted scanning.
pub struct ScannedItem {
    pub item: LibraryItem,
    pub metadata_path: Option<PathBuf>,
}

/// Parse only the given book file paths (no directory walk).
///
/// Returns a `ScannedItem` per successfully parsed file, with the resolved
/// metadata path so the caller can store correct fingerprints.
pub async fn scan_specific_files(
    file_paths: &[PathBuf],
    metadata_location: &MetadataLocation,
) -> Vec<ScannedItem> {
    let scanner = match LibraryScanner::new(metadata_location) {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to create scanner for targeted scan: {}", e);
            return Vec::new();
        }
    };

    let mut results = Vec::new();

    for path in file_paths {
        let format = match LibraryItemFormat::from_path(path) {
            Some(f) => f,
            None => {
                warn!("Unsupported file format for targeted scan: {:?}", path);
                continue;
            }
        };

        let book_info = match scanner.parse_book_info(format, path).await {
            Ok(info) => info,
            Err(e) => {
                warn!("Failed to parse {:?} {:?}: {}", format, path, e);
                continue;
            }
        };

        let metadata_path = scanner.locate_metadata_path(path, format);
        let koreader_metadata = scanner.parse_koreader_metadata(metadata_path.clone()).await;

        let item_id = match scanner.canonical_item_id(path, koreader_metadata.as_ref()) {
            Ok(id) => id,
            Err(e) => {
                warn!("Failed to derive item ID for {:?}: {}", path, e);
                continue;
            }
        };

        let item = LibraryItem {
            id: item_id,
            book_info,
            koreader_metadata,
            file_path: path.to_path_buf(),
            format,
        };

        results.push(ScannedItem {
            item,
            metadata_path,
        });
    }

    if !file_paths.is_empty() {
        info!(
            "Targeted scan: parsed {} of {} files",
            results.len(),
            file_paths.len()
        );
    }
    results
}

pub async fn scan_library(
    library_paths: &[PathBuf],
    metadata_location: &MetadataLocation,
) -> Result<(Vec<LibraryItem>, HashSet<String>)> {
    info!("Scanning {} library paths...", library_paths.len());
    let start = Instant::now();
    let scanner = LibraryScanner::new(metadata_location)?;

    // Set up spinner for scanning progress
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Scanning library...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut books = Vec::new();
    let mut library_md5s = HashSet::new();

    // Walk through all library paths and scan for books/comics
    for library_path in library_paths {
        debug!("Scanning library path: {:?}", library_path);

        for entry in walkdir::WalkDir::new(library_path) {
            let entry = entry?;
            let path = entry.path();

            // Detect book format from extension
            let format = match LibraryItemFormat::from_path(path) {
                Some(f) => f,
                None => continue, // Skip unsupported formats
            };

            debug!("Processing {:?}: {:?}", format, path);

            // Parse book based on format
            let book_info = match scanner.parse_book_info(format, path).await {
                Ok(info) => info,
                Err(e) => {
                    log::warn!("Failed to parse {:?} {:?}: {}", format, path, e);
                    continue;
                }
            };

            let metadata_path = scanner.locate_metadata_path(path, format);
            let koreader_metadata = scanner.parse_koreader_metadata(metadata_path).await;
            scanner.collect_md5_for_item(&mut library_md5s, path, &koreader_metadata);
            let item_id = scanner.canonical_item_id(path, koreader_metadata.as_ref())?;

            let book = LibraryItem {
                id: item_id,
                book_info,
                koreader_metadata,
                file_path: path.to_path_buf(),
                format,
            };

            books.push(book);

            // Update spinner with current count
            spinner.set_message(format!("Scanning library... {} items found", books.len()));
        }
    }

    // Clear spinner and log summary
    spinner.finish_and_clear();
    let elapsed = start.elapsed();
    info!(
        "Found {} items ({} books, {} comics) in {:.1}s",
        books.len(),
        books.iter().filter(|b| b.is_book()).count(),
        books.iter().filter(|b| b.is_comic()).count(),
        elapsed.as_secs_f64()
    );
    Ok((books, library_md5s))
}
