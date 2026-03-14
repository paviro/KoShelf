//! Page scaling: converts rendered page counts to synthetic stable-page equivalents.
//!
//! KOReader's synthetic pagination produces a stable page total (`pagemap_doc_pages`)
//! that differs from the rendered page count stored in the statistics DB. The scaling
//! factor `pagemap_doc_pages / stat_book.pages` is applied to all page counts so that
//! reading statistics reflect synthetic pages when the feature is enabled.

use std::collections::HashMap;

use crate::koreader::types::StatisticsData;

/// Scaling factors for converting rendered page numbers to synthetic stable-page
/// equivalents used by KOReader.
#[derive(Debug, Clone)]
pub struct PageScaling {
    factor_by_book_id: HashMap<i64, f64>,
    factor_by_md5: HashMap<String, f64>,
}

impl PageScaling {
    /// Build scaling factors from DB-stored page data and statistics data.
    ///
    /// `scaling_inputs_by_md5` maps item MD5 → `(pagemap_doc_pages, doc_pages)`.
    /// `pagemap_doc_pages` is the synthetic stable total (numerator).
    /// `doc_pages` is the KOReader rendered page count, used as fallback denominator
    /// when `stat_book.pages` is unavailable.
    pub fn from_db_inputs(
        scaling_inputs_by_md5: &HashMap<String, (i32, Option<i32>)>,
        stats_data: &StatisticsData,
    ) -> Self {
        let mut factor_by_book_id = HashMap::new();
        let mut factor_by_md5 = HashMap::new();

        for stat_book in &stats_data.books {
            let md5_key = stat_book.md5.to_lowercase();
            let Some(&(pagemap_doc_pages, fallback_doc_pages)) =
                scaling_inputs_by_md5.get(&md5_key)
            else {
                continue;
            };

            let rendered_total = stat_book
                .pages
                .filter(|&p| p > 0)
                .or(fallback_doc_pages.map(i64::from).filter(|&p| p > 0));

            let Some(rendered_total) = rendered_total else {
                continue;
            };

            let factor = pagemap_doc_pages as f64 / rendered_total as f64;
            if !factor.is_finite() || factor <= 0.0 {
                continue;
            }

            factor_by_book_id.insert(stat_book.id, factor);
            factor_by_md5.insert(md5_key, factor);
        }

        Self {
            factor_by_book_id,
            factor_by_md5,
        }
    }

    /// Create a no-op instance that performs no scaling.
    pub fn disabled() -> Self {
        Self {
            factor_by_book_id: HashMap::new(),
            factor_by_md5: HashMap::new(),
        }
    }

    /// Return the raw scaling factor for a stat book ID, if any.
    ///
    /// Use this when accumulating page counts as floats before rounding once
    /// per aggregation bucket (day/month/etc.) to avoid per-page rounding errors.
    pub fn factor_for_book_id(&self, book_id: i64) -> f64 {
        self.factor_by_book_id.get(&book_id).copied().unwrap_or(1.0)
    }

    /// Scale a page count using the factor for the given stat book ID.
    pub fn scale_pages_for_book_id(&self, book_id: i64, pages: i64) -> i64 {
        match self.factor_by_book_id.get(&book_id) {
            Some(&factor) => scale_pages_with_factor(pages, factor),
            None => pages,
        }
    }

    /// Scale a page count using the factor for the given MD5.
    pub fn scale_pages_for_md5(&self, md5: &str, pages: i64) -> i64 {
        match self.factor_by_md5.get(&md5.to_lowercase()) {
            Some(&factor) => scale_pages_with_factor(pages, factor),
            None => pages,
        }
    }
}

/// Round a floating-point page total to the nearest integer.
///
/// Used after accumulating scaled page counts as floats to round once per
/// aggregation bucket rather than per individual page stat.
pub fn round_pages(value: f64) -> i64 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    value.round() as i64
}

