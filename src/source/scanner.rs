//! Library filesystem scanning: path collection and metadata location.
//!
//! Stripped to pure path discovery — all parsing, metadata resolution, and
//! cover generation now live in `runtime::ingest::library`.

use crate::shelf::models::LibraryItemFormat;
use crate::source::kobo::{KoboDbParser, KoboFileHints};
use log::{info, warn};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

/// Configuration for where to find KOReader metadata
#[derive(Clone, Debug, Default)]
pub enum MetadataLocation {
    /// Default: metadata stored in .sdr folder next to each book
    #[default]
    InBookFolder,
    /// Metadata stored in docsettings folder with full path structure
    DocSettings(PathBuf),
    /// Metadata stored in hashdocsettings folder organized by partial MD5 hash
    HashDocSettings(PathBuf),
}

#[derive(Clone, Debug)]
pub struct CollectedItem {
    pub path: PathBuf,
    pub format: LibraryItemFormat,
    pub kobo_hints: Option<KoboFileHints>,
}

#[derive(Clone, Debug, Default)]
pub struct CollectionOptions {
    pub kobo_db_path: Option<PathBuf>,
}

/// Walk library directories and collect paths with supported formats.
///
/// Returns all file paths matching supported book/comic extensions
/// (epub, fb2, mobi, cbz, cbr), plus extensionless Kobo kepub
/// entries matched by KoboReader.sqlite when configured. No parsing, no
/// metadata, no covers.
pub async fn collect_paths(
    library_paths: &[PathBuf],
    options: &CollectionOptions,
) -> Vec<CollectedItem> {
    let kobo_index = match &options.kobo_db_path {
        Some(path) => match KoboMatchIndex::from_db(path).await {
            Ok(index) => Some(index),
            Err(e) => {
                warn!("Failed to read Kobo database {:?}: {}", path, e);
                None
            }
        },
        None => None,
    };

    let mut items = Vec::new();

    for library_path in library_paths {
        for entry in walkdir::WalkDir::new(library_path) {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to read directory entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();
            if let Some(format) = LibraryItemFormat::from_path(path) {
                items.push(CollectedItem {
                    path: entry.into_path(),
                    format,
                    kobo_hints: None,
                });
                continue;
            }

            if let Some(index) = &kobo_index
                && is_extensionless_file(path)
                && let Some(hints) = index.match_path(path)
            {
                items.push(CollectedItem {
                    path: entry.into_path(),
                    format: LibraryItemFormat::Epub,
                    kobo_hints: Some(hints),
                });
            }
        }
    }

    info!(
        "Collected {} items from {} library directories",
        items.len(),
        library_paths.len()
    );
    items
}

fn is_extensionless_file(path: &Path) -> bool {
    path.is_file() && path.extension().is_none()
}

#[derive(Debug)]
struct KoboMatchIndex {
    by_filename: HashMap<String, KoboFileHints>,
}

impl KoboMatchIndex {
    async fn from_db(path: &Path) -> anyhow::Result<Self> {
        let entries = KoboDbParser::parse(path).await?;
        let mut by_filename = HashMap::new();

        for entry in entries {
            for candidate in filename_candidates(&entry.hints.content_id) {
                by_filename
                    .entry(candidate)
                    .or_insert_with(|| entry.hints.clone());
            }
        }

        Ok(Self { by_filename })
    }

    fn match_path(&self, path: &Path) -> Option<KoboFileHints> {
        let filename = path.file_name()?.to_str()?;
        self.by_filename.get(filename).cloned()
    }
}

