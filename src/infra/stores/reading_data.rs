//! Shared in-memory store for processed reading statistics data.
//!
//! Reading endpoints compute responses on demand from this data,
//! applying scope, date-range, and timezone filters at request time.

use crate::domain::reading::PageScaling;
use crate::koreader::types::StatisticsData;
use crate::time_config::TimeConfig;
use std::sync::{Arc, RwLock};

/// Bundle of statistics data and time configuration used by reading
/// and library domain services for on-demand query computation.
#[derive(Debug, Clone)]
pub struct ReadingData {
    pub stats_data: StatisticsData,
    pub time_config: TimeConfig,
    pub heatmap_scale_max: Option<u32>,
    /// Page scaling factors for synthetic page counts.
    pub page_scaling: PageScaling,
}

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
