//! HTTP API module.

pub mod error;
pub mod extractors;
pub mod handlers;
pub mod params;
pub mod responses;
pub mod server;

pub use server::WebServer;
