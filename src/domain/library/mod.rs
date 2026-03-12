//! Library-domain boundaries for list/detail queries and build/update orchestration.

pub mod build;
pub mod collision;
pub mod item_mapping;
pub mod projections;
pub mod queries;
pub mod service;

pub use build::{LibraryBuildMode, LibraryBuildPipeline, LibraryBuildResult};
pub use queries::{IncludeSet, LibraryDetailQuery, LibraryListQuery};
pub use service::LibraryService;
