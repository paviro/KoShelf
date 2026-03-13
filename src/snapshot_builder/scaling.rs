//! Synthetic stable-page scaling helpers.

use crate::models::{LibraryItem, StatisticsData};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PageScaling {
    enabled: bool,
    factor_by_md5: HashMap<String, f64>,
}

impl PageScaling {
    pub fn from_inputs(
        enabled: bool,
        items: &[LibraryItem],
        stats_data: Option<&StatisticsData>,
    ) -> Self {
        if !enabled {
            return Self {
                enabled: false,
                factor_by_md5: HashMap::new(),
            };
        }

        let Some(stats_data) = stats_data else {
            return Self {
                enabled: true,
                factor_by_md5: HashMap::new(),
            };
        };

        let md5_to_item: HashMap<String, &LibraryItem> = items
            .iter()
            .filter_map(|item| {
                item.koreader_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.partial_md5_checksum.as_ref())
                    .map(|md5| (md5.to_lowercase(), item))
            })
            .collect();

        let mut factor_by_md5: HashMap<String, f64> = HashMap::new();

        for stat_book in &stats_data.books {
            let md5_key = stat_book.md5.to_lowercase();
            let Some(item) = md5_to_item.get(&md5_key) else {
                continue;
            };

            let Some(stable_total) = item.synthetic_scaling_page_total() else {
                continue;
            };

            let rendered_total = stat_book.pages.filter(|pages| *pages > 0).or_else(|| {
                item.koreader_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.doc_pages.map(i64::from))
                    .filter(|pages| *pages > 0)
            });

            let Some(rendered_total) = rendered_total else {
                continue;
            };

            let factor = stable_total as f64 / rendered_total as f64;
            if !factor.is_finite() || factor <= 0.0 {
                continue;
            }

            factor_by_md5.insert(md5_key, factor);
        }

        Self {
            enabled: true,
            factor_by_md5,
        }
    }

    pub fn factor_for_md5(&self, md5: &str) -> Option<f64> {
        if !self.enabled {
            return None;
        }
        self.factor_by_md5.get(&md5.to_lowercase()).copied()
    }

    pub fn scale_pages_for_md5(&self, md5: &str, pages: i64) -> i64 {
        match self.factor_for_md5(md5) {
            Some(factor) => Self::scale_pages_with_factor(pages, factor),
            None => pages,
        }
    }

    pub fn scale_pages_with_factor(pages: i64, factor: f64) -> i64 {
        if pages <= 0 || !factor.is_finite() || factor <= 0.0 {
            return 0;
        }

        Self::round_pages(pages as f64 * factor)
    }

    fn round_pages(value: f64) -> i64 {
        if !value.is_finite() || value <= 0.0 {
            return 0;
        }

        value.round() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ContentType;
    use crate::tests::fixtures;

    fn test_item(id: &str, md5: &str, synthetic: bool, stable_total: u32) -> LibraryItem {
        let mut metadata =
            fixtures::koreader_metadata_for_pages(md5, true, synthetic, stable_total);
        metadata.pagemap_current_page_label = None;
        metadata.pagemap_last_page_label = None;
        fixtures::library_item(id, Some(metadata))
    }

    #[test]
    fn builds_factors_for_synthetic_items_only() {
        let item_synthetic = test_item("1", "md5-synth", true, 300);
        let item_publisher_only = test_item("2", "md5-pub", false, 400);

        let books = vec![
            fixtures::stat_book(1, "md5-synth", 200, ContentType::Book),
            fixtures::stat_book(2, "md5-pub", 200, ContentType::Book),
        ];
        let stats_data = fixtures::statistics_data(books.clone(), Vec::new());

        let scaling = PageScaling::from_inputs(
            true,
            &[item_synthetic.clone(), item_publisher_only.clone()],
            Some(&stats_data),
        );
        assert_eq!(scaling.factor_for_md5("md5-synth"), Some(1.5));
        assert_eq!(scaling.factor_for_md5("md5-pub"), None);

        let off = PageScaling::from_inputs(false, &[item_synthetic], Some(&stats_data));
        assert_eq!(off.factor_for_md5("md5-synth"), None);
    }

    #[test]
    fn builds_factors_even_when_page_labels_are_disabled() {
        let mut item = test_item("1", "md5-synth-no-labels", true, 300);
        if let Some(metadata) = item.koreader_metadata.as_mut() {
            metadata.pagemap_use_page_labels = Some(false);
        }

        let books = vec![fixtures::stat_book(
            1,
            "md5-synth-no-labels",
            200,
            ContentType::Book,
        )];
        let stats_data = fixtures::statistics_data(books.clone(), Vec::new());

        let scaling = PageScaling::from_inputs(true, &[item], Some(&stats_data));
        assert_eq!(scaling.factor_for_md5("md5-synth-no-labels"), Some(1.5));
    }
}
