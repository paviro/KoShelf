//! KoShelf library crate.
//!
//! This crate backs the `koshelf` binary. Keeping most logic in `lib.rs` makes the
//! codebase easier to test and refactor while keeping `src/main.rs` minimal.

pub mod app;
pub mod i18n;
pub mod pipeline;
pub mod server;
pub mod shelf;
pub mod source;
pub mod store;

pub use app::Cli;
pub use app::run;

#[cfg(test)]
mod tests;
