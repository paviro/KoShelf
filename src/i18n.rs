//! Internationalization (i18n) module using Fluent-flavored translations.
//! Supports a subset of Fluent syntax: simple messages and plural selectors.

use anyhow::Result;
use std::collections::HashMap;

/// Embedded translation files
const EN_FTL: &str = include_str!("../locales/en.ftl");
const DE_FTL: &str = include_str!("../locales/de.ftl");

#[derive(Debug, Clone)]
enum TranslationValue {
    Simple(String),
    Plural {
        one: String,
        other: String,
    },
}

impl TranslationValue {
    fn as_simple(&self) -> Option<&str> {
        match self {
            Self::Simple(s) => Some(s),
            Self::Plural { other, .. } => Some(other), // Fallback to 'other' if accessed directly
        }
    }
}

/// Translations wrapper for key-value lookups with pluralization and fallback support.
pub struct Translations {
    /// Current language code (e.g., "en", "de")
    language: String,
    /// Current language translations
    translations: HashMap<String, TranslationValue>,
    /// Fallback (English) translations for non-English languages
    fallback: HashMap<String, TranslationValue>,
}

impl Translations {
    /// Load translations for the specified language.
    /// Falls back to English for missing keys.
    pub fn load(language: &str) -> Result<Self> {
        let ftl_string = match language {
            "de" => DE_FTL,
            _ => EN_FTL,
        };

        let translations = parse_ftl(ftl_string)?;

        let fallback = if language != "en" {
            parse_ftl(EN_FTL)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            language: language.to_string(),
            translations,
            fallback,
        })
    }

    /// Generate a JSON string compatible with the frontend logic.
    /// Flattens plurals into `key_one` and `key_other`.
    /// Returns a JSON object with format: { "language": "en", "translations": { "key": "value", ... } }
    pub fn to_json_string(&self) -> String {
        let mut flat_translations_map = HashMap::new();
        
        // Helper to insert into map with fallback
        let mut insert_entries = |entries: &HashMap<String, TranslationValue>| {
            for (key, value) in entries {
                match value {
                    TranslationValue::Simple(val) => {
                        flat_translations_map.insert(key.clone(), val.clone());
                    }
                    TranslationValue::Plural { one, other } => {
                        flat_translations_map.insert(format!("{}_one", key), one.clone());
                        flat_translations_map.insert(format!("{}_other", key), other.clone());
                    }
                }
            }
        };
        
        // Insert fallback first, then override with primary language translations
        insert_entries(&self.fallback);
        insert_entries(&self.translations);
        
        // Create wrapper JSON with language field
        let output = serde_json::json!({
            "language": self.language,
            "translations": flat_translations_map
        });
        
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get the raw JSON string (compatibility method name)
    pub fn raw_json(&self) -> String {
        self.to_json_string()
    }

    /// Get a translation by key.
    pub fn get(&self, key: &str) -> String {
        self.lookup(key).unwrap_or_else(|| key.to_string())
    }

    /// Get a translation by key with a numeric argument for pluralization.
    /// Looks up key in FTL.
    /// If Simple: return value with substitution.
    /// If Plural: select based on count.
    pub fn get_with_num<T: std::fmt::Display>(&self, key: &str, count: T) -> String {
        let count_str = count.to_string();
        let num: f64 = count_str.parse().unwrap_or(0.0);
        
        match self.lookup_value(key) {
            Some(TranslationValue::Plural { one, other }) => {
                let template = if num == 1.0 { one } else { other };
                // Replace {$count}, { $count } (standard with spaces), and {{ count }} (legacy)
                template.replace("{$count}", &count_str)
                       .replace("{ $count }", &count_str)
                       .replace("{{ count }}", &count_str)
            }
            Some(TranslationValue::Simple(s)) => {
                 s.replace("{$count}", &count_str)
                  .replace("{ $count }", &count_str)
                  .replace("{{ count }}", &count_str)
            }
            None => key.to_string(),
        }
    }

    /// Internal: lookup key in primary translations, then fallback.
    fn lookup(&self, key: &str) -> Option<String> {
        self.lookup_value(key).and_then(|v| v.as_simple()).map(|s| s.to_string())
    }
    
    fn lookup_value(&self, key: &str) -> Option<&TranslationValue> {
        self.translations.get(key)
            .or_else(|| self.fallback.get(key))
    }
    
    /// Get the chrono Locale for this translation language
    pub fn locale(&self) -> chrono::Locale {
        match self.language.as_str() {
            "de" => chrono::Locale::de_DE,
            "en" => chrono::Locale::en_US,
            _ => chrono::Locale::en_US, // Default to English for unknown languages
        }
    }
}

