use crate::models::{BookInfo, Identifier};
use crate::utils::sanitize_html;
use anyhow::{Context, Result, anyhow};
use log::{debug, warn};
use std::path::{Path, PathBuf};

pub struct MobiParser;

impl Default for MobiParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MobiParser {
    pub fn new() -> Self {
        Self
    }

    pub async fn parse(&self, mobi_path: &Path) -> Result<BookInfo> {
        let path = mobi_path.to_path_buf();
        tokio::task::spawn_blocking(move || Self::parse_sync(&path))
            .await
            .with_context(|| "Task join error")?
    }

    fn parse_sync(mobi_path: &PathBuf) -> Result<BookInfo> {
        debug!("Opening MOBI: {:?}", mobi_path);
        let data = std::fs::read(mobi_path)
            .with_context(|| format!("Failed to read MOBI file: {:?}", mobi_path))?;

        // Default fallback: filename stem.
        let fallback_title = mobi_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        // Parse Palm Database (PDB) header to get record 0.
        let record0 = match Self::extract_pdb_record0(&data) {
            Ok(r0) => r0,
            Err(e) => {
                warn!(
                    "Failed to parse PDB/MOBI structure for {:?}: {}",
                    mobi_path, e
                );
                return Ok(Self::book_info_from_filename(&fallback_title));
            }
        };

        let mobi_start = Self::find_bytes(&record0, b"MOBI")
            .ok_or_else(|| anyhow!("MOBI header not found in record 0"))?;

        // MOBI header length at +4
        let mobi_header_len = Self::read_u32_be(&record0, mobi_start + 4)
            .context("Failed to read MOBI header length")? as usize;
        let mobi_end = mobi_start
            .checked_add(mobi_header_len)
            .ok_or_else(|| anyhow!("Invalid MOBI header length (overflow)"))?;
        if mobi_end > record0.len() {
            return Err(anyhow!("MOBI header extends beyond record 0"));
        }

        // Text encoding at +12 (common: 65001 = UTF-8, 1252 = CP1252)
        let _text_encoding = Self::read_u32_be(&record0, mobi_start + 12).unwrap_or(0);

        // First image record index (PDB record number). Common MOBI field; used as a fallback.
        // Offset is relative to the MOBI header start.
        let first_image_index = Self::read_u32_be(&record0, mobi_start + 0x6C)
            .map(|v| v as usize)
            .unwrap_or(0);

        // "Full name" (often the title) is stored as an offset/length from MOBI header start.
        let title_from_full_name = Self::extract_full_name_title(&record0, mobi_start);

        // Parse EXTH (if present) for richer metadata (author/title/subjects/etc.)
        let exth = Self::parse_exth(&record0, mobi_end);

        let mut title: Option<String> = None;
        let mut authors: Vec<String> = Vec::new();
        let mut description: Option<String> = None;
        let mut language: Option<String> = None;
        let mut publisher: Option<String> = None;
        let mut subjects: Vec<String> = Vec::new();
        let mut identifiers: Vec<Identifier> = Vec::new();
        let mut cover_data: Option<Vec<u8>> = None;
        let mut cover_mime_type: Option<String> = None;

        // Precompute PDB record ranges for cover extraction.
        let record_ranges = Self::pdb_record_ranges(&data).unwrap_or_default();

        if let Some(exth) = exth {
            // Title: prefer EXTH 503 ("Updated Title") if present, else fall back to MOBI full name.
            if let Some(t) = exth.get_string(503) {
                title = Some(t);
            }

            // Author(s): EXTH 100
            if let Some(author_raw) = exth.get_string(100) {
                authors.extend(Self::split_people_list(&author_raw));
            }

            // Publisher: EXTH 101
            if let Some(p) = exth.get_string(101) {
                publisher = Some(p);
            }

            // Description: EXTH 103
            if let Some(d) = exth.get_string(103) {
                let cleaned = sanitize_html(&d);
                let trimmed = cleaned.trim();
                if !trimmed.is_empty() {
                    description = Some(trimmed.to_string());
                }
            }

            // Subjects: EXTH 105 (may appear multiple times in some files)
            for s in exth.get_strings(105) {
                subjects.extend(Self::split_subject_list(&s));
            }

            // ISBN: commonly stored as EXTH 104 by some producers (e.g., Calibre).
            if let Some(isbn_raw) = exth.get_string(104) {
                let isbn = Self::normalize_isbn_like(&isbn_raw);
                if !isbn.is_empty() {
                    identifiers.push(Identifier::new("isbn".to_string(), isbn));
                }
            }

            // Language: EXTH 524 (often values like "en", "en-US", "de", etc.)
            if language.is_none()
                && let Some(lang_raw) = exth.get_string(524) {
                    let lang = Self::normalize_language_tag(&lang_raw);
                    if !lang.is_empty() {
                        language = Some(lang);
                    }
                }

            // ASIN: commonly seen as EXTH 113 or 504 depending on producer/tooling.
            if let Some(asin) = exth.get_string(113).or_else(|| exth.get_string(504)) {
                let asin_trim = asin.trim().to_string();
                if !asin_trim.is_empty() {
                    identifiers.push(Identifier::new("mobi-asin".to_string(), asin_trim));
                }
            }

            // Cover extraction: EXTH 201 is commonly the cover image record index/offset.
            // Different producers interpret this slightly differently, so we try a couple
            // candidate mappings and validate by image magic bytes.
            if cover_data.is_none() && !record_ranges.is_empty()
                && let Some(cover_rec) = exth.get_u32(201).map(|v| v as usize) {
                    let mut candidates: Vec<usize> = Vec::new();
                    candidates.push(cover_rec);
                    candidates.push(cover_rec.saturating_add(1));
                    if first_image_index > 0 {
                        candidates.push(first_image_index.saturating_add(cover_rec));
                        candidates.push(
                            first_image_index
                                .saturating_add(cover_rec)
                                .saturating_add(1),
                        );
                    }
                    candidates.sort();
                    candidates.dedup();

                    for idx in candidates {
                        if let Some((bytes, mime)) =
                            Self::extract_image_record(&data, &record_ranges, idx)
                        {
                            cover_data = Some(bytes);
                            cover_mime_type = Some(mime);
                            break;
                        }
                    }
                }
        }

        // If we didn't get a cover via EXTH 201, do a best-effort fallback:
        // scan from first_image_index for the first record that looks like an image.
        if cover_data.is_none() && !record_ranges.is_empty() {
            let start = first_image_index.min(record_ranges.len().saturating_sub(1));
            for idx in start..record_ranges.len() {
                if let Some((bytes, mime)) = Self::extract_image_record(&data, &record_ranges, idx)
                {
                    cover_data = Some(bytes);
                    cover_mime_type = Some(mime);
                    break;
                }
            }
        }

        // Fallback title sources
        let final_title = title
            .or(title_from_full_name)
            .unwrap_or_else(|| fallback_title.clone());

        if authors.is_empty() {
            // Some MOBI producers store author as EXTH 100 only; if missing, keep empty.
        }

        // De-dupe subjects/authors
        authors = Self::dedupe_preserve_order(authors);
        subjects = Self::dedupe_preserve_order(subjects);

        Ok(BookInfo {
            title: final_title,
            authors,
            description,
            language,
            publisher,
            identifiers,
            subjects,
            series: None,
            series_number: None,
            pages: None,
            cover_data,
            cover_mime_type: cover_mime_type.map(|m| m.to_string()),
        })
    }

