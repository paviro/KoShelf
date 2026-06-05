//! External data sources: KOReader integration, book format parsers, and filesystem scanning.

pub mod fingerprints;
pub mod kobo;
pub mod koreader;
pub mod parsers;
pub mod scanner;
pub(crate) mod sqlite_snapshot;

pub use fingerprints::{
    FileFingerprint, ItemFingerprints, ReconcileAction, classify_reconcile_action,
};
