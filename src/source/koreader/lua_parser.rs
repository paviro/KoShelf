use crate::shelf::models::{
    Annotation, BookStatus, DocProps, FlowPoint, KoReaderMetadata, ReaderPresentation, Stats,
    Summary,
};
use anyhow::{Context, Result, anyhow};
use log::{debug, warn};
use mlua::{ChunkMode, Lua, LuaOptions, StdLib, Table, Value};
use std::fs;
use std::path::Path;

/// Parses KOReader `.sdr/metadata.*.lua` sidecar files into [`KoReaderMetadata`].
pub struct LuaParser {
    lua: Lua,
}

impl Default for LuaParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaParser {
    pub fn new() -> Self {
        Self {
            lua: Lua::new_with(StdLib::NONE, LuaOptions::default())
                .expect("Failed to create sandboxed Lua state"),
        }
    }

    /// Evaluate a KOReader Lua metadata file and extract its fields.
    pub fn parse(&self, lua_path: &Path) -> Result<KoReaderMetadata> {
        debug!("Parsing Lua metadata: {:?}", lua_path);

        let bytes = fs::read(lua_path)
            .with_context(|| format!("Failed to read Lua file: {:?}", lua_path))?;
        let content = String::from_utf8_lossy(&bytes);

        let value: Value = self
            .lua
            .load(&*content)
            .set_mode(ChunkMode::Text)
            .eval()
            .map_err(|e| anyhow!("Failed to parse Lua file {:?}: {}", lua_path, e))?;

        match value {
            Value::Table(table) => self.parse_metadata_table(table),
            _ => Err(anyhow!("Expected Lua file to return a table")),
        }
    }

    fn parse_metadata_table(&self, table: Table) -> Result<KoReaderMetadata> {
        let annotations = self.parse_annotations(&table)?;
        let reader_presentation = self.parse_reader_presentation(&table)?;
        let doc_pages = self.get_optional_u32(&table, "doc_pages")?;
        let doc_path = self.get_optional_string(&table, "doc_path")?;
        let doc_props = self.parse_doc_props(&table)?;
        let handmade_flows_enabled = self.get_optional_bool(&table, "handmade_flows_enabled")?;
        let handmade_flow_points = self.parse_flow_points(&table)?;
        let pagemap_use_page_labels = self.get_optional_bool(&table, "pagemap_use_page_labels")?;
        let pagemap_chars_per_synthetic_page =
            self.get_optional_u32(&table, "pagemap_chars_per_synthetic_page")?;
        let pagemap_doc_pages = self.get_optional_u32(&table, "pagemap_doc_pages")?;
        let pagemap_current_page_label =
            self.get_optional_string(&table, "pagemap_current_page_label")?;
        let pagemap_last_page_label =
            self.get_optional_string(&table, "pagemap_last_page_label")?;
        let partial_md5_checksum = self.get_optional_string(&table, "partial_md5_checksum")?;
        let percent_finished = self.get_optional_f64(&table, "percent_finished")?;
        let stats = self.parse_stats(&table)?;
        let summary = self.parse_summary(&table)?;
        let text_lang = self.get_optional_string(&table, "text_lang")?;

        Ok(KoReaderMetadata {
            annotations,
            reader_presentation,
            doc_pages,
            doc_path,
            doc_props,
            handmade_flows_enabled,
            handmade_flow_points,
            pagemap_use_page_labels,
            pagemap_chars_per_synthetic_page,
            pagemap_doc_pages,
            pagemap_current_page_label,
            pagemap_last_page_label,
            partial_md5_checksum,
            percent_finished,
            stats,
            summary,
            text_lang,
        })
    }

    fn parse_reader_presentation(&self, table: &Table) -> Result<Option<ReaderPresentation>> {
        let reader_presentation = ReaderPresentation {
            font_face: self.get_optional_string(table, "font_face")?,
            font_size_pt: self.get_optional_f64(table, "copt_font_size")?,
            line_spacing_percentage: self.get_optional_u32(table, "copt_line_spacing")?,
            horizontal_margins: self.get_optional_u32_pair(table, "copt_h_page_margins")?,
            top_margin: self.get_optional_u32(table, "copt_t_page_margin")?,
            bottom_margin: self.get_optional_u32(table, "copt_b_page_margin")?,
            embedded_fonts: self.get_optional_bool_or_flag(table, "copt_embedded_fonts")?,
            hyphenation: self.get_optional_bool_or_flag(table, "hyphenation")?,
            floating_punctuation: self.get_optional_bool_or_flag(table, "floating_punctuation")?,
            word_spacing: self.get_optional_u32_pair(table, "copt_word_spacing")?,
        };

        Ok((!reader_presentation.is_empty()).then_some(reader_presentation))
    }

