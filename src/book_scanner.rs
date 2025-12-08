use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use anyhow::{Result, bail};
use log::{debug, info, warn};
use walkdir;

use crate::models::Book;
use crate::epub_parser::EpubParser;
use crate::lua_parser::LuaParser;
use crate::utils::generate_book_id;
use crate::partial_md5::calculate_partial_md5;

/// Configuration for where to find KOReader metadata
#[derive(Clone, Debug)]
pub enum MetadataLocation {
    /// Default: metadata stored in .sdr folder next to each book
    InBookFolder,
    /// Metadata stored in docsettings folder with full path structure
    DocSettings(PathBuf),
    /// Metadata stored in hashdocsettings folder organized by partial MD5 hash
    HashDocSettings(PathBuf),
}

impl Default for MetadataLocation {
    fn default() -> Self {
        MetadataLocation::InBookFolder
    }
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
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) {
                if dir_name.ends_with(".sdr") {
                    // Extract the book filename from the sdr directory name
                    // e.g., "MyBook.sdr" -> "MyBook.epub"
                    let book_stem = &dir_name[..dir_name.len() - 4]; // Remove ".sdr"
                    
                    // Check for metadata.epub.lua inside the .sdr folder
                    let metadata_path = path.join("metadata.epub.lua");
                    if metadata_path.exists() {
                        let book_filename = format!("{}.epub", book_stem);
                        
                        if index.contains_key(&book_filename) {
                            duplicates.push(book_filename.clone());
                        } else {
                            debug!("Found docsettings metadata for: {}", book_filename);
                            index.insert(book_filename, metadata_path);
                        }
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
    
    info!("Scanning hashdocsettings folder: {:?}", hashdocsettings_path);
    
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
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) {
                if dir_name.ends_with(".sdr") {
                    // Extract the hash from the directory name
                    // e.g., "570615f811d504e628db1ef262bea270.sdr" -> "570615f811d504e628db1ef262bea270"
                    let hash = &dir_name[..dir_name.len() - 4]; // Remove ".sdr"
                    
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
                                    if let Some(name) = entry.file_name().to_str() {
                                        if name.starts_with("metadata.") && name.ends_with(".lua") {
                                            debug!("Found hashdocsettings metadata for hash: {} ({})", hash, name);
                                            index.insert(hash.to_lowercase(), entry.path());
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    info!("Found {} metadata files in hashdocsettings folder", index.len());
    Ok(index)
}

pub async fn scan_books(
    books_path: &PathBuf,
    metadata_location: &MetadataLocation,
) -> Result<(Vec<Book>, HashSet<String>)> {
    info!("Scanning books in directory: {:?}", books_path);
    let epub_parser = EpubParser::new();
    let lua_parser = LuaParser::new();
    
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
    
    // Walk through all epub files
    for entry in walkdir::WalkDir::new(books_path) {
        let entry = entry?;
        let path = entry.path();
        
        if path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("epub"))
            .unwrap_or(false)
        {
            debug!("Processing: {:?}", path);
            
            // Parse epub
            let epub_info = match epub_parser.parse(path).await {
                Ok(info) => info,
                Err(e) => {
                    log::warn!("Failed to parse epub {:?}: {}", path, e);
                    continue;
                }
            };
            
            // Track the MD5 for this book (for statistics filtering)
            // We may already have it from hashdocsettings lookup, or will get it from metadata
            let mut book_md5: Option<String> = None;
            
            // Find metadata based on the configured location
            let metadata_path = match metadata_location {
                MetadataLocation::InBookFolder => {
                    // Default: look for .sdr directory next to the book
                    let epub_stem = path.file_stem().unwrap().to_str().unwrap();
                    let sdr_path = path.parent().unwrap().join(format!("{}.sdr", epub_stem));
                    let metadata_file = sdr_path.join("metadata.epub.lua");
                    if metadata_file.exists() {
                        Some(metadata_file)
                    } else {
                        None
                    }
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
            
            let book = Book {
                id: generate_book_id(&epub_info.title),
                epub_info,
                koreader_metadata,
                epub_path: path.to_path_buf(),
            };
            
            books.push(book);
        }
    }
    
    info!("Found {} books!", books.len());
    Ok((books, library_md5s))
}
