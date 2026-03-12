use std::collections::HashSet;

use crate::contracts::common::ContentTypeFilter;
use crate::contracts::error::ApiErrorCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemSort {
    #[default]
    Title,
    Author,
    Status,
    Progress,
    Rating,
    Annotations,
    LastOpenAt,
}

impl ItemSort {
    pub fn parse(value: &str) -> Result<Self, ApiErrorCode> {
        match value {
            "title" => Ok(Self::Title),
            "author" => Ok(Self::Author),
            "status" => Ok(Self::Status),
            "progress" => Ok(Self::Progress),
            "rating" => Ok(Self::Rating),
            "annotations" => Ok(Self::Annotations),
            "last_open_at" => Ok(Self::LastOpenAt),
            _ => Err(ApiErrorCode::InvalidQuery),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn parse(value: &str) -> Result<Self, ApiErrorCode> {
        match value {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err(ApiErrorCode::InvalidQuery),
        }
    }
}

// ── Include tokens for detail endpoint ────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IncludeToken {
    Highlights,
    Bookmarks,
    Statistics,
    Completions,
}

impl IncludeToken {
    pub const ALL: &[IncludeToken] = &[
        IncludeToken::Highlights,
        IncludeToken::Bookmarks,
        IncludeToken::Statistics,
        IncludeToken::Completions,
    ];

    fn parse(value: &str) -> Option<Self> {
        match value {
            "highlights" => Some(Self::Highlights),
            "bookmarks" => Some(Self::Bookmarks),
            "statistics" => Some(Self::Statistics),
            "completions" => Some(Self::Completions),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct IncludeSet {
    tokens: HashSet<IncludeToken>,
}

impl IncludeSet {
    /// Parse a comma-separated include string with strict validation.
    ///
    /// Rules per API contract:
    /// - comma-separated lowercase tokens
    /// - duplicates ignored
    /// - `all` supersedes any other token
    /// - unknown token returns `Err` with a descriptive message
    pub fn parse(value: Option<&str>) -> Result<Self, (ApiErrorCode, String)> {
        let Some(value) = value else {
            return Ok(Self::default());
        };

        let value = value.trim();
        if value.is_empty() {
            return Ok(Self::default());
        }

        let mut tokens = HashSet::new();

        for raw in value.split(',') {
            let token = raw.trim();
            if token.is_empty() {
                continue;
            }
            if token == "all" {
                return Ok(Self::all());
            }
            match IncludeToken::parse(token) {
                Some(t) => {
                    tokens.insert(t);
                }
                None => {
                    return Err((
                        ApiErrorCode::InvalidQuery,
                        format!(
                            "unknown include token '{token}'; \
                             valid tokens are: highlights, bookmarks, statistics, completions, all"
                        ),
                    ));
                }
            }
        }

        Ok(Self { tokens })
    }

    pub fn all() -> Self {
        Self {
            tokens: IncludeToken::ALL.iter().copied().collect(),
        }
    }

    pub fn has(&self, token: IncludeToken) -> bool {
        self.tokens.contains(&token)
    }
}

// ── Query types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LibraryListQuery {
    pub scope: ContentTypeFilter,
    pub sort: ItemSort,
    pub order: Option<SortOrder>,
}

#[derive(Debug, Clone)]
pub struct LibraryDetailQuery {
    pub id: String,
    pub includes: IncludeSet,
}

impl LibraryDetailQuery {
    pub fn new(id: impl Into<String>, includes: IncludeSet) -> Self {
        Self {
            id: id.into(),
            includes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn include_set_none_returns_empty() {
        let set = IncludeSet::parse(None).unwrap();
        assert!(!set.has(IncludeToken::Highlights));
        assert!(!set.has(IncludeToken::Bookmarks));
        assert!(!set.has(IncludeToken::Statistics));
        assert!(!set.has(IncludeToken::Completions));
    }

    #[test]
    fn include_set_empty_string_returns_empty() {
        let set = IncludeSet::parse(Some("")).unwrap();
        assert!(!set.has(IncludeToken::Highlights));
    }

    #[test]
    fn include_set_single_token() {
        let set = IncludeSet::parse(Some("highlights")).unwrap();
        assert!(set.has(IncludeToken::Highlights));
        assert!(!set.has(IncludeToken::Bookmarks));
    }

    #[test]
    fn include_set_multiple_tokens() {
        let set = IncludeSet::parse(Some("highlights,bookmarks")).unwrap();
        assert!(set.has(IncludeToken::Highlights));
        assert!(set.has(IncludeToken::Bookmarks));
        assert!(!set.has(IncludeToken::Statistics));
    }

    #[test]
    fn include_set_all_supersedes_individual_tokens() {
        let set = IncludeSet::parse(Some("highlights,all,bookmarks")).unwrap();
        assert!(set.has(IncludeToken::Highlights));
        assert!(set.has(IncludeToken::Bookmarks));
        assert!(set.has(IncludeToken::Statistics));
        assert!(set.has(IncludeToken::Completions));
    }

    #[test]
    fn include_set_all_alone() {
        let set = IncludeSet::parse(Some("all")).unwrap();
        assert!(set.has(IncludeToken::Highlights));
        assert!(set.has(IncludeToken::Bookmarks));
        assert!(set.has(IncludeToken::Statistics));
        assert!(set.has(IncludeToken::Completions));
    }

    #[test]
    fn include_set_duplicates_ignored() {
        let set = IncludeSet::parse(Some("highlights,highlights")).unwrap();
        assert!(set.has(IncludeToken::Highlights));
    }

    #[test]
    fn include_set_unknown_token_returns_error() {
        let err = IncludeSet::parse(Some("highlights,unknown")).unwrap_err();
        assert_eq!(err.0, ApiErrorCode::InvalidQuery);
        assert!(err.1.contains("unknown"));
    }

    #[test]
    fn include_set_whitespace_trimmed() {
        let set = IncludeSet::parse(Some(" highlights , bookmarks ")).unwrap();
        assert!(set.has(IncludeToken::Highlights));
        assert!(set.has(IncludeToken::Bookmarks));
    }

    #[test]
    fn include_set_trailing_comma_tolerated() {
        let set = IncludeSet::parse(Some("highlights,")).unwrap();
        assert!(set.has(IncludeToken::Highlights));
    }
}
