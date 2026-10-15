//! Ingest pipeline: library sync, item processing, cleanup, and statistics loading.

mod batch;
mod cleanup;
mod library;
mod metadata;
pub(crate) mod preservation;
mod processor;
mod reconcile;
pub(crate) mod sources;
mod statistics;

pub(crate) use batch::ingest_items;
pub(crate) use cleanup::delete_item_for_book_path;
pub(crate) use library::sync_library;
pub(crate) use statistics::load_reading_data;

pub(crate) use processor::refresh_capture;
