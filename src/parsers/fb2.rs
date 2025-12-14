use crate::models::{BookInfo, Identifier};
use crate::utils::sanitize_html;
use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use log::{debug, warn};
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::Event;
use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub struct Fb2Parser;

impl Fb2Parser {
    pub fn new() -> Self {
        Self
    }

    pub async fn parse(&self, fb2_path: &Path) -> Result<BookInfo> {
        let path = fb2_path.to_path_buf();

        // Run blocking I/O and parsing on the blocking threadpool
        tokio::task::spawn_blocking(move || Self::parse_sync(&path))
            .await
            .with_context(|| "Task join error")?
    }

    fn parse_sync(fb2_path: &PathBuf) -> Result<BookInfo> {
        debug!("Opening FB2: {:?}", fb2_path);

        // Read the FB2 content, handling both plain .fb2 and .fb2.zip
        let xml_content = Self::read_fb2_content(fb2_path)?;

        // Parse FB2 XML
        let (fb2_info, cover_href) = Self::parse_fb2_metadata(&xml_content)?;

        // Extract cover image if referenced
        let (cover_data, cover_mime_type) = if let Some(ref cover_href) = cover_href {
            Self::extract_cover_image(&xml_content, cover_href)?
        } else {
            (None, None)
        };

        Ok(BookInfo {
            cover_data,
            cover_mime_type,
            ..fb2_info
        })
    }

