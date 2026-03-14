//! Library-domain boundaries for list/detail queries and item persistence.

pub mod build;
pub mod item_mapping;
pub mod queries;
pub mod service;

pub use build::upsert_single_item;
pub use queries::{IncludeSet, LibraryDetailQuery, LibraryListQuery};
pub use service::{detail, list};
