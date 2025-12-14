//! Format-specific parsers for extracting metadata from ebooks and comics.

pub mod comic;
pub mod epub;
pub mod fb2;
pub mod mobi;

pub use comic::ComicParser;
pub use epub::EpubParser;
pub use fb2::Fb2Parser;
pub use mobi::MobiParser;