    fn parse_flow_points(&self, table: &Table) -> Result<Vec<FlowPoint>> {
        let mut flow_points = Vec::new();

        if let Ok(Value::Table(points_table)) = table.get("handmade_flow_points") {
            let mut index = 1;
            while let Ok(Value::Table(point_table)) = points_table.get(index) {
                let hidden = self
                    .get_optional_bool(&point_table, "hidden")?
                    .unwrap_or(false);
                let page = self.get_optional_u32(&point_table, "page")?;
                if let Some(page) = page {
                    flow_points.push(FlowPoint { hidden, page });
                }
                index += 1;
            }
        }

        Ok(flow_points)
    }

    fn parse_annotations(&self, table: &Table) -> Result<Vec<Annotation>> {
        let mut annotations = Vec::new();

        if let Ok(Value::Table(annotations_table)) = table.get("annotations") {
            // Lua arrays are 1-indexed
            let mut index = 1;
            while let Ok(Value::Table(annotation_table)) = annotations_table.get(index) {
                let annotation = self.parse_annotation(annotation_table)?;
                annotations.push(annotation);
                index += 1;
            }
        }

        Ok(annotations)
    }

    fn parse_annotation(&self, table: Table) -> Result<Annotation> {
        Ok(Annotation {
            chapter: self.get_optional_string(&table, "chapter")?,
            datetime: self.get_optional_string(&table, "datetime")?,
            datetime_updated: self.get_optional_string(&table, "datetime_updated")?,
            pageno: self.get_optional_u32(&table, "pageno")?,
            pos0: self.get_optional_string(&table, "pos0")?,
            pos1: self.get_optional_string(&table, "pos1")?,
            text: self.get_optional_string(&table, "text")?,
            note: self.get_optional_string(&table, "note")?,
            color: self.get_optional_string(&table, "color")?,
            drawer: self.get_optional_string(&table, "drawer")?,
        })
    }

    fn parse_doc_props(&self, table: &Table) -> Result<Option<DocProps>> {
        if let Ok(Value::Table(props_table)) = table.get("doc_props") {
            Ok(Some(DocProps {
                authors: self.get_optional_string(&props_table, "authors")?,
                description: self.get_optional_string(&props_table, "description")?,
                identifiers: self.get_optional_string(&props_table, "identifiers")?,
                keywords: self.get_optional_string(&props_table, "keywords")?,
                language: self.get_optional_string(&props_table, "language")?,
                title: self.get_optional_string(&props_table, "title")?,
            }))
        } else {
            Ok(None)
        }
    }

    fn parse_stats(&self, table: &Table) -> Result<Option<Stats>> {
        if let Ok(Value::Table(stats_table)) = table.get("stats") {
            Ok(Some(Stats {
                authors: self.get_optional_string(&stats_table, "authors")?,
                highlights: self.get_optional_u32(&stats_table, "highlights")?,
                language: self.get_optional_string(&stats_table, "language")?,
                notes: self.get_optional_u32(&stats_table, "notes")?,
                pages: self.get_optional_u32(&stats_table, "pages")?,
                series: self.get_optional_string(&stats_table, "series")?,
                title: self.get_optional_string(&stats_table, "title")?,
            }))
        } else {
            Ok(None)
        }
    }

