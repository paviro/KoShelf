use super::*;
use crate::statistics_parser::StatisticsParser;
use mlua::{Lua, LuaOptions, StdLib, Table};
use rusqlite::{Connection, params};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Debug)]
struct StatisticsLuaArtifacts {
    statements: Vec<String>,
    totals_query: String,
}

#[derive(Debug, PartialEq, Eq)]
struct ColumnDescriptor {
    name: String,
    data_type: String,
    notnull: bool,
    default_value: Option<String>,
}

impl ColumnDescriptor {
    fn new(name: &str, data_type: &str, notnull: bool, default_value: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            data_type: data_type.to_string(),
            notnull,
            default_value: default_value.map(|s| s.to_string()),
        }
    }
}

struct TestDatabase {
    _temp_dir: TempDir,
    conn: Connection,
    path: PathBuf,
}

impl TestDatabase {
    fn new(statements: &[String]) -> Self {
        let temp_dir =
            TempDir::new().expect("Failed to create temporary directory for statistics DB");
        let path = temp_dir.path().join("statistics.sqlite3");
        let conn = Connection::open(&path).expect("Failed to open temporary statistics DB");
        for stmt in statements {
            conn.execute_batch(stmt).unwrap_or_else(|err| {
                panic!(
                    "Failed to execute KoReader schema statement:\n{}\nError: {}",
                    stmt, err
                )
            });
        }
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE;")
            .expect("Failed to normalize SQLite journal mode for test database");

        Self {
            _temp_dir: temp_dir,
            conn,
            path,
        }
    }
}

#[test]
fn test_statistics_schema_matches_koreader() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db = TestDatabase::new(&artifacts.statements);

    let data_columns = fetch_table_info(&db.conn, "page_stat_data");
    assert_eq!(
        data_columns,
        vec![
            ColumnDescriptor::new("id_book", "INTEGER", false, None),
            ColumnDescriptor::new("page", "INTEGER", true, Some("0")),
            ColumnDescriptor::new("start_time", "INTEGER", true, Some("0")),
            ColumnDescriptor::new("duration", "INTEGER", true, Some("0")),
            ColumnDescriptor::new("total_pages", "INTEGER", true, Some("0")),
        ],
        "KoReader page_stat_data schema changed. Update parser expectations."
    );

    let view_columns = fetch_table_info(&db.conn, "page_stat");
    let view_column_names: Vec<&str> = view_columns.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        view_column_names,
        vec!["id_book", "page", "start_time", "duration"],
        "page_stat view columns changed unexpectedly"
    );

    let view_sql: String = db
        .conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='view' AND name='page_stat'",
            [],
            |row| row.get(0),
        )
        .expect("Unable to read page_stat view SQL");
    let lowered = view_sql.to_lowercase();
    for snippet in [
        "create view",
        "first_page + idx - 1 as page",
        "duration / (last_page - first_page + 1) as duration",
        "((page - 1) * pages) / total_pages + 1 as first_page",
        "join (select number as idx from numbers)",
    ] {
        assert!(
            lowered.contains(snippet),
            "Expected KoReader view definition to contain '{}', actual SQL:\n{}",
            snippet,
            view_sql
        );
    }
}

