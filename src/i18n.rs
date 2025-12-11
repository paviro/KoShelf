//! Internationalization (i18n) module using Fluent-flavored translations.
//! Supports a subset of Fluent syntax: simple messages and plural selectors.
//! 
//! Locale hierarchy (from highest to lowest priority):
//! 1. Regional variant (e.g., `de_AT.ftl`) - only contains overrides
//! 2. Base language (e.g., `de.ftl`) - contains all keys for the language
//! 3. English fallback (`en.ftl`) - used for any missing keys

use anyhow::Result;
use std::collections::HashMap;
use std::str::FromStr;

use include_dir::{include_dir, Dir};

/// Embedded locales directory
static LOCALES: Dir = include_dir!("$CARGO_MANIFEST_DIR/locales");

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
/// 
/// Supports a three-tier fallback hierarchy:
/// 1. Regional variant (e.g., `de_AT`) - sparse, only overrides
/// 2. Base language (e.g., `de`) - complete translation for the language
/// 3. English (`en`) - final fallback for any missing keys
pub struct Translations {
    /// The requested language code, preserved for chrono locale (e.g., "en_US", "de_DE")
    language: String,
    /// Merged translations (regional overrides + base language)
    translations: HashMap<String, TranslationValue>,
    /// English fallback translations
    fallback: HashMap<String, TranslationValue>,
}

impl Translations {
    /// Load translations for the specified language.
    /// 
    /// Accepts locale codes in various formats:
    /// - Full POSIX format: `en_US`, `de_DE`
    /// - Hyphenated format: `en-US`, `de-DE`
    /// 
    /// A full locale code (language + region) is required for proper date formatting.
    /// 
    /// Loading hierarchy:
    /// - If `de_AT` is requested: loads `de_AT.ftl` (if exists) merged over `de.ftl`
    /// - If `de_DE` is requested: loads `de_DE.ftl` (if exists) merged over `de.ftl`
    /// - Always falls back to `en.ftl` for missing keys (unless language is English)
    /// 
    /// # Panics
    /// Panics if only a language code is provided without a region (e.g., `de` instead of `de_DE`).
    pub fn load(language: &str) -> Result<Self> {
        let normalized = normalize_locale(language);
        
        // Require full locale code (language + region)
        if !normalized.contains('_') {
            panic!(
                "Invalid locale '{}': a full locale code is required (e.g., 'de_DE', 'en_US', 'pt_BR'). \
                 Just the language code '{}' is not sufficient for proper date formatting.",
                language,
                normalized
            );
        }
        
        // Extract language and region
        let parts: Vec<&str> = normalized.split('_').collect();
        let lang_code = parts[0].to_string();
        let region_code = parts[1..].join("_");
        
        // Load English fallback first (unless we're loading English)
        let fallback = if lang_code != "en" {
            let en_file = LOCALES.get_file("en.ftl").expect("en.ftl must exist");
            parse_ftl(en_file.contents_utf8().unwrap_or(""))?
        } else {
            HashMap::new()
        };
        
        // Load base language file
        let base_filename = format!("{}.ftl", lang_code);
        let base_translations = if let Some(file) = LOCALES.get_file(&base_filename) {
            parse_ftl(file.contents_utf8().unwrap_or(""))?
        } else if lang_code != "en" {
            // Base language file doesn't exist, fall back to English
            let en_file = LOCALES.get_file("en.ftl").expect("en.ftl must exist");
            parse_ftl(en_file.contents_utf8().unwrap_or(""))?
        } else {
            HashMap::new()
        };
        
        // Load regional variant if it exists
        let regional_filename = format!("{}_{}.ftl", lang_code, region_code);
        let final_translations = if let Some(file) = LOCALES.get_file(&regional_filename) {
            // Merge: base first, then regional overrides
            let regional = parse_ftl(file.contents_utf8().unwrap_or(""))?;
            let mut merged = base_translations;
            merged.extend(regional);
            merged
        } else {
            // Regional file doesn't exist, use base language translations
            base_translations
        };
        
        // Preserve the requested language code for chrono locale purposes
        // e.g., if user requests "de_DE", keep "de_DE" even if only "de.ftl" exists
        Ok(Self {
            language: normalized,
            translations: final_translations,
            fallback,
        })
    }

