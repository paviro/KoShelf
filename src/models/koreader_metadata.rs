use serde::{Deserialize, Serialize};

use super::BookStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoReaderMetadata {
    pub annotations: Vec<Annotation>,
    pub doc_pages: Option<u32>,
    pub doc_path: Option<String>,
    pub doc_props: Option<DocProps>,
    pub pagemap_use_page_labels: Option<bool>,
    pub pagemap_chars_per_synthetic_page: Option<u32>,
    pub pagemap_doc_pages: Option<u32>,
    pub pagemap_current_page_label: Option<String>,
    pub pagemap_last_page_label: Option<String>,
    pub partial_md5_checksum: Option<String>,
    pub percent_finished: Option<f64>,
    pub stats: Option<Stats>,
    pub summary: Option<Summary>,
    pub text_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub chapter: Option<String>,
    pub datetime: Option<String>,
    pub pageno: Option<u32>,
    pub pos0: Option<String>,
    pub pos1: Option<String>,
    pub text: Option<String>, // Optional: highlights have text, bookmarks don't
    pub note: Option<String>,
}

impl Annotation {
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
