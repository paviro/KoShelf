//! KOReader custom properties, with base values kept independently of overrides.
use crate::shelf::models::BookInfo;
use crate::source::custom::{Capture, Resolution};
use anyhow::{Context, Result, anyhow, ensure};
use mlua::{Lua, LuaOptions, StdLib, Table, Value, chunk::ChunkMode};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

pub const FIELDS: [&str; 7] = [
    "title",
    "authors",
    "series",
    "series_index",
    "language",
    "keywords",
    "description",
];
pub type Properties = BTreeMap<String, String>;
/// Custom state of one book, stored with its library record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Metadata {
    /// Document values before overrides.
    pub base: Properties,
    pub overrides: Properties,
    /// Document page count, recorded in KOReader's `doc_props`.
    pub base_pages: Option<u32>,
    /// Digests of the applied custom files, to detect changes without reparsing.
    pub metadata_digest: Option<String>,
    pub cover_digest: Option<String>,
    pub error: Option<String>,
}
pub fn base(info: &BookInfo) -> Properties {
    let mut props = Properties::new();
    props.insert("title".into(), info.title.clone());
    props.insert("authors".into(), info.authors.join("\n"));
    props.insert("keywords".into(), info.subjects.join("\n"));
    for (key, value) in [
        ("series", &info.series),
        ("series_index", &info.series_number),
        ("language", &info.language),
        ("description", &info.description),
    ] {
        props.insert(key.into(), value.clone().unwrap_or_default());
    }
    props
}
/// KOReader stores series indices as Lua numbers and rejects other input.
pub fn series_index(value: &str) -> Option<f64> {
    value.trim().parse::<f64>().ok().filter(|n| n.is_finite())
}
/// One entry per line, without surrounding whitespace or blank lines.
fn list(value: &str) -> impl Iterator<Item = &str> {
    value.lines().map(str::trim).filter(|line| !line.is_empty())
}
/// Compare values the way they are displayed: surrounding whitespace, empty
/// list entries and number formatting carry no meaning, so they never justify
/// an override.
fn normalized(key: &str, value: &str) -> String {
    match key {
        "authors" | "keywords" => list(value).collect::<Vec<_>>().join("\n"),
        "series_index" => {
            series_index(value).map_or_else(|| value.trim().into(), |n| n.to_string())
        }
        _ => value.trim().to_string(),
    }
}
/// The Lua value KOReader would store: series indices are numbers.
fn lua_value(lua: &Lua, key: &str, value: &str) -> mlua::Result<Value> {
    if key != "series_index" {
        return lua.create_string(value).map(Value::String);
    }
    let n =
        series_index(value).ok_or_else(|| mlua::Error::runtime("series index must be a number"))?;
    Ok(
        // `mlua::Integer` is i32 on 32-bit LuaJIT/Lua 5.1 targets.
        if n.fract() == 0.0 && n >= mlua::Integer::MIN as f64 && n < -(mlua::Integer::MIN as f64) {
            Value::Integer(n as mlua::Integer)
        } else {
            Value::Number(n)
        },
    )
}
pub fn parse(bytes: &[u8]) -> Result<Properties> {
    let lua = Lua::new_with(StdLib::TABLE, LuaOptions::default()).map_err(|e| anyhow!("{e}"))?;
    lua.set_memory_limit(32 * 1024 * 1024)
        .map_err(|e| anyhow!("{e}"))?;
    lua.set_hook(
        mlua::HookTriggers::new().every_nth_instruction(100_000),
        |_, _| {
            Err(mlua::Error::RuntimeError(
                "custom metadata instruction limit".into(),
            ))
        },
    )
    .map_err(|e| anyhow!("{e}"))?;
    let root: Table = lua
        .load(std::str::from_utf8(bytes)?)
        .set_mode(ChunkMode::Text)
        .eval()
        .map_err(|e| anyhow!("{e}"))?;
    let props: Table = root.get("custom_props").map_err(|e| anyhow!("{e}"))?;
    let mut result = Properties::new();
    for key in FIELDS {
        let value: Value = props.get(key).map_err(|e| anyhow!("{e}"))?;
        let value = match value {
            Value::Nil => continue,
            Value::String(s) => s.to_str().map_err(|e| anyhow!("{e}"))?.to_string(),
            Value::Integer(n) if key == "series_index" => n.to_string(),
            Value::Number(n) if key == "series_index" && n.is_finite() => n.to_string(),
            _ => return Err(anyhow!("invalid custom property {key}")),
        };
        result.insert(key.into(), value);
    }
    Ok(result)
}
pub fn apply(info: &mut BookInfo, props: &Properties) {
    for (key, value) in props {
        match key.as_str() {
            "title" => info.title = value.clone(),
            "authors" => info.authors = list(value).map(str::to_owned).collect(),
            "keywords" => info.subjects = list(value).map(str::to_owned).collect(),
            "series" => info.series = Some(value.clone()),
            "series_index" => info.series_number = Some(value.clone()),
            "language" => info.language = Some(value.clone()),
            "description" => info.description = Some(value.clone()),
            _ => {}
        }
    }
}
pub fn write(
    path: &Path,
    captured: Option<&Capture>,
    original: &Properties,
    pages: Option<u32>,
    patch: &BTreeMap<String, Option<String>>,
) -> Result<()> {
    ensure!(
        patch.keys().all(|k| FIELDS.contains(&k.as_str())),
        "unknown custom property"
    );
    let initial = b"return { custom_props = {} }";
    super::LuaWriter::new()
        .write_custom(
            path,
            captured.map_or(initial.as_slice(), |c| c.bytes.as_slice()),
            || {
                if let Some(c) = captured {
                    c.verify()
                } else {
                    ensure!(!path.try_exists()?, "custom metadata appeared");
                    Ok(())
                }
            },
            |lua, root| {
                if captured.is_none() {
                    // Like KOReader's `doc_props`: missing values are absent,
                    // never empty strings.
                    let base = lua.create_table()?;
                    for (key, value) in original {
                        if value.trim().is_empty()
                            || (key == "series_index" && series_index(value).is_none())
                        {
                            continue;
                        }
                        base.set(key.as_str(), lua_value(lua, key, value)?)?;
                    }
                    if let Some(pages) = pages {
                        base.set("pages", pages)?;
                    }
                    root.set("doc_props", base)?;
                }
                let props: Table = root.get("custom_props")?;
                for (key, value) in patch {
                    // Blank values and values matching the ebook clear the override.
                    let value = value.as_deref().filter(|v| {
                        !v.trim().is_empty()
                            && original
                                .get(key)
                                .is_none_or(|base| normalized(key, v) != normalized(key, base))
                    });
                    match value {
                        Some(value) => props.set(key.as_str(), lua_value(lua, key, value)?)?,
                        None => props.set(key.as_str(), Value::Nil)?,
                    }
                }
                Ok(props.pairs::<Value, Value>().next().is_none())
            },
        )
        .context("cannot write custom metadata")
}

