use fluent_syntax::ast;
use fluent_syntax::parser;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use rayon::prelude::*;
use regex::Regex;

const EN_FTL: &str = include_str!("../../locales/en.ftl");

/// Required metadata keys for base language files (not regional variants)
const REQUIRED_METADATA: &[&str] = &["-lang-code", "-lang-name", "-lang-dialect"];

/// Helper to extract keys from FTL content using the official parser
fn extract_ftl_keys(content: &str) -> HashSet<String> {
    let resource = parser::parse(content).expect("Failed to parse FTL");
    let mut keys = HashSet::new();

    for entry in resource.body {
        match entry {
            ast::Entry::Message(msg) => {
                keys.insert(msg.id.name.to_string());
            }
            ast::Entry::Term(term) => {
                keys.insert(format!("-{}", term.id.name));
            }
            _ => {}
        }
    }
    keys
}

/// Check if a filename represents a regional variant (e.g., de_AT.ftl, pt_BR.ftl)
/// vs a base language file (e.g., de.ftl, en.ftl)
fn is_regional_variant(filename: &str) -> bool {
    let stem = filename.strip_suffix(".ftl").unwrap_or(filename);
    stem.contains('_')
}

#[test]
fn test_missing_translation_keys() {
    let en_keys = extract_ftl_keys(EN_FTL);

    let locales_dir = Path::new("locales");
    for entry in fs::read_dir(locales_dir).expect("Failed to read locales directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("ftl") {
            let filename = path.file_name().unwrap().to_str().unwrap();
            if filename == "en.ftl" {
                continue;
            }
            
            let content = fs::read_to_string(&path).expect("Failed to read locale file");
            let locale_keys = extract_ftl_keys(&content);

            // Regional variants (e.g., de_AT.ftl) are allowed to have a subset of keys
            // They only override specific translations from their base language
            // Base language files (e.g., de.ftl) must have all keys
            if is_regional_variant(filename) {
                // Regional variants: only check for extra keys (no keys that don't exist in en.ftl)
                let extra_keys: Vec<_> = locale_keys.difference(&en_keys).collect();
                if !extra_keys.is_empty() {
                    panic!(
                        "Regional variant {} has extra keys (not in en.ftl): {:?}",
                        filename, extra_keys
                    );
                }
            } else {
                // Base language files: must have all keys from en.ftl
                let missing_keys: Vec<_> = en_keys.difference(&locale_keys).collect();
                let extra_keys: Vec<_> = locale_keys.difference(&en_keys).collect();
                
                if !missing_keys.is_empty() {
                    panic!(
                        "Base language file {} is missing keys: {:?}",
                        filename, missing_keys
                    );
                }
                if !extra_keys.is_empty() {
                    panic!(
                        "Base language file {} has extra keys (not in en.ftl): {:?}",
                        filename, extra_keys
                    );
                }
            }
        }
    }
}

#[test]
fn test_unused_translation_keys() {
    let en_keys = extract_ftl_keys(EN_FTL);
    
    // Collect all source files first
    let source_files: Vec<_> = WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            let s = path.to_string_lossy();
            
            // Skip target, .git, and non-files
            if s.contains("target") || s.contains(".git") || !path.is_file() {
                return false;
            }
            
            // Only check relevant extensions
            ["rs", "ts", "html", "js"].contains(&path.extension().and_then(|s| s.to_str()).unwrap_or(""))
        })
        .map(|e| e.path().to_owned())
        .collect();

    // Compile regex once
    // Matches quoted strings that look like FTL keys: "key", 'key', "key-name", 'key_name'
    let re = Regex::new(r#"["']([-a-zA-Z0-9_]+)["']"#).expect("Invalid Regex");

    // Process files in parallel to find all potential key usages
    let found_tokens: HashSet<String> = source_files
        .par_iter()
        .map(|path| {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => return HashSet::new(),
            };
            
            let mut found = HashSet::new();
            for cap in re.captures_iter(&content) {
                if let Some(m) = cap.get(1) {
                    found.insert(m.as_str().to_string());
                }
            }
            found
        })
        .reduce(HashSet::new, |mut a, b| {
            a.extend(b);
            a
        });

    // Check which EN keys are unused
    let mut unused_list: Vec<_> = en_keys
        .into_iter()
        .filter(|k| {
            // A key is used if we found "key", "key_one", or "key_other"
            !found_tokens.contains(k) 
            && !found_tokens.contains(&format!("{}_one", k)) 
            && !found_tokens.contains(&format!("{}_other", k))
        })
        .collect();

    unused_list.sort();
    
    if !unused_list.is_empty() {
        panic!("Found unused translation keys: {:?}", unused_list);
    }
}

