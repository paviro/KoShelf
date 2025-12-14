use anyhow::{Result, bail};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use crate::koreader::{LuaParser, calculate_partial_md5};
use crate::models::{LibraryItem, LibraryItemFormat};
use crate::parsers::{ComicParser, EpubParser, Fb2Parser, MobiParser};
use crate::utils::generate_book_id;

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

pub async fn scan_library(
    library_paths: &[PathBuf],
    metadata_location: &MetadataLocation,
) -> Result<(Vec<LibraryItem>, HashSet<String>)> {
    info!("Scanning {} library paths...", library_paths.len());
    let start = Instant::now();
    let epub_parser = EpubParser::new();
    let fb2_parser = Fb2Parser::new();
    let comic_parser = ComicParser::new();
    let mobi_parser = MobiParser::new();
    let lua_parser = LuaParser::new();

    // Set up spinner for scanning progress
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Scanning library...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    // Pre-build metadata indices for external storage modes
    let docsettings_index = match metadata_location {
        MetadataLocation::DocSettings(path) => Some(build_docsettings_index(path)?),
        _ => None,
    };

    let hashdocsettings_index = match metadata_location {
        MetadataLocation::HashDocSettings(path) => Some(build_hashdocsettings_index(path)?),
        _ => None,
    };

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
            let book_info = match format {
                LibraryItemFormat::Epub => match epub_parser.parse(path).await {
                    Ok(info) => info,
                    Err(e) => {
                        log::warn!("Failed to parse epub {:?}: {}", path, e);
                        continue;
                    }
                },
                LibraryItemFormat::Fb2 => match fb2_parser.parse(path).await {
                    Ok(info) => info,
                    Err(e) => {
                        log::warn!("Failed to parse fb2 {:?}: {}", path, e);
                        continue;
                    }
                },
                LibraryItemFormat::Cbz | LibraryItemFormat::Cbr => {
                    match comic_parser.parse(path).await {
                        Ok(info) => info,
                        Err(e) => {
                            log::warn!("Failed to parse comic {:?}: {}", path, e);
                            continue;
                        }
                    }
                }
                LibraryItemFormat::Mobi => match mobi_parser.parse(path).await {
                    Ok(info) => info,
                    Err(e) => {
                        log::warn!("Failed to parse mobi {:?}: {}", path, e);
                        continue;
                    }
                },
            };

            // Track the MD5 for this book (for statistics filtering)
            // We may already have it from hashdocsettings lookup, or will get it from metadata
            let mut book_md5: Option<String> = None;

            // Find metadata based on the configured location
            let metadata_path = match metadata_location {
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
                        docsettings_index
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
                            // Store the calculated MD5 for later use
                            book_md5 = Some(hash.clone());
                            hashdocsettings_index
                                .as_ref()
                                .and_then(|idx| idx.get(&hash.to_lowercase()).cloned())
                        }
                        Err(e) => {
                            warn!("Failed to calculate partial MD5 for {:?}: {}", path, e);
                            None
                        }
                    }
                }
            };

            let koreader_metadata = if let Some(metadata_path) = metadata_path {
                match lua_parser.parse(&metadata_path).await {
                    Ok(metadata) => {
                        debug!("Found metadata at: {:?}", metadata_path);
                        Some(metadata)
                    }
                    Err(e) => {
                        log::warn!("Failed to parse metadata {:?}: {}", metadata_path, e);
                        None
                    }
                }
            } else {
                None
            };

            // Collect MD5 for statistics filtering:
            // 1. Prefer MD5 from metadata (stable even if file is updated)
            // 2. Use calculated MD5 from hashdocsettings lookup if available
            // 3. Fall back to calculating MD5 for books without metadata
            if let Some(ref metadata) = koreader_metadata {
                if let Some(ref md5) = metadata.partial_md5_checksum {
                    library_md5s.insert(md5.clone());
                } else if let Some(ref md5) = book_md5 {
                    library_md5s.insert(md5.clone());
                } else if let Ok(md5) = calculate_partial_md5(path) {
                    library_md5s.insert(md5);
                }
            } else if let Some(ref md5) = book_md5 {
                library_md5s.insert(md5.clone());
            } else if let Ok(md5) = calculate_partial_md5(path) {
                library_md5s.insert(md5);
            }

            let book = LibraryItem {
                id: generate_book_id(&book_info.title),
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
