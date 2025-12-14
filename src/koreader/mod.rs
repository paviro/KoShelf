//! KoReader integration: metadata parsing, statistics database, and reading analytics.

pub mod calendar;
pub mod completion;
pub mod database;
pub mod lua;
pub mod partial_md5;
pub mod session;
pub mod statistics;

pub use calendar::CalendarGenerator;
pub use database::StatisticsParser;
pub use lua::LuaParser;
pub use partial_md5::calculate_partial_md5;
pub use statistics::{BookStatistics, StatisticsCalculator};