    fn book_info_from_filename(title: &str) -> BookInfo {
        BookInfo {
            title: title.to_string(),
            authors: Vec::new(),
            description: None,
            language: None,
            publisher: None,
            identifiers: Vec::new(),
            subjects: Vec::new(),
            series: None,
            series_number: None,
            pages: None,
            cover_data: None,
            cover_mime_type: None,
        }
    }

    fn extract_pdb_record0(data: &[u8]) -> Result<Vec<u8>> {
        // PDB header is 78 bytes minimum.
        if data.len() < 78 {
            return Err(anyhow!("File too small to be a PDB/MOBI"));
        }
        let record_count =
            Self::read_u16_be(data, 76).context("Failed to read PDB record count")? as usize;
        if record_count == 0 {
            return Err(anyhow!("PDB record count is zero"));
        }
        let record_list_start = 78;
        let record_list_len = record_count
            .checked_mul(8)
            .ok_or_else(|| anyhow!("Record list length overflow"))?;
        if data.len() < record_list_start + record_list_len {
            return Err(anyhow!("File too small for PDB record list"));
        }

        // Read first two record offsets to bound record 0.
        let off0 = Self::read_u32_be(data, record_list_start)
            .context("Failed to read record 0 offset")? as usize;
        let off1 = if record_count >= 2 {
            Self::read_u32_be(data, record_list_start + 8)
                .context("Failed to read record 1 offset")? as usize
        } else {
            data.len()
        };

        if off0 >= data.len() || off1 > data.len() || off1 <= off0 {
            return Err(anyhow!(
                "Invalid PDB record offsets (off0={}, off1={}, len={})",
                off0,
                off1,
                data.len()
            ));
        }
        Ok(data[off0..off1].to_vec())
    }

