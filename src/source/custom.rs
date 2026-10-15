//! Discovery of KOReader custom files (`custom_metadata.lua`, `cover.*`).
//!
//! KOReader looks in its sidecar locations in preference order and uses the
//! first file it finds; KoShelf does the same.
use super::metadata::MetadataIndices;
use super::scanner::MetadataLocation;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

pub const MAX_SOURCE_BYTES: u64 = 20 * 1024 * 1024;
const METADATA_FILE: &str = "custom_metadata.lua";

/// KOReader's `document_metadata_folder` setting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Location {
    #[default]
    Doc,
    Dir,
    Hash,
}

impl Location {
    /// KOReader's lookup order for this preference.
    fn order(self) -> [Location; 3] {
        match self {
            Location::Doc => [Location::Doc, Location::Dir, Location::Hash],
            Location::Dir => [Location::Dir, Location::Doc, Location::Hash],
            Location::Hash => [Location::Hash, Location::Doc, Location::Dir],
        }
    }
}

/// The exact bytes of a custom file, so writes can detect concurrent changes.
#[derive(Debug, Clone)]
pub struct Capture {
    pub path: PathBuf,
    pub digest: String,
    pub bytes: Vec<u8>,
}

impl Capture {
    pub fn read(path: &Path) -> Result<Self> {
        let file = fs::File::open(path)?;
        ensure!(
            file.metadata()?.len() <= MAX_SOURCE_BYTES,
            "custom file exceeds 20 MiB"
        );
        let mut bytes = Vec::new();
        file.take(MAX_SOURCE_BYTES + 1).read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() as u64 <= MAX_SOURCE_BYTES,
            "custom file exceeds 20 MiB"
        );
        Ok(Self {
            path: path.to_path_buf(),
            digest: format!("{:x}", md5::compute(&bytes)),
            bytes,
        })
    }

    pub fn verify(&self) -> Result<()> {
        ensure!(
            Self::read(&self.path)?.digest == self.digest,
            "custom file changed"
        );
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Resolution {
    pub metadata: Option<Capture>,
    pub cover: Option<Capture>,
    /// Folder for files that don't exist yet.
    pub target: PathBuf,
    pub issues: Vec<String>,
}

impl Resolution {
    pub fn metadata_folder(&self) -> &Path {
        folder(self.metadata.as_ref(), &self.target)
    }
    pub fn cover_folder(&self) -> &Path {
        folder(self.cover.as_ref(), &self.target)
    }
    pub fn metadata_digest(&self) -> Option<&str> {
        self.metadata.as_ref().map(|c| c.digest.as_str())
    }
    pub fn cover_digest(&self) -> Option<&str> {
        self.cover.as_ref().map(|c| c.digest.as_str())
    }
}

fn folder<'a>(capture: Option<&'a Capture>, target: &'a Path) -> &'a Path {
    capture.and_then(|c| c.path.parent()).unwrap_or(target)
}

pub fn cover_extension(path: &Path) -> Option<String> {
    if path.file_stem()?.to_str()? != "cover" {
        return None;
    }
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    matches!(ext.as_str(), "gif" | "jpeg" | "jpg" | "png" | "webp").then_some(ext)
}

/// KOReader strips only the final extension: `Book.fb2.zip` → `Book.fb2.sdr`.
pub fn adjacent(book: &Path) -> PathBuf {
    book.with_extension("sdr")
}

pub fn is_custom_path(path: &Path) -> bool {
    path.parent()
        .and_then(Path::extension)
        .is_some_and(|s| s == "sdr")
        && (path.file_name().is_some_and(|s| s == METADATA_FILE)
            || path.file_stem().is_some_and(|s| s == "cover"))
}

/// Paths of the custom files KOReader would use, without reading them.
#[derive(Debug, Default)]
pub struct Located {
    pub metadata: Option<PathBuf>,
    pub cover: Option<PathBuf>,
    /// Folder for files that don't exist yet.
    pub target: PathBuf,
    pub issues: Vec<String>,
}

/// `sidecar` is the selected reading metadata file, whose folder KOReader
/// writes new custom files into.
pub fn resolve(
    book: &Path,
    checksum: &str,
    sidecar: Option<&Path>,
    indices: &MetadataIndices,
    preference: Option<Location>,
) -> Resolution {
    let located = locate(book, checksum, sidecar, indices, preference);
    let mut issues = located.issues;
    let mut read = |path: Option<PathBuf>| {
        let path = path?;
        Capture::read(&path)
            .map_err(|e| issues.push(format!("{}: {e:#}", path.display())))
            .ok()
    };
    Resolution {
        metadata: read(located.metadata),
        cover: read(located.cover),
        target: located.target,
        issues,
    }
}

/// Like [`resolve`], but only finds the files.
pub fn locate(
    book: &Path,
    checksum: &str,
    sidecar: Option<&Path>,
    indices: &MetadataIndices,
    preference: Option<Location>,
) -> Located {
    let preference = preference.unwrap_or_default();
    let mut result = Located::default();
    let mut folders: Vec<(Location, PathBuf)> = Vec::new();
    for location in preference.order() {
        match location {
            Location::Doc => folders.push((location, adjacent(book))),
            Location::Dir => {
                // A sidecar-only path inside a central root already identifies
                // its exact folder; other books with the same stem are unrelated.
                if indices
                    .location
                    .central_paths()
                    .iter()
                    .any(|root| book.starts_with(root))
                    && let Some(folder) = sidecar.and_then(Path::parent)
                {
                    folders.push((location, folder.to_path_buf()));
                    continue;
                }
                let matches = indices.sdr_folders(book);
                // Same-named books elsewhere share the stem; never apply their files.
                if matches.len() > 1 {
                    result
                        .issues
                        .push(format!("ambiguous docsettings folders: {matches:?}"));
                } else {
                    folders.extend(matches.iter().map(|p| (location, p.clone())));
                }
            }
            Location::Hash => {
                if let Some(root) = hash_root(&indices.location)
                    && checksum.len() >= 2
                {
                    folders.push((
                        location,
                        root.join(&checksum[..2]).join(format!("{checksum}.sdr")),
                    ));
                }
            }
        }
    }
    result.target = sidecar
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .or_else(|| {
            folders
                .iter()
                .find(|(l, _)| *l == preference)
                .map(|(_, p)| p.clone())
        })
        .unwrap_or_else(|| adjacent(book));

    for (_, folder) in &folders {
        let mut entries: Vec<PathBuf> = match fs::read_dir(folder) {
            Ok(entries) => entries.filter_map(|e| e.ok().map(|e| e.path())).collect(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                result.issues.push(format!("{}: {e}", folder.display()));
                continue;
            }
        };
        entries.sort();
        for path in entries {
            let slot = if path.file_name().is_some_and(|n| n == METADATA_FILE) {
                &mut result.metadata
            } else if cover_extension(&path).is_some() {
                &mut result.cover
            } else {
                continue;
            };
            if slot.is_none() {
                *slot = Some(path);
            }
        }
    }
    result
}

fn hash_root(location: &MetadataLocation) -> Option<&Path> {
    match location {
        MetadataLocation::HashDocSettings(root)
        | MetadataLocation::Central {
            hashdocsettings: root,
            ..
        } => Some(root),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compound_extensions_share_only_the_last_stem() {
        assert_eq!(
            adjacent(Path::new("Book.fb2.zip")),
            Path::new("Book.fb2.sdr")
        );
        assert_eq!(
            adjacent(Path::new("Book.epub")),
            adjacent(Path::new("Book.fb2"))
        );
    }

    #[test]
    fn preference_order_picks_the_first_location_like_koreader() {
        let temp = tempfile::tempdir().unwrap();
        let book = temp.path().join("Book.epub");
        let doc = adjacent(&book);
        let dir = temp.path().join("settings/device/Book.sdr");
        for folder in [&doc, &dir] {
            fs::create_dir_all(folder).unwrap();
            fs::write(folder.join(METADATA_FILE), b"return {}").unwrap();
        }
        fs::write(dir.join("cover.PNG"), b"png").unwrap();
        let roots = MetadataLocation::DocSettings(temp.path().join("settings"));
        let indices = MetadataIndices::new(&roots).unwrap();
        let resolve = |p| resolve(&book, "abcd", None, &indices, p);

        let r = resolve(Some(Location::Dir));
        assert_eq!(r.metadata.unwrap().path, dir.join(METADATA_FILE));
        let r = resolve(None);
        assert_eq!(r.metadata.unwrap().path, doc.join(METADATA_FILE));
        // Covers are found independently of the metadata file.
        assert_eq!(r.cover.unwrap().path, dir.join("cover.PNG"));
        assert_eq!(r.target, doc);

        fs::create_dir_all(temp.path().join("settings/other/Book.sdr")).unwrap();
        fs::write(temp.path().join("settings/other/Book.sdr/x.lua"), b"").unwrap();
        let indices = MetadataIndices::new(&roots).unwrap();
        let r = super::resolve(&book, "abcd", None, &indices, None);
        assert!(!r.issues.is_empty());
        // Ambiguous docsettings folders contribute nothing.
        assert_eq!(r.metadata.unwrap().path, doc.join(METADATA_FILE));
        assert!(r.cover.is_none());
    }

    #[test]
    fn hash_location_and_reading_sidecar_target() {
        let temp = tempfile::tempdir().unwrap();
        let book = temp.path().join("Book.epub");
        let hash = temp.path().join("hash/ab/abcd.sdr");
        fs::create_dir_all(&hash).unwrap();
        fs::write(hash.join("cover.jpg"), b"jpg").unwrap();
        let roots = MetadataLocation::HashDocSettings(temp.path().join("hash"));
        let indices = MetadataIndices::new(&roots).unwrap();
        let r = resolve(&book, "abcd", None, &indices, Some(Location::Hash));
        assert_eq!(r.cover.unwrap().path, hash.join("cover.jpg"));
        assert_eq!(r.target, hash);
        let sidecar = adjacent(&book).join("metadata.epub.lua");
        let r = resolve(
            &book,
            "abcd",
            Some(&sidecar),
            &indices,
            Some(Location::Hash),
        );
        assert_eq!(r.target, adjacent(&book));
    }
}