/// Simple FTL parser
fn parse_ftl(content: &str) -> Result<HashMap<String, TranslationValue>> {
    let mut map = HashMap::new();
    let mut lines = content.lines().peekable();
    
    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some((key, rest)) = line.split_once('=') {
            let key = key.trim().to_string();
            let rest = rest.trim();
            
            if rest.is_empty() {
                // Multi-line or plural?
                // Check next line
                if let Some(next_line) = lines.peek() {
                    if next_line.trim().starts_with('{') {
                         // Likely plural start on next line? Or maybe rest contained '{'
                    }
                }
            }
            
            if rest.starts_with('{') && rest.contains("->") {
                // Plural block start: e.g. "key = { $count ->"
                let mut one = String::new();
                let mut other = String::new();
                
                // Parse plural variants until closing brace
                while let Some(variant_line) = lines.next() {
                    let v_line = variant_line.trim();
                    if v_line == "}" {
                        break;
                    }
                    if v_line.starts_with('[') {
                        // [one] Something
                        let close_bracket = v_line.find(']').unwrap_or(0);
                        let variant = &v_line[1..close_bracket];
                        let value = v_line[close_bracket+1..].trim().to_string();
                        
                        match variant {
                            "one" => one = value,
                            "other" => other = value,
                            _ => {} // Ignore others for now
                        }
                    }
                }
                // If one is missing, fallback to other
                if one.is_empty() { one = other.clone(); }
                
                map.insert(key, TranslationValue::Plural { one, other });
            } else {
                // Simple value
                let value = rest.to_string();
                map.insert(key, TranslationValue::Simple(value));
            }
        }
    }
    
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let ftl = "key = Value";
        let map = parse_ftl(ftl).unwrap();
        match map.get("key").unwrap() {
            TranslationValue::Simple(v) => assert_eq!(v, "Value"),
            _ => panic!("Expected simple"),
        }
    }

    #[test]
    fn test_parse_plural() {
        let ftl = r#"
key = { $count ->
    [one] 1 Thing
    [other] Many Things
}
"#;
        let map = parse_ftl(ftl).unwrap();
        match map.get("key").unwrap() {
            TranslationValue::Plural { one, other } => {
                assert_eq!(one, "1 Thing");
                assert_eq!(other, "Many Things");
            }
            _ => panic!("Expected plural"),
        }
    }
    
    #[test]
    fn test_get_with_num() {
        let ftl = r#"
pages = { $count ->
    [one] { $count } page
    [other] { $count } pages
}
simple = Simple { $count }
"#;
        let map = parse_ftl(ftl).unwrap();
        let t = Translations {
            language: "en".to_string(),
            translations: map,
            fallback: HashMap::new(),
        };
        
        assert_eq!(t.get_with_num("pages", 1), "1 page");
        assert_eq!(t.get_with_num("pages", 2), "2 pages");
        assert_eq!(t.get_with_num("simple", 10), "Simple 10");
    }
    
    #[test]
    fn test_load_english() {
        let t = Translations::load("en").unwrap();
        assert_eq!(t.get("books"), "Books");
    }

    #[test]
    fn test_plural_lookup() {
        let t = Translations::load("en").unwrap();
        assert_eq!(t.get_with_num("pages", 1), "1 page");
        assert_eq!(t.get_with_num("pages", 5), "5 pages");
    }
    
    #[test]
    fn test_book_count_lookup() {
        let t = Translations::load("en").unwrap();
        assert!(t.get_with_num("book-label", 1).contains("Book"));
        assert!(t.get_with_num("book-label", 2).contains("Books"));
    }
}
