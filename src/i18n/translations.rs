//! Locale metadata helpers.

include!(concat!(env!("OUT_DIR"), "/locale_manifest.rs"));

pub fn list_supported_languages() -> String {
    let mut output = String::new();
    output.push_str("Supported Languages:\n\n");
    for (code, name) in SUPPORTED_LANGUAGES {
        output.push_str(&format!("- {code}: {name}\n"));
    }
    output.push_str("\nUsage:\n  --language <locale>    (e.g., --language de_DE)\n\n");
    output
}
