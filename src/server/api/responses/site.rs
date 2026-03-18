use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SiteCapabilities {
    pub has_books: bool,
    pub has_comics: bool,
    pub has_reading_data: bool,
    pub has_files: bool,
    pub auth_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_chars: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteData {
    pub title: String,
    pub language: String,
    pub capabilities: SiteCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_policy: Option<PasswordPolicy>,
}

impl Default for SiteData {
    fn default() -> Self {
        Self {
            title: String::new(),
            language: "en_US".to_string(),
            capabilities: SiteCapabilities::default(),
            authenticated: None,
            password_policy: None,
        }
    }
}
