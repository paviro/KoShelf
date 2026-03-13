//! Canonical-ID collision detection for library items.
//!
//! When multiple source files resolve to the same canonical MD5 identity,
//! only one deterministic winner is exposed.  Losers are recorded as
//! diagnostics so operators can investigate file duplication.

use crate::infra::sqlite::library_repo::rows::CollisionDiagnosticRow;
use crate::models::LibraryItem;
use std::collections::HashSet;

/// Result of collision detection across a set of library items.
pub struct CollisionResult {
    /// Items that won their canonical-ID group (unique IDs guaranteed).
    pub winners: Vec<LibraryItem>,
    /// Diagnostic entries for every non-winning path in a collision group.
    pub diagnostics: Vec<CollisionDiagnosticRow>,
}

/// Deduplicate items using first-wins semantics (single pass).
///
/// Iterates items once; the first item seen for each `id` wins.
/// Subsequent items with the same ID are recorded as collision diagnostics.
/// Order is deterministic for a given filesystem state (walkdir order).
pub fn deduplicate_first_wins(items: Vec<LibraryItem>, detected_at: &str) -> CollisionResult {
    let mut seen: HashSet<String> = HashSet::with_capacity(items.len());
    let mut winners = Vec::with_capacity(items.len());
    let mut diagnostics = Vec::new();

    // Track the winning path for each ID so diagnostics can reference it.
    let mut winner_paths: std::collections::HashMap<String, String> =
        std::collections::HashMap::with_capacity(items.len());

    for item in items {
        if seen.insert(item.id.clone()) {
            winner_paths.insert(
                item.id.clone(),
                item.file_path.to_string_lossy().to_string(),
            );
            winners.push(item);
        } else {
            let loser_path = item.file_path.to_string_lossy().to_string();
            let winner_path = winner_paths.get(&item.id).cloned().unwrap_or_default();
            diagnostics.push(CollisionDiagnosticRow {
                canonical_id: item.id.clone(),
                file_path: loser_path.clone(),
                winner_item_id: item.id.clone(),
                reason: format!(
                    "first_wins: {} encountered before {}",
                    winner_path, loser_path
                ),
                detected_at: detected_at.to_string(),
            });
        }
    }

    CollisionResult {
        winners,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BookInfo, LibraryItem, LibraryItemFormat};
    use std::path::PathBuf;

    fn test_item(id: &str, path: &str) -> LibraryItem {
        LibraryItem {
            id: id.to_string(),
            book_info: BookInfo {
                title: format!("Book at {path}"),
                authors: vec![],
                description: None,
                language: None,
                publisher: None,
                identifiers: vec![],
                subjects: vec![],
                series: None,
                series_number: None,
                pages: None,
                cover_data: None,
                cover_mime_type: None,
            },
            koreader_metadata: None,
            file_path: PathBuf::from(path),
            format: LibraryItemFormat::Epub,
        }
    }

    #[test]
    fn no_collisions_returns_all_items() {
        let items = vec![
            test_item("aaa", "/books/a.epub"),
            test_item("bbb", "/books/b.epub"),
        ];

        let result = deduplicate_first_wins(items, "2026-01-01T00:00:00Z");

        assert_eq!(result.winners.len(), 2);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn collision_keeps_first_encountered_item() {
        let items = vec![
            test_item("same-md5", "/books/z.epub"),
            test_item("same-md5", "/books/a.epub"),
        ];

        let result = deduplicate_first_wins(items, "2026-01-01T00:00:00Z");

        assert_eq!(result.winners.len(), 1);
        // First-wins: /books/z.epub was encountered first
        assert_eq!(result.winners[0].file_path, PathBuf::from("/books/z.epub"));

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].file_path, "/books/a.epub");
        assert_eq!(result.diagnostics[0].canonical_id, "same-md5");
        assert!(result.diagnostics[0].reason.contains("first_wins"));
    }

    #[test]
    fn multiple_collisions_produce_correct_diagnostics() {
        let items = vec![
            test_item("md5-x", "/books/c.epub"),
            test_item("md5-x", "/books/a.epub"),
            test_item("md5-x", "/books/b.epub"),
        ];

        let result = deduplicate_first_wins(items, "2026-01-01T00:00:00Z");

        assert_eq!(result.winners.len(), 1);
        // First-wins: /books/c.epub was encountered first
        assert_eq!(result.winners[0].file_path, PathBuf::from("/books/c.epub"));
        assert_eq!(result.diagnostics.len(), 2);
    }
}
