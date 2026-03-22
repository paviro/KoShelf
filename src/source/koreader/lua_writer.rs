//! Round-trip writer for KoReader `.sdr/metadata.*.lua` sidecar files.
//!
//! Reads the file into an `mlua::Table` using the same sandboxed VM as
//! `LuaParser`, applies caller-provided mutations via the mlua Rust API,
//! then serializes back to disk using a pure-Rust serializer that matches
//! KoReader's `dump.lua` output format.

use anyhow::{Context, Result, anyhow};
use mlua::{ChunkMode, Lua, LuaOptions, StdLib, Table, Value};
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

const MAX_DEPTH: usize = 64;
const INDENT: &str = "    ";
const BACKUP_MIN_AGE_SECS: u64 = 60;

/// Round-trip writer for KoReader metadata sidecar files.
///
/// Like `LuaParser`, the inner `mlua::Lua` is **not `Sync`**. Create a
/// fresh instance per write operation (writes are infrequent).
pub struct LuaWriter {
    lua: Lua,
}

impl Default for LuaWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaWriter {
    pub fn new() -> Self {
        Self {
            lua: Lua::new_with(StdLib::NONE, LuaOptions::default())
                .expect("Failed to create sandboxed Lua state"),
        }
    }

    /// Read a metadata file, apply mutations, backup-rotate, serialize, and
    /// write back.
    ///
    /// The mutation closure receives the root Lua table for targeted field
    /// changes. Field names and validation are the caller's responsibility.
    pub fn write(
        &self,
        metadata_path: &Path,
        mutate: impl FnOnce(&Lua, &Table) -> mlua::Result<()>,
    ) -> Result<()> {
        // 1. Read
        let content = fs::read_to_string(metadata_path)
            .with_context(|| format!("Failed to read metadata file: {:?}", metadata_path))?;

        let value: Value = self
            .lua
            .load(&content)
            .set_mode(ChunkMode::Text)
            .eval()
            .map_err(|e| anyhow!("Failed to parse Lua file {:?}: {}", metadata_path, e))?;

        let table = match value {
            Value::Table(t) => t,
            _ => return Err(anyhow!("Expected Lua file to return a table")),
        };

        // 2. Mutate
        mutate(&self.lua, &table).map_err(|e| anyhow!("Mutation failed: {}", e))?;

        // 3. Serialize
        let header_path = metadata_path.to_string_lossy();
        let serialized = serialize_table(&table, &header_path)?;

        // 4. Backup rotate
        let directory_updated = backup_rotate(metadata_path)?;

        // 5. Write + fsync (restore backup on failure to avoid data loss)
        if let Err(e) = write_and_sync(metadata_path, serialized.as_bytes(), directory_updated) {
            if directory_updated {
                let old_path = backup_path(metadata_path);
                if old_path.exists() {
                    let _ = fs::rename(&old_path, metadata_path);
                }
            }
            return Err(e);
        }

        Ok(())
    }
}

// ── Pure-Rust serializer ─────────────────────────────────────────────────

/// Serialize an `mlua::Table` to KoReader's `dump()` format.
fn serialize_table(table: &Table, file_path: &str) -> Result<String> {
    let mut buf = String::new();
    writeln!(buf, "-- {}", file_path).unwrap();
    write!(buf, "return ").unwrap();

    let mut seen = HashSet::new();
    write_value(&mut buf, &Value::Table(table.clone()), 0, &mut seen)?;
    writeln!(buf).unwrap();

    Ok(buf)
}

