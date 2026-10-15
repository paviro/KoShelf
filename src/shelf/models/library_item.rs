use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::shelf::models::koreader_metadata::{Annotation, KoReaderMetadata};
use crate::source::koreader::custom_metadata::Metadata as CustomMetadata;
use crate::source::koreader::merge::resolve_language;

/// Content type classification (broad category)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Book,
    Comic,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Book => write!(f, "book"),
            ContentType::Comic => write!(f, "comic"),
        }
    }
}

/// Library item formats. `Other` is a KOReader document KoShelf cannot parse
/// (PDF, DjVu, ...); its extension always comes from the item path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibraryItemFormat {
    Epub,
    Fb2,
    Mobi,
    Cbz,
    Cbr,
    Other,
}

/// Unparsed document types the scanner collects. Any other extension only
/// enters the library through a KOReader sidecar.
const OTHER_DOCUMENT_EXTENSIONS: &[&str] = &[
    "pdf", "djvu", "djv", "xps", "oxps", "cbt", "cb7", "azw", "azw3", "docx", "doc", "rtf", "odt",
    "chm", "pdb", "prc", "fb3", "htmlz", "cbr",
];

impl LibraryItemFormat {
    /// Try to detect format from a file path's extension
    /// Handles compound extensions like .fb2.zip
    pub fn from_path(path: &Path) -> Option<Self> {
        Self::from_path_with(path, false)
    }

    /// Like [`Self::from_path`], but any extension (or none) maps to `Other`.
    /// Used for documents a KOReader sidecar vouches for.
    pub fn from_document_path(path: &Path) -> Self {
        Self::from_path_with(path, true).unwrap_or(Self::Other)
    }

    fn from_path_with(path: &Path, any: bool) -> Option<Self> {
        let filename = path.file_name()?.to_str()?.to_lowercase();

        // Check for compound extensions first (e.g., .fb2.zip)
        if filename.ends_with(".fb2.zip") {
            return Some(Self::Fb2);
        }

        // Then check simple extensions
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_lowercase();
        match ext.as_str() {
            "epub" => Some(Self::Epub),
            "fb2" => Some(Self::Fb2),
            "mobi" => Some(Self::Mobi),
            "cbz" => Some(Self::Cbz),
            #[cfg(not(windows))]
            "cbr" => Some(Self::Cbr),
            ext if any || OTHER_DOCUMENT_EXTENSIONS.contains(&ext) => Some(Self::Other),
            _ => None,
        }
    }

    /// Get the KOReader metadata filename for this file path.
    ///
    /// KOReader names sidecars after the last suffix only, so `.fb2.zip`
    /// books use `*.fb2.sdr/metadata.zip.lua` and extensionless documents use
    /// `metadata._.lua`.
    pub fn metadata_filename_for_path(&self, path: &Path) -> String {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        if matches!(self, Self::Fb2) && filename.ends_with(".fb2.zip") {
            return "metadata.zip.lua".into();
        }
        if matches!(self, Self::Other) {
            let suffix = path.extension().and_then(|e| e.to_str()).unwrap_or("_");
            return format!("metadata.{suffix}.lua");
        }
        format!("metadata.{}.lua", self.extension_for_path(path))
    }

    /// Build the document filename a sidecar refers to, given the sidecar stem
    /// (the `.sdr` directory name without the suffix) and the metadata
    /// filename found inside it. Inverse of `metadata_filename_for_path`:
    /// `metadata.zip.lua` under `<name>.fb2.sdr` means `<name>.fb2.zip`,
    /// `metadata._.lua` means no extension, and any other metadata filename
    /// means `<stem>.<ext>`.
    pub fn book_filename_from_sidecar(book_stem: &str, metadata_filename: &str) -> Option<String> {
        let filename = metadata_filename
            .strip_suffix(".old")
            .unwrap_or(metadata_filename);
        match Self::metadata_suffix(filename)? {
            "_" => Some(book_stem.to_string()),
            suffix => Some(format!("{book_stem}.{suffix}")),
        }
    }

    /// The document suffix of a `metadata.<suffix>.lua` filename.
    fn metadata_suffix(filename: &str) -> Option<&str> {
        filename
            .strip_prefix("metadata.")?
            .strip_suffix(".lua")
            .filter(|s| !s.is_empty() && !s.contains(['/', '\\']))
    }

