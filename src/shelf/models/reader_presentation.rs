use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderPresentation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_face: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size_pt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_spacing_percent: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h_page_margins: Option<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_page_margin: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b_page_margin: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedded_css: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedded_fonts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyphenation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floating_punctuation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_spacing: Option<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_expansion: Option<u32>,
}

impl ReaderPresentation {
    pub fn is_empty(&self) -> bool {
        self.font_face.is_none()
            && self.font_size_pt.is_none()
            && self.line_spacing_percent.is_none()
            && self.h_page_margins.is_none()
            && self.t_page_margin.is_none()
            && self.b_page_margin.is_none()
            && self.embedded_css.is_none()
            && self.embedded_fonts.is_none()
            && self.hyphenation.is_none()
            && self.floating_punctuation.is_none()
            && self.word_spacing.is_none()
            && self.word_expansion.is_none()
    }
}
