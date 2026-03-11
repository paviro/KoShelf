use crate::contracts::common::ContentTypeFilter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LibraryListQuery {
    pub scope: ContentTypeFilter,
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
