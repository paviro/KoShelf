//! Reading statistics: calendar, completions, metrics, summary, and available periods.

pub mod available_periods;
pub mod calendar;
pub mod completions;
pub mod compute;
pub mod metrics;
pub mod queries;
pub mod shared;
pub mod summary;
pub mod types;

pub use available_periods::available_periods;
pub use calendar::reading_calendar as calendar;
pub use completions::reading_completions as completions;
pub use compute::calculator::{BookStatistics, StatisticsCalculator};
pub use compute::scaling::PageScaling;
pub use metrics::metrics;
pub use summary::summary;
