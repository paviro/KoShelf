//! Source adapter boundaries for scanner, parser, and KOReader integrations.

pub mod fingerprints;

pub use fingerprints::{
    FileFingerprint, FingerprintChange, ItemFingerprints, ReconcileAction, ReparseScope,
    classify_reconcile_action,
};
