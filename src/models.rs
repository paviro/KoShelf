use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Sanitizes HTML content to only allow safe tags while removing styles and dangerous elements
pub fn sanitize_html(html: &str) -> String {
    use ammonia::Builder;
    
    let allowed_tags: &[&str] = &[
        "p", "br", "h1", "h2", "h3", "h4", "h5", "h6",
        "ul", "ol", "li", "strong", "em", "b", "i",
        "blockquote", "pre", "code", "div", "span"
    ];
    
    let empty_attrs: &[&str] = &[];
    
    Builder::default()
        .add_tags(allowed_tags)
        .add_generic_attributes(empty_attrs)  // Remove all attributes including style
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
            "amazon" => "Amazon".to_string(),
            "asin" => "Amazon".to_string(),
            "mobi-asin" => "Amazon".to_string(),
            "goodreads" => "Goodreads".to_string(),
            "doi" => "DOI".to_string(),
            "kobo" => "Kobo".to_string(),
            "oclc" => "WorldCat".to_string(),
            "lccn" => "Library of Congress".to_string(),
            _ => self.scheme.clone(),
        }
    }
    
    /// Get the URL for this identifier if it can be linked
    pub fn url(&self) -> Option<String> {
        match self.scheme.to_lowercase().as_str() {
            "isbn" => Some(format!("https://www.worldcat.org/isbn/{}", self.value)),
            "google" => Some(format!("https://books.google.com/books?id={}", self.value)),
            "amazon" | "asin" | "mobi-asin" => Some(format!("https://www.amazon.com/dp/{}", self.value)),
            "goodreads" => Some(format!("https://www.goodreads.com/book/show/{}", self.value)),
            "doi" => Some(format!("https://doi.org/{}", self.value)),
            "kobo" => Some(format!("https://www.kobo.com/ebook/{}", self.value)),
            "oclc" => Some(format!("https://www.worldcat.org/oclc/{}", self.value)),
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
        self.epub_info.language.as_ref()
            .or_else(|| {
                self.koreader_metadata
                    .as_ref()
                    .and_then(|m| m.text_lang.as_ref())
            })
    }
    
    /// Get publisher from EPUB metadata
    pub fn publisher(&self) -> Option<&String> {
        self.epub_info.publisher.as_ref()
    }
    
    /// Get all identifiers from EPUB metadata, filtered to only include those with proper display schemes
    pub fn identifiers(&self) -> Vec<&Identifier> {
        self.epub_info.identifiers.iter()
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
    pub subjects: Vec<String>, // Genres/subjects/tags
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoReaderMetadata {
    pub annotations: Vec<Annotation>,
    pub doc_pages: Option<u32>,
    pub doc_path: Option<String>,
    pub doc_props: Option<DocProps>,
    pub partial_md5_checksum: Option<String>,
    pub percent_finished: Option<f64>,
    pub stats: Option<Stats>,
    pub summary: Option<Summary>,
    pub text_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub chapter: Option<String>,
    pub color: Option<String>,
    pub datetime: Option<String>,
    pub drawer: Option<String>,
    pub page: Option<String>,
    pub pageno: Option<u32>,
    pub pos0: Option<String>,
    pub pos1: Option<String>,
    pub text: String,
    pub note: Option<String>,
}

impl Annotation {
    pub fn formatted_datetime(&self) -> Option<String> {
        self.datetime.as_ref().and_then(|dt| {
            // Parse "2025-06-17 23:48:38" format
            chrono::NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|ndt| ndt.format("%B %d, %Y at %I:%M %p").to_string())
        })
    }
    
    /// Returns true if this annotation is a bookmark (no pos0/pos1), false if it's a highlight/quote
    pub fn is_bookmark(&self) -> bool {
        self.pos0.is_none() && self.pos1.is_none()
    }
    
    /// Returns true if this annotation is a highlight/quote (has pos0/pos1), false if it's a bookmark
    pub fn is_highlight(&self) -> bool {
        !self.is_bookmark()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocProps {
    pub authors: Option<String>,
    pub description: Option<String>,
    pub identifiers: Option<String>,
    pub keywords: Option<String>,
    pub language: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub authors: Option<String>,
    pub highlights: Option<u32>,
    pub language: Option<String>,
    pub notes: Option<u32>,
    pub pages: Option<u32>,
    pub series: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub modified: Option<String>,
    pub note: Option<String>,
    pub rating: Option<u32>,
    pub status: BookStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BookStatus {
    Reading,
    Complete,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for BookStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookStatus::Reading => write!(f, "reading"),
            BookStatus::Complete => write!(f, "complete"),
            BookStatus::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingStats {
    // Overall stats
    pub total_read_time: i64,          // seconds
    pub total_page_reads: i64,
    pub longest_read_time_in_day: i64, // seconds
    pub most_pages_in_day: i64,
    
    // Weekly stats
    pub weeks: Vec<WeeklyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyStats {
    pub start_date: String,            // ISO format yyyy-mm-dd
    pub end_date: String,              // ISO format yyyy-mm-dd
    pub read_time: i64,                // seconds
    pub pages_read: i64,
    pub avg_pages_per_day: f64,
    pub avg_read_time_per_day: f64,    // seconds
} 