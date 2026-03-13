//! Reading-domain boundaries for statistics, calendar, and completions data.

pub mod available_periods;
pub mod calendar;
pub mod completions;
pub mod metrics;
pub mod queries;
pub mod service;
pub mod shared;
pub mod summary;

pub use service::ReadingService;
