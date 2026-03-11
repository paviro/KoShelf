//! Library-domain boundaries for list/detail queries and sync orchestration.

pub mod queries;
pub mod service;
pub mod sync;

pub use queries::{LibraryDetailQuery, LibraryListQuery};
pub use service::LibraryService;
pub use sync::{LibrarySyncMode, LibrarySyncRequest, LibrarySyncResult, LibrarySyncService};
