//! Library filesystem scanning: path collection and metadata location.
//!
//! Stripped to pure path discovery — all parsing, metadata resolution, and
//! cover generation now live in `runtime::ingest::library`.

use crate::models::LibraryItemFormat;
use log::{info, warn};
use std::path::PathBuf;

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

/// Walk library directories and collect paths with supported extensions.
///
/// Returns all file paths matching supported book/comic formats
/// (epub, fb2, mobi, cbz, cbr).  No parsing, no metadata, no covers.
pub fn collect_paths(library_paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for library_path in library_paths {
        for entry in walkdir::WalkDir::new(library_path) {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to read directory entry: {}", e);
                    continue;
                }
            };

            if LibraryItemFormat::from_path(entry.path()).is_some() {
                paths.push(entry.into_path());
            }
        }
    }

    info!(
        "Collected {} items from {} library directories",
        paths.len(),
        library_paths.len()
    );
    paths
}
