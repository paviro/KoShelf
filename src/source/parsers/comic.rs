use crate::shelf::models::BookInfo;
use crate::shelf::utils::sanitize_html;
use anyhow::{Context, Result, anyhow};
use log::{debug, warn};
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::Event;
use std::borrow::Cow;
#[cfg(not(windows))]
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Image file extensions we look for as cover candidates
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];

/// Extracts metadata and cover images from CBZ (ZIP) and CBR (RAR) comic archives.
pub struct ComicParser;

impl Default for ComicParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ComicParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a CBZ or CBR comic archive for metadata (ComicInfo.xml) and cover image.
    pub async fn parse(&self, comic_path: &Path) -> Result<BookInfo> {
        let path = comic_path.to_path_buf();
        let is_cbr = path
            .extension()
            .map(|e| e.to_str().unwrap_or("").to_lowercase() == "cbr")
            .unwrap_or(false);

        // Windows builds intentionally don't support CBR/RAR archives (unrar doesn't compile reliably).
        if is_cbr && cfg!(windows) {
            return Err(anyhow!(
                "CBR (.cbr) is not supported on Windows builds; please convert to CBZ (.cbz) or use the linux subsystem for windows."
            ));
        }

        tokio::task::spawn_blocking(move || {
            if is_cbr {
                Self::parse_cbr_sync(&path)
            } else {
                Self::parse_cbz_sync(&path)
            }
        })
        .await
        .with_context(|| "Task join error")?
    }

    /// Parse a CBZ (ZIP-based) comic archive
    fn parse_cbz_sync(cbz_path: &PathBuf) -> Result<BookInfo> {
        debug!("Opening CBZ: {:?}", cbz_path);
        let file = File::open(cbz_path)
            .with_context(|| format!("Failed to open CBZ file: {:?}", cbz_path))?;
        let mut zip = ZipArchive::new(file)
            .with_context(|| format!("Failed to read CBZ as zip: {:?}", cbz_path))?;

        let image_count = zip
            .file_names()
            .filter(|name| {
                let lower_name = name.to_lowercase();
                IMAGE_EXTENSIONS.iter().any(|ext| lower_name.ends_with(ext))
            })
            .count();

        let mut book_info = if let Ok(mut comic_info_file) = zip.by_name("ComicInfo.xml") {
            let mut xml_content = String::new();
            comic_info_file.read_to_string(&mut xml_content)?;
            Self::parse_comic_info_xml(&xml_content)?
        } else {
            Self::book_info_from_filename(cbz_path)
        };

        if image_count > 0 {
            book_info.pages = Some(image_count as u32);
        }

        let (cover_data, cover_mime_type) = Self::extract_cover_from_cbz(&mut zip)?;
        book_info.cover_data = cover_data;
        book_info.cover_mime_type = cover_mime_type;

        Ok(book_info)
    }

    /// Parse a CBR (RAR-based) comic archive
    #[cfg(not(windows))]
    fn parse_cbr_sync(cbr_path: &PathBuf) -> Result<BookInfo> {
        debug!("Opening CBR: {:?}", cbr_path);

        let temp_dir = tempfile::tempdir().with_context(|| "Failed to create temp directory")?;

        let archive = unrar::Archive::new(cbr_path)
            .open_for_processing()
            .map_err(|e| anyhow!("Failed to open CBR file: {:?}", e))?;

        let mut comic_info_xml: Option<String> = None;
        let mut image_files: Vec<String> = Vec::new();

        let mut archive = archive;
        loop {
            match archive.read_header() {
                Ok(Some(header)) => {
                    let filename = header.entry().filename.to_string_lossy().to_string();

                    let basename = Path::new(&filename)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&filename);

                    if basename.eq_ignore_ascii_case("ComicInfo.xml") {
                        let extract_path = temp_dir.path().join("ComicInfo.xml");
                        archive = header
                            .extract_to(&extract_path)
                            .map_err(|e| anyhow!("Failed to extract ComicInfo.xml: {:?}", e))?;
                        if let Ok(content) = fs::read_to_string(&extract_path) {
                            comic_info_xml = Some(content);
                        }
                        continue;
                    }

                    let is_image = Path::new(&filename)
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| {
                            let lower = e.to_lowercase();
                            IMAGE_EXTENSIONS.contains(&lower.as_str())
                        })
                        .unwrap_or(false);

                    if is_image {
                        image_files.push(filename.clone());
                    }

                    archive = header
                        .skip()
                        .map_err(|e| anyhow!("Failed to skip entry: {:?}", e))?;
                }
                Ok(None) => break, // No more entries
                Err(e) => return Err(anyhow!("Failed to read RAR header: {:?}", e)),
            }
        }

        let mut book_info = if let Some(ref xml) = comic_info_xml {
            Self::parse_comic_info_xml(xml)?
        } else {
            Self::book_info_from_filename(cbr_path)
        };

        if !image_files.is_empty() {
            book_info.pages = Some(image_files.len() as u32);
        }

        image_files.sort();

        if let Some(cover_filename) = image_files.first() {
            debug!("Selected cover image for CBR: {}", cover_filename);

            let archive = unrar::Archive::new(cbr_path)
                .open_for_processing()
                .map_err(|e| anyhow!("Failed to re-open CBR file for cover extraction: {:?}", e))?;

            let mut archive = archive;
            let mut cover_extracted = false;

            loop {
                match archive.read_header() {
                    Ok(Some(header)) => {
                        let filename = header.entry().filename.to_string_lossy().to_string();

                        if &filename == cover_filename {
                            // Use a safe fixed filename to prevent path traversal
                            // from malicious archive entry names containing "../".
                            let safe_name = Path::new(&filename)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            let extract_path = temp_dir.path().join(if safe_name.is_empty() {
                                "cover".to_string()
                            } else {
                                safe_name
                            });

                            header
                                .extract_to(&extract_path)
                                .map_err(|e| anyhow!("Failed to extract cover image: {:?}", e))?;

                            if let Ok(data) = fs::read(&extract_path) {
                                let mime =
                                    Self::mime_type_from_extension(&extract_path.to_string_lossy());
                                book_info.cover_data = Some(data);
                                book_info.cover_mime_type = mime;
                            }
                            cover_extracted = true;
                            break;
                        } else {
                            archive = header
                                .skip()
                                .map_err(|e| anyhow!("Failed to skip entry: {:?}", e))?;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        warn!("Error reading RAR header during cover extraction: {:?}", e);
                        break;
                    }
                }
            }

            if !cover_extracted {
                warn!(
                    "Failed to find selected cover image '{}' in second pass",
                    cover_filename
                );
            }
        }

        Ok(book_info)
    }

    /// Parse a CBR (RAR-based) comic archive (unsupported on Windows).
    #[cfg(windows)]
    fn parse_cbr_sync(_cbr_path: &PathBuf) -> Result<BookInfo> {
        Err(anyhow!(
            "CBR (.cbr) is not supported on Windows builds; please convert to CBZ (.cbz)."
        ))
    }

    /// Parse ComicInfo.xml metadata
    fn parse_comic_info_xml(xml: &str) -> Result<BookInfo> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        let mut title: Option<String> = None;
        let mut series: Option<String> = None;
        let mut number: Option<String> = None;
        let mut summary: Option<String> = None;
        let mut publisher: Option<String> = None;
        let mut language: Option<String> = None;
        let mut authors: Vec<String> = Vec::new();
        let mut subjects: Vec<String> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = e.local_name();

                    // Skip the root ComicInfo element - we want its children
                    if tag_name.as_ref() == b"ComicInfo" {
                        continue;
                    }

                    // Read the text content for this element
                    if let Ok(text) = reader.read_text(e.name()) {
                        let text = unescape(&text).unwrap_or(Cow::Borrowed(&text)).into_owned();
                        if text.is_empty() {
                            continue;
                        }

                        match tag_name.as_ref() {
                            b"Title" => title = Some(text),
                            b"Series" => series = Some(text),
                            b"Number" => number = Some(text),
                            b"Summary" => {
                                let cleaned = sanitize_html(&text);
                                let trimmed = cleaned.trim();
                                if !trimmed.is_empty() {
                                    summary = Some(trimmed.to_string());
                                }
                            }
                            b"Publisher" => publisher = Some(text),
                            b"LanguageISO" => language = Some(text),
                            b"Writer" => {
                                // Writers are comma-separated
                                for writer in text.split(',').map(|s| s.trim().to_string()) {
                                    if !writer.is_empty() && !authors.contains(&writer) {
                                        authors.push(writer);
                                    }
                                }
                            }
                            b"Penciller" | b"Inker" | b"Colorist" | b"Letterer"
                            | b"CoverArtist" | b"Editor" => {
                                // Add other creators to authors list
                                for creator in text.split(',').map(|s| s.trim().to_string()) {
                                    if !creator.is_empty() && !authors.contains(&creator) {
                                        authors.push(creator);
                                    }
                                }
                            }
                            b"Genre" => {
                                // Genres are comma-separated
                                for genre in text.split(',').map(|s| s.trim().to_string()) {
                                    if !genre.is_empty() && !subjects.contains(&genre) {
                                        subjects.push(genre);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    warn!("Error parsing ComicInfo.xml: {}", e);
                    break;
                }
                _ => {}
            }
            buf.clear();
        }
        // Use Title if present, otherwise fall back to Series (common in comics)
        let final_title = title
            .or_else(|| series.clone())
            .unwrap_or_else(|| "Unknown Comic".to_string());

        Ok(BookInfo {
            title: final_title,
            authors,
            description: summary,
            language,
            publisher,
            identifiers: Vec::new(),
            subjects,
            series,
            series_number: number,
            pages: None,
            chapters: Vec::new(),
            cover_data: None,
            cover_mime_type: None,
        })
    }

    /// Create BookInfo from filename (fallback when no ComicInfo.xml)
    fn book_info_from_filename(path: &Path) -> BookInfo {
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown Comic".to_string());

        BookInfo {
            title,
            authors: Vec::new(),
            description: None,
            language: None,
            publisher: None,
            identifiers: Vec::new(),
            subjects: Vec::new(),
            series: None,
            series_number: None,
            pages: None,
            chapters: Vec::new(),
            cover_data: None,
            cover_mime_type: None,
        }
    }

    /// Extract cover image from CBZ archive (first image when sorted)
    fn extract_cover_from_cbz(
        zip: &mut ZipArchive<File>,
    ) -> Result<(Option<Vec<u8>>, Option<String>)> {
        // Collect all image file names
        // Find the first image file alphabetically (which is usually the cover page)
        // using an iterator to avoid checking every file if not needed and avoiding allocations
        let cover_file_name = zip
            .file_names()
            .filter(|name| {
                let lower_name = name.to_lowercase();
                IMAGE_EXTENSIONS.iter().any(|ext| lower_name.ends_with(ext))
            })
            // Lexicographical minimum is equivalent to sorting and taking the first
            .min()
            .map(|s| s.to_string());

        if let Some(first_image) = cover_file_name
            && let Ok(mut file) = zip.by_name(&first_image)
        {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            let mime = Self::mime_type_from_extension(&first_image);
            return Ok((Some(buf), mime));
        }

        Ok((None, None))
    }

    /// Get MIME type from file extension
    fn mime_type_from_extension(filename: &str) -> Option<String> {
        let lower = filename.to_lowercase();
        if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
            Some("image/jpeg".to_string())
        } else if lower.ends_with(".png") {
            Some("image/png".to_string())
        } else if lower.ends_with(".webp") {
            Some("image/webp".to_string())
        } else if lower.ends_with(".gif") {
            Some("image/gif".to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ComicParser;

    #[test]
    fn comic_info_summary_is_sanitized() {
        let xml = r#"
            <ComicInfo>
                <Title>Test Comic</Title>
                <Summary><![CDATA[<p>Hello <strong>world</strong></p><script>alert('x')</script>]]></Summary>
            </ComicInfo>
        "#;

        let parsed = ComicParser::parse_comic_info_xml(xml).expect("comic info should parse");

        let description = parsed
            .description
            .expect("sanitized non-empty summary should be retained");
        assert!(description.contains("Hello"));
        assert!(description.contains("world"));
        assert!(!description.contains("<script"));
    }

    #[test]
    fn comic_info_empty_summary_is_ignored() {
        let xml = r#"
            <ComicInfo>
                <Title>Test Comic</Title>
                <Summary>   </Summary>
            </ComicInfo>
        "#;

        let parsed = ComicParser::parse_comic_info_xml(xml).expect("comic info should parse");

        assert!(parsed.description.is_none());
    }
}
