use crate::server::api::responses::common::ContentTypeFilter;
use crate::shelf::token_set::{self, SetToken};

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

query_enum_bare! {
    pub enum ItemSort {
        Title => "title",
        Author => "author",
        Status => "status",
        Progress => "progress",
        Rating => "rating",
        Annotations => "annotations",
        LastOpenAt => "last_open_at",
    }
}

impl ItemSort {
    pub fn sql_column(self) -> &'static str {
        match self {
            Self::Title => "LOWER(title)",
            Self::Author => "LOWER(JSON_EXTRACT(authors_json, '$[0]'))",
            Self::Status => "status",
            Self::Progress => "progress_percentage",
            Self::Rating => "rating",
            Self::Annotations => "annotation_count",
            Self::LastOpenAt => "last_open_at",
        }
    }

    pub fn default_order(self) -> SortOrder {
        match self {
            Self::Title | Self::Author | Self::Status => SortOrder::Asc,
            Self::Progress | Self::Rating | Self::Annotations | Self::LastOpenAt => SortOrder::Desc,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

query_enum_bare! {
    pub enum SortOrder {
        Asc => "asc",
        Desc => "desc",
    }
}

impl SortOrder {
    pub fn sql_keyword(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
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
    ReaderPresentation,
    Chapters,
}

impl IncludeToken {
    pub const ALL: &[IncludeToken] = &[
        IncludeToken::Highlights,
        IncludeToken::Bookmarks,
        IncludeToken::Statistics,
        IncludeToken::Completions,
        IncludeToken::ReaderPresentation,
        IncludeToken::Chapters,
    ];
}

impl SetToken for IncludeToken {
    fn parse_token(value: &str) -> Option<Self> {
        match value {
            "highlights" => Some(Self::Highlights),
            "bookmarks" => Some(Self::Bookmarks),
            "statistics" => Some(Self::Statistics),
            "completions" => Some(Self::Completions),
            "reader_presentation" => Some(Self::ReaderPresentation),
            "chapters" => Some(Self::Chapters),
            _ => None,
        }
    }

    fn valid_tokens() -> &'static str {
        "highlights, bookmarks, statistics, completions, reader_presentation, chapters, all"
    }

    fn all_variants() -> Option<&'static [Self]> {
        Some(Self::ALL)
    }
}

pub type IncludeSet = token_set::TokenSet<IncludeToken>;

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
    use crate::server::api::responses::error::ApiErrorCode;

    #[test]
    fn include_set_none_returns_empty() {
        let set = IncludeSet::parse(None).unwrap();
        assert!(!set.has(IncludeToken::Highlights));
        assert!(!set.has(IncludeToken::Bookmarks));
        assert!(!set.has(IncludeToken::Statistics));
        assert!(!set.has(IncludeToken::Completions));
        assert!(!set.has(IncludeToken::ReaderPresentation));
        assert!(!set.has(IncludeToken::Chapters));
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
        assert!(set.has(IncludeToken::ReaderPresentation));
        assert!(set.has(IncludeToken::Chapters));
    }

    #[test]
    fn include_set_all_alone() {
        let set = IncludeSet::parse(Some("all")).unwrap();
        assert!(set.has(IncludeToken::Highlights));
        assert!(set.has(IncludeToken::Bookmarks));
        assert!(set.has(IncludeToken::Statistics));
        assert!(set.has(IncludeToken::Completions));
        assert!(set.has(IncludeToken::ReaderPresentation));
        assert!(set.has(IncludeToken::Chapters));
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
