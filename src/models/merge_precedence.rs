//! Merge-precedence helpers shared across library ingestion and projections.

/// Source used to resolve a canonical partial MD5 value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalPartialMd5Source {
    /// KOReader metadata `partial_md5_checksum`.
    Metadata,
    /// File MD5 calculated from item content.
    File,
}

/// Canonical partial MD5 resolution result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalPartialMd5 {
    pub value: String,
    pub source: CanonicalPartialMd5Source,
}

/// Normalize and validate a partial MD5 candidate.
///
/// Valid values are exactly 32 hex characters. Input is trimmed and lowercased.
pub(crate) fn normalize_partial_md5(candidate: &str) -> Option<String> {
    let normalized = candidate.trim().to_lowercase();
    if normalized.len() == 32 && normalized.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(normalized)
    } else {
        None
    }
}

/// Resolve canonical item identity MD5 using runtime precedence:
/// metadata MD5 -> file MD5.
pub(crate) fn resolve_canonical_partial_md5(
    metadata_partial_md5: Option<&str>,
    file_partial_md5: Option<&str>,
) -> Option<CanonicalPartialMd5> {
    if let Some(value) = metadata_partial_md5.and_then(normalize_partial_md5) {
        return Some(CanonicalPartialMd5 {
            value,
            source: CanonicalPartialMd5Source::Metadata,
        });
    }

    file_partial_md5
        .and_then(normalize_partial_md5)
        .map(|value| CanonicalPartialMd5 {
            value,
            source: CanonicalPartialMd5Source::File,
        })
}

/// Resolve language using runtime precedence: parser language, then KOReader `text_lang`.
pub(crate) fn resolve_language<'a>(
    parser_language: Option<&'a String>,
    metadata_text_lang: Option<&'a String>,
) -> Option<&'a String> {
    parser_language.or(metadata_text_lang)
}

/// Resolve page total using runtime precedence:
/// stable page labels (optional) -> KOReader rendered pages -> parser pages.
pub(crate) fn resolve_page_total(
    use_stable_page_metadata: bool,
    stable_page_total: Option<u32>,
    rendered_page_total: Option<u32>,
    parser_page_total: Option<u32>,
) -> Option<u32> {
    let stable_pages = if use_stable_page_metadata {
        stable_page_total.filter(|pages| *pages > 0)
    } else {
        None
    };

    stable_pages
        .or_else(|| rendered_page_total.filter(|pages| *pages > 0))
        .or(parser_page_total)
}

#[cfg(test)]
mod tests {
    use super::{
        CanonicalPartialMd5Source, normalize_partial_md5, resolve_canonical_partial_md5,
        resolve_language, resolve_page_total,
    };

    const VALID_MD5_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const VALID_MD5_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const VALID_MD5_C: &str = "cccccccccccccccccccccccccccccccc";

    #[test]
    fn normalize_partial_md5_accepts_trimmed_mixed_case_hex() {
        let normalized = normalize_partial_md5("  AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n")
            .expect("md5 should normalize");

        assert_eq!(normalized, VALID_MD5_A);
    }

    #[test]
    fn normalize_partial_md5_rejects_non_hex_and_wrong_length_values() {
        assert_eq!(normalize_partial_md5("abc"), None);
        assert_eq!(
            normalize_partial_md5("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"),
            None
        );
    }

    #[test]
    fn canonical_md5_resolution_prefers_metadata_then_file_hashes() {
        let metadata = resolve_canonical_partial_md5(Some(VALID_MD5_A), Some(VALID_MD5_B))
            .expect("metadata md5 should resolve");
        assert_eq!(metadata.value, VALID_MD5_A);
        assert_eq!(metadata.source, CanonicalPartialMd5Source::Metadata);

        let file = resolve_canonical_partial_md5(Some("invalid"), Some(VALID_MD5_C))
            .expect("file md5 should resolve");
        assert_eq!(file.value, VALID_MD5_C);
        assert_eq!(file.source, CanonicalPartialMd5Source::File);
    }

    #[test]
    fn canonical_md5_resolution_returns_none_when_all_candidates_are_missing_or_invalid() {
        assert_eq!(resolve_canonical_partial_md5(None, None), None);
        assert_eq!(
            resolve_canonical_partial_md5(Some("invalid"), Some("invalid")),
            None
        );
    }

    #[test]
    fn resolve_language_prefers_parser_value_before_koreader_text_lang() {
        let parser_language = "de-DE".to_string();
        let metadata_text_lang = "en-US".to_string();

        let language = resolve_language(Some(&parser_language), Some(&metadata_text_lang));

        assert_eq!(language, Some(&parser_language));
    }

    #[test]
    fn resolve_page_total_uses_stable_then_rendered_then_parser_fallback_order() {
        assert_eq!(
            resolve_page_total(true, Some(300), Some(250), Some(200)),
            Some(300)
        );
        assert_eq!(
            resolve_page_total(false, Some(300), Some(250), Some(200)),
            Some(250)
        );
        assert_eq!(resolve_page_total(false, None, None, Some(200)), Some(200));
    }
}
