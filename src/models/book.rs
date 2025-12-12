use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::koreader_metadata::KoReaderMetadata;

/// Supported ebook formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BookFormat {
    Epub,
    Fb2,
    Mobi,
    Cbz,
    Cbr,
}

impl BookFormat {
    /// Try to detect format from a file path's extension
    /// Handles compound extensions like .fb2.zip
    pub fn from_path(path: &Path) -> Option<Self> {
        let filename = path.file_name()?.to_str()?.to_lowercase();
        
        // Check for compound extensions first (e.g., .fb2.zip)
        if filename.ends_with(".fb2.zip") {
            return Some(Self::Fb2);
        }
        
        // Then check simple extensions
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "epub" => Some(Self::Epub),
            "fb2" => Some(Self::Fb2),
            "mobi" => Some(Self::Mobi),
            "cbz" => Some(Self::Cbz),
            #[cfg(not(windows))]
            "cbr" => Some(Self::Cbr),
            _ => None,
        }
    }

    /// Get the KOReader metadata filename for this format
    pub fn metadata_filename(&self) -> &'static str {
        match self {
            Self::Epub => "metadata.epub.lua",
            Self::Fb2 => "metadata.fb2.lua",
            Self::Mobi => "metadata.mobi.lua",
            Self::Cbz => "metadata.cbz.lua",
            Self::Cbr => "metadata.cbr.lua",
        }
    }

    /// Check if a filename is a KOReader metadata file for any supported format
    pub fn is_metadata_file(filename: &str) -> bool {
        matches!(
            filename,
            "metadata.epub.lua"
                | "metadata.fb2.lua"
                | "metadata.mobi.lua"
                | "metadata.cbz.lua"
                | "metadata.cbr.lua"
        )
    }
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

    /// Check if this identifier can be linked to an external service
    pub fn is_linkable(&self) -> bool {
        self.url().is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub book_info: BookInfo,
    pub koreader_metadata: Option<KoReaderMetadata>,
    pub file_path: PathBuf,
}

impl Book {
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

    pub fn star_display(&self) -> [bool; 5] {
        let rating = self.rating().unwrap_or(0);
        [
            rating >= 1,
            rating >= 2,
            rating >= 3,
            rating >= 4,
            rating >= 5,
        ]
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

    pub fn progress_percentage_display(&self) -> u32 {
        self.progress_percentage()
            .map(|p| (p * 100.0).round() as u32)
            .unwrap_or(0)
    }

    pub fn annotations(&self) -> &[super::koreader_metadata::Annotation] {
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

    pub fn doc_pages(&self) -> Option<u32> {
        self.koreader_metadata.as_ref()?.doc_pages
    }

    pub fn note_count(&self) -> usize {
        self.koreader_metadata
            .as_ref()
            .and_then(|m| m.stats.as_ref())
            .and_then(|s| s.notes)
            .map(|n| n as usize)
            .unwrap_or(0)
    }

    /// Get language, preferring EPUB metadata over KoReader metadata
    pub fn language(&self) -> Option<&String> {
        self.book_info.language.as_ref().or_else(|| {
            self.koreader_metadata
                .as_ref()
                .and_then(|m| m.text_lang.as_ref())
        })
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

    /// Get a formatted display string for subjects/genres
    pub fn subjects_display(&self) -> Option<String> {
        if self.book_info.subjects.is_empty() {
            None
        } else {
            Some(self.book_info.subjects.join(", "))
        }
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

    /// Get formatted series display (e.g., "Series Name #1")
    pub fn series_display(&self) -> Option<String> {
        match (self.series(), self.series_number()) {
            (Some(series), Some(number)) => Some(format!("{} #{}", series, number)),
            (Some(series), None) => Some(series.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookInfo {
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub identifiers: Vec<Identifier>, // Changed from single identifier to vector
    pub subjects: Vec<String>,        // Genres/subjects/tags
    pub series: Option<String>,
    pub series_number: Option<String>,
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
