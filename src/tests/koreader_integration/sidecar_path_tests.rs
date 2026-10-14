use super::lua_mocks::compose_lua_mocks;
use super::*;
use mlua::Lua;

const LUA_DOCSETTINGS_MOCKS_EXTRA: &str = r#"
    package.preload["luasettings"] = function()
        local LuaSettings = {}
        LuaSettings.__index = LuaSettings

        function LuaSettings:extend(base)
            base = base or {}
            base.__index = base
            setmetatable(base, self)
            return base
        end

        function LuaSettings:readSetting(_, default)
            return default
        end

        return LuaSettings
    end

    package.preload["dump"] = function()
        return function() return "" end
    end
"#;

fn load_koreader_docsettings(lua: &Lua, source: &str) -> (mlua::Function, mlua::Function) {
    let mocks = compose_lua_mocks(LUA_DOCSETTINGS_MOCKS_EXTRA);

    lua.load(&mocks)
        .set_name("docsettings_mocks")
        .exec()
        .expect("Failed to set up Lua mocks for docsettings.lua");

    let docsettings: mlua::Table = lua
        .load(source)
        .set_name("docsettings.lua")
        .eval()
        .expect("Failed to load KoReader docsettings.lua");

    let adapter: mlua::Function = lua
        .load(
            r#"
            return function(docsettings)
                return {
                    getSidecarDir = function(doc_path)
                        return docsettings.getSidecarDir(docsettings, doc_path)
                    end,
                    getSidecarFilename = docsettings.getSidecarFilename,
                }
            end
            "#,
        )
        .eval()
        .expect("Failed to create docsettings adapter");

    let exports: mlua::Table = adapter
        .call(docsettings)
        .expect("Failed to adapt docsettings helper functions");

    let get_sidecar_dir: mlua::Function =
        exports.get("getSidecarDir").expect("Missing getSidecarDir");
    let get_sidecar_filename: mlua::Function = exports
        .get("getSidecarFilename")
        .expect("Missing getSidecarFilename");

    (get_sidecar_dir, get_sidecar_filename)
}

/// Test that our sidecar path logic matches KoReader's docsettings.lua
#[test]
fn test_sidecar_paths_against_koreader() {
    let koreader_dir = get_koreader_dir();
    let lua = Lua::new();

    let docsettings_path = koreader_dir.path().join("frontend/docsettings.lua");
    let docsettings_src = std::fs::read_to_string(&docsettings_path).unwrap_or_else(|_| {
        panic!(
            "Failed to read KoReader docsettings.lua at {}",
            docsettings_path.display()
        )
    });

    let (get_sidecar_dir, get_sidecar_filename) = load_koreader_docsettings(&lua, &docsettings_src);

    let test_cases = vec![
        ("../../foo.pdf", "../../foo.sdr", "metadata.pdf.lua"),
        ("/foo/bar.pdf", "/foo/bar.sdr", "metadata.pdf.lua"),
        ("baz.pdf", "baz.sdr", "metadata.pdf.lua"),
        (
            "/path/to/book.epub",
            "/path/to/book.sdr",
            "metadata.epub.lua",
        ),
        ("book.djvu", "book.sdr", "metadata.djvu.lua"),
    ];

    for (doc_path, expected_dir, expected_filename) in test_cases {
        let lua_dir: String = get_sidecar_dir.call(doc_path).unwrap();
        let lua_filename: String = get_sidecar_filename.call(doc_path).unwrap();

        assert_eq!(
            lua_dir, expected_dir,
            "Sidecar dir for '{}' should be '{}'",
            doc_path, expected_dir
        );
        assert_eq!(
            lua_filename, expected_filename,
            "Sidecar filename for '{}' should be '{}'",
            doc_path, expected_filename
        );

        eprintln!(
            "✓ {} -> dir: {}, filename: {}",
            doc_path, lua_dir, lua_filename
        );
    }
}