    /// Generate a JSON string compatible with the frontend logic.
    /// Flattens plurals into `key_one` and `key_other`.
    /// Returns a JSON object with format: { "language": "de-AT", "translations": { "key": "value", ... } }
    /// Note: The language field uses BCP 47 format (hyphenated) for JavaScript Intl API compatibility.
    pub fn to_json_string(&self) -> String {
        let mut flat_translations_map = HashMap::new();
        
        // Helper to insert into map
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
        
        // Create wrapper JSON with language field (convert to BCP 47 format for JS Intl APIs)
        let bcp47_language = self.language.replace('_', "-");
        let output = serde_json::json!({
            "language": bcp47_language,
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
    
    /// Get the chrono Locale for this translation language.
    /// Requires a full locale code (e.g., `de_DE`, `en_US`) for proper date formatting.
    /// Falls back to `en_US` if the locale code is not recognized by chrono.
    pub fn locale(&self) -> chrono::Locale {
        chrono::Locale::from_str(&self.language).unwrap_or(chrono::Locale::en_US)
    }
}

/// Normalize a locale string to POSIX format (ll_CC).
/// Handles various input formats:
/// - `pt-br` → `pt_BR`
/// - `pt-BR` → `pt_BR`
/// - `pt_br` → `pt_BR`
/// - `de-DE` → `de_DE`
fn normalize_locale(locale: &str) -> String {
    // Replace hyphens with underscores
    let with_underscore = locale.replace('-', "_");
    
    // Split into parts
    let parts: Vec<&str> = with_underscore.split('_').collect();
    
    match parts.len() {
        1 => {
            // Just language code, return lowercase
            parts[0].to_lowercase()
        }
        2 => {
            // Language and region: lowercase language, uppercase region
            format!("{}_{}", parts[0].to_lowercase(), parts[1].to_uppercase())
        }
        _ => {
            // More complex locale (e.g., with script), join with underscores
            // First part lowercase, rest uppercase
            let mut result = parts[0].to_lowercase();
            for part in &parts[1..] {
                result.push('_');
                result.push_str(&part.to_uppercase());
            }
            result
        }
    }
}

/// List all supported languages by reading metadata from FTL files.
/// Returns a formatted string suitable for CLI output.
pub fn list_supported_languages() -> String {
    use std::collections::BTreeMap;
    
    // Map: lang_code -> (name, vec of supported locales)
    let mut languages: BTreeMap<String, (String, Vec<String>)> = BTreeMap::new();
    
    // Single pass: collect all .ftl files
    for file in LOCALES.files() {
        let filename = file.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !filename.ends_with(".ftl") {
            continue;
        }
        
        let is_base = !filename.contains('_');
        let content = file.contents_utf8().unwrap_or("");
        
        if is_base {
            // Base language file - extract metadata
            let mut code = String::new();
            let mut name = String::new();
            let mut dialect = String::new();
            
            for line in content.lines() {
                let line = line.trim();
                if let Some((key, value)) = line.split_once('=') {
                    match key.trim() {
                        "-lang-code" => code = value.trim().to_string(),
                        "-lang-name" => name = value.trim().to_string(),
                        "-lang-dialect" => dialect = value.trim().to_string(),
                        _ => {}
                    }
                }
            }
            
            if !code.is_empty() && !name.is_empty() {
                let entry = languages.entry(code).or_insert_with(|| (name, Vec::new()));
                if !dialect.is_empty() && !entry.1.contains(&dialect) {
                    entry.1.push(dialect);
                }
            }
        } else {
            // Regional variant file (e.g., "de_AT.ftl")
            let locale = filename.trim_end_matches(".ftl");
            let lang_code = locale.split('_').next().unwrap_or("").to_string();
            
            if let Some(entry) = languages.get_mut(&lang_code) {
                if !entry.1.contains(&locale.to_string()) {
                    entry.1.push(locale.to_string());
                }
            }
        }
    }
    
    // Format output
    let mut output = String::new();
    output.push_str("Supported Languages:\n\n");
    
    for (code, (name, dialects)) in &languages {
        output.push_str(&format!("  {} - {}\n", code, name));
        if !dialects.is_empty() {
            output.push_str(&format!("      Full support: {}\n", dialects.join(", ")));
        }
    }
    
    output.push_str("\nUsage:\n");
    output.push_str("  --language <locale>    (e.g., --language de_DE)\n\n");
    output.push_str("Note:\n");
    output.push_str("  You can use any regional variant even if not listed above.\n");
    output.push_str("  Unlisted variants (e.g., de_AT, en_GB) will use the base language\n");
    output.push_str("  translations with region-specific date formatting from chrono.\n");
    output.push_str("  For full support, the locale needs translated date format strings.\n");
    
    output
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
        let t = Translations::load("en_US").unwrap();
        assert_eq!(t.language, "en_US");
        assert_eq!(t.get("books"), "Books");
    }

    #[test]
    fn test_load_german() {
        let t = Translations::load("de_DE").unwrap();
        assert_eq!(t.language, "de_DE");
        assert_eq!(t.get("books"), "Bücher");
    }
    
    #[test]
    #[should_panic(expected = "Invalid locale")]
    fn test_reject_bare_language_code() {
        // Bare language codes should be rejected
        let _ = Translations::load("de");
    }

    #[test]
    fn test_plural_lookup() {
        let t = Translations::load("en_US").unwrap();
        assert_eq!(t.get_with_num("pages", 1), "1 page");
        assert_eq!(t.get_with_num("pages", 5), "5 pages");
    }
    
    #[test]
    fn test_book_count_lookup() {
        let t = Translations::load("en_US").unwrap();
        assert!(t.get_with_num("book-label", 1).contains("Book"));
        assert!(t.get_with_num("book-label", 2).contains("Books"));
    }
    
    #[test]
    fn test_normalize_locale() {
        assert_eq!(normalize_locale("pt-br"), "pt_BR");
        assert_eq!(normalize_locale("pt-BR"), "pt_BR");
        assert_eq!(normalize_locale("pt_br"), "pt_BR");
        assert_eq!(normalize_locale("PT_BR"), "pt_BR");
        assert_eq!(normalize_locale("en-US"), "en_US");
        assert_eq!(normalize_locale("EN-us"), "en_US");
        // All German locale variants should normalize correctly
        assert_eq!(normalize_locale("de-DE"), "de_DE");
        assert_eq!(normalize_locale("de-de"), "de_DE");
        assert_eq!(normalize_locale("de_DE"), "de_DE");
        assert_eq!(normalize_locale("de_de"), "de_DE");
        assert_eq!(normalize_locale("DE_DE"), "de_DE");
        assert_eq!(normalize_locale("de-AT"), "de_AT");
        assert_eq!(normalize_locale("de_AT"), "de_AT");
    }
    
    #[test]
    fn test_locale_chrono() {
        // Full locale codes work with chrono even if only base translation file exists
        let t = Translations::load("de_DE").unwrap();
        assert_eq!(t.language, "de_DE");
        assert_eq!(t.locale(), chrono::Locale::de_DE);
        
        let t = Translations::load("en_US").unwrap();
        assert_eq!(t.language, "en_US");
        assert_eq!(t.locale(), chrono::Locale::en_US);
        
        // Unknown locale falls back to en_US for chrono
        let t = Translations::load("xx_YY").unwrap();
        assert_eq!(t.locale(), chrono::Locale::en_US);
    }
    
    #[test]
    fn test_load_with_hyphenated_locale() {
        // Should work with hyphenated format, preserving full locale
        let t = Translations::load("de-DE").unwrap();
        assert_eq!(t.language, "de_DE"); // Normalized and preserved
        // Translations come from de.ftl (base), but locale is de_DE
        assert_eq!(t.get("books"), "Bücher");
    }
    
    #[test]
    fn test_fallback_to_english() {
        // Unknown locale - requested language is preserved, translations come from English
        let t = Translations::load("xx_YY").unwrap();
        assert_eq!(t.language, "xx_YY");
        // But translations come from English fallback
        assert_eq!(t.get("books"), "Books");
    }
    
    #[test]
    fn test_json_output_uses_bcp47() {
        let t = Translations::load("de_DE").unwrap();
        let json = t.to_json_string();
        // Should contain hyphenated language code
        assert!(json.contains("\"language\": \"de-DE\""));
        
        let t = Translations::load("de_AT").unwrap();
        let json = t.to_json_string();
        assert!(json.contains("\"language\": \"de-AT\""));
    }
    
    #[test]
    fn test_regional_variant_falls_back_to_base_language() {
        // de_AT should fall back to de.ftl (base German), NOT de_DE.ftl
        // This is important: the fallback chain is:
        //   de_AT.ftl (if exists) → de.ftl → en.ftl
        // NOT:
        //   de_AT.ftl → de_DE.ftl → de.ftl → en.ftl
        let t = Translations::load("de_AT").unwrap();
        assert_eq!(t.language, "de_AT");
        
        // Should get German translations from de.ftl (base)
        assert_eq!(t.get("books"), "Bücher");
        
        // Verify chrono locale works for Austrian German
        assert_eq!(t.locale(), chrono::Locale::de_AT);
    }
}