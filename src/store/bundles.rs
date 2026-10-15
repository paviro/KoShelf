//! Durable library records. SQLite is only a projection of these captures.
use crate::{
    shelf::models::{BookInfo, LibraryItem, LibraryItemFormat},
    source::metadata::{CapturedMetadata, SourceKind},
};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retention {
    Immediate,
    Days(u32),
    Forever,
}
impl Default for Retention {
    fn default() -> Self {
        Self::Days(30)
    }
}
impl std::fmt::Display for Retention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate => write!(f, "0"),
            Self::Days(n) => write!(f, "{n}"),
            Self::Forever => write!(f, "forever"),
        }
    }
}
impl std::str::FromStr for Retention {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s == "forever" {
            return Ok(Self::Forever);
        }
        let n = s
            .parse::<u32>()
            .map_err(|_| "expected a nonnegative integer or forever".to_string())?;
        Ok(if n == 0 {
            Self::Immediate
        } else {
            Self::Days(n)
        })
    }
}
/// API shape: days as a number (0 = immediate) or the string `"forever"`.
impl Serialize for Retention {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Immediate => s.serialize_u32(0),
            Self::Days(n) => s.serialize_u32(*n),
            Self::Forever => s.serialize_str("forever"),
        }
    }
}
impl<'de> Deserialize<'de> for Retention {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Value {
            Number(u32),
            Text(String),
        }
        match Value::deserialize(d)? {
            Value::Number(0) => Ok(Self::Immediate),
            Value::Number(n) => Ok(Self::Days(n)),
            Value::Text(s) if s == "forever" => Ok(Self::Forever),
            _ => Err(serde::de::Error::custom(
                "expected a nonnegative integer or 'forever'",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Available,
    Archived,
    Retired,
}
impl State {
    fn directory(self) -> &'static str {
        match self {
            Self::Available => "snapshots",
            Self::Archived => "archived",
            Self::Retired => "retired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub path: PathBuf,
    pub filename: String,
    pub kind: SourceKind,
    pub modified_ns: u128,
    pub ranking_modified_secs: u128,
    pub revision: String,
    pub bytes_md5: String,
    pub is_backup: bool,
}

/// Versioned disk format, independent of database rows and HTTP responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub version: u32,
    pub id: String,
    pub checksum: String,
    pub book_path: PathBuf,
    pub format: LibraryItemFormat,
    pub book_info: BookInfo,
    pub source: Option<Source>,
    #[serde(skip, default = "available_state")]
    pub state: State,
    pub created_at: i64,
    pub updated_at: i64,
}

fn available_state() -> State {
    State::Available
}

const TEMP_PREFIX: &str = ".koshelf-tmp-";
/// A replaced book directory parked by the portable (two-rename) swap.
const PARKED_PREFIX: &str = ".koshelf-old-";
const MANIFEST: &str = "book.json";
const COVER: &str = "cover.webp";

#[derive(Clone)]
pub struct Bundle {
    pub record: Record,
    pub directory: PathBuf,
    // Loaded under the store lock, so readers own their bytes across later saves.
    source_bytes: Option<Arc<[u8]>>,
}
impl Bundle {
    /// The saved cover, if any. Saves replace whole book directories, so an
    /// opened cover stays readable.
    pub fn cover(&self) -> Option<PathBuf> {
        let path = self.directory.join(COVER);
        path.is_file().then_some(path)
    }
    pub fn revision(&self) -> Result<String> {
        Ok(format!(
            "{:x}",
            md5::compute(serde_json::to_vec(&(&self.record, self.record.state))?)
        ))
    }
    pub fn item(&self) -> Result<LibraryItem> {
        let metadata = self
            .record
            .source
            .as_ref()
            .map(|s| {
                let bytes = self
                    .source_bytes
                    .as_deref()
                    .context("missing captured bytes")?;
                ensure!(
                    format!("{:x}", md5::compute(bytes)) == s.bytes_md5,
                    "captured sidecar checksum mismatch"
                );
                crate::source::koreader::LuaParser::new().parse_bytes(&s.path, bytes)
            })
            .transpose()?;
        Ok(LibraryItem {
            id: self.record.id.clone(),
            file_path: self.record.book_path.clone(),
            format: self.record.format,
            book_info: self.record.book_info.clone(),
            koreader_metadata: metadata,
        })
    }
}

pub struct BundleStore {
    pub root: PathBuf,
    pub retention: Retention,
    gate: Mutex<()>,
    _lock: std::sync::Arc<fs::File>,
    #[cfg(test)]
    pub(crate) fail_publication: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub(crate) fail_transition: std::sync::atomic::AtomicBool,
}
fn sync_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    fs::File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}
fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
/// Hard-link `from` to `to` (one copy on disk), copying across filesystems.
/// Linked files are shared, so writers must replace them, never modify in place.
pub(crate) fn link_or_copy(from: &Path, to: &Path) -> Result<()> {
    if fs::hard_link(from, to).is_err() {
        write_file(to, &fs::read(from)?)?;
    }
    Ok(())
}
/// Atomically exchange two existing directories. `Ok(false)` when the platform
/// or filesystem cannot, so callers fall back to the portable swap.
fn exchange(a: &Path, b: &Path) -> std::io::Result<bool> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::{ffi::CString, os::unix::ffi::OsStrExt};
        let a = CString::new(a.as_os_str().as_bytes())?;
        let b = CString::new(b.as_os_str().as_bytes())?;
        // The raw syscall works with every glibc and musl version.
        #[cfg(target_os = "linux")]
        // SAFETY: both paths are valid NUL-terminated strings for the call's duration.
        let status = unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                libc::AT_FDCWD,
                a.as_ptr(),
                libc::AT_FDCWD,
                b.as_ptr(),
                libc::RENAME_EXCHANGE,
            )
        };
        #[cfg(target_os = "macos")]
        // SAFETY: both paths are valid NUL-terminated strings for the call's duration.
        let status = unsafe { libc::renamex_np(a.as_ptr(), b.as_ptr(), libc::RENAME_SWAP) };
        if status == 0 {
            return Ok(true);
        }
        let error = std::io::Error::last_os_error();
        match error.raw_os_error() {
            Some(libc::ENOSYS | libc::EINVAL | libc::ENOTSUP) => Ok(false),
            _ => Err(error),
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (a, b);
        Ok(false)
    }
}
/// Replace the book directory `current` with the complete `staged` one. After
/// the atomic exchange, `staged` holds the previous version for its owner to
/// drop. The portable fallback parks the previous version; startup recovery
/// restores it if a crash leaves no current directory.
fn swap_in(staged: &Path, current: &Path) -> Result<()> {
    let parent = current.parent().context("book parent")?;
    if !exchange(staged, current)? {
        let name = current.file_name().context("book name")?.to_string_lossy();
        let parked = parent.join(format!("{PARKED_PREFIX}{name}"));
        fs::rename(current, &parked)?;
        fs::rename(staged, current)?;
        sync_dir(parent)?;
        if let Err(e) = fs::remove_dir_all(&parked) {
            log::warn!("Cannot remove replaced capture {}: {e}", parked.display());
        }
    }
    sync_dir(parent)
}
fn canonical(id: &str) -> Result<()> {
    ensure!(
        crate::pipeline::media::is_canonical_item_id(id),
        "invalid record identity"
    );
    Ok(())
}
pub(crate) fn acquire_storage_lock(data: &Path) -> Result<std::sync::Arc<fs::File>> {
    let data = std::path::absolute(data)?;
    let root = data.join("library");
    fs::create_dir_all(&root)?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("writer.lock"))?;
    lock.try_lock()
        .context("another process is using this library storage")?;
    Ok(std::sync::Arc::new(lock))
}