/// Apply a scaling factor to a page count, rounding to nearest integer.
fn scale_pages_with_factor(pages: i64, factor: f64) -> i64 {
    if pages <= 0 || !factor.is_finite() || factor <= 0.0 {
        return 0;
    }
    let value = pages as f64 * factor;
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    value.round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ContentType;
    use crate::tests::fixtures;

    #[test]
    fn builds_factors_for_matching_books() {
        let books = vec![
            fixtures::stat_book(1, "md5-a", 200, ContentType::Book),
            fixtures::stat_book(2, "md5-b", 400, ContentType::Book),
        ];
        let stats_data = fixtures::statistics_data(books, Vec::new());

        let mut inputs = HashMap::new();
        inputs.insert("md5-a".to_string(), (300, Some(200)));
        // md5-b not in inputs — should get no factor

        let scaling = PageScaling::from_db_inputs(&inputs, &stats_data);

        assert_eq!(scaling.scale_pages_for_book_id(1, 1), 2); // 1 * 1.5 = 1.5 → 2
        assert_eq!(scaling.scale_pages_for_book_id(2, 1), 1); // no factor
        assert_eq!(scaling.scale_pages_for_md5("md5-a", 10), 15); // 10 * 1.5 = 15
        assert_eq!(scaling.scale_pages_for_md5("md5-b", 10), 10); // no factor
    }

    #[test]
    fn factor_for_book_id_returns_raw_factor_or_one() {
        let books = vec![fixtures::stat_book(1, "md5-a", 200, ContentType::Book)];
        let stats_data = fixtures::statistics_data(books, Vec::new());
        let mut inputs = HashMap::new();
        inputs.insert("md5-a".to_string(), (300, Some(200)));

        let scaling = PageScaling::from_db_inputs(&inputs, &stats_data);
        assert!((scaling.factor_for_book_id(1) - 1.5).abs() < f64::EPSILON);
        assert!((scaling.factor_for_book_id(999) - 1.0).abs() < f64::EPSILON); // no factor → 1.0
    }

    #[test]
    fn disabled_performs_no_scaling() {
        let scaling = PageScaling::disabled();
        assert_eq!(scaling.scale_pages_for_book_id(1, 5), 5);
        assert_eq!(scaling.scale_pages_for_md5("any", 5), 5);
        assert!((scaling.factor_for_book_id(1) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn skips_books_with_zero_pages() {
        let books = vec![fixtures::stat_book(1, "md5-zero", 0, ContentType::Book)];
        let stats_data = fixtures::statistics_data(books, Vec::new());
        let mut inputs = HashMap::new();
        inputs.insert("md5-zero".to_string(), (300, None));

        let scaling = PageScaling::from_db_inputs(&inputs, &stats_data);
        assert_eq!(scaling.scale_pages_for_book_id(1, 5), 5);
    }

    #[test]
    fn falls_back_to_doc_pages_when_stat_pages_missing() {
        // stat_book.pages is None, but doc_pages (200) is available as fallback
        let mut book = fixtures::stat_book(1, "md5-fb", 0, ContentType::Book);
        book.pages = None;
        let stats_data = fixtures::statistics_data(vec![book], Vec::new());
        let mut inputs = HashMap::new();
        inputs.insert("md5-fb".to_string(), (300, Some(200)));

        let scaling = PageScaling::from_db_inputs(&inputs, &stats_data);
        assert_eq!(scaling.scale_pages_for_md5("md5-fb", 10), 15); // 300/200 = 1.5
    }

    #[test]
    fn case_insensitive_md5_lookup() {
        let books = vec![fixtures::stat_book(1, "MD5-UPPER", 200, ContentType::Book)];
        let stats_data = fixtures::statistics_data(books, Vec::new());
        let mut inputs = HashMap::new();
        inputs.insert("md5-upper".to_string(), (300, Some(200)));

        let scaling = PageScaling::from_db_inputs(&inputs, &stats_data);
        assert_eq!(scaling.scale_pages_for_md5("MD5-UPPER", 10), 15);
    }

    #[test]
    fn scale_pages_with_factor_guards() {
        assert_eq!(scale_pages_with_factor(0, 1.5), 0);
        assert_eq!(scale_pages_with_factor(-1, 1.5), 0);
        assert_eq!(scale_pages_with_factor(10, 0.0), 0);
        assert_eq!(scale_pages_with_factor(10, f64::NAN), 0);
        assert_eq!(scale_pages_with_factor(10, f64::INFINITY), 0);
        assert_eq!(scale_pages_with_factor(10, 1.5), 15);
    }
}
