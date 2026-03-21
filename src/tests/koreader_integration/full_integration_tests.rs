use super::*;
use mlua::Lua;

/// Integration tests that use KoReader's actual code to validate compatibility.
#[test]
fn test_parse_koreader_generated_metadata() {
    let koreader_dir = get_koreader_dir();
    let dump_lua_path = koreader_dir.path().join("frontend/dump.lua");

    if !dump_lua_path.exists() {
        panic!("KoReader dump.lua not found at {}", dump_lua_path.display());
    }

    let lua = Lua::new();

    let dump_source = std::fs::read_to_string(&dump_lua_path).expect("Failed to read dump.lua");
    let dump_fn: mlua::Function = lua
        .load(&dump_source)
        .eval()
        .expect("Failed to load dump.lua");

    // Create test data as a Lua table
    let test_table: mlua::Table = lua
        .load(
            r#"{
            annotations = {
                {
                    chapter = "Chapter 1: Introduction",
                    datetime = "2024-03-15 14:30:00",
                    pageno = 15,
                    pos0 = "/body/section[1]/p[3]/text().0",
                    pos1 = "/body/section[1]/p[3]/text().50",
                    text = "This is an important passage that was highlighted.",
                    note = "Remember this section for later",
                    color = "yellow",
                    drawer = "underscore",
                },
            },
            font_face = "Noto Serif",
            copt_font_size = 24,
            copt_line_spacing = 110,
            copt_h_page_margins = { 30, 30 },
            copt_t_page_margin = 30,
            copt_b_page_margin = 15,
            copt_embedded_css = 1,
            copt_embedded_fonts = 0,
            copt_word_spacing = { 100, 90 },
            copt_word_expansion = 15,
            floating_punctuation = 0,
            hyphenation = true,
            doc_pages = 250,
            doc_path = "/storage/Books/My Book.epub",
            pagemap_use_page_labels = true,
            pagemap_chars_per_synthetic_page = 1500,
            pagemap_doc_pages = 320,
            pagemap_current_page_label = "12",
            pagemap_last_page_label = "320",
            doc_props = {
                authors = "John Smith",
                description = "A fascinating book about programming.",
                language = "en",
                title = "My Programming Book",
            },
            partial_md5_checksum = "abc123def456789",
            percent_finished = 0.25,
            stats = {
                authors = "John Smith",
                highlights = 5,
                language = "en",
                notes = 2,
                pages = 250,
                title = "My Programming Book",
            },
            summary = {
                modified = "2024-03-15",
                note = "Great book, still reading",
                rating = 4,
                status = "reading",
            },
            text_lang = "en-US",
        }"#,
        )
        .eval()
        .expect("Failed to create test table");

    // Use KoReader's dump() to serialize the table
    let serialized: String = dump_fn.call(test_table).expect("Failed to call dump()");
    let lua_output = format!("return {}", serialized);

    eprintln!("KoReader dump() output:\n{}", lua_output);

    // Write to temp file and parse with our Rust parser
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("metadata.epub.lua");
    std::fs::write(&test_file, &lua_output).unwrap();

    let parser = crate::source::koreader::LuaParser::new();
    let metadata = parser
        .parse(&test_file)
        .expect("Failed to parse KoReader-generated metadata");

    // Verify all fields were parsed correctly
    assert_eq!(metadata.doc_pages, Some(250));
    assert_eq!(
        metadata.partial_md5_checksum.as_deref(),
        Some("abc123def456789")
    );
    assert!((metadata.percent_finished.unwrap() - 0.25).abs() < 0.0001);
    assert_eq!(metadata.text_lang.as_deref(), Some("en-US"));
    assert_eq!(metadata.pagemap_use_page_labels, Some(true));
    assert_eq!(metadata.pagemap_chars_per_synthetic_page, Some(1500));
    assert_eq!(metadata.pagemap_doc_pages, Some(320));
    assert_eq!(metadata.pagemap_current_page_label.as_deref(), Some("12"));
    assert_eq!(metadata.pagemap_last_page_label.as_deref(), Some("320"));

    let presentation = metadata
        .reader_presentation
        .expect("reader_presentation should be present");
    assert_eq!(presentation.font_face.as_deref(), Some("Noto Serif"));
    assert_eq!(presentation.font_size_pt, Some(24.0));
    assert_eq!(presentation.line_spacing_percent, Some(110));
    assert_eq!(presentation.h_page_margins, Some([30, 30]));
    assert_eq!(presentation.t_page_margin, Some(30));
    assert_eq!(presentation.b_page_margin, Some(15));
    assert_eq!(presentation.embedded_css, Some(true));
    assert_eq!(presentation.embedded_fonts, Some(false));
    assert_eq!(presentation.hyphenation, Some(true));
    assert_eq!(presentation.floating_punctuation, Some(false));
    assert_eq!(presentation.word_spacing, Some([100, 90]));

    // Verify doc_props
    let doc_props = metadata.doc_props.expect("doc_props should be present");
    assert_eq!(doc_props.authors.as_deref(), Some("John Smith"));
    assert_eq!(doc_props.title.as_deref(), Some("My Programming Book"));

    // Verify stats
    let stats = metadata.stats.expect("stats should be present");
    assert_eq!(stats.highlights, Some(5));
    assert_eq!(stats.notes, Some(2));

    // Verify summary
    let summary = metadata.summary.expect("summary should be present");
    assert_eq!(summary.rating, Some(4));
    assert_eq!(summary.status, crate::shelf::models::BookStatus::Reading);

    // Verify annotation
    assert_eq!(metadata.annotations.len(), 1);
    let annotation = &metadata.annotations[0];
    assert_eq!(
        annotation.chapter.as_deref(),
        Some("Chapter 1: Introduction")
    );
    assert_eq!(
        annotation.note.as_deref(),
        Some("Remember this section for later")
    );
    assert_eq!(annotation.color.as_deref(), Some("yellow"));
    assert_eq!(annotation.drawer.as_deref(), Some("underscore"));
    assert!(annotation.is_highlight());
}
