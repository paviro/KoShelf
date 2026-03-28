use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SiteCapabilities {
    pub has_books: bool,
    pub has_comics: bool,
    pub has_reading_data: bool,
    pub has_files: bool,

    pub has_writeback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_chars: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteAuth {
    pub authenticated: bool,
    pub password_policy: PasswordPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteData {
    pub title: String,
    pub language: String,
    pub capabilities: SiteCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<SiteAuth>,
}

impl Default for SiteData {
    fn default() -> Self {
        Self {
            title: String::new(),
            language: "en_US".to_string(),
            capabilities: SiteCapabilities::default(),
            auth: None,
        }
    }
}
