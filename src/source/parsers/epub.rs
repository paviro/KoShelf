use crate::shelf::models::{BookInfo, ChapterEntry, Identifier};
use crate::shelf::utils::sanitize_html;
use anyhow::{Context, Result, anyhow};
use log::{debug, warn};
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::Event;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use zip::ZipArchive;

/// Extracts metadata and cover images from EPUB files via OPF parsing.
pub struct EpubParser;

impl Default for EpubParser {
    fn default() -> Self {
        Self::new()
    }
}

impl EpubParser {
    pub fn new() -> Self {
        Self
    }

    /// Normalize a relative path inside a ZIP archive, stripping `..` components
    /// to prevent path traversal within the archive.
    fn normalize_zip_path(path: &str) -> String {
        let mut parts: Vec<&str> = Vec::new();
        for component in Path::new(path).components() {
            match component {
                Component::Normal(s) => {
                    if let Some(s) = s.to_str() {
                        parts.push(s);
                    }
                }
                Component::ParentDir => {
                    parts.pop();
                }
                _ => {} // Skip RootDir, CurDir, Prefix
            }
        }
        parts.join("/")
    }

    /// Parse an EPUB file for metadata, page count, and cover image.
    pub async fn parse(&self, epub_path: &Path) -> Result<BookInfo> {
        let path = epub_path.to_path_buf();

        tokio::task::spawn_blocking(move || Self::parse_sync(&path))
            .await
            .with_context(|| "Task join error")?
    }

    fn parse_sync(epub_path: &PathBuf) -> Result<BookInfo> {
        debug!("Opening EPUB: {:?}", epub_path);
        let file = File::open(epub_path)
            .with_context(|| format!("Failed to open EPUB file: {:?}", epub_path))?;
        let mut zip = ZipArchive::new(file)
            .with_context(|| format!("Failed to read EPUB as zip: {:?}", epub_path))?;

        let opf_path = {
            let mut container_xml = String::new();
            let mut container_file = zip
                .by_name("META-INF/container.xml")
                .with_context(|| "META-INF/container.xml not found in EPUB")?;
            container_file.read_to_string(&mut container_xml)?;
            Self::normalize_zip_path(&Self::find_opf_path(&container_xml)?)
        };
        debug!("Found OPF file path: {}", opf_path);

        let opf_xml = {
            let mut opf_xml = String::new();
            {
                let mut opf_file = zip
                    .by_name(&opf_path)
                    .with_context(|| format!("OPF file '{}' not found in EPUB", opf_path))?;
                opf_file.read_to_string(&mut opf_xml)?;
            }
            opf_xml
        };

        let (mut book_info, cover_id, nav_path) = Self::parse_opf_metadata(&opf_xml)?;

        let opf_parent = Path::new(&opf_path).parent();

        // If no page count from OPF metadata, try the nav document's page-list.
        if book_info.pages.is_none()
            && let Some(ref nav_rel_path) = nav_path
        {
            let resolved_nav_path = Self::resolve_relative_path(opf_parent, nav_rel_path);

            if let Ok(mut nav_file) = zip.by_name(&resolved_nav_path) {
                let mut nav_xml = String::new();
                if nav_file.read_to_string(&mut nav_xml).is_ok() {
                    book_info.pages = Self::parse_page_list(&nav_xml);
                }
            }
        }

        // Extract chapter TOC entries with fractional positions.
        book_info.chapters =
            Self::extract_chapters(&mut zip, opf_parent, &opf_xml, nav_path.as_deref());

        let (cover_path, cover_mime_type) = Self::find_cover_path(&opf_xml, &cover_id)?;
        debug!(
            "Cover image path: {:?}, MIME type: {:?}",
            cover_path, cover_mime_type
        );

        let resolved_cover_path = cover_path
            .as_deref()
            .map(|p| Self::resolve_relative_path(opf_parent, p));

        let cover_data = if let Some(ref cover_path) = resolved_cover_path {
            match zip.by_name(cover_path) {
                Ok(mut cover_file) => {
                    let mut buf = Vec::new();
                    cover_file.read_to_end(&mut buf)?;
                    Some(buf)
                }
                Err(e) => {
                    warn!("Cover image file '{}' not found: {}", cover_path, e);
                    None
                }
            }
        } else {
            None
        };

        Ok(BookInfo {
            cover_data,
            cover_mime_type,
            ..book_info
        })
    }