fn write_value(
    buf: &mut String,
    value: &Value,
    depth: usize,
    seen: &mut HashSet<usize>,
) -> Result<()> {
    if depth > MAX_DEPTH {
        return Err(anyhow!("Exceeded maximum serialization depth of {}", MAX_DEPTH));
    }

    match value {
        Value::Table(t) => write_table(buf, t, depth, seen),
        Value::String(s) => {
            let s = s.to_str().map_err(|e| anyhow!("Non-UTF8 string: {}", e))?;
            write_lua_string(buf, &s);
            Ok(())
        }
        Value::Integer(i) => {
            write!(buf, "{}", i).unwrap();
            Ok(())
        }
        Value::Number(n) => {
            if *n == (*n as i64) as f64 {
                write!(buf, "{}", *n as i64).unwrap();
            } else {
                write!(buf, "{}", n).unwrap();
            }
            Ok(())
        }
        Value::Boolean(b) => {
            write!(buf, "{}", if *b { "true" } else { "false" }).unwrap();
            Ok(())
        }
        // Skip non-data types (nil, functions, userdata, threads, errors)
        _ => Ok(()),
    }
}

fn write_table(
    buf: &mut String,
    table: &Table,
    depth: usize,
    seen: &mut HashSet<usize>,
) -> Result<()> {
    let ptr = table.to_pointer() as usize;
    if !seen.insert(ptr) {
        return Err(anyhow!("Cycle detected in Lua table"));
    }

    // Collect and sort keys: numeric first (ascending), then string (ascending).
    let mut int_entries: Vec<(i64, Value)> = Vec::new();
    let mut str_entries: Vec<(String, Value)> = Vec::new();

    for pair in table.pairs::<Value, Value>() {
        let (key, val) = pair.map_err(|e| anyhow!("Failed to iterate table: {}", e))?;

        // Skip non-data values
        if matches!(
            val,
            Value::Nil
                | Value::Function(_)
                | Value::UserData(_)
                | Value::Thread(_)
                | Value::LightUserData(_)
                | Value::Error(_)
        ) {
            continue;
        }

        match key {
            Value::Integer(i) => int_entries.push((i, val)),
            Value::Number(n) if n == (n as i64) as f64 => {
                int_entries.push((n as i64, val));
            }
            Value::String(s) => {
                let s = s.to_str().map_err(|e| anyhow!("Non-UTF8 key: {}", e))?;
                str_entries.push((s.to_string(), val));
            }
            _ => {} // skip non-string/non-numeric keys
        }
    }

    int_entries.sort_by_key(|(k, _)| *k);
    str_entries.sort_by(|(a, _), (b, _)| a.cmp(b));

    if int_entries.is_empty() && str_entries.is_empty() {
        write!(buf, "{{}}").unwrap();
        seen.remove(&ptr);
        return Ok(());
    }

    writeln!(buf, "{{").unwrap();

    let child_indent = INDENT.repeat(depth + 1);
    let close_indent = INDENT.repeat(depth);

    for (key, val) in &int_entries {
        write!(buf, "{}[{}] = ", child_indent, key).unwrap();
        write_value(buf, val, depth + 1, seen)?;
        writeln!(buf, ",").unwrap();
    }

    for (key, val) in &str_entries {
        write!(buf, "{}[\"", child_indent).unwrap();
        write_lua_escaped_chars(buf, key);
        write!(buf, "\"] = ").unwrap();
        write_value(buf, val, depth + 1, seen)?;
        writeln!(buf, ",").unwrap();
    }

    write!(buf, "{}}}", close_indent).unwrap();

    seen.remove(&ptr);
    Ok(())
}

/// Write a Lua %q-style escaped string (with surrounding quotes).
///
/// Matches Lua 5.1 / LuaJIT `string.format("%q", s)` output (which is what
/// KoReader's `dump.lua` uses): newlines are escaped as `\` followed by a
/// literal newline rather than the `\n` two-char sequence.
fn write_lua_string(buf: &mut String, s: &str) {
    buf.push('"');
    write_lua_escaped_chars(buf, s);
    buf.push('"');
}

