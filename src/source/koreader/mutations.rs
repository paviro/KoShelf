//! KoReader-specific field mutations for metadata sidecar files.
//!
//! These functions coordinate `LuaWriter` with KoReader's table structure
//! knowledge (e.g. `summary.note`, `annotations[n].color`).
//! HTTP handlers call into this module rather than using `LuaWriter` directly.

use super::LuaWriter;
use anyhow::Result;
use mlua::{Lua, Table, Value};
use std::path::Path;

// NOTE: All timestamps (`summary.modified`, `datetime_updated`) are computed
// by the caller using the configured timezone, so the Lua file and DB always
// receive the same value and both match the KoReader device's local time.

// ── Public write operations ──────────────────────────────────────────────

/// Write item-level field changes to a KoReader metadata sidecar file.
///
/// `modified_date` is the caller-formatted date string (YYYY-MM-DD) stamped
/// on `summary.modified`, matching KoReader's `filemanagerutil.saveSummary()`.
pub fn write_item_metadata(
    metadata_path: &Path,
    review_note: Option<Option<&str>>,
    rating: Option<u32>,
    status: Option<&str>,
    modified_date: &str,
) -> Result<()> {
    let writer = LuaWriter::new();
    writer.write(metadata_path, |lua, table| {
        apply_item_mutations(lua, table, review_note, rating, status, modified_date)
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
    note: Option<Option<&str>>,
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
/// The correct stats counter is determined by reading the annotation's `drawer`
/// and `note` fields, matching KOReader's `getBookmarkType` classification.
pub fn delete_annotation(metadata_path: &Path, lua_index: i32) -> Result<()> {
    let lua_array_index = (lua_index + 1) as i64;
    let writer = LuaWriter::new();
    writer.write(metadata_path, |lua, table| {
        apply_annotation_deletion(lua, table, lua_array_index)
    })
}

// ── Mutation helpers ─────────────────────────────────────────────────────

fn apply_item_mutations(
    lua: &Lua,
    table: &Table,
    review_note: Option<Option<&str>>,
    rating: Option<u32>,
    status: Option<&str>,
    modified_date: &str,
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

    // None = don't touch, Some(None) = clear, Some(Some(v)) = set
    match review_note {
        Some(Some(note)) => summary.set("note", note)?,
        Some(None) => summary.set("note", Value::Nil)?,
        None => {}
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
    summary.set("modified", modified_date)?;

    Ok(())
}

fn apply_annotation_mutations(
    table: &Table,
    lua_array_index: i64,
    note: Option<Option<&str>>,
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

    // Capture the note status before mutation so we can update stats if it changes.
    // KOReader's getBookmarkType: drawer + note → "note", drawer + no note → "highlight".
    let had_note = if note.is_some() {
        let has_drawer = !matches!(annotation.get::<Value>("drawer")?, Value::Nil);
        if has_drawer {
            Some(!matches!(annotation.get::<Value>("note")?, Value::Nil))
        } else {
            None // bookmark — not tracked in stats
        }
    } else {
        None // note field not being changed
    };

    match note {
        Some(Some(text)) => annotation.set("note", text)?,
        Some(None) => annotation.set("note", Value::Nil)?,
        None => {}
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

    // Update stats counters if the annotation transitioned between highlight ↔ note.
    // Matches KOReader's setBookmarkNote / deleteItemNote AnnotationsModified events.
    if let Some(had_note) = had_note {
        let has_note = !matches!(annotation.get::<Value>("note")?, Value::Nil);
        if had_note != has_note {
            if let Value::Table(stats) = table.get("stats")? {
                if has_note {
                    // highlight → note
                    if let Value::Integer(c) = stats.get("highlights")? {
                        stats.set("highlights", (c - 1).max(0))?;
                    }
                    if let Value::Integer(c) = stats.get("notes")? {
                        stats.set("notes", c + 1)?;
                    }
                } else {
                    // note → highlight
                    if let Value::Integer(c) = stats.get("highlights")? {
                        stats.set("highlights", c + 1)?;
                    }
                    if let Value::Integer(c) = stats.get("notes")? {
                        stats.set("notes", (c - 1).max(0))?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn apply_annotation_deletion(
    lua: &Lua,
    table: &Table,
    lua_array_index: i64,
) -> mlua::Result<()> {
    let annotations: Table = table.get("annotations")?;

    // Read the annotation before removing to determine its KOReader type.
    let annotation: Table = match annotations.get::<Value>(lua_array_index)? {
        Value::Table(t) => t,
        _ => {
            return Err(mlua::Error::external(format!(
                "no annotation at Lua index {}",
                lua_array_index
            )));
        }
    };

    // KOReader's getBookmarkType: no drawer → "bookmark", drawer + note → "note",
    // drawer + no note → "highlight". Only highlights and notes are tracked in stats.
    let has_drawer = !matches!(annotation.get::<Value>("drawer")?, Value::Nil);
    let stats_key = if has_drawer {
        let has_note = !matches!(annotation.get::<Value>("note")?, Value::Nil);
        Some(if has_note { "notes" } else { "highlights" })
    } else {
        None // bookmark — not tracked in stats
    };

    // Use Lua's table.remove() which shifts subsequent elements down,
    // matching KoReader's readerbookmark:removeItemByIndex().
    let table_lib: Table = lua.globals().get("table")?;
    let remove_fn: mlua::Function = table_lib.get("remove")?;
    remove_fn.call::<Value>((annotations, lua_array_index))?;

    // Decrement the stats counter matching KoReader's AnnotationsModified event.
    if let Some(key) = stats_key {
        if let Value::Table(stats) = table.get("stats")? {
            if let Value::Integer(count) = stats.get(key)? {
                stats.set(key, (count - 1).max(0))?;
            }
        }
    }

    Ok(())
}
