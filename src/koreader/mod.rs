//! KoReader integration: metadata parsing and statistics database access.

pub mod database;
pub mod lua;
pub(crate) mod merge_precedence;
pub mod partial_md5;
pub mod types;

pub use database::StatisticsParser;
pub use lua::LuaParser;
pub use partial_md5::calculate_partial_md5;
pub use types::*;

// Re-exported from canonical home in `domain::reading`.
pub use crate::domain::reading::{BookStatistics, CalendarGenerator, StatisticsCalculator};