    fn find_opf_path(container_xml: &str) -> Result<String> {
        let mut reader = Reader::from_str(container_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    if e.local_name().as_ref() == b"rootfile" {
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            if key == b"full-path" {
                                return Ok(attr.unescape_value()?.into_owned());
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow!("Error parsing container.xml: {}", e)),
                _ => {}
            }
            buf.clear();
        }
        Err(anyhow!("No rootfile/full-path found in container.xml"))
    }

    /// Parse OPF metadata - returns (BookInfo, cover_id, nav_path)
    fn parse_opf_metadata(opf_xml: &str) -> Result<(BookInfo, Option<String>, Option<String>)> {
        let mut reader = Reader::from_str(opf_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let mut in_metadata = false;
        let mut in_manifest = false;
        let mut title = None;
        let mut authors = Vec::new();
        let mut description = None;
        let mut publisher = None;
        let mut language = None;
        let mut identifiers = Vec::new();
        let mut subjects = Vec::new();

        let mut meta_cover_id: Option<String> = None;
        let mut cal_series: Option<String> = None;
        let mut cal_series_number: Option<String> = None;
        let mut number_of_pages: Option<u32> = None;
        let mut nav_path: Option<String> = None;

        // EPUB3 collection tracking
        let mut epub3_collections: HashMap<String, String> = HashMap::new(); // id -> name
        let mut epub3_indices: HashMap<String, String> = HashMap::new(); // refines (#id) -> index

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    if local_name.as_ref() == b"metadata" {
                        in_metadata = true;
                    } else if local_name.as_ref() == b"manifest" {
                        in_manifest = true;
                    } else if in_metadata {
                        match local_name.as_ref() {
                            b"title" => {
                                if let Ok(text) = reader.read_text(e.name()) {
                                    title = Some(
                                        unescape(&text)
                                            .unwrap_or(Cow::Borrowed(&text))
                                            .into_owned(),
                                    );
                                }
                            }
                            b"creator" => {
                                if let Ok(text_content) = reader.read_text(e.name()) {
                                    authors.push(
                                        unescape(&text_content)
                                            .unwrap_or(Cow::Borrowed(&text_content))
                                            .into_owned(),
                                    );
                                }
                            }
                            b"description" => match reader.read_text(e.name()) {
                                Ok(raw_text) => {
                                    let cleaned = sanitize_html(&raw_text);

                                    let trimmed = cleaned.trim();
                                    if !trimmed.is_empty() {
                                        description = Some(trimmed.to_string());
                                    }
                                }
                                Err(e) => {
                                    debug!("Error reading description: {:?}", e);
                                }
                            },
                            b"publisher" => {
                                if let Ok(text) = reader.read_text(e.name()) {
                                    publisher = Some(
                                        unescape(&text)
                                            .unwrap_or(Cow::Borrowed(&text))
                                            .into_owned(),
                                    );
                                }
                            }
                            b"language" => {
                                if let Ok(text) = reader.read_text(e.name()) {
                                    language = Some(
                                        unescape(&text)
                                            .unwrap_or(Cow::Borrowed(&text))
                                            .into_owned(),
                                    );
                                }
                            }
                            b"identifier" => {
                                let mut scheme = None;
                                for attr in e.attributes().flatten() {
                                    let key = attr.key.as_ref();
                                    if key == b"opf:scheme" || key == b"scheme" {
                                        scheme = Some(attr.unescape_value()?.into_owned());
                                    }
                                }
                                if let Ok(text_content) = reader.read_text(e.name()) {
                                    let value = unescape(&text_content)
                                        .unwrap_or(Cow::Borrowed(&text_content))
                                        .into_owned();
                                    let (final_scheme, final_value) = if let Some(s) = scheme {
                                        (s, value.clone())
                                    } else if let Some(colon_pos) = value.find(':') {
                                        let potential_scheme = &value[..colon_pos];
                                        let potential_value = &value[colon_pos + 1..];
                                        (potential_scheme.to_string(), potential_value.to_string())
                                    } else {
                                        ("unknown".to_string(), value.clone())
                                    };
                                    identifiers.push(Identifier::new(final_scheme, final_value));
                                }
                            }
                            b"subject" => {
                                if let Ok(text_content) = reader.read_text(e.name()) {
                                    let subject = unescape(&text_content)
                                        .unwrap_or(Cow::Borrowed(&text_content))
                                        .into_owned();
                                    if !subject.is_empty() {
                                        subjects.push(subject);
                                    }
                                }
                            }
                            b"meta" => {
                                let mut property = None;
                                let mut id = None;
                                let mut refines = None;

                                let mut name_attr = None;
                                let mut content_attr = None;

                                for attr in e.attributes().flatten() {
                                    let key = attr.key.as_ref();
                                    match key {
                                        b"property" => {
                                            property = Some(attr.unescape_value()?.into_owned())
                                        }
                                        b"id" => id = Some(attr.unescape_value()?.into_owned()),
                                        b"refines" => {
                                            refines = Some(attr.unescape_value()?.into_owned())
                                        }
                                        b"name" => {
                                            name_attr = Some(attr.unescape_value()?.into_owned())
                                        }
                                        b"content" => {
                                            content_attr = Some(attr.unescape_value()?.into_owned())
                                        }
                                        _ => {}
                                    }
                                }

                                if let (Some(n), Some(c)) = (&name_attr, &content_attr) {
                                    if n == "cover" {
                                        meta_cover_id = Some(c.clone());
                                    }
                                    if n == "calibre:series" {
                                        cal_series = Some(c.clone());
                                    }
                                    if n == "calibre:series_index" {
                                        cal_series_number = Some(c.clone());
                                    }
                                }

                                if let Some(prop) = property {
                                    if prop == "belongs-to-collection" {
                                        if let (Ok(text_content), Some(i)) =
                                            (reader.read_text(e.name()), id)
                                        {
                                            epub3_collections.insert(
                                                i,
                                                unescape(&text_content)
                                                    .unwrap_or(Cow::Borrowed(&text_content))
                                                    .into_owned(),
                                            );
                                        }
                                    } else if prop == "group-position" {
                                        if let (Ok(text_content), Some(r)) =
                                            (reader.read_text(e.name()), refines)
                                        {
                                            let clean_refines = r.trim_start_matches('#');
                                            epub3_indices.insert(
                                                clean_refines.to_string(),
                                                unescape(&text_content)
                                                    .unwrap_or(Cow::Borrowed(&text_content))
                                                    .into_owned(),
                                            );
                                        }
                                    } else if prop == "schema:numberOfPages"
                                        && let Ok(text_content) = reader.read_text(e.name())
                                        && let Ok(pages) = text_content.trim().parse::<u32>()
                                    {
                                        number_of_pages = Some(pages);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    if in_metadata && local_name.as_ref() == b"meta" {
                        let mut name_attr = None;
                        let mut content_attr = None;
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            match key {
                                b"name" => name_attr = Some(attr.unescape_value()?.into_owned()),
                                b"content" => {
                                    content_attr = Some(attr.unescape_value()?.into_owned())
                                }
                                _ => {}
                            }
                        }
                        if let (Some(n), Some(c)) = (name_attr, content_attr) {
                            if n == "cover" {
                                meta_cover_id = Some(c);
                            } else if n == "calibre:series" {
                                cal_series = Some(c);
                            } else if n == "calibre:series_index" {
                                cal_series_number = Some(c);
                            }
                        }
                    } else if in_manifest && local_name.as_ref() == b"item" {
                        // Look for nav document in manifest
                        let mut href = None;
                        let mut properties = None;
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            if key == b"href" {
                                if let Ok(h) = attr.unescape_value() {
                                    href = Some(h.into_owned());
                                }
                            } else if key == b"properties"
                                && let Ok(p) = attr.unescape_value()
                            {
                                properties = Some(p.into_owned());
                            }
                        }
                        if let (Some(h), Some(p)) = (href, properties)
                            && p.contains("nav")
                        {
                            nav_path = Some(h);
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.local_name().as_ref() == b"metadata" {
                        in_metadata = false;
                    } else if e.local_name().as_ref() == b"manifest" {
                        in_manifest = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow!("Error parsing OPF: {}", e)),
                _ => {}
            }
            buf.clear();
        }

        let (series, series_number) = if !epub3_collections.is_empty() {
            let mut best = None;
            for (id, name) in &epub3_collections {
                if let Some(idx) = epub3_indices.get(id) {
                    best = Some((Some(name.clone()), Some(idx.clone())));
                    break;
                }
            }
            best.unwrap_or_else(|| {
                if let Some((_, name)) = epub3_collections.iter().next() {
                    (Some(name.clone()), None)
                } else {
                    (None, None)
                }
            })
        } else {
            (cal_series, cal_series_number)
        };

        let cover_id = meta_cover_id;
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
            pages: number_of_pages,
            chapters: Vec::new(),
            cover_data: None,
            cover_mime_type: None,
        };
        Ok((info, cover_id, nav_path))
    }

    fn find_cover_path(
        opf_xml: &str,
        cover_id: &Option<String>,
    ) -> Result<(Option<String>, Option<String>)> {
        let mut reader = Reader::from_str(opf_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    if e.local_name().as_ref() == b"item" {
                        let mut id = None;
                        let mut href = None;
                        let mut media_type = None;
                        let mut properties = None;

                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            if key == b"id" {
                                id = Some(attr.unescape_value()?.into_owned());
                            } else if key == b"href" {
                                href = Some(attr.unescape_value()?.into_owned());
                            } else if key == b"media-type" {
                                media_type = Some(attr.unescape_value()?.into_owned());
                            } else if key == b"properties" {
                                properties = Some(attr.unescape_value()?.into_owned());
                            }
                        }

                        if let (Some(href), Some(media_type)) = (href, media_type)
                            && media_type.starts_with("image/")
                        {
                            // Check if this is the cover using EPUB 3.0 properties
                            if let Some(props) = &properties
                                && props.contains("cover-image")
                            {
                                return Ok((Some(href), Some(media_type)));
                            }

                            // Check if this matches the cover_id from meta tags (EPUB 2.0 style)
                            if let (Some(cover_id), Some(id)) = (cover_id, &id)
                                && id == cover_id
                            {
                                return Ok((Some(href), Some(media_type)));
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow!("Error parsing manifest for cover: {}", e)),
                _ => {}
            }
            buf.clear();
        }
        Ok((None, None))
    }

    /// Resolve a relative path against the OPF parent directory inside the ZIP.
    fn resolve_relative_path(opf_parent: Option<&Path>, rel_path: &str) -> String {
        let joined = if let Some(parent) = opf_parent {
            parent.join(rel_path)
        } else {
            Path::new(rel_path).to_path_buf()
        };
        Self::normalize_zip_path(&joined.to_string_lossy().replace('\\', "/"))
    }

    /// Parse page-list from EPUB3 navigation document to get page count
    /// The page-list is a nav element with epub:type="page-list" containing anchor elements
    fn parse_page_list(nav_xml: &str) -> Option<u32> {
        let mut reader = Reader::from_str(nav_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        let mut in_page_list = false;
        let mut page_count = 0u32;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();

                    // Check for nav with epub:type="page-list"
                    if local_name.as_ref() == b"nav" {
                        for attr in e.attributes().flatten() {
                            // Look for epub:type attribute (either with or without namespace prefix)
                            let key = attr.key.as_ref();
                            if (key == b"epub:type" || key.ends_with(b":type") || key == b"type")
                                && let Ok(val) = attr.unescape_value()
                                && val.contains("page-list")
                            {
                                in_page_list = true;
                            }
                        }
                    }

                    // Count anchor elements in page-list (each represents a page)
                    if in_page_list && local_name.as_ref() == b"a" {
                        page_count += 1;
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.local_name().as_ref() == b"nav" {
                        in_page_list = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        if page_count > 0 {
            Some(page_count)
        } else {
            None
        }
    }

    // ── Chapter / TOC extraction ─────────────────────────────────────────

    /// Orchestrate chapter extraction from an EPUB: parse spine, try EPUB3
    /// nav TOC first, fall back to NCX, and compute byte-weighted fractions.
    fn extract_chapters(
        zip: &mut ZipArchive<File>,
        opf_parent: Option<&Path>,
        opf_xml: &str,
        nav_path: Option<&str>,
    ) -> Vec<ChapterEntry> {
        let (spine_hrefs, ncx_path) = Self::parse_opf_spine(opf_xml);
        if spine_hrefs.is_empty() {
            return Vec::new();
        }

        let spine_byte_map = Self::build_spine_byte_map(zip, opf_parent, &spine_hrefs);

        // Try EPUB3 nav document TOC first.
        let mut toc_entries: Vec<(String, String)> = Vec::new();
        if let Some(nav_rel) = nav_path {
            let resolved = Self::resolve_relative_path(opf_parent, nav_rel);
            let nav_parent = Path::new(nav_rel).parent().map(Path::to_path_buf);
            if let Ok(mut f) = zip.by_name(&resolved) {
                let mut xml = String::new();
                if f.read_to_string(&mut xml).is_ok() {
                    toc_entries = Self::parse_nav_toc(&xml);
                    // Nav hrefs are relative to the nav document — resolve them
                    // to be relative to the OPF (same base as spine hrefs).
                    if let Some(ref nav_dir) = nav_parent {
                        for entry in &mut toc_entries {
                            entry.0 = Self::normalize_zip_path(
                                &nav_dir.join(&entry.0).to_string_lossy().replace('\\', "/"),
                            );
                        }
                    }
                }
            }
        }

        // Fall back to EPUB2 NCX if no EPUB3 TOC was found.
        if toc_entries.is_empty()
            && let Some(ref ncx_rel) = ncx_path
        {
            let resolved = Self::resolve_relative_path(opf_parent, ncx_rel);
            let ncx_parent = Path::new(ncx_rel.as_str()).parent().map(Path::to_path_buf);
            if let Ok(mut f) = zip.by_name(&resolved) {
                let mut xml = String::new();
                if f.read_to_string(&mut xml).is_ok() {
                    toc_entries = Self::parse_ncx_toc(&xml);
                    // NCX hrefs are relative to the NCX file location.
                    if let Some(ref ncx_dir) = ncx_parent {
                        for entry in &mut toc_entries {
                            entry.0 = Self::normalize_zip_path(
                                &ncx_dir.join(&entry.0).to_string_lossy().replace('\\', "/"),
                            );
                        }
                    }
                }
            }
        }

        if toc_entries.is_empty() {
            return Vec::new();
        }

        Self::map_toc_to_fractions(&toc_entries, &spine_byte_map)
    }

    /// Parse the OPF spine to get the ordered list of content document hrefs
    /// and the NCX path (for EPUB2 fallback).
    fn parse_opf_spine(opf_xml: &str) -> (Vec<String>, Option<String>) {
        let mut reader = Reader::from_str(opf_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        let mut manifest_map: HashMap<String, String> = HashMap::new(); // id -> href
        let mut spine_idrefs: Vec<String> = Vec::new();
        let mut ncx_toc_id: Option<String> = None;
        let mut in_manifest = false;
        let mut in_spine = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == b"manifest" {
                        in_manifest = true;
                    } else if local.as_ref() == b"spine" {
                        in_spine = true;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"toc"
                                && let Ok(v) = attr.unescape_value()
                            {
                                ncx_toc_id = Some(v.into_owned());
                            }
                        }
                    } else if in_manifest && local.as_ref() == b"item" {
                        let mut id = None;
                        let mut href = None;
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"id" => {
                                    if let Ok(v) = attr.unescape_value() {
                                        id = Some(v.into_owned());
                                    }
                                }
                                b"href" => {
                                    if let Ok(v) = attr.unescape_value() {
                                        href = Some(v.into_owned());
                                    }
                                }
                                _ => {}
                            }
                        }
                        if let (Some(id), Some(href)) = (id, href) {
                            manifest_map.insert(id, href);
                        }
                    } else if in_spine && local.as_ref() == b"itemref" {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"idref"
                                && let Ok(v) = attr.unescape_value()
                            {
                                spine_idrefs.push(v.into_owned());
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == b"manifest" {
                        in_manifest = false;
                    } else if local.as_ref() == b"spine" {
                        in_spine = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        // Resolve spine idrefs to hrefs via manifest.
        let spine_hrefs: Vec<String> = spine_idrefs
            .iter()
            .filter_map(|idref| manifest_map.get(idref).cloned())
            .collect();

        // Resolve NCX toc id to href.
        let ncx_path = ncx_toc_id.and_then(|id| manifest_map.get(&id).cloned());

        (spine_hrefs, ncx_path)
    }

    /// Look up each spine item's uncompressed byte size from the ZIP directory.
    fn build_spine_byte_map(
        zip: &mut ZipArchive<File>,
        opf_parent: Option<&Path>,
        spine_hrefs: &[String],
    ) -> Vec<(String, u64)> {
        spine_hrefs
            .iter()
            .map(|href| {
                let resolved = Self::resolve_relative_path(opf_parent, href);
                let size = zip.by_name(&resolved).map(|f| f.size()).unwrap_or(0);
                (href.clone(), size)
            })
            .collect()
    }

    /// Parse `<nav epub:type="toc">` from an EPUB3 navigation document.
    /// Returns (href, title) pairs for all anchors (including nested chapters).
    fn parse_nav_toc(nav_xml: &str) -> Vec<(String, String)> {
        let mut reader = Reader::from_str(nav_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        let mut in_toc = false;
        let mut depth = 0u32;
        let mut entries: Vec<(String, String)> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == b"nav" {
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            if (key == b"epub:type" || key.ends_with(b":type") || key == b"type")
                                && let Ok(val) = attr.unescape_value()
                                && val.contains("toc")
                            {
                                in_toc = true;
                            }
                        }
                    } else if in_toc && local.as_ref() == b"ol" {
                        depth += 1;
                    } else if in_toc && depth >= 1 && local.as_ref() == b"a" {
                        // Extract href and text from anchors at any nesting depth.
                        let mut href = None;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"href"
                                && let Ok(v) = attr.unescape_value()
                            {
                                href = Some(v.into_owned());
                            }
                        }
                        if let (Some(href), Ok(text)) = (href, reader.read_text(e.name())) {
                            let title =
                                unescape(&text).unwrap_or(Cow::Borrowed(&text)).into_owned();
                            let title = title.trim().to_string();
                            if !title.is_empty() {
                                entries.push((href, title));
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == b"nav" {
                        in_toc = false;
                        depth = 0;
                    } else if in_toc && local.as_ref() == b"ol" {
                        depth = depth.saturating_sub(1);
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        entries
    }

    /// Parse `<navMap>` from an EPUB2 NCX file.
    /// Returns (src, title) pairs for all navPoints (including nested).
    fn parse_ncx_toc(ncx_xml: &str) -> Vec<(String, String)> {
        let mut reader = Reader::from_str(ncx_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        let mut in_nav_map = false;
        let mut nav_point_depth = 0u32;
        let mut current_title: Option<String> = None;
        let mut current_src: Option<String> = None;
        let mut entries: Vec<(String, String)> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == b"navMap" {
                        in_nav_map = true;
                    } else if in_nav_map && local.as_ref() == b"navPoint" {
                        nav_point_depth += 1;
                        current_title = None;
                        current_src = None;
                    } else if in_nav_map
                        && nav_point_depth >= 1
                        && local.as_ref() == b"text"
                        && let Ok(text) = reader.read_text(e.name())
                    {
                        current_title = Some(
                            unescape(&text)
                                .unwrap_or(Cow::Borrowed(&text))
                                .trim()
                                .to_string(),
                        );
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let local = e.local_name();
                    if in_nav_map && nav_point_depth >= 1 && local.as_ref() == b"content" {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"src"
                                && let Ok(v) = attr.unescape_value()
                            {
                                current_src = Some(v.into_owned());
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == b"navMap" {
                        in_nav_map = false;
                    } else if in_nav_map && local.as_ref() == b"navPoint" {
                        if let (Some(src), Some(title)) = (current_src.take(), current_title.take())
                            && !title.is_empty()
                        {
                            entries.push((src, title));
                        }
                        nav_point_depth = nav_point_depth.saturating_sub(1);
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        entries
    }

    /// Map TOC entries to fractional positions using spine byte-size weighting.
    fn map_toc_to_fractions(
        toc_entries: &[(String, String)],
        spine_byte_map: &[(String, u64)],
    ) -> Vec<ChapterEntry> {
        let total_bytes: u64 = spine_byte_map.iter().map(|(_, s)| s).sum();
        if total_bytes == 0 {
            return Vec::new();
        }

        // Build cumulative byte offsets and an index for fast href lookup.
        let mut cumulative: Vec<u64> = Vec::with_capacity(spine_byte_map.len());
        let mut running = 0u64;
        for (_, size) in spine_byte_map {
            cumulative.push(running);
            running += size;
        }
        let spine_index: HashMap<&str, usize> = spine_byte_map
            .iter()
            .enumerate()
            .map(|(i, (href, _))| (href.as_str(), i))
            .collect();

        let mut chapters = Vec::new();
        for (href, title) in toc_entries {
            // Strip fragment (e.g. "chapter.xhtml#sec1" → "chapter.xhtml").
            let file_part = href.split('#').next().unwrap_or(href);
            if let Some(&idx) = spine_index.get(file_part) {
                let position = cumulative[idx] as f64 / total_bytes as f64;
                chapters.push(ChapterEntry {
                    title: title.clone(),
                    position,
                });
            }
        }

        chapters
    }
}
