use serde::{Deserialize, Serialize};

use super::common::ApiMeta;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteCapabilities {
    pub has_books: bool,
    pub has_comics: bool,
    pub has_reading_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteResponse {
    pub meta: ApiMeta,
    pub title: String,
    pub language: String,
    pub capabilities: SiteCapabilities,
}
