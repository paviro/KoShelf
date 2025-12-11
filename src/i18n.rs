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
use intl_pluralrules::{PluralRules, PluralRuleType};
use unic_langid::LanguageIdentifier;

/// Embedded locales directory
static LOCALES: Dir = include_dir!("$CARGO_MANIFEST_DIR/locales");

/// CLDR plural categories
/// See: https://cldr.unicode.org/index/cldr-spec/plural-rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl PluralCategory {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "zero" => Some(Self::Zero),
            "one" => Some(Self::One),
            "two" => Some(Self::Two),
            "few" => Some(Self::Few),
            "many" => Some(Self::Many),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
enum TranslationValue {
    Simple(String),
    /// Plural with all CLDR categories. The HashMap stores only the categories
    /// that are defined in the FTL file. `other` is required and used as fallback.
    Plural(HashMap<PluralCategory, String>),
}

impl TranslationValue {
    fn as_simple(&self) -> Option<&str> {
        match self {
            Self::Simple(s) => Some(s),
            Self::Plural(variants) => variants.get(&PluralCategory::Other).map(|s| s.as_str()),
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
    /// Flattens plurals into `key_zero`, `key_one`, `key_two`, `key_few`, `key_many`, `key_other`.
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
                    TranslationValue::Plural(variants) => {
                        for (category, val) in variants {
                            let suffix = match category {
                                PluralCategory::Zero => "zero",
                                PluralCategory::One => "one",
                                PluralCategory::Two => "two",
                                PluralCategory::Few => "few",
                                PluralCategory::Many => "many",
                                PluralCategory::Other => "other",
                            };
                            flat_translations_map.insert(format!("{}_{}", key, suffix), val.clone());
                        }
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
    /// If Plural: select CLDR plural category based on count and language.
    /// 
    /// The selector logic works as follows:
    /// 1. First, try exact numeric match (e.g., [0], [1], [2] variants)
    /// 2. Then, try CLDR plural category for the language (zero, one, two, few, many)
    /// 3. Finally, fall back to 'other' (required default)
    pub fn get_with_num<T: std::fmt::Display>(&self, key: &str, count: T) -> String {
        let count_str = count.to_string();
        let num: f64 = count_str.parse().unwrap_or(0.0);
        let num_int = num as i64;
        
        let replace_placeholders = |template: &str| -> String {
            // Standard Fluent placeholder format: {$var}
            // Also accept common variant with spaces: { $var }
            template.replace("{$count}", &count_str)
                   .replace("{ $count }", &count_str)
        };
        
        match self.lookup_value(key) {
            Some(TranslationValue::Plural(variants)) => {
                // Select the appropriate variant using CLDR rules
                let category = self.select_plural_category(num_int);
                let template = variants.get(&category)
                    .or_else(|| variants.get(&PluralCategory::Other))
                    .map(|s| s.as_str())
                    .unwrap_or(key);
                replace_placeholders(template)
            }
            Some(TranslationValue::Simple(s)) => replace_placeholders(s),
            None => key.to_string(),
        }
    }
    
    /// Select the CLDR plural category for a number based on the current language.
    /// Uses the `intl_pluralrules` crate for proper Unicode CLDR compliance.
    fn select_plural_category(&self, n: i64) -> PluralCategory {
        // Parse the language identifier
        let lang_code = self.language.split('_').next().unwrap_or("en");
        let langid: LanguageIdentifier = lang_code.parse()
            .unwrap_or_else(|_| "en".parse::<LanguageIdentifier>().unwrap());
        
        // Create plural rules for this language
        let pr = PluralRules::create(langid, PluralRuleType::CARDINAL)
            .unwrap_or_else(|_| {
                let en: LanguageIdentifier = "en".parse().unwrap();
                PluralRules::create(en, PluralRuleType::CARDINAL).unwrap()
            });
        
        // Select the category and convert to our enum
        match pr.select(n) {
            Ok(intl_pluralrules::PluralCategory::ZERO) => PluralCategory::Zero,
            Ok(intl_pluralrules::PluralCategory::ONE) => PluralCategory::One,
            Ok(intl_pluralrules::PluralCategory::TWO) => PluralCategory::Two,
            Ok(intl_pluralrules::PluralCategory::FEW) => PluralCategory::Few,
            Ok(intl_pluralrules::PluralCategory::MANY) => PluralCategory::Many,
            Ok(intl_pluralrules::PluralCategory::OTHER) | Err(_) => PluralCategory::Other,
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
    
    // Map: dialect_code -> name
    let mut languages: BTreeMap<String, String> = BTreeMap::new();
    
    // Single pass: collect all .ftl files and parse metadata
    for file in LOCALES.files() {
        let filename = file.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !filename.ends_with(".ftl") {
            continue;
        }
        
        let content = file.contents_utf8().unwrap_or("");
        
        // Extract metadata
        let mut name = String::new();
        let mut dialect = String::new();
        
        for line in content.lines() {
            let line = line.trim();
            // Skip comments that aren't metadata
            if !line.starts_with("-") {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "-lang-name" => name = value.trim().to_string(),
                    "-lang-dialect" => dialect = value.trim().to_string(),
                    _ => {}
                }
            }
        }
        
        // If metadata is present, add to list
        if !dialect.is_empty() && !name.is_empty() {
            languages.insert(dialect, name);
        } else {
            // Fallback for files without metadata (shouldn't happen with current files)
            // Use filename stem as code and output warning in name
            let stem = filename.trim_end_matches(".ftl");
            if !languages.contains_key(stem) {
                 languages.insert(stem.to_string(), format!("Unknown ({})", stem));
            }
        }
    }
    
    // Format output
    let mut output = String::new();
    output.push_str("Supported Languages:\n\n");
    
    for (code, name) in &languages {
        output.push_str(&format!("- {}: {}\n", code, name));
    }
    
    output.push_str("\nUsage:\n");
    output.push_str("  --language <locale>    (e.g., --language de_DE)\n\n");
    output.push_str("Note:\n");
    output.push_str("  You can use any country variant even if not listed above.\n");
    output.push_str("  Unlisted variants will use the base language\n");
    output.push_str("  and region-specific date formatting from chrono.\n");
    output.push_str("  For full support, the locale needs a translated date format string.\n");

    output
}

/// FTL parser supporting a subset of Fluent syntax:
/// - Simple messages (single and multiline)
/// - Plural selectors with CLDR categories (zero, one, two, few, many, other)
/// - Comments (lines starting with #)
///
/// Multiline messages follow Fluent spec: continuation lines must be indented.
/// Exact numeric selectors (e.g., [0], [1]) are parsed but ignored — use CLDR categories instead.
fn parse_ftl(content: &str) -> Result<HashMap<String, TranslationValue>> {
    let mut map = HashMap::new();
    let mut lines = content.lines().peekable();
    
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        // Messages start with identifier at column 0 (not indented)
        if line.starts_with(char::is_whitespace) {
            // Indented line without a message context — skip
            continue;
        }
        
        if let Some((key, rest)) = trimmed.split_once('=') {
            let key = key.trim().to_string();
            let rest = rest.trim();
            
            // Check if this is a plural block (contains selector arrow)
            if rest.starts_with('{') && rest.contains("->") {
                // Plural block: e.g. "key = { $count ->"
                let mut variants: HashMap<PluralCategory, String> = HashMap::new();
                let mut found_default = false;
                
                // Parse plural variants until closing brace
                while let Some(variant_line) = lines.next() {
                    let v_line = variant_line.trim();
                    
                    // Skip empty lines and comments inside plural block
                    if v_line.is_empty() || v_line.starts_with('#') {
                        continue;
                    }
                    
                    if v_line == "}" {
                        break;
                    }
                    
                    // Check for default marker (*)
                    let is_default = v_line.starts_with('*');
                    let v_line = v_line.trim_start_matches('*');
                    
                    if v_line.starts_with('[') {
                        if let Some(close_bracket) = v_line.find(']') {
                            let variant_key = &v_line[1..close_bracket];
                            let value = v_line[close_bracket + 1..].trim().to_string();
                            
                            // Parse as CLDR category
                            if let Some(category) = PluralCategory::from_str(variant_key) {
                                variants.insert(category, value);
                                if is_default {
                                    found_default = true;
                                }
                            }
                            // Exact numeric selectors are parsed but ignored
                            // They don't map cleanly to CLDR categories across languages
                        }
                    }
                }
                
                // Log warning if no default variant was marked (but don't fail)
                if !found_default && !variants.contains_key(&PluralCategory::Other) {
                    eprintln!("Warning: plural key '{}' missing required *[other] default variant", key);
                }
                
                map.insert(key, TranslationValue::Plural(variants));
            } else if rest.is_empty() {
                // Value starts on next line(s) — multiline message
                let mut multiline_value = String::new();
                
                // Collect all indented continuation lines
                while let Some(next_line) = lines.peek() {
                    // Continuation lines must be indented
                    if next_line.is_empty() {
                        // Blank lines are preserved in multiline text
                        lines.next();
                        if !multiline_value.is_empty() {
                            multiline_value.push('\n');
                        }
                        continue;
                    }
                    
                    if !next_line.starts_with(char::is_whitespace) {
                        // Not indented — end of multiline value
                        break;
                    }
                    
                    let content_line = lines.next().unwrap();
                    let trimmed_content = content_line.trim();
                    
                    // Skip comments in multiline blocks
                    if trimmed_content.starts_with('#') {
                        continue;
                    }
                    
                    // Check if this starts a plural block
                    if trimmed_content.starts_with('{') && trimmed_content.contains("->") {
                        // Actually a plural block that started on next line
                        let mut variants: HashMap<PluralCategory, String> = HashMap::new();
                        
                        while let Some(variant_line) = lines.next() {
                            let v_line = variant_line.trim();
                            
                            if v_line.is_empty() || v_line.starts_with('#') {
                                continue;
                            }
                            
                            if v_line == "}" {
                                break;
                            }
                            
                            let v_line = v_line.trim_start_matches('*');
                            if v_line.starts_with('[') {
                                if let Some(close_bracket) = v_line.find(']') {
                                    let variant_key = &v_line[1..close_bracket];
                                    let value = v_line[close_bracket + 1..].trim().to_string();
                                    
                                    if let Some(category) = PluralCategory::from_str(variant_key) {
                                        variants.insert(category, value);
                                    }
                                }
                            }
                        }
                        
                        map.insert(key.clone(), TranslationValue::Plural(variants));
                        break;
                    }
                    
                    // Regular multiline text
                    if !multiline_value.is_empty() {
                        multiline_value.push('\n');
                    }
                    multiline_value.push_str(trimmed_content);
                }
                
                // Only insert as Simple if we didn't already insert as Plural
                if !map.contains_key(&key) && !multiline_value.is_empty() {
                    map.insert(key, TranslationValue::Simple(multiline_value));
                }
            } else {
                // Simple single-line value
                map.insert(key, TranslationValue::Simple(rest.to_string()));
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
            TranslationValue::Plural(variants) => {
                assert_eq!(variants.get(&PluralCategory::One).unwrap(), "1 Thing");
                assert_eq!(variants.get(&PluralCategory::Other).unwrap(), "Many Things");
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
    
    #[test]
    fn test_parse_multiline_message() {
        let ftl = r#"
long-message =
    This is a long message
    that spans multiple lines
"#;
        let map = parse_ftl(ftl).unwrap();
        match map.get("long-message").unwrap() {
            TranslationValue::Simple(v) => {
                assert!(v.contains("This is a long message"));
                assert!(v.contains("that spans multiple lines"));
            }
            _ => panic!("Expected simple multiline message"),
        }
    }
    
    #[test]
    fn test_parse_plural_with_comments() {
        let ftl = r#"
items = { $count ->
    # This is a comment inside plural block
    [one] One item
    # Another comment
   *[other] Many items
}
"#;
        let map = parse_ftl(ftl).unwrap();
        match map.get("items").unwrap() {
            TranslationValue::Plural(variants) => {
                assert_eq!(variants.get(&PluralCategory::One).unwrap(), "One item");
                assert_eq!(variants.get(&PluralCategory::Other).unwrap(), "Many items");
            }
            _ => panic!("Expected plural"),
        }
    }
    
    #[test]
    fn test_parse_plural_on_next_line() {
        // Plural block starting on the line after =
        let ftl = r#"
pages =
    { $count ->
        [one] { $count } page
       *[other] { $count } pages
    }
"#;
        let map = parse_ftl(ftl).unwrap();
        match map.get("pages").unwrap() {
            TranslationValue::Plural(variants) => {
                assert_eq!(variants.get(&PluralCategory::One).unwrap(), "{ $count } page");
                assert_eq!(variants.get(&PluralCategory::Other).unwrap(), "{ $count } pages");
            }
            _ => panic!("Expected plural"),
        }
    }
}