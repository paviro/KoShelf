use crate::shelf::models::{BookInfo, ChapterEntry, Identifier};
use crate::shelf::utils::sanitize_html;
use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use log::{debug, warn};
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::Event;
use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

/// Extracts metadata and cover images from FB2 files (plain XML or `.fb2.zip`).
pub struct Fb2Parser;

impl Default for Fb2Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Fb2Parser {
    pub fn new() -> Self {
        Self
    }

    /// Parse an FB2 file for metadata and cover image.
    pub async fn parse(&self, fb2_path: &Path) -> Result<BookInfo> {
        let path = fb2_path.to_path_buf();

        tokio::task::spawn_blocking(move || Self::parse_sync(&path))
            .await
            .with_context(|| "Task join error")?
    }

    fn parse_sync(fb2_path: &Path) -> Result<BookInfo> {
        debug!("Opening FB2: {:?}", fb2_path);

        let xml_content = Self::read_fb2_content(fb2_path)?;
        let (fb2_info, cover_href) = Self::parse_fb2_metadata(&xml_content)?;
        let chapters = Self::extract_chapters(&xml_content);

        let (cover_data, cover_mime_type) = if let Some(ref cover_href) = cover_href {
            Self::extract_cover_image(&xml_content, cover_href)?
        } else {
            (None, None)
        };

        Ok(BookInfo {
            chapters,
            cover_data,
            cover_mime_type,
            ..fb2_info
        })
    }

    /// Read FB2 content, handling both plain .fb2 and .fb2.zip files
    /// Also detects .fb2 files that are actually ZIP archives (by checking magic bytes)
    fn read_fb2_content(path: &Path) -> Result<String> {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check if it's explicitly a .fb2.zip file or if the .fb2 file is actually a ZIP archive
        let is_zip = if filename.ends_with(".fb2.zip") {
            true
        } else {
            // Check magic bytes for ZIP signature (PK\x03\x04)
            Self::is_zip_file(path)?
        };

        if is_zip {
            Self::extract_fb2_from_zip(path)
        } else {
            // Plain .fb2 file (XML)
            let mut file =
                File::open(path).with_context(|| format!("Failed to open FB2 file: {:?}", path))?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok(content)
        }
    }

    /// Check if a file is a ZIP archive by reading the magic bytes
    fn is_zip_file(path: &Path) -> Result<bool> {
        let mut file =
            File::open(path).with_context(|| format!("Failed to open file: {:?}", path))?;
        let mut magic = [0u8; 4];
        match file.read_exact(&mut magic) {
            Ok(_) => {
                // ZIP magic bytes: PK\x03\x04
                Ok(magic[0] == 0x50 && magic[1] == 0x4B && magic[2] == 0x03 && magic[3] == 0x04)
            }
            Err(_) => Ok(false), // File too small, not a ZIP
        }
    }

    /// Extract FB2 content from a ZIP archive
    fn extract_fb2_from_zip(path: &Path) -> Result<String> {
        let file =
            File::open(path).with_context(|| format!("Failed to open FB2 zip file: {:?}", path))?;
        let mut archive = ZipArchive::new(file)
            .with_context(|| format!("Failed to read FB2 zip archive: {:?}", path))?;

        // Find the .fb2 or .xml file inside the archive
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let entry_name = entry.name().to_lowercase();
            if entry_name.ends_with(".fb2") || entry_name.ends_with(".xml") {
                let mut content = String::new();
                entry.read_to_string(&mut content)?;
                debug!(
                    "Extracted FB2 or XML from zip: {} ({} bytes)",
                    entry.name(),
                    content.len()
                );
                return Ok(content);
            }
        }

        Err(anyhow!(
            "No .fb2 or .xml file found inside zip archive: {:?}",
            path
        ))
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
                                    && let Ok(href) = attr.unescape_value()
                                {
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
                                && let Ok(href) = attr.unescape_value()
                            {
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
            chapters: Vec::new(),
            cover_data: None,
            cover_mime_type: None,
        };
        Ok((info, cover_href))
    }

