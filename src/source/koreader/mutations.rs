//! KoReader-specific field mutations for metadata sidecar files.
//!
//! These functions coordinate `LuaWriter` with KoReader's table structure
//! knowledge (e.g. `summary.note`, `annotations[n].color`).
//! HTTP handlers call into this module rather than using `LuaWriter` directly.

use super::LuaWriter;
use anyhow::Result;
use chrono::Local;
use mlua::{Lua, Table, Value};
use std::path::Path;

// NOTE: `chrono::Local` is used only for `summary.modified` in item mutations.
// Annotation `datetime_updated` is computed by the caller and passed in, so
// the Lua file and DB always receive the same timestamp.

// ── Public write operations ──────────────────────────────────────────────

/// Write item-level field changes to a KoReader metadata sidecar file.
pub fn write_item_metadata(
    metadata_path: &Path,
    review_note: Option<&str>,
    rating: Option<u32>,
    status: Option<&str>,
) -> Result<()> {
    let writer = LuaWriter::new();
    writer.write(metadata_path, |lua, table| {
        apply_item_mutations(lua, table, review_note, rating, status)
    })
}

/// Write annotation-level field changes to a KoReader metadata sidecar file.
///
/// `lua_index` is the 0-based position from the database; this function
/// converts to Lua's 1-based indexing internally.
///
/// `datetime_updated` is pre-computed by the caller so the Lua file and DB
/// receive the same timestamp.
pub fn write_annotation_metadata(
    metadata_path: &Path,
    lua_index: i32,
    note: Option<&str>,
    color: Option<&str>,
    drawer: Option<&str>,
    datetime_updated: Option<&str>,
) -> Result<()> {
    let lua_array_index = (lua_index + 1) as i64;
    let writer = LuaWriter::new();
    writer.write(metadata_path, |_lua, table| {
        apply_annotation_mutations(
            table,
            lua_array_index,
            note,
            color,
            drawer,
            datetime_updated,
        )
    })
}

/// Delete an annotation from a KoReader metadata sidecar file.
///
/// `lua_index` is the 0-based position from the database.
/// `is_highlight` indicates whether the deleted annotation is a highlight (true)
/// or a bookmark (false), used to decrement the correct stats counter.
pub fn delete_annotation(metadata_path: &Path, lua_index: i32, is_highlight: bool) -> Result<()> {
    let lua_array_index = (lua_index + 1) as i64;
    let writer = LuaWriter::new();
    writer.write(metadata_path, |lua, table| {
        apply_annotation_deletion(lua, table, lua_array_index, is_highlight)
    })
}

// ── Mutation helpers ─────────────────────────────────────────────────────

fn apply_item_mutations(
    lua: &Lua,
    table: &Table,
    review_note: Option<&str>,
    rating: Option<u32>,
    status: Option<&str>,
) -> mlua::Result<()> {
    // Ensure summary table exists
    let summary: Table = match table.get("summary")? {
        Value::Table(t) => t,
        _ => {
            let t = lua.create_table()?;
            table.set("summary", t.clone())?;
            t
        }
    };

    if let Some(note) = review_note {
        summary.set("note", note)?;
    }

    if let Some(rating) = rating {
        if rating == 0 {
            summary.set("rating", Value::Nil)?;
        } else {
            summary.set("rating", rating)?;
        }
    }

    if let Some(status) = status {
        summary.set("status", status)?;
    }

    // Match KoReader's filemanagerutil.saveSummary(): always stamp the modified date.
    summary.set("modified", Local::now().format("%Y-%m-%d").to_string())?;

    Ok(())
}

fn apply_annotation_mutations(
    table: &Table,
    lua_array_index: i64,
    note: Option<&str>,
    color: Option<&str>,
    drawer: Option<&str>,
    datetime_updated: Option<&str>,
) -> mlua::Result<()> {
    let annotations: Table = table.get("annotations")?;
    let annotation: Table = match annotations.get::<Value>(lua_array_index)? {
        Value::Table(t) => t,
        _ => {
            return Err(mlua::Error::external(format!(
                "no annotation at Lua index {}",
                lua_array_index
            )));
        }
    };

    if let Some(note) = note {
        annotation.set("note", note)?;
    }

    if let Some(color) = color {
        annotation.set("color", color)?;
    }

    if let Some(drawer) = drawer {
        annotation.set("drawer", drawer)?;
    }

    if let Some(dt) = datetime_updated {
        annotation.set("datetime_updated", dt)?;
    }

    Ok(())
}

fn apply_annotation_deletion(
    lua: &Lua,
    table: &Table,
    lua_array_index: i64,
    is_highlight: bool,
) -> mlua::Result<()> {
    let annotations: Table = table.get("annotations")?;

    // Verify the annotation exists before removing.
    match annotations.get::<Value>(lua_array_index)? {
        Value::Table(_) => {}
        _ => {
            return Err(mlua::Error::external(format!(
                "no annotation at Lua index {}",
                lua_array_index
            )));
        }
    }

    // Use Lua's table.remove() which shifts subsequent elements down,
    // matching KoReader's readerbookmark:removeItemByIndex().
    let table_lib: Table = lua.globals().get("table")?;
    let remove_fn: mlua::Function = table_lib.get("remove")?;
    remove_fn.call::<Value>((annotations, lua_array_index))?;

    // Decrement the stats counter matching KoReader's AnnotationsModified event.
    if let Value::Table(stats) = table.get("stats")? {
        let key = if is_highlight { "highlights" } else { "notes" };
        if let Value::Integer(count) = stats.get(key)? {
            stats.set(key, (count - 1).max(0))?;
        }
    }

    Ok(())
}