    /// Compute record byte ranges for all PDB records.
    /// Returns a vector of (start, end) slices into the file buffer.
    fn pdb_record_ranges(data: &[u8]) -> Result<Vec<(usize, usize)>> {
        if data.len() < 78 {
            return Err(anyhow!("File too small to be a PDB/MOBI"));
        }
        let record_count =
            Self::read_u16_be(data, 76).context("Failed to read PDB record count")? as usize;
        if record_count == 0 {
            return Err(anyhow!("PDB record count is zero"));
        }
        let record_list_start = 78;
        let record_list_len = record_count
            .checked_mul(8)
            .ok_or_else(|| anyhow!("Record list length overflow"))?;
        if data.len() < record_list_start + record_list_len {
            return Err(anyhow!("File too small for PDB record list"));
        }

        let mut offsets: Vec<usize> = Vec::with_capacity(record_count);
        for i in 0..record_count {
            let off = Self::read_u32_be(data, record_list_start + (i * 8))
                .with_context(|| format!("Failed to read record {} offset", i))?
                as usize;
            offsets.push(off);
        }
        // Add end sentinel
        offsets.push(data.len());

        let mut ranges: Vec<(usize, usize)> = Vec::with_capacity(record_count);
        for i in 0..record_count {
            let start = offsets[i];
            let end = offsets[i + 1];
            if start >= data.len() || end > data.len() || end <= start {
                return Err(anyhow!(
                    "Invalid PDB record offsets at {} (start={}, end={}, len={})",
                    i,
                    start,
                    end,
                    data.len()
                ));
            }
            ranges.push((start, end));
        }
        Ok(ranges)
    }

    fn extract_image_record(
        data: &[u8],
        ranges: &[(usize, usize)],
        record_index: usize,
    ) -> Option<(Vec<u8>, String)> {
        let (start, end) = *ranges.get(record_index)?;
        let bytes = &data[start..end];
        let mime = Self::guess_image_mime(bytes)?;
        Some((bytes.to_vec(), mime))
    }

