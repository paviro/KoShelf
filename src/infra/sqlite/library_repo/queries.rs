//! Query parameter types for library list operations.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LibrarySort {
    #[default]
    Title,
    Author,
    Status,
    Progress,
    Rating,
    Annotations,
    LastOpenAt,
}

impl LibrarySort {
    pub(super) fn sql_column(self) -> &'static str {
        match self {
            Self::Title => "title_sort",
            Self::Author => "primary_author_sort",
            Self::Status => "status",
            Self::Progress => "progress_percentage",
            Self::Rating => "rating",
            Self::Annotations => "annotation_count",
            Self::LastOpenAt => "last_open_at",
        }
    }

    pub(super) fn default_direction(self) -> SortDirection {
        match self {
            Self::Title | Self::Author | Self::Status => SortDirection::Asc,
            Self::Progress | Self::Rating | Self::Annotations | Self::LastOpenAt => {
                SortDirection::Desc
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub(super) fn sql_keyword(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LibraryListFilter {
    /// `"book"` or `"comic"`. `None` returns all content types.
    pub content_type: Option<String>,
    pub sort: LibrarySort,
    /// When `None`, uses the sort column's natural default direction.
    pub sort_direction: Option<SortDirection>,
}
