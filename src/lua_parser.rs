use crate::models::*;
use anyhow::{Result, Context};
use std::path::Path;
use std::fs;
use mlua::{Lua, Value, Table};
use log::{debug, warn};

pub struct LuaParser {
    lua: Lua,
}

impl LuaParser {
    pub fn new() -> Self {
        Self {
            lua: Lua::new(),
        }
    }
    
    pub async fn parse(&self, lua_path: &Path) -> Result<KoReaderMetadata> {
        debug!("Parsing Lua metadata: {:?}", lua_path);
        
        let content = fs::read_to_string(lua_path)
            .with_context(|| format!("Failed to read Lua file: {:?}", lua_path))?;
        
        // Execute the Lua code which returns a table
        let value: Value = self.lua.load(&content).eval()
            .with_context(|| format!("Failed to parse Lua file: {:?}", lua_path))?;
        
        match value {
            Value::Table(table) => self.parse_metadata_table(table),
            _ => anyhow::bail!("Expected Lua file to return a table"),
        }
    }
    
    fn parse_metadata_table(&self, table: Table) -> Result<KoReaderMetadata> {
        let annotations = self.parse_annotations(&table)?;
        let doc_pages = self.get_optional_u32(&table, "doc_pages")?;
        let doc_path = self.get_optional_string(&table, "doc_path")?;
        let doc_props = self.parse_doc_props(&table)?;
        let partial_md5_checksum = self.get_optional_string(&table, "partial_md5_checksum")?;
        let percent_finished = self.get_optional_f64(&table, "percent_finished")?;
        let stats = self.parse_stats(&table)?;
        let summary = self.parse_summary(&table)?;
        let text_lang = self.get_optional_string(&table, "text_lang")?;
        
        Ok(KoReaderMetadata {
            annotations,
            doc_pages,
            doc_path,
            doc_props,
            partial_md5_checksum,
            percent_finished,
            stats,
            summary,
            text_lang,
        })
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
            color: self.get_optional_string(&table, "color")?,
            datetime: self.get_optional_string(&table, "datetime")?,
            drawer: self.get_optional_string(&table, "drawer")?,
            page: self.get_optional_string(&table, "page")?,
            pageno: self.get_optional_u32(&table, "pageno")?,
            pos0: self.get_optional_string(&table, "pos0")?,
            pos1: self.get_optional_string(&table, "pos1")?,
            text: self.get_required_string(&table, "text")?,
            note: self.get_optional_string(&table, "note")?,
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
            let status = self.get_optional_string(&summary_table, "status")?
                .map(|s| match s.as_str() {
                    "reading" => BookStatus::Reading,
                    "complete" => BookStatus::Complete,
                    _ => BookStatus::Unknown,
                })
                .unwrap_or(BookStatus::Unknown);
            
            Ok(Some(Summary { modified, note, rating, status }))
        } else {
            Ok(None)
        }
    }
    
    fn get_optional_string(&self, table: &Table, key: &str) -> Result<Option<String>> {
        match table.get(key) {
            Ok(Value::String(s)) => Ok(Some(s.to_str()?.to_string())),
            Ok(Value::Nil) => Ok(None),
            Ok(_) => {
                warn!("Expected string for key '{}', got different type", key);
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }
    
    fn get_required_string(&self, table: &Table, key: &str) -> Result<String> {
        match table.get(key) {
            Ok(Value::String(s)) => Ok(s.to_str()?.to_string()),
            Ok(Value::Nil) => anyhow::bail!("Required field '{}' is nil", key),
            Ok(_) => anyhow::bail!("Expected string for key '{}', got different type", key),
            Err(e) => anyhow::bail!("Failed to get required field '{}': {}", key, e),
        }
    }
    
    fn get_optional_u32(&self, table: &Table, key: &str) -> Result<Option<u32>> {
        match table.get(key) {
            Ok(Value::Integer(i)) => Ok(Some(i as u32)),
            Ok(Value::Number(n)) => Ok(Some(n as u32)),
            Ok(Value::Nil) => Ok(None),
            Ok(_) => {
                warn!("Expected number for key '{}', got different type", key);
                Ok(None)
            }
            Err(_) => Ok(None),
        }
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