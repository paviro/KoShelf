//! Disposable inventory of observed ebooks, including unselected duplicates.
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use crate::source::scanner::CollectedItem;

#[derive(Clone)]
pub(crate) struct Source {
    pub item: CollectedItem,
    pub checksum: String,
    pub identity: Option<String>,
}

#[derive(Default)]
pub(crate) struct SourceIndex {
    paths: BTreeMap<PathBuf, Source>,
    checksums: HashMap<String, BTreeSet<PathBuf>>,
    identities: HashMap<String, BTreeSet<PathBuf>>,
}

impl SourceIndex {
    pub fn get(&self, path: &Path) -> Option<&Source> {
        self.paths.get(path)
    }

    pub fn remove(&mut self, path: &Path) -> Option<Source> {
        let source = self.paths.remove(path)?;
        if let Some(paths) = self.checksums.get_mut(&source.checksum) {
            paths.remove(path);
            if paths.is_empty() {
                self.checksums.remove(&source.checksum);
            }
        }
        if let Some(id) = &source.identity
            && let Some(paths) = self.identities.get_mut(id)
        {
            paths.remove(path);
            if paths.is_empty() {
                self.identities.remove(id);
            }
        }
        Some(source)
    }

    pub fn insert(&mut self, source: Source) {
        let path = source.item.path.clone();
        self.remove(&path);
        self.checksums
            .entry(source.checksum.clone())
            .or_default()
            .insert(path.clone());
        if let Some(id) = &source.identity {
            self.identities
                .entry(id.clone())
                .or_default()
                .insert(path.clone());
        }
        self.paths.insert(path, source);
    }

    pub fn candidates(&self, id: &str, checksum: Option<&str>) -> Vec<Source> {
        let mut paths = self.identities.get(id).cloned().unwrap_or_default();
        if let Some(checksum) = checksum
            && let Some(matches) = self.checksums.get(checksum)
        {
            paths.extend(matches.iter().cloned());
        }
        paths
            .into_iter()
            .filter_map(|p| self.paths.get(&p).cloned())
            .collect()
    }
}
