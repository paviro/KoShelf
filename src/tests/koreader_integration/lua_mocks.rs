pub(super) const LUA_BASE_MOCKS: &str = r#"
package = package or {}
package.preload = package.preload or {}

local noop = function(...) return nil end

local function stub_module(name, value)
    package.preload[name] = function()
        return value
    end
end

stub_module("datastorage", {
    getDataDir = function() return "/tmp/koreader" end,
    getSettingsDir = function() return "/tmp/koreader" end,
    getHistoryDir = function() return "/tmp/koreader/history" end,
    getDocSettingsDir = function() return "/tmp/koreader/docsettings" end,
    getDocSettingsHashDir = function() return "/tmp/koreader/hashdocsettings" end,
})

stub_module("ffi/util", {
    template = function(_, text) return text end,
    basename = function(path) return path end,
    copyFile = function(...) return true end,
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
        return "koreader-test-checksum"
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
    makePath = noop,
    writeToFile = function() return true end,
    splitFileNameSuffix = function() return "" end,
    getFileNameSuffix = function() return "" end,
})

_G.G_reader_settings = {
    readSetting = function(_, _, default) return default end,
    has = function() return false end,
    isTrue = function() return false end,
}
"#;

pub(super) fn compose_lua_mocks(extra: &str) -> String {
    format!("{}\n{}", LUA_BASE_MOCKS, extra)
}
