use crate::models::{EpubInfo, Identifier};
use anyhow::{Result, Context, anyhow};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use log::{debug, warn};
use quick_xml::Reader;
use quick_xml::events::Event;
use quick_xml::escape::unescape;
use std::borrow::Cow;
use base64::{Engine as _, engine::general_purpose};

pub struct Fb2Parser;

impl Fb2Parser {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn parse(&self, fb2_path: &Path) -> Result<EpubInfo> {
        debug!("Opening FB2: {:?}", fb2_path);
        let mut file = File::open(fb2_path)
            .with_context(|| format!("Failed to open FB2 file: {:?}", fb2_path))?;
        let mut xml_content = String::new();
        file.read_to_string(&mut xml_content)?;

        // Parse FB2 XML
        let (fb2_info, cover_href) = Self::parse_fb2_metadata(&xml_content)?;

        // Extract cover image if referenced
        let (cover_data, cover_mime_type) = if let Some(ref cover_href) = cover_href {
            Self::extract_cover_image(&xml_content, cover_href)?
        } else {
            (None, None)
        };

        Ok(EpubInfo {
            cover_data,
            cover_mime_type,
            ..fb2_info
        })
    }

    fn parse_fb2_metadata(fb2_xml: &str) -> Result<(EpubInfo, Option<String>)> {
        let mut reader = Reader::from_str(fb2_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        
        let mut in_description = false;
        let mut in_title_info = false;
        let mut in_publish_info = false;
        let mut in_coverpage = false;
        
        let mut title = None;
        let mut authors = Vec::new();
        let mut description = None;
        let mut publisher = None;
        let mut language = None;
        let mut identifiers = Vec::new();
        let mut subjects = Vec::new();
        let series = None;
        let series_number = None;
        let mut cover_href = None;
        
        // Track current element text
        let mut current_text = String::new();
        let mut current_element: Option<Vec<u8>> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    let name_str = String::from_utf8_lossy(local_name.as_ref());
                    
                    if name_str == "description" {
                        in_description = true;
                    } else if in_description && name_str == "title-info" {
                        in_title_info = true;
                    } else if in_description && name_str == "publish-info" {
                        in_publish_info = true;
                    } else if in_description && name_str == "coverpage" {
                        in_coverpage = true;
                    } else if in_title_info || in_publish_info {
                        current_element = Some(local_name.as_ref().to_vec());
                        current_text.clear();
                    } else if in_coverpage && name_str == "image" {
                        // Extract cover image href
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            if key == b"href" || key == b"l:href" {
                                if let Ok(href) = attr.unescape_value() {
                                    let href_str = href.into_owned();
                                    // Remove # prefix if present
                                    cover_href = Some(href_str.trim_start_matches('#').to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref elem) = current_element {
                        let elem_str = String::from_utf8_lossy(elem);
                        let text_bytes = e.as_ref();
                        let text_str = String::from_utf8_lossy(text_bytes);
                        let text = unescape(&text_str).unwrap_or(Cow::Borrowed(&text_str));
                        current_text.push_str(&text);
                        
                        if in_title_info {
                            match &*elem_str {
                                "book-title" => {
                                    title = Some(text.into_owned());
                                }
                                "first-name" | "middle-name" | "last-name" => {
                                    // We'll collect author name parts separately
                                }
                                "lang" => {
                                    language = Some(text.into_owned());
                                }
                                "genre" => {
                                    // Extract genre value from attributes or text
                                    subjects.push(text.into_owned());
                                }
                                "annotation" => {
                                    // Description can be in annotation
                                    if description.is_none() {
                                        let cleaned = Self::clean_text(&text);
                                        if !cleaned.trim().is_empty() {
                                            description = Some(cleaned);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        } else if in_publish_info {
                            match &*elem_str {
                                "publisher" => {
                                    publisher = Some(text.into_owned());
                                }
                                "isbn" => {
                                    identifiers.push(Identifier::new("isbn".to_string(), text.into_owned()));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name_str = String::from_utf8_lossy(local_name.as_ref());
                    
                    if name_str == "description" {
                        in_description = false;
                        in_title_info = false;
                        in_publish_info = false;
                        in_coverpage = false;
                    } else if name_str == "title-info" {
                        in_title_info = false;
                    } else if name_str == "publish-info" {
                        in_publish_info = false;
                    } else if name_str == "coverpage" {
                        in_coverpage = false;
                    } else if name_str == "author" && in_title_info {
                        // Collect author name parts
                        if !current_text.trim().is_empty() {
                            authors.push(current_text.trim().to_string());
                            current_text.clear();
                        }
                    }
                    
                    current_element = None;
                }
                Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    let name_str = String::from_utf8_lossy(local_name.as_ref());
                    
                    if in_coverpage && name_str == "image" {
                        // Extract cover image href
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            if key == b"href" || key == b"l:href" {
                                if let Ok(href) = attr.unescape_value() {
                                    let href_str = href.into_owned();
                                    cover_href = Some(href_str.trim_start_matches('#').to_string());
                                    break;
                                }
                            }
                        }
                    } else if in_title_info && name_str == "genre" {
                        // Extract genre value
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            if key == b"match" {
                                if let Ok(genre) = attr.unescape_value() {
                                    subjects.push(genre.into_owned());
                                }
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow!("Error parsing FB2: {}", e)),
                _ => {}
            }
            buf.clear();
        }

        // Parse author names more carefully
        let mut reader2 = Reader::from_str(fb2_xml);
        reader2.config_mut().trim_text(true);
        let mut buf2 = Vec::new();
        let mut authors_detailed = Vec::new();
        let mut in_author = false;
        let mut first_name = String::new();
        let mut middle_name = String::new();
        let mut last_name = String::new();
        let mut in_title_info2 = false;
        let mut in_description2 = false;

        loop {
            match reader2.read_event_into(&mut buf2) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    let name_str = String::from_utf8_lossy(local_name.as_ref());
                    if name_str == "description" {
                        in_description2 = true;
                    } else if in_description2 && name_str == "title-info" {
                        in_title_info2 = true;
                    } else if in_title_info2 && name_str == "author" {
                        in_author = true;
                        first_name.clear();
                        middle_name.clear();
                        last_name.clear();
                    } else if in_author {
                        let local_name = e.local_name();
                        let elem_name = String::from_utf8_lossy(local_name.as_ref());
                        if elem_name == "first-name" || elem_name == "middle-name" || elem_name == "last-name" {
                            // Read text content for this element
                            let mut text_content = String::new();
                            let mut nested_buf = Vec::new();
                            loop {
                                match reader2.read_event_into(&mut nested_buf) {
                                    Ok(Event::Text(text_event)) => {
                                        let text_bytes = text_event.as_ref();
                                        let text_str = String::from_utf8_lossy(text_bytes);
                                        let text_unescaped = unescape(&text_str).unwrap_or(Cow::Borrowed(&text_str));
                                        text_content.push_str(&text_unescaped);
                                    }
                                    Ok(Event::End(end_event)) => {
                                        if end_event.local_name().as_ref() == local_name.as_ref() {
                                            nested_buf.clear();
                                            break;
                                        }
                                    }
                                    Ok(Event::Eof) => break,
                                    Err(_) => break,
                                    _ => {}
                                }
                                nested_buf.clear();
                            }
                            match &*elem_name {
                                "first-name" => first_name = text_content,
                                "middle-name" => middle_name = text_content,
                                "last-name" => last_name = text_content,
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name_str = String::from_utf8_lossy(local_name.as_ref());
                    if name_str == "author" && in_author {
                        // Build full author name
                        let mut full_name = Vec::new();
                        if !first_name.is_empty() {
                            full_name.push(first_name.clone());
                        }
                        if !middle_name.is_empty() {
                            full_name.push(middle_name.clone());
                        }
                        if !last_name.is_empty() {
                            full_name.push(last_name.clone());
                        }
                        if !full_name.is_empty() {
                            authors_detailed.push(full_name.join(" "));
                        }
                        in_author = false;
                    } else if name_str == "title-info" {
                        in_title_info2 = false;
                    } else if name_str == "description" {
                        in_description2 = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    warn!("Error in second pass for authors: {}", e);
                    break;
                }
                _ => {}
            }
            buf2.clear();
        }

        // Use detailed authors if available, otherwise fall back to simple collection
        let final_authors = if !authors_detailed.is_empty() {
            authors_detailed
        } else if !authors.is_empty() {
            authors
        } else {
            Vec::new()
        };

        let info = EpubInfo {
            title: title.unwrap_or_else(|| "Unknown Title".to_string()),
            authors: final_authors,
            description,
            publisher,
            language,
            identifiers,
            subjects,
            series,
            series_number,
            cover_data: None,
            cover_mime_type: None,
        };
        Ok((info, cover_href))
    }

    fn extract_cover_image(fb2_xml: &str, cover_href: &str) -> Result<(Option<Vec<u8>>, Option<String>)> {
        let mut reader = Reader::from_str(fb2_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        
        let mut in_binary = false;
        let mut current_id = None;
        let mut image_data = None;
        let mut mime_type = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    if local_name.as_ref() == b"binary" {
                        in_binary = true;
                        // Extract id attribute
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            if key == b"id" {
                                if let Ok(id) = attr.unescape_value() {
                                    current_id = Some(id.into_owned());
                                }
                            } else if key == b"content-type" {
                                if let Ok(ct) = attr.unescape_value() {
                                    mime_type = Some(ct.into_owned());
                                }
                            }
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_binary {
                        if let Some(ref id) = current_id {
                            if id == cover_href {
                                // Decode base64 image data
                                let text_bytes = e.as_ref();
                                let text_str = String::from_utf8_lossy(text_bytes);
                                let text = unescape(&text_str).unwrap_or(Cow::Borrowed(&text_str));
                                let text_clean = text.trim().replace('\n', "").replace('\r', "").replace(' ', "");
                                match general_purpose::STANDARD.decode(&text_clean) {
                                    Ok(data) => {
                                        image_data = Some(data);
                                    }
                                    Err(e) => {
                                        warn!("Failed to decode base64 cover image: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.local_name().as_ref() == b"binary" {
                        in_binary = false;
                        current_id = None;
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

        Ok((image_data, mime_type))
    }

    fn clean_text(input: &str) -> String {
        // Remove XML tags and decode entities
        let decoded = unescape(input).unwrap_or(Cow::Borrowed(input));
        decoded.into_owned()
    }
}