    fn guess_image_mime(bytes: &[u8]) -> Option<String> {
        if bytes.len() < 12 {
            return None;
        }
        // JPEG
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some("image/jpeg".to_string());
        }
        // PNG
        if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Some("image/png".to_string());
        }
        // GIF
        if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
            return Some("image/gif".to_string());
        }
        // WEBP (RIFF....WEBP)
        if bytes.starts_with(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WEBP" {
            return Some("image/webp".to_string());
        }
        None
    }

    fn extract_full_name_title(record0: &[u8], mobi_start: usize) -> Option<String> {
        // Offsets relative to MOBI header start (common MOBI layout).
        let off = Self::read_u32_be(record0, mobi_start + 0x54).ok()? as usize;
        let len = Self::read_u32_be(record0, mobi_start + 0x58).ok()? as usize;
        let start = mobi_start.checked_add(off)?;
        let end = start.checked_add(len)?;
        if end > record0.len() || start >= end {
            return None;
        }
        let raw = &record0[start..end];
        let s = Self::decode_text(raw);
        let trimmed = s.trim_matches('\0').trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    }

    fn parse_exth(record0: &[u8], search_from: usize) -> Option<ExthBlock> {
        if search_from >= record0.len() {
            return None;
        }
        let exth_pos_rel = Self::find_bytes(&record0[search_from..], b"EXTH")?;
        let exth_pos = search_from + exth_pos_rel;

        let total_len = Self::read_u32_be(record0, exth_pos + 4).ok()? as usize;
        let record_count = Self::read_u32_be(record0, exth_pos + 8).ok()? as usize;

        let exth_end = exth_pos.checked_add(total_len)?;
        if total_len < 12 || exth_end > record0.len() {
            return None;
        }

        let mut records: Vec<(u32, Vec<u8>)> = Vec::new();
        let mut cursor = exth_pos + 12;
        for _ in 0..record_count {
            if cursor + 8 > exth_end {
                break;
            }
            let rtype = Self::read_u32_be(record0, cursor).ok()?;
            let rlen = Self::read_u32_be(record0, cursor + 4).ok()? as usize;
            if rlen < 8 {
                break;
            }
            let r_end = cursor.checked_add(rlen)?;
            if r_end > exth_end {
                break;
            }
            let value = record0[cursor + 8..r_end].to_vec();
            records.push((rtype, value));
            cursor = r_end;
        }

        Some(ExthBlock { records })
    }

    fn split_people_list(raw: &str) -> Vec<String> {
        let s = raw.trim();
        if s.is_empty() {
            return vec![];
        }
        // Prefer semicolon/newline separation (comma is ambiguous for "Last, First").
        let mut out: Vec<String> = Vec::new();
        for part in s.split(';').flat_map(|p| p.split('\n')).map(|p| p.trim()) {
            if !part.is_empty() {
                out.push(part.to_string());
            }
        }
        if out.is_empty() {
            vec![s.to_string()]
        } else {
            out
        }
    }

    fn split_subject_list(raw: &str) -> Vec<String> {
        let s = raw.trim();
        if s.is_empty() {
            return vec![];
        }
        // Subjects are often semicolon-separated; commas are sometimes used too.
        let mut out: Vec<String> = Vec::new();
        for part in s.split(';').flat_map(|p| p.split(',')).map(|p| p.trim()) {
            if !part.is_empty() {
                out.push(part.to_string());
            }
        }
        if out.is_empty() {
            vec![s.to_string()]
        } else {
            out
        }
    }

    fn dedupe_preserve_order(items: Vec<String>) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for s in items {
            let key = s.to_lowercase();
            if seen.insert(key) {
                out.push(s);
            }
        }
        out
    }

    fn decode_text(bytes: &[u8]) -> String {
        // Most modern MOBI metadata is UTF-8; fall back to lossy UTF-8.
        String::from_utf8_lossy(bytes).into_owned()
    }

    fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.is_empty() || haystack.len() < needle.len() {
            return None;
        }
        haystack.windows(needle.len()).position(|w| w == needle)
    }

    fn read_u16_be(buf: &[u8], offset: usize) -> Result<u16> {
        if offset + 2 > buf.len() {
            return Err(anyhow!("Out of bounds read_u16_be at {}", offset));
        }
        Ok(u16::from_be_bytes([buf[offset], buf[offset + 1]]))
    }

    fn read_u32_be(buf: &[u8], offset: usize) -> Result<u32> {
        if offset + 4 > buf.len() {
            return Err(anyhow!("Out of bounds read_u32_be at {}", offset));
        }
        Ok(u32::from_be_bytes([
            buf[offset],
            buf[offset + 1],
            buf[offset + 2],
            buf[offset + 3],
        ]))
    }

    fn normalize_isbn_like(raw: &str) -> String {
        let mut s = raw.trim().to_string();
        if s.is_empty() {
            return s;
        }
        // Strip common prefixes like "ISBN:" / "isbn " etc.
        let lower = s.to_lowercase();
        if lower.starts_with("isbn") {
            // Remove leading "isbn" and any separators
            s = s[4..].to_string();
            s = s.trim_start_matches([':', ' ', '\t', '-']).to_string();
        }
        // Remove whitespace; keep hyphens if present (harmless) but many sources omit them.
        s = s.split_whitespace().collect::<String>();
        s.trim().to_string()
    }

    fn normalize_language_tag(raw: &str) -> String {
        // Keep this conservative: we just want something stable for display/filtering.
        // Common inputs: "en", "en-US", "en_US", "de", etc.
        let mut s = raw.trim().trim_matches('\0').to_string();
        if s.is_empty() {
            return s;
        }
        // Normalize separators and casing mildly (we don't attempt full BCP-47 validation here).
        s = s.replace('_', "-");
        s
    }
}

#[derive(Debug, Clone)]
struct ExthBlock {
    records: Vec<(u32, Vec<u8>)>,
}

impl ExthBlock {
    fn get_u32(&self, typ: u32) -> Option<u32> {
        self.records
            .iter()
            .find(|(t, _)| *t == typ)
            .and_then(|(_, v)| {
                if v.len() < 4 {
                    None
                } else {
                    Some(u32::from_be_bytes([v[0], v[1], v[2], v[3]]))
                }
            })
    }

    fn get_string(&self, typ: u32) -> Option<String> {
        self.records
            .iter()
            .find(|(t, _)| *t == typ)
            .map(|(_, v)| {
                let s = String::from_utf8_lossy(v).into_owned();
                s.trim_matches('\0').trim().to_string()
            })
            .filter(|s| !s.is_empty())
    }

    fn get_strings(&self, typ: u32) -> Vec<String> {
        self.records
            .iter()
            .filter(|(t, _)| *t == typ)
            .map(|(_, v)| {
                let s = String::from_utf8_lossy(v).into_owned();
                s.trim_matches('\0').trim().to_string()
            })
            .filter(|s| !s.is_empty())
            .collect()
    }
}
