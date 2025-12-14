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
                },
            },
            doc_pages = 250,
            doc_path = "/storage/Books/My Book.epub",
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

    let parser = crate::koreader::LuaParser::new();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let metadata = rt
        .block_on(parser.parse(&test_file))
        .expect("Failed to parse KoReader-generated metadata");

    // Verify all fields were parsed correctly
    assert_eq!(metadata.doc_pages, Some(250));
    assert_eq!(
        metadata.partial_md5_checksum.as_deref(),
        Some("abc123def456789")
    );
    assert!((metadata.percent_finished.unwrap() - 0.25).abs() < 0.0001);
    assert_eq!(metadata.text_lang.as_deref(), Some("en-US"));

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
    assert_eq!(summary.status, crate::models::BookStatus::Reading);

    // Verify annotation
    assert_eq!(metadata.annotations.len(), 1);
    let annotation = &metadata.annotations[0];
    assert_eq!(
        annotation.chapter.as_deref(),
        Some("Chapter 1: Introduction")
    );
    assert!(annotation.is_highlight());
}