#[test]
fn test_statistics_totals_query_matches_parser() {
    let koreader_dir = get_koreader_dir();
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
    let artifacts = load_statistics_artifacts(&lua, koreader_dir.path());
    let db = TestDatabase::new(&artifacts.statements);
    seed_sample_statistics(&db.conn);

    let query = format_percent_d(&artifacts.totals_query, &[1]);
    let (expected_pages, expected_duration): (i64, i64) = db
        .conn
        .query_row(&query, [], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("KoReader totals query failed");

    let stats_data =
        StatisticsParser::parse(&db.path).expect("Rust parser failed to read statistics DB");
    let mut unique_pages = HashSet::new();
    let mut total_duration = 0;
    for stat in stats_data.page_stats.iter().filter(|ps| ps.id_book == 1) {
        unique_pages.insert(stat.page);
        total_duration += stat.duration;
    }

    assert_eq!(
        unique_pages.len() as i64,
        expected_pages,
        "Rust unique page counting diverged from KoReader SQL"
    );
    assert_eq!(
        total_duration, expected_duration,
        "Rust duration aggregation diverged from KoReader SQL"
    );
}

fn fetch_table_info(conn: &Connection, table: &str) -> Vec<ColumnDescriptor> {
    let pragma = format!("PRAGMA table_info({})", table);
    let mut stmt = conn
        .prepare(&pragma)
        .expect("Failed to prepare pragma query");
    let iter = stmt
        .query_map([], |row| {
            Ok(ColumnDescriptor {
                name: row.get(1)?,
                data_type: row.get(2)?,
                notnull: row.get::<_, i64>(3)? != 0,
                default_value: row.get(4)?,
            })
        })
        .expect("Failed to inspect schema");

    iter.filter_map(Result::ok).collect()
}

fn seed_sample_statistics(conn: &Connection) {
    conn.execute(
        "INSERT INTO book (id, title, authors, notes, last_open, highlights, pages, series, language, md5, total_read_time, total_read_pages)
         VALUES (?1, ?2, ?3, NULL, NULL, NULL, ?4, NULL, ?5, ?6, NULL, NULL);",
        params![1, "Test Book", "Test Author", 300_i64, "en", "test-md5"],
    )
    .expect("Failed to seed book row");

    let inserts = [
        (1_i64, 1_i64, 1_000_i64, 90_i64, 100_i64),
        (1_i64, 25_i64, 2_000_i64, 60_i64, 120_i64),
    ];

    for (id_book, page, start_time, duration, total_pages) in inserts {
        conn.execute(
            "INSERT INTO page_stat_data (id_book, page, start_time, duration, total_pages)
             VALUES (?1, ?2, ?3, ?4, ?5);",
            params![id_book, page, start_time, duration, total_pages],
        )
        .expect("Failed to seed page_stat_data row");
    }
}

fn load_statistics_artifacts(lua: &Lua, root: &Path) -> StatisticsLuaArtifacts {
    let script = build_statistics_lua_script(root);
    let result: Table = lua
        .load(&script)
        .eval()
        .expect("Failed to bootstrap KoReader statistics module");

    let executed_table: Table = result
        .get("executed_sql")
        .expect("Missing executed_sql results from Lua");
    let statements = executed_table
        .sequence_values::<String>()
        .collect::<mlua::Result<Vec<_>>>()
        .expect("Failed to collect KoReader schema statements");
    let totals_query: String = result
        .get("totals_query")
        .expect("Failed to capture KoReader totals query");

    StatisticsLuaArtifacts {
        statements,
        totals_query,
    }
}

fn format_percent_d(template: &str, values: &[i64]) -> String {
    let mut formatted = template.to_string();
    for value in values {
        if let Some(idx) = formatted.find("%d") {
            formatted.replace_range(idx..idx + 2, &value.to_string());
        } else {
            panic!(
                "Template '{}' is missing enough placeholders for provided values",
                template
            );
        }
    }

    assert!(
        !formatted.contains("%d"),
        "Unused %d placeholders remain after formatting SQL: {}",
        formatted
    );
    formatted
}

fn build_statistics_lua_script(root: &Path) -> String {
    LUA_STATISTICS_TEMPLATE
        .replace("__ROOT__", &root.display().to_string())
        .replace("__MOCKS__", LUA_STATISTICS_MOCKS)
}

const LUA_STATISTICS_TEMPLATE: &str = r#"
local root = [[__ROOT__]]

__MOCKS__

local executed_sql = {}
local conn = {}

function conn:exec(sql)
    table.insert(executed_sql, sql)
end

local ReaderStatistics = dofile(root .. "/plugins/statistics.koplugin/main.lua")
ReaderStatistics:createDB(conn)

local function find_upvalue(func, target)
    local idx = 1
    while true do
        local name, value = debug.getupvalue(func, idx)
        if not name then
            break
        end
        if name == target then
            return value
        end
        idx = idx + 1
    end
    return nil
end

local function locate_totals_query()
    local candidates = {
        "initData",
        "addBookStatToDB",
        "getPageTimeTotalStats",
        "getPageTimeTotalCompletions",
    }

    for _, name in ipairs(candidates) do
        local func = ReaderStatistics[name]
        if type(func) == "function" then
            local value = find_upvalue(func, "STATISTICS_SQL_BOOK_TOTALS_QUERY")
            if value then
                return value
            end
        end
    end
    return nil
end

local totals_query = locate_totals_query()
assert(totals_query, "Unable to locate STATISTICS_SQL_BOOK_TOTALS_QUERY upvalue in ReaderStatistics")

return {
    executed_sql = executed_sql,
    totals_query = totals_query,
}
"#;

const LUA_STATISTICS_MOCKS: &str = r#"
package.path = table.concat({
    root .. "/?.lua",
    root .. "/?/init.lua",
    root .. "/frontend/?.lua",
    root .. "/frontend/?/init.lua",
    root .. "/plugins/?.lua",
    root .. "/plugins/?/init.lua",
    package.path,
}, ";")

local noop = function() end

local function stub_module(name, value)
    package.preload[name] = function()
        return value
    end
end

stub_module("ui/bidi", {})

local ButtonDialog = {}
function ButtonDialog:new()
    return setmetatable({}, { __index = self })
end
stub_module("ui/widget/buttondialog", ButtonDialog)

local ConfirmBox = {}
function ConfirmBox:new()
    return setmetatable({}, { __index = self })
end
stub_module("ui/widget/confirmbox", ConfirmBox)

stub_module("datastorage", {
    getDataDir = function() return "/tmp/koreader" end,
    getSettingsDir = function() return "/tmp/koreader" end,
})

stub_module("device", {
    canUseWAL = function() return true end,
    hasColorScreen = function() return false end,
})

stub_module("dispatcher", {
    registerAction = noop,
})

local DocSettings = {}
function DocSettings:open()
    return {
        readSetting = function(_, _, default) return default end,
    }
end
stub_module("docsettings", DocSettings)

local InfoMessage = {}
function InfoMessage:new(opts)
    return setmetatable(opts or {}, { __index = self })
end
stub_module("ui/widget/infomessage", InfoMessage)

local KeyValuePage = {}
function KeyValuePage:new()
    return setmetatable({}, { __index = self })
end
stub_module("ui/widget/keyvaluepage", KeyValuePage)

stub_module("optmath", {
    round = function(value) return value end,
})

stub_module("readerprogress", {})

stub_module("readhistory", {
    hist = {},
})

stub_module("lua-ljsqlite3/init", {
    open = function()
        error("sqlite access not available in tests")
    end,
})

stub_module("frontend/apps/cloudstorage/syncservice", {
    new = function() return {} end,
})

stub_module("ui/uimanager", {
    show = noop,
    close = noop,
    forceRePaint = noop,
    broadcastEvent = noop,
})

local Widget = {}
Widget.__index = Widget
function Widget:extend(props)
    props = props or {}
    return setmetatable(props, { __index = self })
end
stub_module("ui/widget/widget", Widget)

stub_module("datetime", {})

stub_module("ffi/util", {
    template = function(_, text) return text end,
    basename = function(path) return path end,
    copyFile = noop,
    fsyncDirectory = noop,
    joinPath = function(path, name) return path .. "/" .. name end,
})

stub_module("libs/libkoreader-lfs", {
    attributes = function()
        return nil
    end,
})

stub_module("logger", {
    dbg = noop,
    info = noop,
    warn = noop,
})

stub_module("util", {
    partialMD5 = function()
        return "deadbeefdeadbeefdeadbeefdeadbeef"
    end,
    tableSize = function(tbl)
        local count = 0
        if tbl then
            for _ in pairs(tbl) do
                count = count + 1
            end
        end
        return count
    end,
    splitFilePathName = function(path)
        return "", path
    end,
})

local gettext = {}
function gettext.pgettext(_, text)
    return text
end
function gettext.ngettext(_, singular, plural, n)
    if n == 1 then
        return singular
    else
        return plural
    end
end
setmetatable(gettext, {
    __call = function(_, text)
        return text
    end,
})
stub_module("gettext", gettext)

_G.G_reader_settings = {
    readSetting = function(_, _, default) return default end,
    has = function() return false end,
    isTrue = function() return false end,
}
"#;
