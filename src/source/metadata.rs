//! Current-source discovery and stable captures shared by ingestion and writeback.
use crate::{
    shelf::models::{KoReaderMetadata, LibraryItemFormat},
    source::{
        koreader::{LuaParser, merge::normalize_partial_md5},
        scanner::MetadataLocation,
    },
};
use anyhow::Result;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub(crate) enum SourceKind {
    Adjacent,
    Docsettings,
    Hashdocsettings,
}

#[derive(Debug)]
pub(crate) struct MetadataChanged;
impl std::fmt::Display for MetadataChanged {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("metadata changed since capture")
    }
}
impl std::error::Error for MetadataChanged {}

#[derive(Debug, Clone)]
struct SelectionContext {
    excluded: Vec<PathBuf>,
    location: MetadataLocation,
    book_path: PathBuf,
    format: LibraryItemFormat,
    checksum: String,
    existing_id: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct CapturedMetadata {
    pub path: PathBuf,
    pub kind: SourceKind,
    pub modified_ns: u128,
    pub bytes: Vec<u8>,
    pub revision: String,
    pub primary_path: PathBuf,
    pub is_backup: bool,
    pub ranking_modified_secs: u128,
    selection: Option<SelectionContext>,
}
impl CapturedMetadata {
    pub fn capture(path: &Path, kind: SourceKind) -> Result<Self> {
        let file = fs::File::open(path)?;
        let before = file.metadata()?;
        let mut bytes = Vec::new();
        use std::io::Read;
        (&file).read_to_end(&mut bytes)?;
        let after = file.metadata()?;
        let current = fs::metadata(path)?;
        let stamp = |m: &fs::Metadata| -> Result<_> {
            Ok((
                m.len(),
                m.modified()?.duration_since(UNIX_EPOCH)?.as_nanos(),
            ))
        };
        if stamp(&before)? != stamp(&after)?
            || stamp(&after)? != stamp(&current)?
            || bytes.len() as u64 != after.len()
        {
            return Err(MetadataChanged.into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if before.ino() != current.ino()
                || before.dev() != current.dev()
                || before.ctime() != current.ctime()
                || before.ctime_nsec() != current.ctime_nsec()
            {
                return Err(MetadataChanged.into());
            }
        }
        let modified_ns = stamp(&after)?.1;
        let mut digest = md5::Context::new();
        digest.consume(path.as_os_str().as_encoded_bytes());
        digest.consume(format!("{kind:?}:{modified_ns}:"));
        digest.consume(&bytes);
        let is_backup = path.extension().is_some_and(|ext| ext == "old");
        let primary_path = if is_backup {
            path.with_extension("")
        } else {
            path.to_path_buf()
        };
        if is_backup {
            // Include the destination state, even when its Lua contents are invalid.
            match Self::capture(&primary_path, kind) {
                Ok(primary) => digest.consume(primary.revision.as_bytes()),
                Err(e)
                    if e.downcast_ref::<std::io::Error>()
                        .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
                {
                    digest.consume(b"missing-primary")
                }
                Err(e) => return Err(e),
            }
        }
        let revision = format!("{:x}", digest.finalize());
        Ok(Self {
            path: path.to_path_buf(),
            kind,
            modified_ns,
            bytes,
            revision,
            primary_path,
            is_backup,
            ranking_modified_secs: modified_ns / 1_000_000_000,
            selection: None,
        })
    }
    pub fn parse(&self) -> Result<KoReaderMetadata> {
        LuaParser::new().parse_bytes(&self.path, &self.bytes)
    }
    pub fn verify(&self) -> Result<()> {
        let current = Self::capture(&self.path, self.kind).map_err(|_| MetadataChanged)?;
        if current.revision != self.revision {
            return Err(MetadataChanged.into());
        }
        if let Some(context) = &self.selection {
            let resolution = MetadataIndices::new_excluding(&context.location, &context.excluded)?
                .resolve(
                    &context.book_path,
                    context.format,
                    &context.checksum,
                    context.existing_id.as_deref(),
                );
            if resolution
                .selected
                .is_none_or(|s| s.revision != self.revision)
            {
                return Err(MetadataChanged.into());
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum DiscoveryIssue {
    Inaccessible(PathBuf, String),
    Invalid(PathBuf, String),
    Unstable(PathBuf),
    Ambiguous(Vec<PathBuf>),
    ConflictingIdentities,
}
impl std::fmt::Display for DiscoveryIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inaccessible(p, e) => write!(f, "inaccessible metadata {}: {e}", p.display()),
            Self::Unstable(p) => write!(f, "metadata changed during capture: {}", p.display()),
            Self::Invalid(p, e) => write!(f, "invalid or unstable metadata {}: {e}", p.display()),
            Self::Ambiguous(p) => write!(
                f,
                "duplicate book sidecars; ambiguous metadata matches: {p:?}"
            ),
            Self::ConflictingIdentities => write!(f, "conflicting metadata identities"),
        }
    }
}
pub(crate) struct Resolution {
    pub selected: Option<CapturedMetadata>,
    pub issues: Vec<DiscoveryIssue>,
    pub blocked: bool,
    /// Normalized identities named by any parseable candidate, including blocked ones.
    pub identities: HashSet<String>,
}
#[derive(Hash, PartialEq, Eq)]
struct DocsettingsKey {
    sidecar_stem: String,
    format: LibraryItemFormat,
}
pub(crate) struct MetadataIndices {
    excluded: Vec<PathBuf>,
    pub(crate) location: MetadataLocation,
    doc: HashMap<DocsettingsKey, Vec<PathBuf>>,
    hash: HashMap<(String, String), Vec<PathBuf>>,
    /// Docsettings `.sdr` folders by stem, for custom files without reading metadata.
    sdr: HashMap<String, Vec<PathBuf>>,
    inaccessible: Vec<(PathBuf, String)>,
}
impl MetadataIndices {
    pub fn new(location: &MetadataLocation) -> Result<Self> {
        Self::new_excluding(location, &[])
    }
    pub fn new_excluding(location: &MetadataLocation, excluded: &[PathBuf]) -> Result<Self> {
        let excluded: Vec<_> = excluded
            .iter()
            .map(|p| fs::canonicalize(p).or_else(|_| std::path::absolute(p)))
            .collect::<std::io::Result<_>>()?;
        let mut this = Self {
            excluded: excluded.clone(),
            location: location.clone(),
            doc: HashMap::new(),
            hash: HashMap::new(),
            sdr: HashMap::new(),
            inaccessible: vec![],
        };
        for root in location.central_paths() {
            let is_hash = match location {
                MetadataLocation::HashDocSettings(_) => true,
                MetadataLocation::Central {
                    hashdocsettings, ..
                } => root == hashdocsettings,
                _ => false,
            };
            for entry in walkdir::WalkDir::new(root)
                .follow_links(true)
                .into_iter()
                .filter_entry(|entry| {
                    // Only directories can lead into excluded storage; skip the
                    // realpath call for plain files and when nothing is excluded.
                    if excluded.is_empty() || !entry.file_type().is_dir() {
                        return true;
                    }
                    let path = fs::canonicalize(entry.path())
                        .unwrap_or_else(|_| entry.path().to_path_buf());
                    !excluded.iter().any(|root| path.starts_with(root))
                })
            {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        this.inaccessible
                            .push((e.path().unwrap_or(root).to_path_buf(), e.to_string()));
                        continue;
                    }
                };
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                let Some(stem) = path
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|s| s.to_str())
                    .and_then(|s| s.strip_suffix(".sdr"))
                else {
                    continue;
                };
                let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                let name = name.strip_suffix(".old").unwrap_or(name);
                let primary_path = path.with_file_name(name);
                if is_hash {
                    if let Some(hash) = normalize_partial_md5(stem)
                        && name.starts_with("metadata.")
                        && name.ends_with(".lua")
                    {
                        this.hash
                            .entry((hash, name.to_string()))
                            .or_default()
                            .push(primary_path.clone());
                    }
                } else {
                    if let Some(parent) = path.parent() {
                        this.sdr
                            .entry(stem.to_string())
                            .or_default()
                            .push(parent.to_path_buf());
                    }
                    if let Some(format) =
                        LibraryItemFormat::from_sidecar_metadata_filename(stem, name)
                    {
                        this.doc
                            .entry(DocsettingsKey {
                                sidecar_stem: stem.to_string(),
                                format,
                            })
                            .or_default()
                            .push(primary_path.clone());
                    }
                }
            }
        }
        for paths in this
            .doc
            .values_mut()
            .chain(this.hash.values_mut())
            .chain(this.sdr.values_mut())
        {
            paths.sort();
            paths.dedup();
        }
        Ok(this)
    }
    /// Docsettings `.sdr` folders matching the book's sidecar stem.
    pub fn sdr_folders(&self, book: &Path) -> &[PathBuf] {
        let stem = book
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        self.sdr.get(stem).map_or(&[], Vec::as_slice)
    }
    fn candidates(
        &self,
        path: &Path,
        format: LibraryItemFormat,
        checksum: &str,
    ) -> Vec<(SourceKind, Vec<PathBuf>)> {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let mut candidates = vec![(
            SourceKind::Adjacent,
            vec![
                path.with_file_name(format!("{stem}.sdr"))
                    .join(format.metadata_filename_for_path(path)),
            ],
        )];
        if matches!(
            self.location,
            MetadataLocation::DocSettings(_) | MetadataLocation::Central { .. }
        ) {
            candidates.push((
                SourceKind::Docsettings,
                self.doc
                    .get(&DocsettingsKey {
                        sidecar_stem: stem.to_string(),
                        format,
                    })
                    .cloned()
                    .unwrap_or_default(),
            ));
        }
        if matches!(
            self.location,
            MetadataLocation::HashDocSettings(_) | MetadataLocation::Central { .. }
        ) {
            candidates.push((
                SourceKind::Hashdocsettings,
                self.hash
                    .get(&(
                        checksum.to_string(),
                        format.metadata_filename_for_path(path).to_string(),
                    ))
                    .cloned()
                    .unwrap_or_default(),
            ));
        }
        candidates
    }
    pub fn resolve(
        &self,
        path: &Path,
        format: LibraryItemFormat,
        checksum: &str,
        existing_id: Option<&str>,
    ) -> Resolution {
        let mut result = Resolution {
            selected: None,
            issues: self
                .inaccessible
                .iter()
                .map(|(p, e)| DiscoveryIssue::Inaccessible(p.clone(), e.clone()))
                .collect(),
            blocked: false,
            identities: HashSet::new(),
        };
        let mut valid = vec![];
        let mut identities = HashSet::new();
        for (kind, paths) in self.candidates(path, format, checksum) {
            if paths.len() > 1 {
                result.identities.extend(paths.iter().filter_map(|p| {
                    CapturedMetadata::capture(p, kind)
                        .and_then(|c| c.parse())
                        .ok()?
                        .partial_md5_checksum
                        .as_deref()
                        .and_then(normalize_partial_md5)
                }));
                result.issues.push(DiscoveryIssue::Ambiguous(paths));
                result.blocked = true;
                continue;
            }
            // No sidecar is the normal state for books never opened in KOReader.
            let Some(path) = paths.first() else {
                continue;
            };
            let backup_path = crate::source::koreader::lua_writer::backup_path(path);
            let mut primary = CapturedMetadata::capture(path, kind);
            let backup = CapturedMetadata::capture(&backup_path, kind);
            // KOReader promotes the primary before parsing either file.
            if let (Ok(primary), Ok(backup)) = (&mut primary, &backup) {
                primary.ranking_modified_secs = primary
                    .ranking_modified_secs
                    .max(backup.ranking_modified_secs);
            }
            for (candidate_path, capture) in [(path, primary), (&backup_path, backup)] {
                match capture {
                    Ok(captured) => match captured.parse() {
                        Ok(metadata) => {
                            if let Some(id) = metadata
                                .partial_md5_checksum
                                .as_deref()
                                .and_then(normalize_partial_md5)
                            {
                                identities.insert(id);
                            }
                            valid.push(captured);
                        }
                        Err(e) => result.issues.push(DiscoveryIssue::Invalid(
                            candidate_path.clone(),
                            e.to_string(),
                        )),
                    },
                    Err(e) => {
                        if e.is::<MetadataChanged>() {
                            result
                                .issues
                                .push(DiscoveryIssue::Unstable(candidate_path.clone()));
                        } else if e
                            .downcast_ref::<std::io::Error>()
                            .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                        {
                            // A missing sidecar or `.old` backup is expected.
                        } else {
                            result.issues.push(DiscoveryIssue::Inaccessible(
                                candidate_path.clone(),
                                e.to_string(),
                            ));
                        }
                    }
                }
            }
        }
        if identities.len() > 1 || existing_id.is_some_and(|id| identities.iter().any(|v| v != id))
        {
            result.issues.push(DiscoveryIssue::ConflictingIdentities);
            result.blocked = true;
        }
        result.identities.extend(identities);
        valid.sort_by(|a, b| {
            b.ranking_modified_secs
                .cmp(&a.ranking_modified_secs)
                .then(a.kind.cmp(&b.kind))
                .then(a.is_backup.cmp(&b.is_backup))
        });
        if !result.blocked {
            result.selected = valid.into_iter().next().map(|mut selected| {
                selected.selection = Some(SelectionContext {
                    excluded: self.excluded.clone(),
                    location: self.location.clone(),
                    book_path: path.to_path_buf(),
                    format,
                    checksum: checksum.to_string(),
                    existing_id: existing_id.map(str::to_string),
                });
                selected
            });
        }
        result
    }
}

