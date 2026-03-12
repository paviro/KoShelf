use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryBuildMode {
    #[default]
    Full,
    Incremental,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LibraryBuildRequest {
    pub mode: LibraryBuildMode,
    pub changed_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LibraryBuildResult {
    pub scanned_files: u64,
    pub upserted_items: u64,
    pub removed_items: u64,
    pub collision_count: u64,
}

pub trait LibraryBuildService: Send + Sync {
    fn build(&self, request: &LibraryBuildRequest) -> Result<LibraryBuildResult>;
}
