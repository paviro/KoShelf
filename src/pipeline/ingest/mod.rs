//! Ingest pipeline: library sync, item processing, cleanup, and statistics loading.

mod batch;
mod cleanup;
mod library;
mod metadata;
mod processor;
mod reconcile;
mod statistics;

pub(crate) use batch::ingest_items;
pub(crate) use cleanup::delete_item_for_book_path;
pub(crate) use library::sync_library;
pub(crate) use statistics::load_reading_data;
