//! Ingest pipeline: library sync, item processing, cleanup, and statistics loading.

mod batch;
mod cleanup;
mod inventory;
mod library;
mod metadata;
pub(crate) mod preservation;
mod processor;
mod reconcile;
pub(crate) mod sources;
mod statistics;

#[cfg(test)]
pub(crate) use batch::ingest_items;
pub(crate) use library::{reconcile_paths, sync_library};
pub(crate) use statistics::load_reading_data;

pub(crate) use processor::refresh_capture;
