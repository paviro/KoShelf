//! Library asset management helpers (cover cleanup).

use super::SnapshotBuilder;
use crate::models::LibraryItem;
use anyhow::Result;
use log::{info, warn};
use std::collections::HashSet;
use std::fs;

impl SnapshotBuilder {
    /// Clean up cover files for books that no longer exist in the library
    pub(crate) fn cleanup_stale_covers(&self, books: &[LibraryItem]) -> Result<()> {
        let covers_dir = self.covers_dir();

        // If covers directory doesn't exist, nothing to clean up
        if !covers_dir.exists() {
            return Ok(());
        }

        // Build set of current book IDs
        let current_ids: HashSet<String> = books.iter().map(|b| b.id.clone()).collect();

        // Iterate over existing cover files
        let entries = fs::read_dir(&covers_dir)?;
        for entry in entries.flatten() {
            let path = entry.path();

            // Skip if not a file
            if !path.is_file() {
                continue;
            }

            // Get filename without extension (book ID)
            if let Some(file_stem) = path.file_stem().and_then(|n| n.to_str()) {
                // If this book ID is not in current books, remove the cover
                if !current_ids.contains(file_stem) {
                    info!("Removing stale cover: {:?}", path);
                    if let Err(e) = fs::remove_file(&path) {
                        warn!("Failed to remove stale cover {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }
}
