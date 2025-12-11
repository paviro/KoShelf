use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

const EN_FTL: &str = include_str!("../../locales/en.ftl");

/// Simple helper to extract keys from FTL content
fn extract_ftl_keys(content: &str) -> HashSet<String> {
    let mut keys = HashSet::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Check for "key =" pattern at start of line (not indented)
        // Fluent keys must be at start of line
        if !line.starts_with(' ') && !line.starts_with('.') && line.contains('=') {
            if let Some((key, _)) = line.split_once('=') {
                keys.insert(key.trim().to_string());
            }
        }
    }
    keys
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

            let missing_keys: Vec<_> = en_keys.difference(&locale_keys).collect();
            let extra_keys: Vec<_> = locale_keys.difference(&en_keys).collect();
            
            if !missing_keys.is_empty() {
                panic!("Locales file {} is missing keys: {:?}", filename, missing_keys);
            }
            if !extra_keys.is_empty() {
                panic!("Locales file {} has extra keys (not in en.ftl): {:?}", filename, extra_keys);
            }
        }
    }
}

#[test]
fn test_unused_translation_keys() {
    let en_keys = extract_ftl_keys(EN_FTL);
    
    // The previous test mapped keys. Here `en_keys` are just the keys in FTL.
    let mut unused_keys: HashSet<String> = en_keys;
    
    // Iterate through all source files
    let walker = WalkDir::new(".").into_iter();
    for entry in walker.filter_entry(|e| !e.path().to_string_lossy().contains("target") && !e.path().to_string_lossy().contains(".git")) {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if !path.is_file() {
            continue;
        }

        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        
        if extension == "rs" || extension == "ts" || extension == "html" || extension == "js" {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut found = Vec::new();
            for key in &unused_keys {
                // Check usage.
                let check_patterns = |k: &str| {
                    let double = format!("\"{}\"", k);
                    let single = format!("'{}'", k);
                    content.contains(&double) || content.contains(&single)
                };

                // Check for exact key match or generated plural keys (_one, _other)
                if check_patterns(key) 
                   || check_patterns(&format!("{}_one", key)) 
                   || check_patterns(&format!("{}_other", key)) {
                    found.push(key.clone());
                }
            }
            
            for key in found {
                unused_keys.remove(&key);
            }
            
            if unused_keys.is_empty() {
                break;
            }
        }
    }

    let mut unused_list: Vec<_> = unused_keys.into_iter().collect();
    unused_list.sort();
    
    if !unused_list.is_empty() {
        panic!("Found unused translation keys: {:?}", unused_list);
    }
}

