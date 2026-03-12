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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LibraryListQuery {
    pub scope: ContentTypeFilter,
    pub sort: ItemSort,
    pub order: Option<SortOrder>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryDetailQuery {
    pub id: String,
}

impl LibraryDetailQuery {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}
