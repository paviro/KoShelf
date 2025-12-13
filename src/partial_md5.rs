//! Partial MD5 hash calculation compatible with KOReader's `util.partialMD5()` function.
//!
//! KOReader uses a non-uniform sampling algorithm that weights file content
//! more heavily at the beginning and less at the end. This helps maintain
//! consistent hashes even when PDFs are modified by appending highlights.

use anyhow::Result;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Calculate the partial MD5 hash of a file using KOReader's algorithm.
///
/// The algorithm samples 1024 bytes from positions determined by the formula:
/// `position = lshift(1024, 2*i)` where i ranges from -1 to 10.
///
/// In LuaJIT's bit library, lshift uses 32-bit arithmetic and the shift amount
/// is masked to 5 bits (mod 32). This means:
/// - For i = -1: lshift(1024, -2) = lshift(1024, 30) = 0 (overflow in 32-bit)
/// - For i = 0: lshift(1024, 0) = 1024
/// - For i = 1: lshift(1024, 2) = 4096
/// - For i = 2: lshift(1024, 4) = 16384
/// - etc.
pub fn calculate_partial_md5(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = md5::Context::new();

    let step: u32 = 1024;
    let size: usize = 1024;
    let mut buffer = vec![0u8; size];

    // i ranges from -1 to 10 in KOReader's algorithm
    // We need to emulate LuaJIT's 32-bit lshift behavior where the shift amount
    // is masked to 5 bits (mod 32)
    for i in -1i32..=10 {
        // Calculate shift amount using Lua's bit library semantics
        // In LuaJIT: shift_amount = (2 * i) mod 32
        let shift_amount = ((2 * i) as u32).wrapping_add(0) & 31;

        // Calculate position using 32-bit arithmetic (like LuaJIT)
        let position = if shift_amount >= 32 {
            0u64
        } else {
            (step.wrapping_shl(shift_amount)) as u64
        };

        // Seek to position
        if file.seek(SeekFrom::Start(position)).is_err() {
            break;
        }

        // Read sample
        match file.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(bytes_read) => {
                hasher.consume(&buffer[..bytes_read]);
            }
            Err(_) => break,
        }
    }

    let digest = hasher.finalize();
    Ok(format!("{:x}", digest))
}