    fn parse_summary(&self, table: &Table) -> Result<Option<Summary>> {
        if let Ok(Value::Table(summary_table)) = table.get("summary") {
            let modified = self.get_optional_string(&summary_table, "modified")?;
            let note = self.get_optional_string(&summary_table, "note")?;
            let rating = self.get_optional_u32(&summary_table, "rating")?;
            let status = self
                .get_optional_string(&summary_table, "status")?
                .map(|s| match s.as_str() {
                    "reading" => BookStatus::Reading,
                    "complete" => BookStatus::Complete,
                    "abandoned" => BookStatus::Abandoned,
                    _ => BookStatus::Unknown,
                })
                .unwrap_or(BookStatus::Unknown);

            Ok(Some(Summary {
                modified,
                note,
                rating,
                status,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_optional_string(&self, table: &Table, key: &str) -> Result<Option<String>> {
        match table.get(key) {
            Ok(Value::String(s)) => match s.to_str() {
                Ok(string_val) => Ok(Some(string_val.to_string())),
                Err(e) => {
                    warn!("Failed to convert string for key '{}': {}", key, e);
                    Ok(None)
                }
            },
            Ok(Value::Nil) => Ok(None),
            Ok(_) => {
                warn!("Expected string for key '{}', got different type", key);
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }

    fn get_optional_u32(&self, table: &Table, key: &str) -> Result<Option<u32>> {
        match table.get(key) {
            Ok(Value::Integer(i)) => match u32::try_from(i) {
                Ok(value) => Ok(Some(value)),
                Err(_) => {
                    warn!("Ignoring out-of-range integer for key '{}': {}", key, i);
                    Ok(None)
                }
            },
            Ok(Value::Number(n)) => Ok(self.parse_u32_number(n, &format!("key '{}'", key))),
            Ok(Value::Nil) => Ok(None),
            Ok(_) => {
                warn!("Expected number for key '{}', got different type", key);
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }

    fn get_optional_bool(&self, table: &Table, key: &str) -> Result<Option<bool>> {
        match table.get(key) {
            Ok(Value::Boolean(value)) => Ok(Some(value)),
            Ok(Value::Nil) => Ok(None),
            Ok(_) => {
                warn!("Expected boolean for key '{}', got different type", key);
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }

    fn get_optional_bool_or_flag(&self, table: &Table, key: &str) -> Result<Option<bool>> {
        match table.get(key) {
            Ok(Value::Boolean(value)) => Ok(Some(value)),
            Ok(Value::Integer(i)) => {
                if i == 0 {
                    Ok(Some(false))
                } else if i == 1 {
                    Ok(Some(true))
                } else {
                    warn!(
                        "Ignoring integer flag other than 0/1 for key '{}': {}",
                        key, i
                    );
                    Ok(None)
                }
            }
            Ok(Value::Number(n)) => {
                if !n.is_finite() {
                    warn!("Ignoring non-finite number for key '{}': {}", key, n);
                    return Ok(None);
                }

                if (n - 0.0).abs() < f64::EPSILON {
                    Ok(Some(false))
                } else if (n - 1.0).abs() < f64::EPSILON {
                    Ok(Some(true))
                } else {
                    warn!(
                        "Ignoring numeric flag other than 0/1 for key '{}': {}",
                        key, n
                    );
                    Ok(None)
                }
            }
            Ok(Value::Nil) => Ok(None),
            Ok(_) => {
                warn!(
                    "Expected bool/int flag for key '{}', got different type",
                    key
                );
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }

    fn get_optional_u32_pair(&self, table: &Table, key: &str) -> Result<Option<[u32; 2]>> {
        let values_table = match table.get(key) {
            Ok(Value::Table(values)) => values,
            Ok(Value::Nil) => return Ok(None),
            Ok(_) => {
                warn!("Expected table for key '{}', got different type", key);
                return Ok(None);
            }
            Err(_) => return Ok(None),
        };

        let first = self.get_u32_from_table_index(&values_table, key, 1)?;
        let second = self.get_u32_from_table_index(&values_table, key, 2)?;

        match (first, second) {
            (Some(a), Some(b)) => Ok(Some([a, b])),
            (None, None) => Ok(None),
            _ => {
                warn!(
                    "Ignoring partially defined two-value table for key '{}'",
                    key
                );
                Ok(None)
            }
        }
    }

    fn get_u32_from_table_index(
        &self,
        table: &Table,
        key: &str,
        index: i64,
    ) -> Result<Option<u32>> {
        match table.get(index) {
            Ok(Value::Integer(i)) => match u32::try_from(i) {
                Ok(value) => Ok(Some(value)),
                Err(_) => {
                    warn!(
                        "Ignoring out-of-range integer for key '{}' at index {}: {}",
                        key, index, i
                    );
                    Ok(None)
                }
            },
            Ok(Value::Number(n)) => {
                Ok(self.parse_u32_number(n, &format!("key '{}' at index {}", key, index)))
            }
            Ok(Value::Nil) => Ok(None),
            Ok(_) => {
                warn!(
                    "Expected number in table for key '{}' at index {}, got different type",
                    key, index
                );
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }

    fn parse_u32_number(&self, value: f64, context: &str) -> Option<u32> {
        if !value.is_finite() {
            warn!("Ignoring non-finite number for {}: {}", context, value);
            return None;
        }

        if value < 0.0 || value > u32::MAX as f64 {
            warn!("Ignoring out-of-range number for {}: {}", context, value);
            return None;
        }

        if value.trunc() != value {
            warn!("Ignoring non-integer number for {}: {}", context, value);
            return None;
        }

        Some(value as u32)
    }

    fn get_optional_f64(&self, table: &Table, key: &str) -> Result<Option<f64>> {
        match table.get(key) {
            Ok(Value::Integer(i)) => Ok(Some(i as f64)),
            Ok(Value::Number(n)) => Ok(Some(n)),
            Ok(Value::Nil) => Ok(None),
            Ok(_) => {
                warn!("Expected number for key '{}', got different type", key);
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::LuaParser;

    #[test]
    fn ignores_fractional_numbers_for_u32_fields() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let lua_path = temp_dir.path().join("metadata.epub.lua");
        fs::write(
            &lua_path,
            r#"return {
                font_face = "Noto Serif",
                copt_line_spacing = 110.5,
                copt_h_page_margins = { 20.5, 20 },
                copt_t_page_margin = 10.2,
            }"#,
        )
        .expect("lua fixture should be written");

        let parser = LuaParser::new();
        let metadata = parser
            .parse(&lua_path)
            .expect("metadata should parse successfully");

        let presentation = metadata
            .reader_presentation
            .expect("reader presentation should still be present");

        assert_eq!(presentation.font_face.as_deref(), Some("Noto Serif"));
        assert_eq!(presentation.line_spacing_percentage, None);
        assert_eq!(presentation.horizontal_margins, None);
        assert_eq!(presentation.top_margin, None);
    }

    #[test]
    fn parses_handmade_flow_points() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let lua_path = temp_dir.path().join("metadata.epub.lua");
        fs::write(
            &lua_path,
            r#"return {
                doc_pages = 1000,
                handmade_flows_enabled = true,
                handmade_flow_points = {
                    [1] = {
                        hidden = true,
                        page = 540,
                        xpointer = "/body/DocFragment[23]/body/div/p[183]/span/text().206",
                    },
                },
            }"#,
        )
        .expect("lua fixture should be written");

        let parser = LuaParser::new();
        let metadata = parser
            .parse(&lua_path)
            .expect("metadata should parse successfully");

        assert_eq!(metadata.handmade_flows_enabled, Some(true));
        assert_eq!(metadata.handmade_flow_points.len(), 1);
        assert!(metadata.handmade_flow_points[0].hidden);
        assert_eq!(metadata.handmade_flow_points[0].page, 540);

        // 1000 - 540 + 1 = 461 hidden pages (from page 540 to end)
        assert_eq!(metadata.hidden_flow_pages(), Some(461));
    }

    #[test]
    fn parses_multiple_flow_points() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let lua_path = temp_dir.path().join("metadata.epub.lua");
        fs::write(
            &lua_path,
            r#"return {
                doc_pages = 800,
                handmade_flows_enabled = true,
                handmade_flow_points = {
                    [1] = { hidden = true, page = 100 },
                    [2] = { hidden = false, page = 150 },
                    [3] = { hidden = true, page = 700 },
                },
            }"#,
        )
        .expect("lua fixture should be written");

        let parser = LuaParser::new();
        let metadata = parser
            .parse(&lua_path)
            .expect("metadata should parse successfully");

        assert_eq!(metadata.handmade_flow_points.len(), 3);
        // 50 hidden (100..150) + 101 hidden (700..800 inclusive) = 151
        assert_eq!(metadata.hidden_flow_pages(), Some(151));
    }

    #[test]
    fn parses_file_with_invalid_utf8() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let lua_path = temp_dir.path().join("metadata.epub.lua");

        // Simulate a truncated UTF-8 sequence: "Fran\xc3..." where \xc3 is the
        // start of "ç" (U+00E7, encoded as \xc3\xa7) but the continuation byte
        // is missing — KoReader can truncate highlight text mid-character.
        let mut content: Vec<u8> = Vec::new();
        content.extend_from_slice(b"return {\n");
        content.extend_from_slice(b"    [\"annotations\"] = {\n");
        content.extend_from_slice(b"        [1] = {\n");
        content.extend_from_slice(b"            [\"text\"] = \"Fran");
        content.push(0xc3); // leading byte of ç without continuation
        content.extend_from_slice(b"...\",\n");
        content.extend_from_slice(b"        },\n");
        content.extend_from_slice(b"    },\n");
        content.extend_from_slice(b"}");

        fs::write(&lua_path, &content).expect("lua fixture should be written");

        let parser = LuaParser::new();
        let metadata = parser
            .parse(&lua_path)
            .expect("should parse despite invalid UTF-8");

        assert_eq!(metadata.annotations.len(), 1);
        // The replacement character replaces the broken byte
        let text = metadata.annotations[0].text.as_deref().unwrap();
        assert!(text.starts_with("Fran"));
        assert!(text.ends_with("..."));
    }

    #[test]
    fn no_flow_points_returns_empty_vec() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let lua_path = temp_dir.path().join("metadata.epub.lua");
        fs::write(
            &lua_path,
            r#"return {
                doc_pages = 500,
            }"#,
        )
        .expect("lua fixture should be written");

        let parser = LuaParser::new();
        let metadata = parser
            .parse(&lua_path)
            .expect("metadata should parse successfully");

        assert_eq!(metadata.handmade_flows_enabled, None);
        assert!(metadata.handmade_flow_points.is_empty());
        assert_eq!(metadata.hidden_flow_pages(), None);
    }
}
