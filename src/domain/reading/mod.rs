//! Reading-domain boundaries for statistics, calendar, and completions data.

pub mod available_periods;
pub mod calendar;
pub mod completions;
pub mod metrics;
pub mod queries;
pub mod scaling;
pub mod sessions;
pub mod shared;
pub mod statistics;
pub mod summary;
pub mod types;

pub use available_periods::available_periods;
pub use calendar::reading_calendar as calendar;
pub use completions::reading_completions as completions;
pub use metrics::metrics;
pub use scaling::PageScaling;
pub use statistics::{BookStatistics, StatisticsCalculator};
pub use summary::summary;
