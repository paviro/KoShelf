use serde::{Deserialize, Serialize};

use super::{BookStatus, ReaderPresentation};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoReaderMetadata {
    pub annotations: Vec<Annotation>,
    pub reader_presentation: Option<ReaderPresentation>,
    pub doc_pages: Option<u32>,
    pub doc_path: Option<String>,
    pub doc_props: Option<DocProps>,
    pub handmade_flows_enabled: Option<bool>,
    pub handmade_flow_points: Vec<FlowPoint>,
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

impl KoReaderMetadata {
    /// Calculate the number of pages in hidden flows.
    ///
    /// Mirrors KOReader's `ReaderHandMade:updateDocFlows()` algorithm:
    /// flow points are processed in order; a `hidden=true` point starts a hidden
    /// range, and the next `hidden=false` point (or end of book) closes it.
    /// Returns `None` if hidden flows are not enabled or there are no hidden
    /// flow points.
    pub fn hidden_flow_pages(&self) -> Option<u32> {
        if self.handmade_flows_enabled != Some(true) {
            return None;
        }
        let total_pages = self.doc_pages?;
        if self.handmade_flow_points.is_empty() {
            return None;
        }

        let mut hidden_pages: u32 = 0;
        let mut cur_hidden_start: Option<u32> = None;

        for point in &self.handmade_flow_points {
            if point.hidden && cur_hidden_start.is_none() {
                cur_hidden_start = Some(point.page);
            } else if !point.hidden && cur_hidden_start.is_some() {
                let start = cur_hidden_start.unwrap();
                hidden_pages += point.page.saturating_sub(start);
                cur_hidden_start = None;
            }
        }

        // If still in a hidden range at end of book, close it
        if let Some(start) = cur_hidden_start {
            hidden_pages += (total_pages + 1).saturating_sub(start);
        }

        if hidden_pages > 0 {
            Some(hidden_pages)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowPoint {
    pub hidden: bool,
    pub page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub chapter: Option<String>,
    pub datetime: Option<String>,
    pub datetime_updated: Option<String>,
    pub pageno: Option<u32>,
    pub pos0: Option<String>,
    pub pos1: Option<String>,
    pub text: Option<String>, // Optional: highlights have text, bookmarks don't
    pub note: Option<String>,
    pub color: Option<String>,
    pub drawer: Option<String>,
}

impl Annotation {
    /// Returns true if this annotation is a bookmark (no drawer), false if it's a highlight/note.
    /// Matches KOReader's `getBookmarkType`: annotations without `drawer` are bookmarks.
    pub fn is_bookmark(&self) -> bool {
        self.drawer.is_none()
    }

    /// Returns true if this annotation is a highlight or note (has drawer), false if it's a bookmark.
    /// Matches KOReader's `getBookmarkType`: annotations with `drawer` are highlights or notes.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata_with_flows(
        doc_pages: u32,
        enabled: bool,
        flow_points: Vec<FlowPoint>,
    ) -> KoReaderMetadata {
        KoReaderMetadata {
            annotations: Vec::new(),
            reader_presentation: None,
            doc_pages: Some(doc_pages),
            doc_path: None,
            doc_props: None,
            handmade_flows_enabled: Some(enabled),
            handmade_flow_points: flow_points,
            pagemap_use_page_labels: None,
            pagemap_chars_per_synthetic_page: None,
            pagemap_doc_pages: None,
            pagemap_current_page_label: None,
            pagemap_last_page_label: None,
            partial_md5_checksum: None,
            percent_finished: None,
            stats: None,
            summary: None,
            text_lang: None,
        }
    }

    #[test]
    fn hidden_flow_pages_single_hidden_range_at_end() {
        // Book: 1000 pages, appendix starts at page 701 (hidden to end)
        let m = metadata_with_flows(
            1000,
            true,
            vec![FlowPoint {
                hidden: true,
                page: 701,
            }],
        );
        assert_eq!(m.hidden_flow_pages(), Some(300));
    }

    #[test]
    fn hidden_flow_pages_hidden_range_in_middle() {
        // Book: 1000 pages, hidden section from 400-599 (200 pages)
        let m = metadata_with_flows(
            1000,
            true,
            vec![
                FlowPoint {
                    hidden: true,
                    page: 400,
                },
                FlowPoint {
                    hidden: false,
                    page: 600,
                },
            ],
        );
        assert_eq!(m.hidden_flow_pages(), Some(200));
    }

    #[test]
    fn hidden_flow_pages_multiple_hidden_ranges() {
        // Two hidden sections: 100-199 (100 pages) and 500-end (501 pages)
        let m = metadata_with_flows(
            1000,
            true,
            vec![
                FlowPoint {
                    hidden: true,
                    page: 100,
                },
                FlowPoint {
                    hidden: false,
                    page: 200,
                },
                FlowPoint {
                    hidden: true,
                    page: 500,
                },
            ],
        );
        assert_eq!(m.hidden_flow_pages(), Some(601));
    }

    #[test]
    fn hidden_flow_pages_none_when_disabled() {
        let m = metadata_with_flows(
            1000,
            false,
            vec![FlowPoint {
                hidden: true,
                page: 701,
            }],
        );
        assert_eq!(m.hidden_flow_pages(), None);
    }

    #[test]
    fn hidden_flow_pages_none_when_no_flow_points() {
        let m = metadata_with_flows(1000, true, vec![]);
        assert_eq!(m.hidden_flow_pages(), None);
    }

    #[test]
    fn hidden_flow_pages_none_when_no_hidden_points() {
        let m = metadata_with_flows(
            1000,
            true,
            vec![FlowPoint {
                hidden: false,
                page: 500,
            }],
        );
        assert_eq!(m.hidden_flow_pages(), None);
    }

    #[test]
    fn hidden_flow_pages_hidden_range_at_start() {
        // Front matter: pages 1-50 hidden
        let m = metadata_with_flows(
            500,
            true,
            vec![
                FlowPoint {
                    hidden: true,
                    page: 1,
                },
                FlowPoint {
                    hidden: false,
                    page: 51,
                },
            ],
        );
        assert_eq!(m.hidden_flow_pages(), Some(50));
    }
}
