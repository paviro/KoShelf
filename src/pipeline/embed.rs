use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::io::Read;

/// Whether the given file extension was gzip-compressed at build time.
pub fn is_precompressed(path: &str) -> bool {
    matches!(
        path.rsplit('.').next(),
        Some("js" | "mjs" | "css" | "html" | "json" | "map" | "svg")
    )
}

/// Decompress a gzip-compressed embedded asset.
pub fn gz_decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut buf = Vec::new();
    decoder
        .read_to_end(&mut buf)
        .context("Failed to decompress embedded asset")?;
    Ok(buf)
}