/// Inspect saved files without initializing or cleaning up archive storage.
pub(crate) fn has_saved_files(data: &Path) -> Result<bool> {
    for name in ["snapshots", "archived", "retired"] {
        match fs::read_dir(data.join("library").join(name)) {
            Ok(mut entries) => {
                if entries.next().transpose()?.is_some() {
                    return Ok(true);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(false)
}

impl BundleStore {
    #[cfg(test)]
    pub fn open(data: &Path, retention: Retention) -> Result<Self> {
        Self::open_locked(data, retention, acquire_storage_lock(data)?)
    }

    pub(crate) fn open_locked(
        data: &Path,
        retention: Retention,
        lock: std::sync::Arc<fs::File>,
    ) -> Result<Self> {
        let data = std::path::absolute(data)?;
        let root = data.join("library");
        for dir in ["snapshots", "archived", "retired"] {
            fs::create_dir_all(root.join(dir))?;
        }
        sync_dir(&root)?;
        sync_dir(&data)?;
        if let Some(parent) = data.parent() {
            sync_dir(parent)?;
        }
        let this = Self {
            root,
            retention,
            gate: Mutex::new(()),
            _lock: lock,
            #[cfg(test)]
            fail_publication: std::sync::atomic::AtomicBool::new(false),
            #[cfg(test)]
            fail_transition: std::sync::atomic::AtomicBool::new(false),
        };
        if let Err(e) = this.cleanup_startup() {
            log::warn!("Temporary capture cleanup pending: {e:#}");
        }
        if let Err(e) = this.expire() {
            log::warn!("Retired file cleanup pending: {e:#}");
        }
        Ok(this)
    }
    fn read(&self, directory: PathBuf) -> Result<Bundle> {
        let mut record: Record = serde_json::from_slice(&fs::read(directory.join(MANIFEST))?)?;
        ensure!(
            record.version == 1,
            "unsupported library bundle version {}",
            record.version
        );
        canonical(&record.id)?;
        let parent = directory.parent().context("missing bundle parent")?;
        record.state = if parent == self.root.join("snapshots") {
            State::Available
        } else if parent == self.root.join("archived") {
            State::Archived
        } else if parent == self.root.join("retired") {
            let (_, id) = retired_identity(&directory)?;
            ensure!(id == record.id, "retired identity mismatch");
            State::Retired
        } else {
            bail!("invalid bundle location");
        };
        if record.state != State::Retired {
            ensure!(
                directory.file_name().and_then(|n| n.to_str()) == Some(record.id.as_str()),
                "bundle identity mismatch"
            );
        }
        if let Some(source) = &record.source {
            ensure!(
                Path::new(&source.filename)
                    .file_name()
                    .and_then(|v| v.to_str())
                    == Some(source.filename.as_str())
                    && LibraryItemFormat::is_metadata_file(&source.filename),
                "invalid captured filename"
            );
        }
        let source_bytes = record
            .source
            .as_ref()
            .map(|source| fs::read(directory.join(&source.filename)).map(Arc::from))
            .transpose()?;
        let bundle = Bundle {
            record,
            directory,
            source_bytes,
        };
        bundle.item()?;
        Ok(bundle)
    }
    /// Enumerate identities separately from parsing captures: corrupt records are
    /// still present and must never be mistaken for explicit removal.
    pub fn present_ids(&self) -> Result<std::collections::HashSet<String>> {
        let _guard = self
            .gate
            .lock()
            .map_err(|_| anyhow::anyhow!("bundle lock poisoned"))?;
        let mut ids = std::collections::HashSet::new();
        for state in ["snapshots", "archived"] {
            for entry in fs::read_dir(self.root.join(state))? {
                let entry = entry?;
                if is_temporary(&entry.path()) || is_foreign(&entry.path()) {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                ensure!(
                    entry.file_type()?.is_dir(),
                    "bundle is not a directory: {name}"
                );
                ensure!(ids.insert(name), "duplicate managed identity");
            }
        }
        Ok(ids)
    }
    pub fn records(&self) -> Result<Vec<Bundle>> {
        let _guard = self
            .gate
            .lock()
            .map_err(|_| anyhow::anyhow!("bundle lock poisoned"))?;
        let mut records = Vec::new();
        for dir in ["snapshots", "archived"] {
            for entry in fs::read_dir(self.root.join(dir))? {
                let path = entry?.path();
                if is_temporary(&path) || is_foreign(&path) {
                    continue;
                }
                match self.read(path.clone()) {
                    Ok(b) => records.push(b),
                    Err(e) => log::warn!("Cannot read managed record {}: {e:#}", path.display()),
                }
            }
        }
        records.sort_by(|a, b| a.record.id.cmp(&b.record.id));
        Ok(records)
    }
    pub fn get(&self, id: &str) -> Result<Option<Bundle>> {
        canonical(id)?;
        let _guard = self
            .gate
            .lock()
            .map_err(|_| anyhow::anyhow!("bundle lock poisoned"))?;
        self.get_unlocked(id)
    }
    fn get_unlocked(&self, id: &str) -> Result<Option<Bundle>> {
        for dir in ["snapshots", "archived"] {
            let path = self.root.join(dir).join(id);
            match fs::metadata(&path) {
                Ok(_) => return self.read(path).map(Some),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(None)
    }
    pub fn capture(
        &self,
        item: &LibraryItem,
        checksum: &str,
        capture: Option<&CapturedMetadata>,
        cover: Option<&Path>,
        state: State,
    ) -> Result<Bundle> {
        canonical(&item.id)?;
        ensure!(
            state != State::Retired,
            "retirement requires a state transition"
        );
        let _guard = self
            .gate
            .lock()
            .map_err(|_| anyhow::anyhow!("bundle lock poisoned"))?;
        let old = self.get_unlocked(&item.id)?;
        let now = chrono::Utc::now().timestamp();
        let source = match capture {
            Some(c) => {
                c.parse()?;
                Some(Source {
                    path: c.path.clone(),
                    // Saved under the primary name (also for `.old` backups), so
                    // restoring is a plain copy next to the ebook.
                    filename: {
                        let name = c
                            .path
                            .file_name()
                            .context("metadata filename")?
                            .to_str()
                            .context("metadata filename encoding")?;
                        name.strip_suffix(".old").unwrap_or(name).to_string()
                    },
                    kind: c.kind,
                    modified_ns: c.modified_ns,
                    ranking_modified_secs: c.ranking_modified_secs,
                    revision: c.revision.clone(),
                    bytes_md5: format!("{:x}", md5::compute(&c.bytes)),
                    is_backup: c.is_backup,
                })
            }
            None => old.as_ref().and_then(|b| b.record.source.clone()),
        };
        let mut book_info = item.book_info.clone();
        book_info.cover_data = None;
        let record = Record {
            version: 1,
            id: item.id.clone(),
            checksum: checksum.to_string(),
            book_path: item.file_path.clone(),
            format: item.format,
            book_info,
            source,
            state,
            created_at: old.as_ref().map_or(now, |b| b.record.created_at),
            updated_at: now,
        };
        self.commit(
            record,
            old.as_ref(),
            capture.map(|c| c.bytes.as_slice()),
            cover,
        )
    }
    pub fn transition(&self, id: &str, state: State) -> Result<Bundle> {
        canonical(id)?;
        let _guard = self
            .gate
            .lock()
            .map_err(|_| anyhow::anyhow!("bundle lock poisoned"))?;
        let old = self.get_unlocked(id)?.context("managed record not found")?;
        self.move_bundle(old, state)
    }
    fn move_bundle(&self, old: Bundle, state: State) -> Result<Bundle> {
        if old.record.state == state {
            return Ok(old);
        }
        let name = if state == State::Retired {
            format!(
                "{}-{}-{}",
                chrono::Utc::now().timestamp(),
                old.record.id,
                uuid::Uuid::new_v4()
            )
        } else {
            old.record.id.clone()
        };
        let destination = self.root.join(state.directory()).join(name);
        ensure!(!destination.try_exists()?, "destination already exists");
        #[cfg(test)]
        ensure!(
            !self
                .fail_transition
                .load(std::sync::atomic::Ordering::Relaxed),
            "injected publication failure"
        );
        fs::rename(&old.directory, &destination)?;
        sync_dir(destination.parent().context("destination parent")?)?;
        sync_dir(old.directory.parent().context("previous parent")?)?;
        self.read(destination)
    }
    /// Every save writes a complete new book directory and swaps it in, so
    /// readers and crashes only ever see a whole previous or whole new capture.
    fn commit(
        &self,
        record: Record,
        old: Option<&Bundle>,
        bytes: Option<&[u8]>,
        cover: Option<&Path>,
    ) -> Result<Bundle> {
        #[cfg(test)]
        ensure!(
            !self
                .fail_publication
                .load(std::sync::atomic::Ordering::Relaxed),
            "injected publication failure"
        );
        let desired_state = record.state;
        let parent = match old {
            Some(b) => b.directory.parent().context("book parent")?.to_path_buf(),
            None => self.root.join(desired_state.directory()),
        };
        // Dropping the staged directory removes it on failure, or the replaced
        // previous version after an atomic exchange.
        let staged = tempfile::Builder::new()
            .prefix(TEMP_PREFIX)
            .tempdir_in(&parent)?;
        let directory = staged.path();
        if let Some(source) = &record.source {
            let destination = directory.join(&source.filename);
            match bytes {
                Some(bytes) => write_file(&destination, bytes)?,
                None => {
                    let old = old.context("missing previous capture")?;
                    link_or_copy(&old.directory.join(&source.filename), &destination)?;
                }
            }
        }
        if let Some(cover) = cover
            .map(Path::to_path_buf)
            .or_else(|| old.and_then(Bundle::cover))
        {
            link_or_copy(&cover, &directory.join(COVER))?;
        }
        write_file(
            &directory.join(MANIFEST),
            &serde_json::to_vec_pretty(&record)?,
        )?;
        sync_dir(directory)?;
        let destination = parent.join(&record.id);
        if old.is_some() {
            swap_in(directory, &destination)?;
        } else {
            ensure!(!destination.try_exists()?, "destination already exists");
            fs::rename(directory, &destination)?;
            sync_dir(&parent)?;
        }
        drop(staged);
        let published = self.read(destination)?;
        self.move_bundle(published, desired_state)
    }
    fn cleanup_startup(&self) -> Result<()> {
        for state in ["snapshots", "archived", "retired"] {
            for entry in fs::read_dir(self.root.join(state))? {
                let path = entry?.path();
                let result = if is_temporary(&path) {
                    remove_entry(&path)
                } else if let Some(id) = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .and_then(|n| n.strip_prefix(PARKED_PREFIX))
                {
                    // An interrupted portable swap: keep the new version if it
                    // was published, otherwise restore the previous one.
                    let current = path.with_file_name(id);
                    match current.try_exists() {
                        Ok(true) => remove_entry(&path),
                        Ok(false) => fs::rename(&path, &current).map_err(Into::into),
                        Err(e) => Err(e.into()),
                    }
                } else {
                    continue;
                };
                if let Err(e) = result {
                    log::warn!("Cannot clean {}: {e:#}", path.display());
                }
            }
        }
        Ok(())
    }
    pub fn expire(&self) -> Result<()> {
        self.expire_at(chrono::Utc::now().timestamp())
    }

    pub(crate) fn expire_at(&self, now: i64) -> Result<()> {
        let _guard = self
            .gate
            .lock()
            .map_err(|_| anyhow::anyhow!("bundle lock poisoned"))?;
        let seconds = match self.retention {
            Retention::Forever => return Ok(()),
            Retention::Immediate => 0,
            Retention::Days(n) => i64::from(n) * 86400,
        };
        for entry in fs::read_dir(self.root.join("retired"))? {
            let result = (|| -> Result<()> {
                let path = entry?.path();
                let (removed_at, _) = retired_identity(&path)
                    .with_context(|| format!("Invalid retired directory {}", path.display()))?;
                if now.saturating_sub(removed_at) >= seconds {
                    fs::remove_dir_all(&path)
                        .with_context(|| format!("Cannot delete {}", path.display()))?;
                }
                Ok(())
            })();
            if let Err(e) = result {
                log::warn!("Retired file cleanup pending: {e:#}");
            }
        }
        sync_dir(&self.root.join("retired"))
    }
}

fn is_temporary(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.starts_with(TEMP_PREFIX))
}
/// Entries that can never be managed records, such as `.DS_Store` or NAS
/// metadata folders. They are left alone and ignored rather than failing
/// enumeration; a corrupt record with a valid identity is still reported.
fn is_foreign(path: &Path) -> bool {
    let in_retired = path
        .parent()
        .and_then(|p| p.file_name())
        .is_some_and(|n| n == "retired");
    let foreign = if in_retired {
        retired_identity(path).is_err()
    } else {
        path.file_name()
            .and_then(|s| s.to_str())
            .is_none_or(|name| canonical(name).is_err())
    };
    if foreign {
        log::debug!(
            "Ignoring foreign entry in library storage: {}",
            path.display()
        );
    }
    foreign
}
fn remove_entry(path: &Path) -> Result<()> {
    if fs::symlink_metadata(path)?.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}
fn retired_identity(path: &Path) -> Result<(i64, &str)> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .context("invalid retired name")?;
    let (timestamp, rest) = name.split_once('-').context("invalid retired name")?;
    let timestamp: i64 = timestamp.parse().context("invalid retirement timestamp")?;
    ensure!(timestamp >= 0, "negative retirement timestamp");
    let (id, token) = rest.split_once('-').context("invalid retired name")?;
    canonical(id)?;
    uuid::Uuid::parse_str(token).context("invalid retired token")?;
    Ok((timestamp, id))
}

#[cfg(test)]
mod tests {
    use super::*;
    const ID: &str = "0123456789abcdef0123456789abcdef";
    fn fixture(dir: &Path) -> (LibraryItem, CapturedMetadata) {
        let path = dir.join("metadata.epub.lua");
        fs::write(&path, format!("-- preserve this comment\nreturn {{partial_md5_checksum='{ID}', annotations={{{{text='saved',drawer='lighten'}}}}}}\n")).unwrap();
        let capture = CapturedMetadata::capture(&path, SourceKind::Adjacent).unwrap();
        let mut item = crate::tests::fixtures::library_item(ID, Some(capture.parse().unwrap()));
        item.file_path = dir.join("book.epub");
        (item, capture)
    }
    #[test]
    fn disabled_storage_preserves_files_and_excludes_other_writers() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!has_saved_files(dir.path()).unwrap());
        let (item, capture) = fixture(dir.path());
        let store = BundleStore::open(dir.path(), Retention::Forever).unwrap();
        store
            .capture(&item, ID, Some(&capture), None, State::Available)
            .unwrap();
        store.transition(ID, State::Retired).unwrap();
        drop(store);
        let retired = fs::read_dir(dir.path().join("library/retired"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let before = fs::read(retired.join(MANIFEST)).unwrap();
        let lock = acquire_storage_lock(dir.path()).unwrap();
        assert!(has_saved_files(dir.path()).unwrap());
        assert!(BundleStore::open(dir.path(), Retention::Immediate).is_err());
        assert_eq!(fs::read(retired.join(MANIFEST)).unwrap(), before);
        drop(lock);
        let _store = BundleStore::open(dir.path(), Retention::Immediate).unwrap();
        assert!(!retired.exists());
    }

    #[test]
    fn exact_bytes_metadata_and_cover_survive_lifecycle_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let (item, capture) = fixture(dir.path());
        let cover = dir.path().join("old.webp");
        fs::write(&cover, b"cover bytes").unwrap();
        let store = BundleStore::open(dir.path(), Retention::Forever).unwrap();
        let saved = store
            .capture(&item, ID, Some(&capture), Some(&cover), State::Available)
            .unwrap();
        assert_eq!(
            fs::read(saved.directory.join("metadata.epub.lua")).unwrap(),
            capture.bytes
        );
        store.transition(ID, State::Archived).unwrap();
        fs::remove_file(capture.path).unwrap();
        drop(store);
        let store = BundleStore::open(dir.path(), Retention::Forever).unwrap();
        let archived = store.get(ID).unwrap().unwrap();
        assert_eq!(
            archived.item().unwrap().annotations()[0].text.as_deref(),
            Some("saved")
        );
        assert_eq!(fs::read(archived.cover().unwrap()).unwrap(), b"cover bytes");
        store.transition(ID, State::Retired).unwrap();
        store.expire().unwrap();
        assert!(store.get(ID).unwrap().is_none());
        assert_eq!(fs::read_dir(store.root.join("retired")).unwrap().count(), 1);
        store
            .capture(&item, ID, None, None, State::Available)
            .unwrap();
        store.transition(ID, State::Archived).unwrap();
        store.transition(ID, State::Retired).unwrap();
        assert_eq!(fs::read_dir(store.root.join("retired")).unwrap().count(), 2);
    }
    #[test]
    fn immediate_expiry_and_policy_change_preserve_originals() {
        let dir = tempfile::tempdir().unwrap();
        let (item, capture) = fixture(dir.path());
        let store = BundleStore::open(dir.path(), Retention::Forever).unwrap();
        store
            .capture(&item, ID, Some(&capture), None, State::Available)
            .unwrap();
        store.transition(ID, State::Archived).unwrap();
        store.transition(ID, State::Retired).unwrap();
        drop(store);
        let store = BundleStore::open(dir.path(), Retention::Immediate).unwrap();
        assert_eq!(fs::read_dir(store.root.join("retired")).unwrap().count(), 0);
        assert_eq!(fs::read(&capture.path).unwrap(), capture.bytes);
    }
    #[test]
    fn missing_and_invalid_sources_keep_last_capture_valid_empty_annotations_replace_it() {
        let dir = tempfile::tempdir().unwrap();
        let (mut item, capture) = fixture(dir.path());
        let store = BundleStore::open(dir.path(), Retention::default()).unwrap();
        store
            .capture(&item, ID, Some(&capture), None, State::Available)
            .unwrap();
        item.koreader_metadata = None;
        let saved = store
            .capture(&item, ID, None, None, State::Available)
            .unwrap();
        assert_eq!(saved.item().unwrap().annotations().len(), 1);
        fs::write(&capture.path, "invalid lua").unwrap();
        let invalid = CapturedMetadata::capture(&capture.path, SourceKind::Adjacent).unwrap();
        assert!(
            store
                .capture(&item, ID, Some(&invalid), None, State::Available)
                .is_err()
        );
        assert_eq!(
            store
                .get(ID)
                .unwrap()
                .unwrap()
                .item()
                .unwrap()
                .annotations()
                .len(),
            1
        );
        fs::write(
            &capture.path,
            format!("return {{partial_md5_checksum='{ID}', annotations={{}}}}"),
        )
        .unwrap();
        let empty = CapturedMetadata::capture(&capture.path, SourceKind::Adjacent).unwrap();
        let saved = store
            .capture(&item, ID, Some(&empty), None, State::Available)
            .unwrap();
        assert!(saved.item().unwrap().annotations().is_empty());
    }
    #[test]
    fn corrupt_and_unsupported_records_are_not_overwritten_or_deleted() {
        let dir = tempfile::tempdir().unwrap();
        let (item, _) = fixture(dir.path());
        let store = BundleStore::open(dir.path(), Retention::Immediate).unwrap();
        let b = store
            .capture(&item, ID, None, None, State::Available)
            .unwrap();
        let mut record = b.record;
        record.version = 99;
        fs::write(
            b.directory.join("book.json"),
            serde_json::to_vec(&record).unwrap(),
        )
        .unwrap();
        assert!(store.get(ID).is_err());
        assert!(
            store
                .capture(&item, ID, None, None, State::Available)
                .is_err()
        );
        drop(store);
        // Reopening (startup cleanup and expiry) must keep the unreadable record.
        let store = BundleStore::open(dir.path(), Retention::Immediate).unwrap();
        assert!(b.directory.exists());
        assert!(store.get(ID).is_err());
        assert!(store.present_ids().unwrap().contains(ID));
    }
    #[test]
    fn retention_accepts_only_documented_values() {
        for (text, expected) in [
            ("0", Retention::Immediate),
            ("30", Retention::Days(30)),
            ("forever", Retention::Forever),
        ] {
            assert_eq!(text.parse::<Retention>().unwrap(), expected);
        }
        for bad in ["-1", "1.5", "never", "4294967296", ""] {
            assert!(bad.parse::<Retention>().is_err());
        }
        #[derive(Deserialize)]
        struct Config {
            value: Retention,
        }
        assert_eq!(
            toml::from_str::<Config>("value = 'forever'").unwrap().value,
            Retention::Forever
        );
        assert_eq!(
            toml::from_str::<Config>("value = 0").unwrap().value,
            Retention::Immediate
        );
        assert!(toml::from_str::<Config>("value = -1").is_err());
        assert_eq!(serde_json::to_value(Retention::Immediate).unwrap(), 0);
        assert_eq!(serde_json::to_value(Retention::Days(30)).unwrap(), 30);
        assert_eq!(serde_json::to_value(Retention::Forever).unwrap(), "forever");
    }
    #[test]
    fn expiration_boundaries_preserve_visible_archives_and_forever_records() {
        for retention in [Retention::Days(2), Retention::Immediate, Retention::Forever] {
            let dir = tempfile::tempdir().unwrap();
            let (item, capture) = fixture(dir.path());
            let store = BundleStore::open(dir.path(), retention).unwrap();
            store
                .capture(&item, ID, Some(&capture), None, State::Archived)
                .unwrap();
            store.expire_at(i64::MAX).unwrap();
            assert_eq!(
                store.get(ID).unwrap().unwrap().record.state,
                State::Archived
            );
            let retired = store.transition(ID, State::Retired).unwrap();
            let (removed_at, _) = retired_identity(&retired.directory).unwrap();
            match retention {
                Retention::Days(days) => {
                    let deadline = removed_at + i64::from(days) * 86400;
                    store.expire_at(deadline - 1).unwrap();
                    assert!(retired.directory.exists());
                    store.expire_at(deadline).unwrap();
                    assert!(!retired.directory.exists());
                }
                Retention::Immediate => {
                    store.expire_at(removed_at).unwrap();
                    assert!(!retired.directory.exists());
                }
                Retention::Forever => {
                    store.expire_at(i64::MAX).unwrap();
                    assert!(retired.directory.exists());
                }
            }
        }
    }

    #[test]
    fn failed_retired_entry_is_retried_without_blocking_other_entries() {
        let dir = tempfile::tempdir().unwrap();
        let store = BundleStore::open(dir.path(), Retention::Immediate).unwrap();
        let blocked = store
            .root
            .join("retired")
            .join(format!("100-{ID}-{}", uuid::Uuid::new_v4()));
        let removable = store
            .root
            .join("retired")
            .join(format!("101-{ID}-{}", uuid::Uuid::new_v4()));
        // A regular file makes remove_dir_all fail without relying on permissions.
        fs::write(&blocked, b"blocked").unwrap();
        fs::create_dir(&removable).unwrap();
        store.expire_at(200).unwrap();
        assert!(blocked.exists());
        assert!(!removable.exists());
        fs::remove_file(&blocked).unwrap();
        fs::create_dir(&blocked).unwrap();
        store.expire_at(200).unwrap();
        assert!(!blocked.exists());
    }

    #[test]
    fn timed_expiry_uses_removal_time_and_covers_share_one_copy() {
        let dir = tempfile::tempdir().unwrap();
        let (item, capture) = fixture(dir.path());
        let store = BundleStore::open(dir.path(), Retention::Days(30)).unwrap();
        let cover = dir.path().join("legacy.webp");
        fs::write(&cover, b"first cover").unwrap();
        store
            .capture(&item, ID, Some(&capture), Some(&cover), State::Available)
            .unwrap();
        // The media cache replaces covers (a new file), as `encode_cover_to_disk` does.
        fs::remove_file(&cover).unwrap();
        fs::write(&cover, b"replacement cover").unwrap();
        let new = store
            .capture(&item, ID, Some(&capture), Some(&cover), State::Available)
            .unwrap();
        assert_eq!(
            fs::read(new.cover().unwrap()).unwrap(),
            b"replacement cover"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let inode = |p: &Path| fs::metadata(p).unwrap().ino();
            // One copy on disk: the saved cover is the media cache file.
            assert_eq!(inode(&new.cover().unwrap()), inode(&cover));
            // Saves without a new cover carry the same file forward.
            let same = store
                .capture(&item, ID, Some(&capture), None, State::Available)
                .unwrap();
            assert_eq!(inode(&same.cover().unwrap()), inode(&cover));
        }
        store.transition(ID, State::Archived).unwrap();
        let retired = store.transition(ID, State::Retired).unwrap();
        store.expire().unwrap();
        assert!(retired.directory.exists());
        let expired_path = store.root.join("retired").join(format!(
            "{}-{ID}-{}",
            chrono::Utc::now().timestamp() - 31 * 86400,
            uuid::Uuid::new_v4()
        ));
        fs::rename(&retired.directory, &expired_path).unwrap();
        // Partial deletion must not prevent subsequent expiry.
        fs::remove_file(expired_path.join("book.json")).unwrap();
        store.expire().unwrap();
        assert!(!expired_path.exists());
        assert!(cover.exists());
        assert!(capture.path.exists());
    }
    #[test]
    fn corrupted_captured_bytes_are_diagnosed_even_if_still_valid_lua() {
        let dir = tempfile::tempdir().unwrap();
        let (item, capture) = fixture(dir.path());
        let store = BundleStore::open(dir.path(), Retention::Forever).unwrap();
        let saved = store
            .capture(&item, ID, Some(&capture), None, State::Available)
            .unwrap();
        fs::write(
            saved.directory.join("metadata.epub.lua"),
            "return {annotations={}}",
        )
        .unwrap();
        assert!(store.get(ID).is_err());
        assert!(saved.directory.exists());
    }
    #[test]
    fn failed_capture_drops_temporary_directories_and_preserves_committed_data() {
        let dir = tempfile::tempdir().unwrap();
        let (item, capture) = fixture(dir.path());
        let store = BundleStore::open(dir.path(), Retention::Forever).unwrap();
        let missing_cover = dir.path().join("missing.webp");
        assert!(
            store
                .capture(
                    &item,
                    ID,
                    Some(&capture),
                    Some(&missing_cover),
                    State::Available
                )
                .is_err()
        );
        assert_eq!(
            fs::read_dir(store.root.join("snapshots")).unwrap().count(),
            0
        );
        let saved = store
            .capture(&item, ID, Some(&capture), None, State::Available)
            .unwrap();
        assert!(
            store
                .capture(
                    &item,
                    ID,
                    Some(&capture),
                    Some(&missing_cover),
                    State::Available
                )
                .is_err()
        );
        assert_eq!(fs::read_dir(&saved.directory).unwrap().count(), 2);
        assert_eq!(
            store.get(ID).unwrap().unwrap().revision().unwrap(),
            saved.revision().unwrap()
        );
    }

    #[test]
    fn saves_swap_whole_directories_and_interrupted_swaps_recover() {
        // `parked`: the previous version was moved aside; `published`: the new
        // version was also moved in. A crash may stop after either step.
        for published in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let (item, capture) = fixture(dir.path());
            let store = BundleStore::open(dir.path(), Retention::Forever).unwrap();
            let old = store
                .capture(&item, ID, Some(&capture), None, State::Available)
                .unwrap();
            let snapshots = store.root.join("snapshots");
            let names = || {
                let mut names: Vec<_> = fs::read_dir(&snapshots)
                    .unwrap()
                    .map(|e| e.unwrap().file_name().into_string().unwrap())
                    .collect();
                names.sort();
                names
            };
            // A completed save leaves exactly one directory per book.
            assert_eq!(names(), [ID]);
            let staged = snapshots.join(format!("{TEMP_PREFIX}book"));
            fs::create_dir(&staged).unwrap();
            let bytes = format!("return {{partial_md5_checksum='{ID}', annotations={{}}}}");
            fs::write(staged.join("metadata.epub.lua"), &bytes).unwrap();
            let mut record = old.record.clone();
            record.source.as_mut().unwrap().bytes_md5 =
                format!("{:x}", md5::compute(bytes.as_bytes()));
            fs::write(staged.join(MANIFEST), serde_json::to_vec(&record).unwrap()).unwrap();
            let parked = snapshots.join(format!("{PARKED_PREFIX}{ID}"));
            fs::rename(&old.directory, &parked).unwrap();
            if published {
                fs::rename(&staged, &old.directory).unwrap();
            }
            drop(store);
            let store = BundleStore::open(dir.path(), Retention::Forever).unwrap();
            let current = store.get(ID).unwrap().unwrap();
            assert_eq!(
                current.item().unwrap().annotations().len(),
                usize::from(!published)
            );
            // Readers own their bytes, whichever version survived.
            assert_eq!(old.item().unwrap().annotations().len(), 1);
            assert_eq!(names(), [ID]);
        }
    }

    #[test]
    fn directory_moves_define_state_and_old_readers_survive_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let (item, capture) = fixture(dir.path());
        let store = BundleStore::open(dir.path(), Retention::Immediate).unwrap();
        let old = store
            .capture(&item, ID, Some(&capture), None, State::Available)
            .unwrap();
        let before = fs::read(old.directory.join("book.json")).unwrap();
        let archived_path = store.root.join("archived").join(ID);
        fs::rename(&old.directory, &archived_path).unwrap();
        drop(store);
        let store = BundleStore::open(dir.path(), Retention::Immediate).unwrap();
        let archived = store.get(ID).unwrap().unwrap();
        assert_eq!(archived.record.state, State::Archived);
        assert_eq!(
            fs::read(archived.directory.join("book.json")).unwrap(),
            before
        );
        assert_ne!(old.revision().unwrap(), archived.revision().unwrap());
        assert!(
            serde_json::from_slice::<serde_json::Value>(&before)
                .unwrap()
                .get("state")
                .is_none()
        );
        let returned = store.transition(ID, State::Available).unwrap();
        assert_eq!(returned.record.state, State::Available);
        store.transition(ID, State::Retired).unwrap();
        store.expire().unwrap();
        assert!(store.present_ids().unwrap().is_empty());
        assert_eq!(old.item().unwrap().annotations().len(), 1);
        assert_eq!(archived.item().unwrap().annotations().len(), 1);
    }

    #[test]
    fn expiry_continues_after_failures_and_does_not_block_startup() {
        let dir = tempfile::tempdir().unwrap();
        let store = BundleStore::open(dir.path(), Retention::Immediate).unwrap();
        let name = || format!("1-{ID}-{}", uuid::Uuid::new_v4());
        // A file where a directory is expected deterministically fails removal.
        let failed = store.root.join("retired").join(name());
        fs::write(&failed, b"not a directory").unwrap();
        let partial = store.root.join("retired").join(name());
        fs::create_dir(&partial).unwrap();
        fs::write(partial.join("leftover"), b"partial deletion").unwrap();
        let malformed = store.root.join("retired/unknown");
        fs::create_dir(&malformed).unwrap();
        drop(store);
        let store = BundleStore::open(dir.path(), Retention::Immediate).unwrap();
        assert!(failed.exists());
        assert!(!partial.exists());
        assert!(malformed.exists());
        fs::remove_file(&failed).unwrap();
        fs::create_dir(&failed).unwrap();
        store.expire().unwrap();
        assert!(!failed.exists());
    }

    #[test]
    fn presence_does_not_confuse_corruption_or_enumeration_failure_with_removal() {
        let dir = tempfile::tempdir().unwrap();
        let (item, capture) = fixture(dir.path());
        let store = BundleStore::open(dir.path(), Retention::Forever).unwrap();
        let saved = store
            .capture(&item, ID, Some(&capture), None, State::Available)
            .unwrap();
        fs::write(saved.directory.join("book.json"), b"broken").unwrap();
        assert!(store.present_ids().unwrap().contains(ID));
        assert!(store.get(ID).is_err());
        fs::rename(store.root.join("snapshots"), store.root.join("unavailable")).unwrap();
        assert!(store.present_ids().is_err());
    }

    #[test]
    fn foreign_entries_do_not_break_enumeration() {
        let dir = tempfile::tempdir().unwrap();
        let (item, capture) = fixture(dir.path());
        let store = BundleStore::open(dir.path(), Retention::Forever).unwrap();
        store
            .capture(&item, ID, Some(&capture), None, State::Available)
            .unwrap();
        fs::write(store.root.join("snapshots/.DS_Store"), b"finder").unwrap();
        fs::create_dir(store.root.join("archived/@eaDir")).unwrap();
        fs::write(store.root.join("retired/.DS_Store"), b"finder").unwrap();
        assert_eq!(store.present_ids().unwrap().len(), 1);
        assert_eq!(store.records().unwrap().len(), 1);
        drop(store);
        let store = BundleStore::open(dir.path(), Retention::Immediate).unwrap();
        assert!(store.root.join("snapshots/.DS_Store").exists());
        assert!(store.root.join("retired/.DS_Store").exists());
        assert!(store.present_ids().unwrap().contains(ID));
    }

    #[test]
    fn lock_child() {
        let Some(path) = std::env::var_os("KOSHELF_LOCK_TEST_DATA") else {
            return;
        };
        let _store = BundleStore::open(Path::new(&path), Retention::Forever).unwrap();
        fs::write(Path::new(&path).join("ready"), b"locked").unwrap();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    #[test]
    fn lock_is_released_when_process_is_killed_even_with_file_left_behind() {
        let dir = tempfile::tempdir().unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "store::bundles::tests::lock_child"])
            .env("KOSHELF_LOCK_TEST_DATA", dir.path())
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !dir.path().join("ready").exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let ready = dir.path().join("ready").exists();
        let rejected = ready && BundleStore::open(dir.path(), Retention::Forever).is_err();
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(ready);
        assert!(rejected);
        assert!(dir.path().join("library/writer.lock").exists());
        assert!(BundleStore::open(dir.path(), Retention::Forever).is_ok());
    }
}