    /// Extract chapter entries from `<body>` sections.
    ///
    /// Proper FB2 files use `<section><title><p>…</p></title>…</section>`.
    /// Sections may be nested (parts containing chapters). We collect every
    /// `<title>` element at any section depth. When no `<title>` elements exist
    /// (e.g. Calibre-converted FB2s), we fall back to the first short `<p>` of
    /// each top-level section.
    fn extract_chapters(fb2_xml: &str) -> Vec<ChapterEntry> {
        let mut reader = Reader::from_str(fb2_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        let mut in_body = false;
        let mut body_done = false; // ignore <body name="notes"> etc.
        let mut body_start: u64 = 0;
        let mut body_end: u64 = 0;
        let mut section_depth: u32 = 0;

        // Collected from <title> elements at any depth.
        let mut titled_entries: Vec<(u64, String)> = Vec::new();
        // Fallback: first short <p> per top-level section (depth 1).
        let mut fallback_entries: Vec<(u64, String)> = Vec::new();

        // State for the current top-level section (fallback path only).
        let mut current_section_pos: u64 = 0;
        let mut current_first_p: Option<String> = None;
        let mut awaiting_first_p = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.local_name();
                    match name.as_ref() {
                        b"body" if !in_body && !body_done => {
                            in_body = true;
                            body_start = reader.buffer_position();
                        }
                        b"section" if in_body => {
                            section_depth += 1;
                            if section_depth == 1 {
                                current_section_pos = reader.buffer_position();
                                current_first_p = None;
                                awaiting_first_p = true;
                            }
                        }
                        b"title" if in_body && section_depth >= 1 => {
                            let pos = reader.buffer_position();
                            if let Ok(inner) = reader.read_text(e.name()) {
                                let text = Self::strip_xml_tags(
                                    &unescape(&inner).unwrap_or(Cow::Borrowed(&inner)),
                                );
                                if !text.is_empty() {
                                    titled_entries.push((pos, text));
                                }
                            }
                            awaiting_first_p = false;
                        }
                        b"p" if in_body && section_depth == 1 && awaiting_first_p => {
                            if let Ok(inner) = reader.read_text(e.name()) {
                                let text = Self::strip_xml_tags(
                                    &unescape(&inner).unwrap_or(Cow::Borrowed(&inner)),
                                );
                                if !text.is_empty() && text.len() <= 80 {
                                    current_first_p = Some(text);
                                }
                            }
                            awaiting_first_p = false;
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.local_name();
                    match name.as_ref() {
                        b"body" if in_body => {
                            in_body = false;
                            body_done = true;
                            body_end = reader.buffer_position();
                        }
                        b"section" if in_body => {
                            if section_depth == 1
                                && let Some(text) = current_first_p.take()
                            {
                                fallback_entries.push((current_section_pos, text));
                            }
                            section_depth = section_depth.saturating_sub(1);
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        let body_len = body_end.saturating_sub(body_start);
        if body_len == 0 {
            return Vec::new();
        }
        let body_len_f = body_len as f64;

        let entries = if !titled_entries.is_empty() {
            &titled_entries
        } else {
            &fallback_entries
        };

        entries
            .iter()
            .map(|(pos, title)| {
                let position = (pos.saturating_sub(body_start) as f64 / body_len_f).clamp(0.0, 1.0);
                ChapterEntry {
                    title: title.clone(),
                    position,
                }
            })
            .collect()
    }

    /// Strip all XML/HTML tags from a string, returning only text content.
    fn strip_xml_tags(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut in_tag = false;
        for ch in input.chars() {
            if ch == '<' {
                in_tag = true;
            } else if ch == '>' {
                in_tag = false;
            } else if !in_tag {
                out.push(ch);
            }
        }
        // Collapse internal whitespace runs into single spaces.
        out.split_whitespace().collect::<Vec<_>>().join(" ")
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
                                        && id.as_ref() == cover_href
                                    {
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
                                let text_clean = text.trim().replace(['\n', '\r', ' '], "");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_xml_tags_basic() {
        assert_eq!(Fb2Parser::strip_xml_tags("<p>Hello</p>"), "Hello");
        assert_eq!(
            Fb2Parser::strip_xml_tags("<p>Part <strong>One</strong></p>"),
            "Part One"
        );
        assert_eq!(
            Fb2Parser::strip_xml_tags("<p>  spaced  out  </p>"),
            "spaced out"
        );
    }

    #[test]
    fn chapters_from_title_elements() {
        let xml = r#"<?xml version="1.0"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description><title-info><book-title>Test</book-title></title-info></description>
<body>
<section><title><p>Chapter 1</p></title><p>Some content here.</p></section>
<section><title><p>Chapter 2</p></title><p>More content here.</p></section>
</body>
</FictionBook>"#;

        let chapters = Fb2Parser::extract_chapters(xml);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Chapter 1");
        assert_eq!(chapters[1].title, "Chapter 2");
        assert!(chapters[0].position < chapters[1].position);
        assert!(chapters[0].position >= 0.0);
        assert!(chapters[1].position <= 1.0);
    }

    #[test]
    fn chapters_fallback_to_first_p() {
        let xml = r#"<?xml version="1.0"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description><title-info><book-title>Test</book-title></title-info></description>
<body>
<section><p>Part One</p><p>Long content paragraph that is definitely not a title.</p></section>
<section><p>1</p><p>Another long content paragraph here.</p></section>
<section><p>2</p><p>Yet more content follows.</p></section>
</body>
</FictionBook>"#;

        let chapters = Fb2Parser::extract_chapters(xml);
        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[0].title, "Part One");
        assert_eq!(chapters[1].title, "1");
        assert_eq!(chapters[2].title, "2");
    }

    #[test]
    fn chapters_from_nested_sections() {
        let xml = r#"<?xml version="1.0"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description><title-info><book-title>Test</book-title></title-info></description>
<body>
<section><title><p>Part One</p></title>
  <section><title><p>Chapter 1</p></title><p>Content A.</p></section>
  <section><title><p>Chapter 2</p></title><p>Content B.</p></section>
</section>
<section><title><p>Part Two</p></title>
  <section><title><p>Chapter 3</p></title><p>Content C.</p></section>
</section>
</body>
</FictionBook>"#;

        let chapters = Fb2Parser::extract_chapters(xml);
        assert_eq!(chapters.len(), 5);
        assert_eq!(chapters[0].title, "Part One");
        assert_eq!(chapters[1].title, "Chapter 1");
        assert_eq!(chapters[4].title, "Chapter 3");
    }

    #[test]
    fn ignores_notes_body() {
        let xml = r#"<?xml version="1.0"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description><title-info><book-title>Test</book-title></title-info></description>
<body>
<section><title><p>Chapter 1</p></title><p>Content.</p></section>
</body>
<body name="notes">
<section><title><p>Note 1</p></title><p>Footnote text.</p></section>
<section><title><p>Note 2</p></title><p>More footnotes.</p></section>
</body>
</FictionBook>"#;

        let chapters = Fb2Parser::extract_chapters(xml);
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "Chapter 1");
    }

    #[test]
    fn skips_sections_without_titles_when_titles_exist() {
        let xml = r#"<?xml version="1.0"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description><title-info><book-title>Test</book-title></title-info></description>
<body>
<section><p>Just an image section with no title element.</p></section>
<section><title><p>Real Chapter</p></title><p>Content.</p></section>
</body>
</FictionBook>"#;

        let chapters = Fb2Parser::extract_chapters(xml);
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "Real Chapter");
    }
}