#[test]
fn test_ftl_files_are_valid() {
    let locales_dir = Path::new("locales");
    
    for entry in fs::read_dir(locales_dir).expect("Failed to read locales directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("ftl") {
            let filename = path.file_name().unwrap().to_str().unwrap();
            let content = fs::read_to_string(&path).expect("Failed to read FTL file");
            
            // Use fluent-syntax parser to validate the FTL file
            if let Err(errors) = parser::parse(content.as_str()) {
                let error_msgs: Vec<_> = errors.1.iter().map(|e| format!("{:?}", e)).collect();
                panic!(
                    "FTL file {} has syntax errors:\n{}",
                    filename,
                    error_msgs.join("\n")
                );
            }
        }
    }
}

#[test]
fn test_required_metadata_present() {
    let locales_dir = Path::new("locales");
    
    for entry in fs::read_dir(locales_dir).expect("Failed to read locales directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("ftl") {
            let filename = path.file_name().unwrap().to_str().unwrap();
            
            // Only check base language files (not regional variants like de_AT.ftl)
            if is_regional_variant(filename) {
                continue;
            }
            
            let content = fs::read_to_string(&path).expect("Failed to read FTL file");
            let keys = extract_ftl_keys(&content);
            
            let mut missing_metadata: Vec<&str> = Vec::new();
            for required_key in REQUIRED_METADATA {
                if !keys.contains(*required_key) {
                    missing_metadata.push(required_key);
                }
            }
            
            if !missing_metadata.is_empty() {
                panic!(
                    "Base language file {} is missing required metadata keys: {:?}\n\
                     Add these to your FTL file (see locales/README.md for format):\n\
                     -lang-code = <ISO 639-1 code>\n\
                     -lang-name = <Native language name>\n\
                     -lang-dialect = <Full locale code for date formatting>",
                    filename, missing_metadata
                );
            }
        }
    }
}

#[test]
fn test_all_locales_loadable_and_functional() {
    use crate::i18n::Translations;
    
    // Dynamically discover and test all FTL files
    // This validates our custom parse_ftl parser handles all translation patterns
    let locales_dir = Path::new("locales");
    
    for entry in fs::read_dir(locales_dir).expect("Failed to read locales directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) != Some("ftl") {
            continue;
        }
        
        let filename = path.file_name().unwrap().to_str().unwrap();
        
        // Skip regional variants (they need their base file)
        if is_regional_variant(filename) {
            continue;
        }
        
        // Read the file to extract the -lang-dialect metadata
        let content = fs::read_to_string(&path).expect("Failed to read FTL file");
        let dialect = content.lines()
            .find(|line| line.trim().starts_with("-lang-dialect"))
            .and_then(|line| line.split_once('='))
            .map(|(_, v)| v.trim().to_string())
            .unwrap_or_else(|| {
                // Fallback: construct from filename (e.g., "de.ftl" -> "de_DE")
                let lang = filename.trim_end_matches(".ftl");
                format!("{}_{}", lang, lang.to_uppercase())
            });
        
        let translations = Translations::load(&dialect)
            .unwrap_or_else(|e| panic!("Locale {} (from {}) failed to load: {}", dialect, filename, e));
        
        // Verify metadata keys are present (parsed correctly)
        // This confirms the parser loaded the file and extracted keys
        let lang_code = translations.get("-lang-code");
        assert!(
            lang_code != "-lang-code",
            "{} missing '-lang-code' metadata",
            filename
        );
        
        // Ensure language is correctly set
        assert!(!translations.to_json_string().is_empty());
    }
}