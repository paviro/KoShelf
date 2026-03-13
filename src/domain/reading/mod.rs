//! Reading-domain boundaries for statistics, calendar, and completions data.

pub mod available_periods;
pub mod calendar;
pub mod completion_calc;
pub mod completions;
pub mod metrics;
pub mod queries;
pub mod session_calc;
pub mod shared;
pub mod statistics_calc;
pub mod summary;
pub mod types;

pub use available_periods::available_periods;
pub use calendar::reading_calendar as calendar;
pub use completions::reading_completions as completions;
pub use metrics::metrics;
pub use statistics_calc::{BookStatistics, StatisticsCalculator};
pub use summary::summary;
