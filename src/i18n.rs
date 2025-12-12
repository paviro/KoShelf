//! Internationalization (i18n) module using fluent-bundle.
//! 
//! Locale hierarchy (from highest to lowest priority):
//! 1. Regional variant (e.g., `de_AT.ftl`) - only contains overrides
//! 2. Base language (e.g., `de.ftl`) - contains all keys for the language
//! 3. English fallback (`en.ftl`) - used for any missing keys

use anyhow::Result;

use std::str::FromStr;

use include_dir::{include_dir, Dir};
use unic_langid::LanguageIdentifier;

// ICU4X for localized language display names
use icu_displaynames::{DisplayNamesOptions, LanguageDisplayNames};
use icu_locid::{locale, Locale};
use fluent::{FluentBundle, FluentResource, FluentArgs};

/// Embedded locales directory
static LOCALES: Dir = include_dir!("$CARGO_MANIFEST_DIR/locales");

/// Translations wrapper using FluentBundle
pub struct Translations {
    /// The requested language code, preserved for chrono locale
    language: String,
    /// The main bundle containing translations
    bundle: FluentBundle<FluentResource>,
    /// We keep track of the raw FTL content strings to send to the frontend
    resources_content: Vec<String>,
}

impl Translations {
    /// Load translations for the specified language.
    pub fn load(language: &str) -> Result<Self> {
        let lang_id: LanguageIdentifier = language.parse()
            .unwrap_or_else(|_| panic!("Invalid locale code '{}'", language));
        
        let normalized = lang_id.to_string().replace("-", "_");
        
        if !normalized.contains('_') {
             panic!("Invalid locale '{}': full locale required (e.g. de_DE)", language);
        }
        
        let parts: Vec<&str> = normalized.split('_').collect();
        let lang_code = parts[0].to_string();
        let region_code = parts[1..].join("_");

        let mut bundle = FluentBundle::new(vec![lang_id.clone()]);
        let mut ordered_sources = Vec::new();
        
        // 1. Regional Variant (Highest Priority)
        let regional_filename = format!("{}_{}.ftl", lang_code, region_code);
        if let Some(file) = LOCALES.get_file(&regional_filename) {
            ordered_sources.push(file.contents_utf8().unwrap_or("").to_string());
        }
        
        // 2. Base Language
        let base_filename = format!("{}.ftl", lang_code);
        if let Some(file) = LOCALES.get_file(&base_filename) {
            ordered_sources.push(file.contents_utf8().unwrap_or("").to_string());
        }
        
        // 3. English Fallback (always included for non-English locales)
        if lang_code != "en" {
            let en_file = LOCALES.get_file("en.ftl").expect("en.ftl must exist");
            ordered_sources.push(en_file.contents_utf8().unwrap_or("").to_string());
        }
        
        // Add resources in priority order. FluentBundle uses "first definition wins",
        // so regional overrides take precedence over base language, which takes precedence over English.
        for source in &ordered_sources {
            let resource = FluentResource::try_new(source.clone())
                .map_err(|(_, errs)| anyhow::anyhow!("Failed to parse FTL: {:?}", errs))?;
            // Ignore duplicate key errors (expected for our fallback strategy)
            let _ = bundle.add_resource(resource);
        }

        Ok(Self {
            language: normalized,
            bundle,
            resources_content: ordered_sources,
        })
    }