#[cfg(test)]
pub(crate) fn locate_metadata_path(
    indices: &MetadataIndices,
    path: &Path,
    format: LibraryItemFormat,
) -> Option<PathBuf> {
    indices
        .candidates(
            path,
            format,
            &crate::source::koreader::calculate_partial_md5(path).unwrap_or_default(),
        )
        .into_iter()
        .find_map(|(_, paths)| (paths.len() == 1 && paths[0].exists()).then(|| paths[0].clone()))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    fn put(path: &Path, data: &str, seconds: u64) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, data).unwrap();
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(seconds)))
            .unwrap();
    }
    fn setup() -> (tempfile::TempDir, MetadataLocation, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let doc = dir.path().join("doc");
        let hash = dir.path().join("hash");
        fs::create_dir_all(&doc).unwrap();
        fs::create_dir_all(&hash).unwrap();
        let a = doc.join("Book.sdr/metadata.epub.lua");
        let b = hash.join("aa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.sdr/metadata.epub.lua");
        (
            dir,
            MetadataLocation::Central {
                docsettings: doc,
                hashdocsettings: hash,
            },
            a,
            b,
        )
    }
    fn resolve(location: &MetadataLocation) -> Resolution {
        MetadataIndices::new(location).unwrap().resolve(
            Path::new("Book.epub"),
            LibraryItemFormat::Epub,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            None,
        )
    }
    #[test]
    fn timestamps_within_one_second_use_koreader_location_priority() {
        let (_dir, location, doc, hash) = setup();
        put(&doc, "return {percent_finished=0.2}", 100);
        put(&hash, "return {percent_finished=0.3}", 100);
        fs::File::options()
            .write(true)
            .open(&hash)
            .unwrap()
            .set_times(
                fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_millis(100_900)),
            )
            .unwrap();
        assert_eq!(resolve(&location).selected.unwrap().path, doc);
        let captured = CapturedMetadata::capture(&hash, SourceKind::Hashdocsettings).unwrap();
        fs::File::options()
            .write(true)
            .open(&hash)
            .unwrap()
            .set_times(
                fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_millis(100_950)),
            )
            .unwrap();
        assert!(captured.verify().is_err());
    }

    #[test]
    fn migration_both_directions_and_tie_priority() {
        let (_dir, location, a, b) = setup();
        put(&a, "return { percent_finished = 0.2 }", 100);
        assert_eq!(resolve(&location).selected.unwrap().path, a);
        put(&b, "return { percent_finished = 0.3 }", 200);
        assert_eq!(resolve(&location).selected.unwrap().path, b);
        put(&a, "return { percent_finished = 0.4 }", 300);
        assert_eq!(resolve(&location).selected.unwrap().path, a);
        put(&b, "return { percent_finished = 0.5 }", 300);
        assert_eq!(resolve(&location).selected.unwrap().path, a);
    }
    #[test]
    fn older_leftover_is_selected_when_newer_source_disappears() {
        let (_dir, location, a, b) = setup();
        put(&a, "return {percent_finished=0.2}", 100);
        put(&b, "return {percent_finished=0.3}", 200);
        assert_eq!(resolve(&location).selected.unwrap().path, b);
        fs::remove_file(&b).unwrap();
        assert_eq!(resolve(&location).selected.unwrap().path, a);
    }
    #[test]
    fn invalid_and_empty_tables_do_not_erase_valid_metadata() {
        let (_dir, location, a, b) = setup();
        put(&a, "return { annotations = {} }", 100);
        for contents in ["return {}", "not lua", "", "return 42"] {
            put(&b, contents, 200);
            let result = resolve(&location);
            assert_eq!(result.selected.unwrap().path, a);
            assert!(
                result
                    .issues
                    .iter()
                    .any(|i| matches!(i, DiscoveryIssue::Invalid(..)))
            );
        }
        fs::remove_file(a).unwrap();
        assert!(resolve(&location).selected.is_none());
    }
    #[test]
    fn conflicting_identities_and_ambiguous_layouts_are_blocked() {
        let (_dir, location, a, b) = setup();
        put(
            &a,
            "return {partial_md5_checksum='11111111111111111111111111111111'}",
            100,
        );
        put(
            &b,
            "return {partial_md5_checksum='22222222222222222222222222222222'}",
            200,
        );
        assert!(resolve(&location).blocked);
        fs::remove_file(&b).unwrap();
        let MetadataLocation::Central { docsettings, .. } = &location else {
            unreachable!()
        };
        put(
            &docsettings.join("other/Book.sdr/metadata.epub.lua"),
            "return {annotations={}}",
            300,
        );
        let result = resolve(&location);
        assert!(result.blocked);
        assert!(
            result
                .issues
                .iter()
                .any(|i| matches!(i, DiscoveryIssue::Ambiguous(_)))
        );
    }
    #[test]
    fn hash_lookup_uses_exact_format_and_zip_suffix() {
        let (dir, _, _, b) = setup();
        put(&b, "return {percent_finished=0.2}", 100);
        let zip = b.with_file_name("metadata.zip.lua");
        put(&zip, "return {percent_finished=0.3}", 100);
        let indices =
            MetadataIndices::new(&MetadataLocation::HashDocSettings(dir.path().join("hash")))
                .unwrap();
        let r = indices.resolve(
            Path::new("Book.fb2.zip"),
            LibraryItemFormat::Fb2,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            None,
        );
        assert_eq!(r.selected.unwrap().path, zip);
        assert!(
            indices
                .resolve(
                    Path::new("Book.mobi"),
                    LibraryItemFormat::Mobi,
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    None
                )
                .selected
                .is_none()
        );
    }
    #[test]
    fn revision_detects_path_switch_and_equal_stat_content_changes() {
        let (_dir, _, a, b) = setup();
        put(&a, "return {percent_finished=0.2}", 100);
        put(&b, "return {percent_finished=0.2}", 100);
        let saved = CapturedMetadata::capture(&a, SourceKind::Docsettings).unwrap();
        let switched = CapturedMetadata::capture(&b, SourceKind::Hashdocsettings).unwrap();
        assert_ne!(saved.revision, switched.revision);
        put(&a, "return {percent_finished=0.3}", 100);
        assert!(saved.verify().is_err());
    }
    #[test]
    fn backups_are_candidates_but_unconfigured_directories_are_not() {
        let (dir, location, a, _) = setup();
        put(
            &a.with_file_name("metadata.epub.lua.old"),
            "return {percent_finished=0.9}",
            300,
        );
        assert!(resolve(&location).selected.unwrap().is_backup);
        put(&a, "return {annotations={}}", 400);
        let adjacent_only = MetadataIndices::new(&MetadataLocation::InBookFolder).unwrap();
        assert!(
            adjacent_only
                .resolve(
                    &dir.path().join("Book.epub"),
                    LibraryItemFormat::Epub,
                    "",
                    None
                )
                .selected
                .is_none()
        );
    }
    #[test]
    fn inaccessible_root_is_diagnosed() {
        let dir = tempfile::tempdir().unwrap();
        let r = resolve(&MetadataLocation::DocSettings(dir.path().join("missing")));
        assert!(
            r.issues
                .iter()
                .any(|i| matches!(i, DiscoveryIssue::Inaccessible(..)))
        );
        assert!(r.selected.is_none());
    }
    #[test]
    fn all_three_locations_migrate_and_ties_match_koreader() {
        let (dir, location, doc, hash) = setup();
        let book = dir.path().join("Book.epub");
        let adjacent = dir.path().join("Book.sdr/metadata.epub.lua");
        let choose = || {
            MetadataIndices::new(&location)
                .unwrap()
                .resolve(
                    &book,
                    LibraryItemFormat::Epub,
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    None,
                )
                .selected
                .unwrap()
        };
        put(&adjacent, "return {annotations={}}", 100);
        assert_eq!(choose().path, adjacent);
        put(&doc, "return {annotations={}}", 200);
        assert_eq!(choose().path, doc);
        put(&hash, "return {annotations={}}", 300);
        assert_eq!(choose().path, hash);
        put(&adjacent, "return {annotations={}}", 400);
        assert_eq!(choose().path, adjacent);
        put(&doc, "return {annotations={}}", 400);
        put(&hash, "return {annotations={}}", 400);
        assert_eq!(choose().path, adjacent);
        fs::remove_file(&adjacent).unwrap();
        assert_eq!(choose().path, doc);
    }

    #[test]
    fn inaccessible_central_directory_does_not_hide_valid_adjacent_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let book = dir.path().join("Book.epub");
        let adjacent = dir.path().join("Book.sdr/metadata.epub.lua");
        put(&adjacent, "return {annotations={}}", 100);
        let r = MetadataIndices::new(&MetadataLocation::DocSettings(
            dir.path().join("unavailable"),
        ))
        .unwrap()
        .resolve(&book, LibraryItemFormat::Epub, "", None);
        assert_eq!(r.selected.unwrap().path, adjacent);
        assert!(!r.issues.is_empty());
    }
    #[cfg(unix)]
    #[test]
    fn central_sidecar_symlinks_remain_discoverable() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("original/metadata.epub.lua");
        put(&target, "return {annotations={}}", 100);
        let root = dir.path().join("doc");
        fs::create_dir_all(&root).unwrap();
        std::os::unix::fs::symlink(target.parent().unwrap(), root.join("Book.sdr")).unwrap();
        let resolution = resolve(&MetadataLocation::DocSettings(root));
        assert!(resolution.selected.is_some());
    }
    #[test]
    fn backup_revision_tracks_primary_changes_and_verification_tracks_new_sources() {
        let (dir, location, doc, hash) = setup();
        let backup = crate::source::koreader::lua_writer::backup_path(&doc);
        put(&backup, "return {percent_finished=0.2}", 100);
        let selected = resolve(&location).selected.unwrap();
        assert!(selected.is_backup);
        assert_eq!(selected.primary_path, doc);
        put(&doc, "return {}", 50);
        assert!(selected.verify().is_err());
        let selected = resolve(&location).selected.unwrap();
        put(&doc, "return {}", 60);
        assert!(selected.verify().is_err());
        let selected = resolve(&location).selected.unwrap();
        put(&hash, "return {percent_finished=0.8}", 200);
        assert!(selected.verify().is_err());
        assert_eq!(resolve(&location).selected.unwrap().path, hash);
        assert!(dir.path().exists());
    }

    #[test]
    fn a_primary_and_backup_are_one_association_in_each_central_index() {
        let (_dir, location, doc, hash) = setup();
        for path in [&doc, &hash] {
            put(path, "return {percent_finished=0.2}", 100);
            put(
                &crate::source::koreader::lua_writer::backup_path(path),
                "return {percent_finished=0.1}",
                200,
            );
        }
        let result = resolve(&location);
        assert!(!result.blocked);
        assert_eq!(result.selected.unwrap().path, doc);
    }

    #[test]
    fn recreated_central_root_discovers_new_backup_source() {
        let (dir, location, doc, _) = setup();
        put(&doc, "return {percent_finished=0.2}", 100);
        assert_eq!(resolve(&location).selected.unwrap().path, doc);
        std::fs::rename(dir.path().join("doc"), dir.path().join("disconnected")).unwrap();
        assert!(resolve(&location).selected.is_none());
        let backup = crate::source::koreader::lua_writer::backup_path(&doc);
        put(&backup, "return {percent_finished=0.8}", 200);
        assert_eq!(resolve(&location).selected.unwrap().path, backup);
    }
    #[test]
    fn managed_storage_is_excluded_from_configured_metadata_discovery() {
        let (_dir, location, doc, hash) = setup();
        put(
            &doc,
            "return {partial_md5_checksum='aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',summary={rating=5}}",
            200,
        );
        put(
            &hash,
            "return {partial_md5_checksum='aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',summary={rating=1}}",
            100,
        );
        let excluded = doc.parent().unwrap().parent().unwrap().to_path_buf();
        let result = MetadataIndices::new_excluding(&location, &[excluded])
            .unwrap()
            .resolve(
                Path::new("Book.epub"),
                LibraryItemFormat::Epub,
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                None,
            );
        assert_eq!(result.selected.unwrap().path, hash);
    }
}
