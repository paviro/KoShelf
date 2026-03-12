//! Canonical-ID collision detection for library items.
//!
//! When multiple source files resolve to the same canonical MD5 identity,
//! only one deterministic winner is exposed.  Losers are recorded as
//! diagnostics so operators can investigate file duplication.

use crate::infra::sqlite::library_repo::rows::CollisionDiagnosticRow;
use crate::models::LibraryItem;
use std::collections::HashMap;

/// Result of collision detection across a set of library items.
pub struct CollisionResult {
    /// Items that won their canonical-ID group (unique IDs guaranteed).
    pub winners: Vec<LibraryItem>,
    /// Diagnostic entries for every non-winning path in a collision group.
    pub diagnostics: Vec<CollisionDiagnosticRow>,
}

/// Detect canonical-ID collisions and select deterministic winners.
///
/// Winner selection: within each group of items sharing the same `id`,
/// the item with the lexicographically smallest `file_path` wins.
pub fn detect_collisions(items: Vec<LibraryItem>, detected_at: &str) -> CollisionResult {
    let mut groups: HashMap<String, Vec<LibraryItem>> = HashMap::new();
    for item in items {
        groups.entry(item.id.clone()).or_default().push(item);
    }

    let mut winners = Vec::with_capacity(groups.len());
    let mut diagnostics = Vec::new();

    for (_id, mut group) in groups {
        // Sort by file_path for deterministic winner selection.
        group.sort_by(|a, b| a.file_path.cmp(&b.file_path));

        let winner = group.remove(0);
        let winner_path = winner.file_path.to_string_lossy().to_string();

        // Remaining items are collision losers.
        for loser in &group {
            let loser_path = loser.file_path.to_string_lossy().to_string();
            diagnostics.push(CollisionDiagnosticRow {
                canonical_id: loser.id.clone(),
                file_path: loser_path.clone(),
                winner_item_id: winner.id.clone(),
                reason: format!("path_precedence: {} < {}", winner_path, loser_path),
                detected_at: detected_at.to_string(),
            });
        }

        winners.push(winner);
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

        let result = detect_collisions(items, "2026-01-01T00:00:00Z");

        assert_eq!(result.winners.len(), 2);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn collision_selects_earliest_path_as_winner() {
        let items = vec![
            test_item("same-md5", "/books/z.epub"),
            test_item("same-md5", "/books/a.epub"),
        ];

        let result = detect_collisions(items, "2026-01-01T00:00:00Z");

        assert_eq!(result.winners.len(), 1);
        assert_eq!(result.winners[0].file_path, PathBuf::from("/books/a.epub"));

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].file_path, "/books/z.epub");
        assert_eq!(result.diagnostics[0].canonical_id, "same-md5");
        assert!(result.diagnostics[0].reason.contains("path_precedence"));
    }

    #[test]
    fn multiple_collisions_produce_correct_diagnostics() {
        let items = vec![
            test_item("md5-x", "/books/c.epub"),
            test_item("md5-x", "/books/a.epub"),
            test_item("md5-x", "/books/b.epub"),
        ];

        let result = detect_collisions(items, "2026-01-01T00:00:00Z");

        assert_eq!(result.winners.len(), 1);
        assert_eq!(result.winners[0].file_path, PathBuf::from("/books/a.epub"));
        assert_eq!(result.diagnostics.len(), 2);
    }
}
