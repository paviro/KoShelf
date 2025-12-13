use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use super::BookStatus;

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
    pub datetime: Option<String>,
    pub pageno: Option<u32>,
    #[serde(skip_serializing)]
    pub pos0: Option<String>,
    #[serde(skip_serializing)]
    pub pos1: Option<String>,
    pub text: Option<String>, // Optional: highlights have text, bookmarks don't
    pub note: Option<String>,
}

impl Annotation {
    pub fn formatted_datetime(&self, translations: &crate::i18n::Translations) -> Option<String> {
        self.datetime.as_ref().and_then(|dt| {
            NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|ndt| {
                    let locale = translations.locale();
                    let format_str = translations.get("datetime.full");

                    // format_localized is only available on NaiveDate and DateTime<Tz>, not NaiveDateTime - we need to convert to a DateTime<Utc> first
                    let dt_utc: DateTime<Utc> = DateTime::from_naive_utc_and_offset(ndt, Utc);
                    dt_utc.format_localized(&format_str, locale).to_string()
                })
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