/// Exercise KOReader's real open()/candidate sorter, not a Rust copy of it.
#[test]
fn resolver_fallback_matches_koreader_open() {
    use crate::shelf::models::LibraryItemFormat;
    use crate::source::{metadata::MetadataIndices, scanner::MetadataLocation};
    use std::time::{Duration, UNIX_EPOCH};

    let source =
        std::fs::read_to_string(get_koreader_dir().path().join("frontend/docsettings.lua"))
            .unwrap();
    // Slots are adjacent primary/backup, docsettings primary/backup, hash primary/backup.
    let cases: Vec<[Option<(u64, bool)>; 6]> = vec![
        [
            Some((100, true)),
            None,
            Some((200, true)),
            None,
            Some((300, true)),
            None,
        ],
        [
            Some((300, true)),
            None,
            Some((200, true)),
            None,
            Some((100, true)),
            None,
        ],
        [Some((100, true)); 6],
        [
            Some((100, true)),
            Some((400, true)),
            Some((300, true)),
            None,
            None,
            None,
        ],
        [
            Some((100, false)),
            Some((400, true)),
            Some((300, true)),
            None,
            None,
            None,
        ],
        [None, Some((400, true)), Some((300, true)), None, None, None],
        [
            None,
            None,
            Some((100, false)),
            Some((200, true)),
            None,
            Some((300, true)),
        ],
        [
            Some((100, true)),
            None,
            Some((300, false)),
            None,
            None,
            None,
        ],
        [None; 6],
    ];
    for case in cases {
        let dir = TempDir::new().unwrap();
        let book = dir.path().join("Book.epub");
        let doc = dir.path().join("docsettings");
        let hash = dir.path().join("hashdocsettings");
        std::fs::create_dir_all(&doc).unwrap();
        std::fs::create_dir_all(&hash).unwrap();
        let checksum = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let primaries = [
            dir.path().join("Book.sdr/metadata.epub.lua"),
            doc.join(
                format!(
                    "{}.sdr/metadata.epub.lua",
                    book.with_extension("").display()
                )
                .trim_start_matches('/'),
            ),
            hash.join(format!("aa/{checksum}.sdr/metadata.epub.lua")),
        ];
        for (slot, entry) in case.iter().enumerate() {
            if let Some((seconds, valid)) = entry {
                let path = if slot % 2 == 0 {
                    primaries[slot / 2].clone()
                } else {
                    crate::source::koreader::lua_writer::backup_path(&primaries[slot / 2])
                };
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(
                    &path,
                    if *valid {
                        "return {percent_finished=0.5}"
                    } else {
                        "return {}"
                    },
                )
                .unwrap();
                std::fs::File::options()
                    .write(true)
                    .open(&path)
                    .unwrap()
                    .set_times(std::fs::FileTimes::new().set_modified(
                        UNIX_EPOCH
                            + Duration::from_secs(*seconds)
                            + Duration::from_millis(slot as u64 * 100),
                    ))
                    .unwrap();
            }
        }
        let lua = Lua::new();
        lua.load(compose_lua_mocks(LUA_DOCSETTINGS_MOCKS_EXTRA))
            .exec()
            .unwrap();
        let storage: mlua::Table = lua.load("return require('datastorage')").eval().unwrap();
        for (key, path) in [
            ("getDocSettingsDir", &doc),
            ("getDocSettingsHashDir", &hash),
        ] {
            let path = path.to_string_lossy().into_owned();
            storage
                .set(
                    key,
                    lua.create_function(move |_, ()| Ok(path.clone())).unwrap(),
                )
                .unwrap();
        }
        let util: mlua::Table = lua.load("return require('util')").eval().unwrap();
        util.set(
            "partialMD5",
            lua.create_function(move |_, _: String| Ok(checksum))
                .unwrap(),
        )
        .unwrap();
        let lfs: mlua::Table = lua
            .load("return require('libs/libkoreader-lfs')")
            .eval()
            .unwrap();
        lfs.set(
            "attributes",
            lua.create_function(|lua, (path, attribute): (String, String)| {
                let Ok(meta) = std::fs::metadata(path) else {
                    return Ok(mlua::Value::Nil);
                };
                Ok(match attribute.as_str() {
                    "mode" => mlua::Value::String(lua.create_string(if meta.is_dir() {
                        "directory"
                    } else {
                        "file"
                    })?),
                    "size" => mlua::Value::Number(meta.len() as f64),
                    "modification" => mlua::Value::Number(
                        meta.modified()
                            .unwrap()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as f64,
                    ),
                    _ => mlua::Value::Nil,
                })
            })
            .unwrap(),
        )
        .unwrap();
        // KOReader deletes bad candidates; the oracle must not mutate our fixtures.
        lua.load("os.remove = function() return true end")
            .exec()
            .unwrap();
        let settings: mlua::Table = lua.load(&source).eval().unwrap();
        let open: mlua::Function = settings.get("open").unwrap();
        let opened: mlua::Table = open
            .call((settings, book.to_string_lossy().into_owned()))
            .unwrap();
        let expected: Option<String> = opened.get("source_candidate").unwrap();
        let actual = MetadataIndices::new(&MetadataLocation::Central {
            docsettings: doc,
            hashdocsettings: hash,
        })
        .unwrap()
        .resolve(&book, LibraryItemFormat::Epub, checksum, None)
        .selected;
        assert_eq!(
            actual.map(|s| s.path),
            expected.map(PathBuf::from),
            "case: {case:?}"
        );
    }
}
