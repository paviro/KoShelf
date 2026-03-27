use ammonia::Builder;
use quick_xml::escape::unescape;
use std::borrow::Cow;
use std::collections::HashSet;

/// Sanitize HTML content, keeping only safe formatting tags.
/// Used for book descriptions/annotations from EPUB and FB2 files.
pub fn sanitize_html(input: &str) -> String {
    let decoded = unescape(input).unwrap_or(Cow::Borrowed(input));

    Builder::new()
        .tags(
            vec![
                "p",
                "br",
                "h1",
                "h2",
                "h3",
                "h4",
                "h5",
                "h6",
                "ul",
                "ol",
                "li",
                "strong",
                "em",
                "b",
                "i",
                "blockquote",
                "pre",
                "code",
                "div",
                "span",
                "a",
            ]
            .into_iter()
            .collect::<HashSet<_>>(),
        )
        .clean(&decoded)
        .to_string()
}
