//! KoShelf library crate.
//!
//! This crate backs the `koshelf` binary. Keeping most logic in `lib.rs` makes the
//! codebase easier to test and refactor while keeping `src/main.rs` minimal.

pub mod app;
pub mod contracts;
pub mod domain;
pub mod i18n;
pub mod infra;
pub mod models;
pub mod runtime;
pub mod server;
pub mod share;
pub mod source;
pub mod store;
pub mod time_config;
pub mod utils;

pub use app::Cli;
pub use app::run;

#[cfg(test)]
mod tests;