/// Undo [`refresh`]: put the base values back, as if freshly extracted.
pub fn restore_base(info: &mut BookInfo) {
    let Some(metadata) = info.custom.take() else {
        return;
    };
    let text = |key: &str| metadata.base.get(key).filter(|v| !v.is_empty()).cloned();
    info.title = metadata.base.get("title").cloned().unwrap_or_default();
    info.authors = list(&text("authors").unwrap_or_default())
        .map(str::to_owned)
        .collect();
    info.subjects = list(&text("keywords").unwrap_or_default())
        .map(str::to_owned)
        .collect();
    info.series = text("series");
    info.series_number = text("series_index");
    info.language = text("language");
    info.description = text("description");
    info.pages = metadata.base_pages;
}

/// Apply the resolved custom metadata to freshly extracted ebook metadata.
/// Unreadable sources keep the previously accepted overrides.
pub fn refresh(info: &mut BookInfo, resolution: &Resolution, previous: Option<&Metadata>) {
    let mut errors = resolution.issues.clone();
    let parsed = match &resolution.metadata {
        Some(source) => parse(&source.bytes)
            .map_err(|e| errors.push(format!("{}: {e:#}", source.path.display())))
            .ok(),
        None if resolution.issues.is_empty() => Some(Properties::new()),
        None => None,
    };
    let metadata = Metadata {
        base: base(info),
        base_pages: info.pages,
        overrides: parsed
            .or_else(|| previous.map(|p| p.overrides.clone()))
            .unwrap_or_default(),
        metadata_digest: resolution.metadata_digest().map(Into::into),
        cover_digest: resolution.cover_digest().map(Into::into),
        error: (!errors.is_empty()).then(|| errors.join("; ")),
    };
    if let Some(error) = &metadata.error {
        log::warn!("Custom metadata: {error}");
    }
    apply(info, &metadata.overrides);
    info.custom = Some(metadata);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_preserves_unknown_values_and_types() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("custom_metadata.lua");
        std::fs::write(&path, b"return {doc_props={title='Original'}, custom_props={series_index=9223372036854775808, private={[1.5]='yes',[true]='yes'}}}").unwrap();
        let capture = Capture::read(&path).unwrap();
        write(
            &path,
            Some(&capture),
            &Properties::new(),
            None,
            &BTreeMap::from([("title".into(), Some("New".into()))]),
        )
        .unwrap();
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(
            parse(&bytes).unwrap()["series_index"],
            parse(&capture.bytes).unwrap()["series_index"]
        );
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("[1.5]"));
        assert!(text.contains("[true]"));
        assert!(text.contains("Original"));
    }
    #[test]
    fn new_snapshot_and_final_reset() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("custom_metadata.lua");
        let props = BTreeMap::from([("title".into(), "Original".into())]);
        write(
            &path,
            None,
            &props,
            Some(10),
            &BTreeMap::from([("title".into(), Some("Edited".into()))]),
        )
        .unwrap();
        let capture = Capture::read(&path).unwrap();
        assert_eq!(parse(&capture.bytes).unwrap()["title"], "Edited");
        write(
            &path,
            Some(&capture),
            &props,
            None,
            &BTreeMap::from([("title".into(), None)]),
        )
        .unwrap();
        assert!(!path.exists());
    }
    #[test]
    fn values_matching_the_original_remove_the_override() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("custom_metadata.lua");
        let props = BTreeMap::from([
            ("title".into(), "Original".into()),
            ("authors".into(), "A\nB".into()),
        ]);
        write(
            &path,
            None,
            &props,
            None,
            &BTreeMap::from([
                ("title".into(), Some("Edited".into())),
                ("authors".into(), Some("A\nB\nC".into())),
            ]),
        )
        .unwrap();
        let capture = Capture::read(&path).unwrap();
        write(
            &path,
            Some(&capture),
            &props,
            None,
            &BTreeMap::from([("authors".into(), Some(" A\n\nB \n".into()))]),
        )
        .unwrap();
        let overrides = parse(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(overrides.get("title").map(String::as_str), Some("Edited"));
        assert!(!overrides.contains_key("authors"));
        let capture = Capture::read(&path).unwrap();
        write(
            &path,
            Some(&capture),
            &props,
            None,
            &BTreeMap::from([("title".into(), Some(" Original ".into()))]),
        )
        .unwrap();
        assert!(!path.exists());
    }
    #[test]
    fn series_indices_are_numbers_and_lists_are_trimmed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("custom_metadata.lua");
        let props = BTreeMap::from([("series_index".into(), "1.0".into())]);
        write(
            &path,
            None,
            &props,
            None,
            &BTreeMap::from([
                ("series_index".into(), Some("3".into())),
                ("title".into(), Some("T".into())),
            ]),
        )
        .unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("[\"series_index\"] = 3,"), "{text}");
        assert!(text.contains("[\"series_index\"] = 1,"), "{text}");
        // An equal number in another notation clears the override.
        let capture = Capture::read(&path).unwrap();
        write(
            &path,
            Some(&capture),
            &props,
            None,
            &BTreeMap::from([("series_index".into(), Some(" 1 ".into()))]),
        )
        .unwrap();
        assert!(
            !parse(&std::fs::read(&path).unwrap())
                .unwrap()
                .contains_key("series_index")
        );
        let capture = Capture::read(&path).unwrap();
        assert!(
            write(
                &path,
                Some(&capture),
                &props,
                None,
                &BTreeMap::from([("series_index".into(), Some("one".into()))]),
            )
            .is_err()
        );

        let mut info = crate::tests::fixtures::library_item("x", None).book_info;
        apply(
            &mut info,
            &BTreeMap::from([("authors".into(), " A \n\n B\n".into())]),
        );
        assert_eq!(info.authors, ["A", "B"]);
    }
}