fn filename_candidates(content_id: &str) -> Vec<String> {
    let trimmed = content_id.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let without_fragment = trimmed
        .split_once('#')
        .map(|(base, _)| base)
        .unwrap_or(trimmed);
    let without_query = without_fragment
        .split_once('?')
        .map(|(base, _)| base)
        .unwrap_or(without_fragment);

    let mut candidates = Vec::new();
    push_unique(&mut candidates, without_query.to_string());

    if let Some(last_segment) = without_query.rsplit('/').next()
        && !last_segment.is_empty()
    {
        push_unique(&mut candidates, percent_decode(last_segment));
    }

    if let Some(file_path) = without_query.strip_prefix("file://") {
        let decoded = percent_decode(file_path);
        if let Some(last_segment) = decoded.rsplit('/').next()
            && !last_segment.is_empty()
        {
            push_unique(&mut candidates, last_segment.to_string());
        }
    }

    candidates
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !value.is_empty() && !values.contains(&value) {
        values.push(value);
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(high), Some(low)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2]))
        {
            output.push(high * 16 + low);
            i += 3;
            continue;
        }

        output.push(bytes[i]);
        i += 1;
    }

    String::from_utf8_lossy(&output).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{CollectionOptions, collect_paths};
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::fs;
    use std::str::FromStr;

    async fn create_kobo_db(entries: &[(&str, i64)]) -> tempfile::NamedTempFile {
        let file = tempfile::NamedTempFile::new().expect("temp file");
        let url = format!("sqlite:{}?mode=rwc", file.path().display());
        let options = SqliteConnectOptions::from_str(&url).expect("sqlite options");
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("sqlite pool");

        sqlx::query(
            "CREATE TABLE content (
                ContentID TEXT PRIMARY KEY,
                BookID TEXT,
                Title TEXT,
                Attribution TEXT,
                IsEncrypted INTEGER
            )",
        )
        .execute(&pool)
        .await
        .expect("content table");

        for (content_id, is_encrypted) in entries {
            sqlx::query(
                "INSERT INTO content (ContentID, BookID, Title, Attribution, IsEncrypted)
                 VALUES (?1, NULL, 'Title', 'Author', ?2)",
            )
            .bind(content_id)
            .bind(is_encrypted)
            .execute(&pool)
            .await
            .expect("content row");
        }

        pool.close().await;
        file
    }

    #[tokio::test]
    async fn collects_extensionless_kobo_match_as_epub() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join(".kobo").join("kepub").join("matched-book");
        fs::create_dir_all(book_path.parent().expect("parent")).expect("kepub dir");
        fs::write(&book_path, b"not parsed here").expect("book file");

        let db = create_kobo_db(&[("matched-book", 1)]).await;
        let items = collect_paths(
            &[dir.path().to_path_buf()],
            &CollectionOptions {
                kobo_db_path: Some(db.path().to_path_buf()),
            },
        )
        .await;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, book_path);
        assert_eq!(
            items[0].format,
            crate::shelf::models::LibraryItemFormat::Epub
        );
        assert!(items[0].kobo_hints.as_ref().unwrap().is_encrypted);
    }

    #[tokio::test]
    async fn ignores_unrelated_extensionless_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(dir.path().join("notes"), b"not a book").expect("notes file");

        let db = create_kobo_db(&[("matched-book", 0)]).await;
        let items = collect_paths(
            &[dir.path().to_path_buf()],
            &CollectionOptions {
                kobo_db_path: Some(db.path().to_path_buf()),
            },
        )
        .await;

        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn collects_extensionless_kobo_match_outside_device_kepub_dir() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join("synced-library").join("matched-book");
        fs::create_dir_all(book_path.parent().expect("parent")).expect("library dir");
        fs::write(&book_path, b"not parsed here").expect("book file");

        let db = create_kobo_db(&[("matched-book", 0)]).await;
        let items = collect_paths(
            &[dir.path().to_path_buf()],
            &CollectionOptions {
                kobo_db_path: Some(db.path().to_path_buf()),
            },
        )
        .await;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, book_path);
    }

    #[tokio::test]
    async fn collects_regular_epub_without_kobo_db() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join("book.epub");
        fs::write(&book_path, b"not parsed here").expect("book file");

        let items = collect_paths(&[dir.path().to_path_buf()], &CollectionOptions::default()).await;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, book_path);
        assert!(items[0].kobo_hints.is_none());
    }

    #[tokio::test]
    async fn collects_fb2_zip_as_book() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join("book.fb2.zip");
        fs::write(&book_path, b"not parsed here").expect("book file");

        let items = collect_paths(&[dir.path().to_path_buf()], &CollectionOptions::default()).await;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, book_path);
        assert_eq!(
            items[0].format,
            crate::shelf::models::LibraryItemFormat::Fb2
        );
    }

    #[tokio::test]
    async fn matches_file_url_by_final_filename() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join(".kobo").join("kepub").join("file row");
        fs::create_dir_all(book_path.parent().expect("parent")).expect("kepub dir");
        fs::write(&book_path, b"not parsed here").expect("book file");

        let db = create_kobo_db(&[("file:///mnt/onboard/.kobo/kepub/file%20row", 0)]).await;
        let items = collect_paths(
            &[dir.path().to_path_buf()],
            &CollectionOptions {
                kobo_db_path: Some(db.path().to_path_buf()),
            },
        )
        .await;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, book_path);
    }
}
