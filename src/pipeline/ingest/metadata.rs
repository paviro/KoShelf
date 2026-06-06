use anyhow::{Result, bail};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fs;
use std::path::{Path, PathBuf};

use crate::shelf::models::LibraryItemFormat;
use crate::source::koreader::calculate_partial_md5;
use crate::source::scanner::MetadataLocation;

/// Pre-built metadata location indices shared across all workers.
///
/// Building docsettings/hashdocsettings indices involves walking directories,
/// so callers build this once per sync or changed-item batch and reuse it.
pub(super) struct MetadataIndices {
    metadata_location: MetadataLocation,
    docsettings_index: Option<HashMap<DocsettingsKey, PathBuf>>,
    hashdocsettings_index: Option<HashMap<String, PathBuf>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DocsettingsKey {
    sidecar_stem: String,
    format: LibraryItemFormat,
}

impl MetadataIndices {
    pub(super) fn new(metadata_location: &MetadataLocation) -> Result<Self> {
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

pub(super) fn locate_metadata_path(
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
            let sidecar_stem = path.file_stem().and_then(|s| s.to_str())?;
            let key = DocsettingsKey {
                sidecar_stem: sidecar_stem.to_string(),
                format,
            };
            indices
                .docsettings_index
                .as_ref()
                .and_then(|idx| idx.get(&key).cloned())
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

fn build_docsettings_index(docsettings_path: &PathBuf) -> Result<HashMap<DocsettingsKey, PathBuf>> {
    let mut index: HashMap<DocsettingsKey, PathBuf> = HashMap::new();
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

        if path.is_dir()
            && let Some(dir_name) = path.file_name().and_then(|s| s.to_str())
            && let Some(book_stem) = dir_name.strip_suffix(".sdr")
        {
            let entries = match fs::read_dir(path) {
                Ok(entries) => entries,
                Err(e) => {
                    warn!("Failed to read docsettings sidecar {:?}: {}", path, e);
                    continue;
                }
            };

            for entry in entries.flatten() {
                let metadata_path = entry.path();
                if !metadata_path.is_file() {
                    continue;
                }

                let Some(filename) = metadata_path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                let Some(format) = LibraryItemFormat::from_metadata_filename(filename) else {
                    continue;
                };

                let key = DocsettingsKey {
                    sidecar_stem: book_stem.to_string(),
                    format,
                };
                let duplicate_label = format!("{} ({:?})", book_stem, format);

                match index.entry(key) {
                    Entry::Occupied(_) => duplicates.push(duplicate_label),
                    Entry::Vacant(entry) => {
                        debug!(
                            "Found docsettings metadata for sidecar stem '{}' ({:?})",
                            book_stem, format
                        );
                        entry.insert(metadata_path);
                    }
                }
            }
        }
    }

    if !duplicates.is_empty() {
        bail!(
            "Found duplicate book sidecars in docsettings folder: {:?}\n\n\
            The --docsettings-path option matches books by sidecar stem and format only, not by their full path.\n\
            This is because the folder structure inside docsettings reflects the device path where \n\
            KOReader was used (e.g., /home/user/books/), which may differ from your local books path.\n\n\
            Unfortunately, KOShelf cannot distinguish between multiple books with the same sidecar stem and format \n\
            when using --docsettings-path. Consider using --hashdocsettings-path instead, which \n\
            matches books by their content hash and doesn't have this limitation.",
            duplicates
        );
    }

    info!("Found {} metadata files in docsettings folder", index.len());
    Ok(index)
}

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

#[cfg(test)]
mod tests {
    use super::{MetadataIndices, build_docsettings_index, locate_metadata_path};
    use crate::shelf::models::LibraryItemFormat;
    use crate::source::scanner::MetadataLocation;
    use std::path::Path;

    fn write_metadata(path: &Path) {
        std::fs::create_dir_all(path.parent().expect("metadata parent"))
            .expect("metadata parent dir");
        std::fs::write(path, "return {}\n").expect("metadata file");
    }

    #[test]
    fn docsettings_lookup_matches_regular_epub_by_stem_and_format() {
        let dir = tempfile::tempdir().expect("temp dir");
        let metadata_path = dir
            .path()
            .join("mnt")
            .join("onboard")
            .join("books")
            .join("Book Title.sdr")
            .join("metadata.epub.lua");
        write_metadata(&metadata_path);

        let indices =
            MetadataIndices::new(&MetadataLocation::DocSettings(dir.path().to_path_buf()))
                .expect("metadata indices");
        let book_path = dir.path().join("library").join("Book Title.epub");

        assert_eq!(
            locate_metadata_path(&indices, &book_path, LibraryItemFormat::Epub),
            Some(metadata_path)
        );
    }

    #[test]
    fn docsettings_lookup_matches_extensionless_kobo_epub_by_stem_and_format() {
        let dir = tempfile::tempdir().expect("temp dir");
        let kobo_id = "428047c7-8b1e-4d15-80d1-4d12829cd185";
        let metadata_path = dir
            .path()
            .join("mnt")
            .join("onboard")
            .join(".kobo")
            .join("kepub")
            .join(format!("{}.sdr", kobo_id))
            .join("metadata.epub.lua");
        write_metadata(&metadata_path);

        let indices =
            MetadataIndices::new(&MetadataLocation::DocSettings(dir.path().to_path_buf()))
                .expect("metadata indices");
        let book_path = dir.path().join("books").join(kobo_id);

        assert_eq!(
            locate_metadata_path(&indices, &book_path, LibraryItemFormat::Epub),
            Some(metadata_path)
        );
    }

    #[test]
    fn docsettings_lookup_keeps_same_stem_different_formats_distinct() {
        let dir = tempfile::tempdir().expect("temp dir");
        let sidecar_dir = dir.path().join("mnt").join("onboard").join("Book.sdr");
        let epub_metadata_path = sidecar_dir.join("metadata.epub.lua");
        let fb2_metadata_path = sidecar_dir.join("metadata.fb2.lua");
        write_metadata(&epub_metadata_path);
        write_metadata(&fb2_metadata_path);

        let indices =
            MetadataIndices::new(&MetadataLocation::DocSettings(dir.path().to_path_buf()))
                .expect("metadata indices");

        assert_eq!(
            locate_metadata_path(
                &indices,
                &dir.path().join("Book.epub"),
                LibraryItemFormat::Epub
            ),
            Some(epub_metadata_path)
        );
        assert_eq!(
            locate_metadata_path(
                &indices,
                &dir.path().join("Book.fb2"),
                LibraryItemFormat::Fb2
            ),
            Some(fb2_metadata_path)
        );
    }

    #[test]
    fn docsettings_index_rejects_duplicate_same_stem_and_format() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_metadata(
            &dir.path()
                .join("mnt")
                .join("onboard")
                .join("books")
                .join("Book.sdr")
                .join("metadata.epub.lua"),
        );
        write_metadata(
            &dir.path()
                .join("storage")
                .join("emulated")
                .join("0")
                .join("books")
                .join("Book.sdr")
                .join("metadata.epub.lua"),
        );

        let err = build_docsettings_index(&dir.path().to_path_buf())
            .expect_err("duplicate docsettings sidecar should fail");
        assert!(
            err.to_string()
                .contains("Found duplicate book sidecars in docsettings folder")
        );
    }
}
