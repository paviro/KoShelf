use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibrarySyncMode {
    #[default]
    Full,
    Incremental,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LibrarySyncRequest {
    pub mode: LibrarySyncMode,
    pub changed_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LibrarySyncResult {
    pub scanned_files: u64,
    pub upserted_items: u64,
    pub removed_items: u64,
    pub collision_count: u64,
}

pub trait LibrarySyncService: Send + Sync {
    fn sync(&self, request: &LibrarySyncRequest) -> Result<LibrarySyncResult>;
}
