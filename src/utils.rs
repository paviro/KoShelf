use std::borrow::Cow;
use std::collections::HashSet;
use ammonia::Builder;
use quick_xml::escape::unescape;

/// Generates a URL-friendly ID from a book title
pub fn generate_book_id(title: &str) -> String {
    title
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase()
}

/// Sanitize HTML content, keeping only safe formatting tags.
/// Used for book descriptions/annotations from EPUB and FB2 files.
pub fn sanitize_html(input: &str) -> String {
    let decoded = unescape(input).unwrap_or(Cow::Borrowed(input));
    
    Builder::new()
        .tags(vec![
            "p", "br", "h1", "h2", "h3", "h4", "h5", "h6",
            "ul", "ol", "li", "strong", "em", "b", "i",
            "blockquote", "pre", "code", "div", "span", "a"
        ].into_iter().collect::<HashSet<_>>())
        .clean(&decoded)
        .to_string()
}