    /// Check if a filename is a KOReader metadata file (`metadata.<ext>.lua`,
    /// or its `.old` backup).
    pub fn is_metadata_file(filename: &str) -> bool {
        let filename = filename.strip_suffix(".old").unwrap_or(filename);
        Self::metadata_suffix(filename).is_some() && !is_ignored_name(filename)
    }

    /// Check if a path points to a KOReader metadata file inside a `.sdr`
    /// directory.
    pub fn is_metadata_path(path: &Path) -> bool {
        path.file_name()
            .and_then(|s| s.to_str())
            .is_some_and(Self::is_metadata_file)
            && path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .is_some_and(|dir| dir.ends_with(".sdr"))
    }

    /// File extension for an item of this format at `path`.
    pub fn extension_for_path(&self, path: &Path) -> String {
        match self {
            Self::Epub => "epub".into(),
            Self::Fb2 => "fb2".into(),
            Self::Mobi => "mobi".into(),
            Self::Cbz => "cbz".into(),
            Self::Cbr => "cbr".into(),
            Self::Other => path
                .extension()
                .and_then(|e| e.to_str())
                .map(str::to_lowercase)
                .unwrap_or_default(),
        }
    }

    /// Get the content type for an item of this format at `path`
    pub fn content_type_for_path(&self, path: &Path) -> ContentType {
        match self {
            Self::Epub | Self::Fb2 | Self::Mobi => ContentType::Book,
            Self::Cbz | Self::Cbr => ContentType::Comic,
            Self::Other => match self.extension_for_path(path).as_str() {
                "cbz" | "cbr" | "cbt" | "cb7" => ContentType::Comic,
                _ => ContentType::Book,
            },
        }
    }

    /// Whether KoShelf can parse metadata and covers from the document.
    pub fn is_parsed(&self) -> bool {
        !matches!(self, Self::Other)
    }
}

/// Syncthing bookkeeping (`.stversions`, `.stfolder`) and conflict copies
/// (`*.sync-conflict-*`) are never library sources.
pub fn is_ignored_path(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str().to_str().is_some_and(|name| {
            matches!(name, ".stversions" | ".stfolder") || is_ignored_name(name)
        })
    })
}

fn is_ignored_name(name: &str) -> bool {
    name.contains(".sync-conflict-")
}

/// Where an item's data currently comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum Availability {
    /// The document is present.
    #[default]
    Available,
    /// Only a KOReader sidecar is present.
    Metadata,
    /// Neither is present; the saved capture is shown.
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub scheme: String,
    pub value: String,
}

impl Identifier {
    pub fn new(scheme: String, value: String) -> Self {
        Self { scheme, value }
    }

    /// Get a display-friendly version of the scheme name
    pub fn display_scheme(&self) -> String {
        match self.scheme.to_lowercase().as_str() {
            "isbn" => "ISBN".to_string(),
            "google" => "Google Books".to_string(),
            "amazon" | "asin" | "mobi-asin" => "Amazon".to_string(),
            "goodreads" => "Goodreads".to_string(),
            "doi" => "DOI".to_string(),
            "kobo" => "Kobo".to_string(),
            "oclc" => "WorldCat".to_string(),
            "lccn" => "Library of Congress".to_string(),
            "hardcover" | "hardcover-slug" => "Hardcover".to_string(),
            "hardcover-edition" => "Hardcover Edition".to_string(),
            _ => self.scheme.clone(),
        }
    }

