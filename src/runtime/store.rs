//! Shared in-memory site metadata store.

use crate::contracts::site::SiteResponse;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct SiteStore {
    inner: Arc<RwLock<Option<Arc<SiteResponse>>>>,
}

impl SiteStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace(&self, site: SiteResponse) {
        let mut guard = self
            .inner
            .write()
            .expect("site store lock poisoned while writing");
        *guard = Some(Arc::new(site));
    }

    pub fn get(&self) -> Option<Arc<SiteResponse>> {
        let guard = self
            .inner
            .read()
            .expect("site store lock poisoned while reading");
        guard.clone()
    }
}

pub type SharedSiteStore = Arc<SiteStore>;
