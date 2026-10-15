//! Disposable inventory of observed documents and sidecars, including unselected duplicates.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::source::scanner::CollectedItem;

#[derive(Clone)]
pub(crate) struct Source {
    pub item: CollectedItem,
    pub checksum: String,
    pub identity: Option<String>,
    pub folders: Vec<PathBuf>,
}

#[derive(Default, Clone)]
pub(crate) struct SourceIndex {
    paths: BTreeMap<PathBuf, Source>,
}

impl SourceIndex {
    pub fn all(&self) -> impl Iterator<Item = &Source> {
        self.paths.values()
    }

    pub fn affects(source: &Source, event: &Path) -> bool {
        source.item.path.starts_with(event)
            || source
                .folders
                .iter()
                .any(|folder| event.starts_with(folder) || folder.starts_with(event))
    }

    pub fn get(&self, path: &Path) -> Option<&Source> {
        self.paths.get(path)
    }

    pub fn remove(&mut self, path: &Path) -> Option<Source> {
        self.paths.remove(path)
    }

    pub fn insert(&mut self, source: Source) {
        self.paths.insert(source.item.path.clone(), source);
    }
}