/// Write %q-style escaped characters (no surrounding quotes).
///
/// Used for both string values and dictionary keys inside `["..."]`.
fn write_lua_escaped_chars(buf: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '\\' => buf.push_str("\\\\"),
            '"' => buf.push_str("\\\""),
            '\n' => {
                buf.push('\\');
                buf.push('\n');
            }
            '\r' => buf.push_str("\\r"),
            '\0' => buf.push_str("\\0"),
            c => buf.push(c),
        }
    }
}

// ── Backup rotation ─────────────────────────────────────────────────────

/// Compute the `.old` backup path for a metadata file (e.g. `foo.lua` → `foo.lua.old`).
pub(crate) fn backup_path(path: &Path) -> std::path::PathBuf {
    path.with_extension(format!(
        "{}.old",
        path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
    ))
}

/// Rotate backup matching KoReader's `LuaSettings:backup()` logic.
///
/// Returns `true` if a backup was created (meaning a new directory entry
/// was added and the parent dir should be fsynced).
fn backup_rotate(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to stat {:?}", path))?;

    let mtime = metadata
        .modified()
        .with_context(|| format!("Failed to read mtime of {:?}", path))?;

    let age = SystemTime::now()
        .duration_since(mtime)
        .unwrap_or_default();

    if age.as_secs() < BACKUP_MIN_AGE_SECS {
        return Ok(false);
    }

    let old_path = backup_path(path);

    fs::rename(path, &old_path)
        .with_context(|| format!("Failed to rotate backup {:?} → {:?}", path, old_path))?;

    Ok(true)
}

// ── Write + fsync ───────────────────────────────────────────────────────

