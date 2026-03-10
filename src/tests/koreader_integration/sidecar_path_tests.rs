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
