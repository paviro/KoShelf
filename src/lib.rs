//! KoShelf library crate.
//!
//! This crate backs the `koshelf` binary. Keeping most logic in `lib.rs` makes the
//! codebase easier to test and refactor while keeping `src/main.rs` minimal.

pub mod app;
pub mod cli;
pub mod config;
pub mod i18n;
pub mod koreader;
pub mod library;
pub mod models;
pub mod parsers;
pub mod server;
pub mod share;
pub mod site_generator;
pub mod templates;
pub mod time_config;
pub mod utils;

pub use app::run;
pub use cli::Cli;

#[cfg(test)]
mod tests;

