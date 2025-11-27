use std::collections::HashSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::koreader_metadata::KoReaderMetadata;

/// Sanitizes HTML content to only allow safe tags while removing styles and dangerous elements
pub fn sanitize_html(html: &str) -> String {
    use ammonia::Builder;

    let allowed_tags: &[&str] = &[
        "p",
        "br",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "ul",
        "ol",
        "li",
        "strong",
        "em",
        "b",
        "i",
        "blockquote",
        "pre",
        "code",
        "div",
        "span",
    ];

    let empty_attrs: &[&str] = &[];

    Builder::default()
        .add_tags(allowed_tags)
        .add_generic_attributes(empty_attrs) // Remove all attributes including style
        .clean(html)
        .to_string()
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
    pub epub_info: EpubInfo,
    pub koreader_metadata: Option<KoReaderMetadata>,
    pub epub_path: PathBuf,
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

    /// Get language, preferring EPUB metadata over KoReader metadata
    pub fn language(&self) -> Option<&String> {
        self.epub_info.language.as_ref().or_else(|| {
            self.koreader_metadata
                .as_ref()
                .and_then(|m| m.text_lang.as_ref())
        })
    }

    /// Get publisher from EPUB metadata
    pub fn publisher(&self) -> Option<&String> {
        self.epub_info.publisher.as_ref()
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
        for id in &self.epub_info.identifiers {
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
        &self.epub_info.subjects
    }

    /// Get a formatted display string for subjects/genres
    pub fn subjects_display(&self) -> Option<String> {
        if self.epub_info.subjects.is_empty() {
            None
        } else {
            Some(self.epub_info.subjects.join(", "))
        }
    }

    /// Get normalized Hardcover-related identifiers (slug and editions).
    /// - Picks slug from `hardcover` or `hardcover-slug` and emits a single `hardcover` entry
    /// - Emits `hardcover-edition` as `{slug}/editions/{editionId}` only when slug exists
    fn get_normalized_hardcover_identifiers(&self) -> Vec<Identifier> {
        let mut out: Vec<Identifier> = Vec::new();

        // Find Hardcover slug from either scheme
        let slug = self
            .epub_info
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
            for id in &self.epub_info.identifiers {
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
        self.epub_info.series.as_ref()
    }

    /// Get series number from EPUB metadata
    pub fn series_number(&self) -> Option<&String> {
        self.epub_info.series_number.as_ref()
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
pub struct EpubInfo {
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

impl EpubInfo {
    pub fn sanitized_description(&self) -> Option<String> {
        self.description.as_ref().map(|desc| sanitize_html(desc))
    }
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
            BookStatus::Abandoned => write!(f, "complete"),
            BookStatus::Unknown => write!(f, "unknown"),
        }
    }
}
