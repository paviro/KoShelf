use serde::{Deserialize, Serialize};

use crate::shelf::models::LibraryItem;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SiteCapabilities {
    pub has_books: bool,
    pub has_comics: bool,
    pub has_reading_data: bool,
    pub auth_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteData {
    pub title: String,
    pub language: String,
    pub capabilities: SiteCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticated: Option<bool>,
}

impl Default for SiteData {
    fn default() -> Self {
        Self {
            title: String::new(),
            language: "en_US".to_string(),
            capabilities: SiteCapabilities::default(),
            authenticated: None,
        }
    }
}

impl SiteData {
    pub fn from_items(
        items: &[LibraryItem],
        has_reading_data: bool,
        title: impl Into<String>,
        language: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            language: language.into(),
            capabilities: SiteCapabilities {
                has_books: items.iter().any(|item| item.is_book()),
                has_comics: items.iter().any(|item| item.is_comic()),
                has_reading_data,
                auth_enabled: false,
            },
            authenticated: None,
        }
    }
}
