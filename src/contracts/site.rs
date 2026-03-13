use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteCapabilities {
    pub has_books: bool,
    pub has_comics: bool,
    pub has_reading_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteData {
    pub title: String,
    pub language: String,
    pub capabilities: SiteCapabilities,
}