    /// Read FB2 content, handling both plain .fb2 and .fb2.zip files
    fn read_fb2_content(path: &Path) -> Result<String> {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        if filename.ends_with(".fb2.zip") {
            // Extract FB2 from zip archive
            let file = File::open(path)
                .with_context(|| format!("Failed to open FB2 zip file: {:?}", path))?;
            let mut archive = ZipArchive::new(file)
                .with_context(|| format!("Failed to read FB2 zip archive: {:?}", path))?;

            // Find the .fb2 file inside the archive
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;
                let entry_name = entry.name().to_lowercase();
                if entry_name.ends_with(".fb2") {
                    let mut content = String::new();
                    entry.read_to_string(&mut content)?;
                    debug!(
                        "Extracted FB2 from zip: {} ({} bytes)",
                        entry.name(),
                        content.len()
                    );
                    return Ok(content);
                }
            }

            Err(anyhow!("No .fb2 file found inside zip archive: {:?}", path))
        } else {
            // Plain .fb2 file
            let mut file =
                File::open(path).with_context(|| format!("Failed to open FB2 file: {:?}", path))?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok(content)
        }
    }

    fn parse_fb2_metadata(fb2_xml: &str) -> Result<(BookInfo, Option<String>)> {
        let mut reader = Reader::from_str(fb2_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        // State tracking
        let mut in_description = false;
        let mut in_title_info = false;
        let mut in_publish_info = false;
        let mut in_coverpage = false;
        let mut in_author = false;

        // Metadata fields
        let mut title = None;
        let mut authors = Vec::new();
        let mut description = None;
        let mut publisher = None;
        let mut language = None;
        let mut identifiers = Vec::new();
        let mut subjects = Vec::new();
        let mut series = None;
        let mut series_number = None;
        let mut cover_href = None;

        // Current author parts
        let mut first_name = String::new();
        let mut middle_name = String::new();
        let mut last_name = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    let name = local_name.as_ref();

                    match name {
                        b"description" => in_description = true,
                        b"title-info" if in_description => in_title_info = true,
                        b"publish-info" if in_description => in_publish_info = true,
                        b"coverpage" if in_title_info => in_coverpage = true,
                        b"author" if in_title_info => {
                            in_author = true;
                            first_name.clear();
                            middle_name.clear();
                            last_name.clear();
                        }
                        // title-info elements
                        b"book-title" if in_title_info => {
                            if let Ok(text) = reader.read_text(e.name()) {
                                title = Some(
                                    unescape(&text).unwrap_or(Cow::Borrowed(&text)).into_owned(),
                                );
                            }
                        }
                        b"lang" if in_title_info => {
                            if let Ok(text) = reader.read_text(e.name()) {
                                language = Some(
                                    unescape(&text).unwrap_or(Cow::Borrowed(&text)).into_owned(),
                                );
                            }
                        }
                        b"genre" if in_title_info => {
                            if let Ok(text) = reader.read_text(e.name()) {
                                let genre =
                                    unescape(&text).unwrap_or(Cow::Borrowed(&text)).into_owned();
                                if !genre.is_empty() {
                                    subjects.push(genre);
                                }
                            }
                        }
                        b"annotation" if in_title_info => {
                            // Read the entire annotation content (may contain nested XML)
                            if let Ok(text) = reader.read_text(e.name()) {
                                let cleaned = sanitize_html(&text);
                                if !cleaned.trim().is_empty() {
                                    description = Some(cleaned);
                                }
                            }
                        }
                        // Author name parts
                        b"first-name" if in_author => {
                            if let Ok(text) = reader.read_text(e.name()) {
                                first_name =
                                    unescape(&text).unwrap_or(Cow::Borrowed(&text)).into_owned();
                            }
                        }
                        b"middle-name" if in_author => {
                            if let Ok(text) = reader.read_text(e.name()) {
                                middle_name =
                                    unescape(&text).unwrap_or(Cow::Borrowed(&text)).into_owned();
                            }
                        }
                        b"last-name" if in_author => {
                            if let Ok(text) = reader.read_text(e.name()) {
                                last_name =
                                    unescape(&text).unwrap_or(Cow::Borrowed(&text)).into_owned();
                            }
                        }
                        // publish-info elements
                        b"publisher" if in_publish_info => {
                            if let Ok(text) = reader.read_text(e.name()) {
                                publisher = Some(
                                    unescape(&text).unwrap_or(Cow::Borrowed(&text)).into_owned(),
                                );
                            }
                        }
                        b"isbn" if in_publish_info => {
                            if let Ok(text) = reader.read_text(e.name()) {
                                let isbn =
                                    unescape(&text).unwrap_or(Cow::Borrowed(&text)).into_owned();
                                if !isbn.is_empty() {
                                    identifiers.push(Identifier::new("isbn".to_string(), isbn));
                                }
                            }
                        }
                        // Cover image
                        b"image" if in_coverpage => {
                            for attr in e.attributes().flatten() {
                                let key = attr.key.as_ref();
                                // Handle both href and l:href (XLink namespace)
                                if (key == b"href" || key.ends_with(b":href"))
                                    && let Ok(href) = attr.unescape_value() {
                                        cover_href = Some(href.trim_start_matches('#').to_string());
                                        break;
                                    }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    let name = local_name.as_ref();

                    // Handle <sequence name="..." number="..."/> for series
                    if name == b"sequence" && in_title_info {
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"name" => {
                                    if let Ok(val) = attr.unescape_value() {
                                        series = Some(val.into_owned());
                                    }
                                }
                                b"number" => {
                                    if let Ok(val) = attr.unescape_value() {
                                        series_number = Some(val.into_owned());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    // Handle self-closing <image .../> in coverpage
                    else if name == b"image" && in_coverpage {
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            if (key == b"href" || key.ends_with(b":href"))
                                && let Ok(href) = attr.unescape_value() {
                                    cover_href = Some(href.trim_start_matches('#').to_string());
                                    break;
                                }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = local_name.as_ref();

                    match name {
                        b"description" => in_description = false,
                        b"title-info" => in_title_info = false,
                        b"publish-info" => in_publish_info = false,
                        b"coverpage" => in_coverpage = false,
                        b"author" if in_author => {
                            // Build full author name from parts
                            let mut parts = Vec::new();
                            if !first_name.is_empty() {
                                parts.push(first_name.clone());
                            }
                            if !middle_name.is_empty() {
                                parts.push(middle_name.clone());
                            }
                            if !last_name.is_empty() {
                                parts.push(last_name.clone());
                            }
                            if !parts.is_empty() {
                                authors.push(parts.join(" "));
                            }
                            in_author = false;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow!("Error parsing FB2: {}", e)),
                _ => {}
            }
            buf.clear();
        }

        let info = BookInfo {
            title: title.unwrap_or_else(|| "Unknown Title".to_string()),
            authors,
            description,
            publisher,
            language,
            identifiers,
            subjects,
            series,
            series_number,
            pages: None,
            cover_data: None,
            cover_mime_type: None,
        };
        Ok((info, cover_href))
    }

    fn extract_cover_image(
        fb2_xml: &str,
        cover_href: &str,
    ) -> Result<(Option<Vec<u8>>, Option<String>)> {
        let mut reader = Reader::from_str(fb2_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if e.local_name().as_ref() == b"binary" {
                        let mut found_id = false;
                        let mut mime_type = None;

                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"id" => {
                                    if let Ok(id) = attr.unescape_value()
                                        && id.as_ref() == cover_href {
                                            found_id = true;
                                        }
                                }
                                b"content-type" => {
                                    if let Ok(ct) = attr.unescape_value() {
                                        mime_type = Some(ct.into_owned());
                                    }
                                }
                                _ => {}
                            }
                        }

                        if found_id {
                            // Read the base64 content
                            if let Ok(text) = reader.read_text(e.name()) {
                                let text_clean = text
                                    .trim()
                                    .replace(['\n', '\r', ' '], "");
                                match general_purpose::STANDARD.decode(&text_clean) {
                                    Ok(data) => return Ok((Some(data), mime_type)),
                                    Err(e) => {
                                        warn!("Failed to decode base64 cover image: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    warn!("Error parsing binary section: {}", e);
                    break;
                }
                _ => {}
            }
            buf.clear();
        }

        Ok((None, None))
    }
}
