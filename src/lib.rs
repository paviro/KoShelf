//! KoShelf library crate.
//!
//! This crate backs the `koshelf` binary. Keeping most logic in `lib.rs` makes the
//! codebase easier to test and refactor while keeping `src/main.rs` minimal.

pub mod app;
pub mod cli;
pub mod config;
pub mod contracts;
pub mod domain;
pub mod i18n;
pub mod infra;
pub mod koreader;
pub mod models;
pub mod parsers;
pub mod runtime;
pub mod server;
pub mod share;
pub mod time_config;
pub mod utils;

pub use app::run;
pub use cli::Cli;

#[cfg(test)]
mod tests;
