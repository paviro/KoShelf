//! Web server and version checking.

pub mod version;
pub mod web;

pub use version::create_version_notifier;
pub use web::WebServer;