    /// Get the URL for this identifier if it can be linked
    pub fn url(&self) -> Option<String> {
        match self.scheme.to_lowercase().as_str() {
            "isbn" => Some(format!("https://www.worldcat.org/isbn/{}", self.value)),
            "google" => Some(format!("https://books.google.com/books?id={}", self.value)),
            "amazon" | "asin" | "mobi-asin" => {
                Some(format!("https://www.amazon.com/dp/{}", self.value))
            }
            "goodreads" => Some(format!(
                "https://www.goodreads.com/book/show/{}",
                self.value
            )),
            "doi" => Some(format!("https://doi.org/{}", self.value)),
            "kobo" => Some(format!("https://www.kobo.com/ebook/{}", self.value)),
            "oclc" => Some(format!("https://www.worldcat.org/oclc/{}", self.value)),
            // For hardcover identifiers, the value is already normalized to the path segment
            // e.g., "the-coldest-touch" or "the-coldest-touch/editions/30409163"
            "hardcover" | "hardcover-edition" => {
                Some(format!("https://hardcover.app/books/{}", self.value))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItem {
    pub id: String,
    pub book_info: BookInfo,
    pub koreader_metadata: Option<KoReaderMetadata>,
    pub file_path: PathBuf,
    pub format: LibraryItemFormat,
    #[serde(default)]
    pub document_state: DocumentState,
}

/// Whether the document file is present. Without it, `file_path` is where it
/// would be, and the KOReader sidecar is the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum DocumentState {
    #[default]
    Present,
    /// Gone, but it was seen: `book_info` keeps its metadata as the base.
    Missing,
    /// Never seen: `book_info` comes from the sidecar's `doc_props`.
    Never,
}

impl DocumentState {
    /// The state after the document disappears.
    pub fn without_file(self) -> Self {
        match self {
            Self::Present | Self::Missing => Self::Missing,
            Self::Never => Self::Never,
        }
    }
}

impl LibraryItem {
    pub fn status(&self) -> BookStatus {
        self.koreader_metadata
            .as_ref()
            .and_then(|m| m.summary.as_ref())
            .map(|s| s.status.clone())
            .unwrap_or(BookStatus::Unknown)
    }

    pub fn rating(&self) -> Option<u32> {
        self.koreader_metadata
            .as_ref()
            .and_then(|m| m.summary.as_ref())
            .and_then(|s| s.rating)
    }

    pub fn review_note(&self) -> Option<&String> {
        self.koreader_metadata
            .as_ref()
            .and_then(|m| m.summary.as_ref())
            .and_then(|s| s.note.as_ref())
    }

    pub fn progress_percentage(&self) -> Option<f64> {
        self.koreader_metadata
            .as_ref()
            .and_then(|m| m.percent_finished)
    }

    pub fn annotations(&self) -> &[Annotation] {
        self.koreader_metadata
            .as_ref()
            .map(|m| m.annotations.as_slice())
            .unwrap_or(&[])
    }

    pub fn annotation_count(&self) -> usize {
        self.koreader_metadata
            .as_ref()
            .map(|m| m.annotations.len())
            .unwrap_or(0)
    }

    pub fn bookmark_count(&self) -> usize {
        self.koreader_metadata
            .as_ref()
            .map(|m| m.annotations.iter().filter(|a| a.is_bookmark()).count())
            .unwrap_or(0)
    }

    pub fn highlight_count(&self) -> usize {
        self.koreader_metadata
            .as_ref()
            .map(|m| m.annotations.iter().filter(|a| a.is_highlight()).count())
            .unwrap_or(0)
    }

    /// Stable page total for display-only usage.
    ///
    /// Valid when `pagemap_doc_pages` is present.
    pub fn stable_display_page_total(&self) -> Option<u32> {
        let metadata = self.koreader_metadata.as_ref()?;
        metadata.pagemap_doc_pages.filter(|pages| *pages > 0)
    }

    /// Stable page total for synthetic scaling usage.
    ///
    /// Valid when KOReader synthetic mode metadata is present and `pagemap_doc_pages` is set.
    pub fn synthetic_scaling_page_total(&self) -> Option<u32> {
        let metadata = self.koreader_metadata.as_ref()?;

        metadata.pagemap_chars_per_synthetic_page?;

        metadata.pagemap_doc_pages.filter(|pages| *pages > 0)
    }

    /// Get language, preferring EPUB metadata over KoReader metadata
    pub fn language(&self) -> Option<&String> {
        resolve_language(
            self.book_info.language.as_ref(),
            self.koreader_metadata
                .as_ref()
                .and_then(|m| m.text_lang.as_ref()),
        )
    }

    /// Get publisher from EPUB metadata
    pub fn publisher(&self) -> Option<&String> {
        self.book_info.publisher.as_ref()
    }

    /// Get identifiers normalized for display and linking.
    /// This includes Hardcover normalization and edition linking.
    pub fn identifiers(&self) -> Vec<Identifier> {
        let mut result: Vec<Identifier> = Vec::new();
        // Tracks (scheme_lowercase, value) pairs to avoid duplicate identifiers
        let mut dedupe_keys: HashSet<(String, String)> = HashSet::new();

        // 1) Add normalized Hardcover identifiers
        for id in self.get_normalized_hardcover_identifiers() {
            let key = (id.scheme.to_lowercase(), id.value.clone());
            if dedupe_keys.insert(key) {
                result.push(id);
            }
        }

        // 2) Add all other identifiers as-is (skip raw Hardcover family)
        for id in &self.book_info.identifiers {
            let scheme_lc = id.scheme.to_lowercase();
            if scheme_lc == "hardcover"
                || scheme_lc == "hardcover-slug"
                || scheme_lc == "hardcover-edition"
            {
                continue;
            }
            let key = (scheme_lc, id.value.clone());
            if dedupe_keys.insert(key) {
                result.push(id.clone());
            }
        }

        // Keep only identifiers with a recognized display scheme
        result
            .into_iter()
            .filter(|id| id.display_scheme() != id.scheme)
            .collect()
    }

    /// Get subjects/genres from EPUB metadata
    pub fn subjects(&self) -> &Vec<String> {
        &self.book_info.subjects
    }

    /// Get normalized Hardcover-related identifiers (slug and editions).
    /// - Picks slug from `hardcover` or `hardcover-slug` and emits a single `hardcover` entry
    /// - Emits `hardcover-edition` as `{slug}/editions/{editionId}` only when slug exists
    fn get_normalized_hardcover_identifiers(&self) -> Vec<Identifier> {
        let mut out: Vec<Identifier> = Vec::new();

        // Find Hardcover slug from either scheme
        let slug = self
            .book_info
            .identifiers
            .iter()
            .find(|id| {
                let s = id.scheme.to_lowercase();
                s == "hardcover" || s == "hardcover-slug"
            })
            .map(|id| id.value.clone());

        if let Some(slug_val) = slug {
            // Normalized hardcover slug
            out.push(Identifier::new("hardcover".to_string(), slug_val.clone()));
            // Normalized hardcover editions
            for id in &self.book_info.identifiers {
                if id.scheme.eq_ignore_ascii_case("hardcover-edition") {
                    out.push(Identifier::new(
                        "hardcover-edition".to_string(),
                        format!("{}/editions/{}", slug_val, id.value),
                    ));
                }
            }
        }

        out
    }

    /// Get series information from EPUB metadata
    pub fn series(&self) -> Option<&String> {
        self.book_info.series.as_ref()
    }

    /// Get series number from EPUB metadata
    pub fn series_number(&self) -> Option<&String> {
        self.book_info.series_number.as_ref()
    }

    /// Get the content type of this book
    pub fn content_type(&self) -> ContentType {
        self.format.content_type_for_path(&self.file_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fixtures;

    #[test]
    fn enables_synthetic_scaling_only_when_synthetic_metadata_exists() {
        let item = fixtures::library_item(
            "id-1",
            Some(fixtures::koreader_metadata_for_pages(
                "md5", true, true, 300,
            )),
        );

        assert_eq!(item.stable_display_page_total(), Some(300));
        assert_eq!(item.synthetic_scaling_page_total(), Some(300));
    }

    #[test]
    fn language_prefers_parser_language_over_koreader_text_lang() {
        let mut metadata = fixtures::koreader_metadata_for_pages("md5", true, false, 300);
        metadata.text_lang = Some("en-US".to_string());

        let mut item = fixtures::library_item("id-1", Some(metadata));
        item.book_info.language = Some("de-DE".to_string());

        assert_eq!(item.language().map(String::as_str), Some("de-DE"));
    }

    #[test]
    fn language_falls_back_to_koreader_text_lang_when_parser_language_is_missing() {
        let mut metadata = fixtures::koreader_metadata_for_pages("md5", true, false, 300);
        metadata.text_lang = Some("en-US".to_string());

        let item = fixtures::library_item("id-1", Some(metadata));

        assert_eq!(item.language().map(String::as_str), Some("en-US"));
    }
}

/// A single chapter entry from a book table of contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterEntry {
    pub title: String,
    /// Fractional position within the book (0.0–1.0), estimated from spine byte sizes.
    pub position: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookInfo {
    #[serde(default)]
    pub custom: Option<CustomMetadata>,
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub identifiers: Vec<Identifier>, // Supports multiple identifiers per item
    pub subjects: Vec<String>,        // Genres/subjects/tags
    pub series: Option<String>,
    pub series_number: Option<String>,
    pub pages: Option<u32>, // Page count from format (EPUB page-list, comic images)
    pub chapters: Vec<ChapterEntry>, // Table of contents (EPUB, FB2)
    pub cover_data: Option<Vec<u8>>,
    pub cover_mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BookStatus {
    Reading,
    Complete,
    Abandoned,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for BookStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookStatus::Reading => write!(f, "reading"),
            BookStatus::Complete => write!(f, "complete"),
            BookStatus::Abandoned => write!(f, "abandoned"),
            BookStatus::Unknown => write!(f, "unknown"),
        }
    }
}
