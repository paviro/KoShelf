//! Shared in-memory store for processed reading statistics data.
//!
//! Reading endpoints compute responses on demand from this data,
//! applying scope, date-range, and timezone filters at request time.
//!
//! The [`ReadingData`] struct itself lives in `models` so that both
//! `domain/library` and `domain/reading` can depend on it without
//! reaching into `runtime`.

use crate::models::ReadingData;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct ReadingDataStore {
    inner: Arc<RwLock<Option<Arc<ReadingData>>>>,
}

impl ReadingDataStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace(&self, data: ReadingData) {
        let mut guard = self
            .inner
            .write()
            .expect("reading data store lock poisoned while writing");
        *guard = Some(Arc::new(data));
    }

    pub fn get(&self) -> Option<Arc<ReadingData>> {
        let guard = self
            .inner
            .read()
            .expect("reading data store lock poisoned while reading");
        guard.clone()
    }
}

pub type SharedReadingDataStore = Arc<ReadingDataStore>;
