//! Reading-domain boundaries for statistics, calendar, and completions data.

pub mod calendar;
pub mod completions;
pub mod metrics;
pub mod queries;
pub mod service;

pub use service::ReadingService;
