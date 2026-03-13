//! Shared in-memory site metadata store.

use crate::contracts::site::SiteData;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct SiteStore {
    inner: Arc<RwLock<Option<Arc<SiteData>>>>,
}

impl SiteStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace(&self, site: SiteData) {
        let mut guard = self
            .inner
            .write()
            .expect("site store lock poisoned while writing");
        *guard = Some(Arc::new(site));
    }

    pub fn get(&self) -> Option<Arc<SiteData>> {
        let guard = self
            .inner
            .read()
            .expect("site store lock poisoned while reading");
        guard.clone()
    }
}

pub type SharedSiteStore = Arc<SiteStore>;