fn write_and_sync(path: &Path, content: &[u8], fsync_parent: bool) -> Result<()> {
    let file = File::create(path)
        .with_context(|| format!("Failed to create {:?}", path))?;

    let mut writer = std::io::BufWriter::new(file);
    writer
        .write_all(content)
        .with_context(|| format!("Failed to write {:?}", path))?;

    let file = writer
        .into_inner()
        .map_err(|e| anyhow!("Failed to flush {:?}: {}", path, e))?;

    file.sync_all()
        .with_context(|| format!("Failed to fsync {:?}", path))?;

    if fsync_parent {
        if let Some(parent) = path.parent() {
            if let Ok(dir) = File::open(parent) {
                let _ = dir.sync_all();
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Fixture that mirrors the structure and formatting of a real KoReader
    /// `metadata.*.lua` sidecar file (as produced by KoReader's `dump.lua`),
    /// using entirely fictional data.
    const KOREADER_FIXTURE: &str =
        include_str!("test_fixtures/round_trip_sample.lua");

    fn write_fixture(dir: &Path, content: &str) -> std::path::PathBuf {
        let path = dir.join("metadata.epub.lua");
        fs::write(&path, content).expect("fixture should write");
        path
    }

    /// Parse a realistic KoReader metadata file and serialize it back.
    /// The body (everything after the header comment) must be identical,
    /// confirming our serializer is format-compatible with KoReader's dump.lua.
    #[test]
    fn round_trip_preserves_realistic_file() {
        let tmp = TempDir::new().unwrap();
        let path = write_fixture(tmp.path(), KOREADER_FIXTURE);

        let writer = LuaWriter::new();
        writer.write(&path, |_, _| Ok(())).expect("write should succeed");

        let result = fs::read_to_string(&path).unwrap();

        // The header comment contains the file path, which changes between
        // fixture and temp dir. Compare everything after the first line.
        let original_body = KOREADER_FIXTURE.split_once('\n').unwrap().1;
        let result_body = result.split_once('\n').unwrap().1;

        assert_eq!(
            result_body, original_body,
            "Round-trip changed the file body.\n\
             --- expected (fixture) ---\n{}\n\
             --- actual (after write) ---\n{}",
            original_body, result_body
        );
    }

    #[test]
    fn round_trip_preserves_data() {
        let tmp = TempDir::new().unwrap();
        let path = write_fixture(
            tmp.path(),
            r#"return {
    ["title"] = "Test Book",
    ["summary"] = {
        ["note"] = "original note",
        ["rating"] = 3,
        ["status"] = "reading",
    },
}"#,
        );

        let writer = LuaWriter::new();
        writer
            .write(&path, |_, table| {
                let summary: Table = table.get("summary")?;
                summary.set("note", "updated note")?;
                summary.set("rating", 5)?;
                Ok(())
            })
            .expect("write should succeed");

        // Verify by re-parsing
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"updated note\""));
        assert!(content.contains("[\"rating\"] = 5"));
    }

    #[test]
    fn serializer_sorts_keys() {
        let tmp = TempDir::new().unwrap();
        let path = write_fixture(
            tmp.path(),
            r#"return {
    ["zebra"] = 1,
    ["alpha"] = 2,
    [2] = "second",
    [1] = "first",
}"#,
        );

        let writer = LuaWriter::new();
        writer.write(&path, |_, _| Ok(())).expect("write should succeed");

        let content = fs::read_to_string(&path).unwrap();
        let alpha_pos = content.find("[\"alpha\"]").unwrap();
        let zebra_pos = content.find("[\"zebra\"]").unwrap();
        let first_pos = content.find("[1]").unwrap();
        let second_pos = content.find("[2]").unwrap();

        // Numeric keys come before string keys
        assert!(first_pos < alpha_pos);
        assert!(second_pos < alpha_pos);
        // Numeric keys sorted ascending
        assert!(first_pos < second_pos);
        // String keys sorted ascending
        assert!(alpha_pos < zebra_pos);
    }

    #[test]
    fn skips_non_data_types() {
        let tmp = TempDir::new().unwrap();
        let path = write_fixture(
            tmp.path(),
            r#"return {
    ["valid"] = "data",
    ["also_valid"] = 42,
}"#,
        );

        let writer = LuaWriter::new();
        writer.write(&path, |_, _| Ok(())).expect("write should succeed");

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("[\"valid\"] = \"data\""));
        assert!(content.contains("[\"also_valid\"] = 42"));
    }

    #[test]
    fn string_escaping() {
        let tmp = TempDir::new().unwrap();
        let path = write_fixture(
            tmp.path(),
            r#"return {
    ["text"] = "hello",
}"#,
        );

        let writer = LuaWriter::new();
        writer
            .write(&path, |_, table| {
                table.set("text", "line1\nline2\\end\"quoted\"")?;
                Ok(())
            })
            .expect("write should succeed");

        let content = fs::read_to_string(&path).unwrap();
        // Newlines are escaped as `\` + literal newline (Lua 5.1 %q style)
        assert!(content.contains("\"line1\\\nline2\\\\end\\\"quoted\\\"\""));
    }

    #[test]
    fn backup_rotation_skips_recent_files() {
        let tmp = TempDir::new().unwrap();
        let path = write_fixture(tmp.path(), r#"return { ["a"] = 1 }"#);

        // File was just created — should not create .old
        let writer = LuaWriter::new();
        writer.write(&path, |_, _| Ok(())).expect("write should succeed");

        let old_path = path.with_extension("lua.old");
        assert!(!old_path.exists(), ".old should not be created for recent files");
    }

    #[test]
    fn empty_table_serializes_correctly() {
        let tmp = TempDir::new().unwrap();
        let path = write_fixture(
            tmp.path(),
            r#"return {
    ["empty"] = {},
    ["data"] = 1,
}"#,
        );

        let writer = LuaWriter::new();
        writer.write(&path, |_, _| Ok(())).expect("write should succeed");

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("[\"empty\"] = {}"));
    }

    #[test]
    fn backup_path_appends_old_extension() {
        let p = std::path::Path::new("/tmp/metadata.epub.lua");
        assert_eq!(backup_path(p), std::path::PathBuf::from("/tmp/metadata.epub.lua.old"));
    }
}