    /// Generate JSON for frontend.
    /// Returns { "language": "...", "resources": [ "content1", "content2" ] }
    /// Frontend will use these to create its own Bundle.
    pub fn to_json_string(&self) -> String {
        let bcp47_language = self.language.replace('_', "-");
        let output = serde_json::json!({
            "language": bcp47_language,
            "resources": self.resources_content
        });
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn raw_json(&self) -> String {
        self.to_json_string()
    }

    /// Get a translation by key.
    pub fn get(&self, key: &str) -> String {
        self.format(key, None)
    }

    /// Get a translation by key with a numeric argument ($count).
    pub fn get_with_num<T: std::fmt::Display + Copy>(&self, key: &str, count: T) -> String {
        let mut args = FluentArgs::new();
        let val_str = count.to_string();
        // FluentBundle requires f64 for proper plural rule selection
        if let Ok(num) = val_str.parse::<f64>() {
            args.set("count", num);
        } else {
            args.set("count", val_str);
        }
        self.format(key, Some(&args))
    }

    /// Format a message using the bundle, handling both simple keys and attribute syntax (key.attr).
    fn format(&self, key: &str, args: Option<&FluentArgs>) -> String {
        let (msg_id, attr_id) = if let Some(idx) = key.find('.') {
            (&key[0..idx], Some(&key[idx+1..]))
        } else {
            (key, None)
        };
        
        let msg = match self.bundle.get_message(msg_id) {
            Some(m) => m,
            None => return key.to_string(),
        };
        
        let pattern = if let Some(attr_name) = attr_id {
             match msg.get_attribute(attr_name) {
                 Some(a) => a.value(),
                 None => return key.to_string(),
             }
        } else {
             match msg.value() {
                 Some(v) => v,
                 None => return key.to_string(),
             }
        };

        let mut errors = vec![];
        let value = self.bundle.format_pattern(pattern, args, &mut errors);
        
        if !errors.is_empty() {
             eprintln!("Formatting errors for key '{}': {:?}", key, errors);
        }
        
        value.into_owned()
    }
    
    pub fn locale(&self) -> chrono::Locale {
        chrono::Locale::from_str(&self.language).unwrap_or(chrono::Locale::en_US)
    }
    
    #[cfg(test)]
    pub fn language(&self) -> &str {
        &self.language
    }
    
    pub fn language_display_name(&self, lang_code: &str) -> String {
        let lang_id: LanguageIdentifier = match lang_code.parse() {
            Ok(l) => l,
            Err(_) => return lang_code.to_uppercase(),
        };
        let target_locale: Locale = match lang_id.to_string().parse() {
            Ok(l) => l,
            Err(_) => return lang_code.to_uppercase(),
        };
        let ui_locale: Locale = {
            let bcp47 = self.language.replace('_', "-");
            bcp47.parse().unwrap_or(locale!("en"))
        };
        let options: DisplayNamesOptions = Default::default();
        match LanguageDisplayNames::try_new(&ui_locale.into(), options) {
            Ok(formatter) => formatter.of(target_locale.id.language).map(|s| s.to_string()),
            Err(_) => None,
        }.unwrap_or_else(|| lang_code.to_uppercase())
    }
}

pub fn list_supported_languages() -> String {
    use std::collections::BTreeMap;
    let mut languages: BTreeMap<String, String> = BTreeMap::new();
    for file in LOCALES.files() {
        let filename = file.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !filename.ends_with(".ftl") { continue; }
        let content = file.contents_utf8().unwrap_or("");
        let mut name = String::new();
        let mut dialect = String::new();
        for line in content.lines() {
            let line = line.trim();
            if !line.starts_with("-") { continue; }
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "-lang-name" => name = value.trim().to_string(),
                    "-lang-dialect" => dialect = value.trim().to_string(),
                    _ => {}
                }
            }
        }
        if !dialect.is_empty() && !name.is_empty() {
            languages.insert(dialect, name);
        } else {
             let stem = filename.trim_end_matches(".ftl");
             if !languages.contains_key(stem) {
                 languages.insert(stem.to_string(), format!("Unknown ({})", stem));
             }
        }
    }
    let mut output = String::new();
    output.push_str("Supported Languages:\n\n");
    for (code, name) in &languages {
        output.push_str(&format!("- {}: {}\n", code, name));
    }
    output.push_str("\nUsage:\n  --language <locale>    (e.g., --language de_DE)\n\n");
    output
}
