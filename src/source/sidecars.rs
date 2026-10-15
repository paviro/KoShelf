//! KOReader reading sidecars found without a document next to them.
use crate::shelf::models::{BookInfo, Identifier, KoReaderMetadata, LibraryItemFormat};
use crate::source::koreader::{LuaParser, merge::normalize_partial_md5};
use crate::source::metadata::MetadataIndices;
use crate::source::scanner::{CollectedItem, MetadataLocation};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// A sidecar whose document is not among the scanned items.
pub(crate) struct Sidecar {
    pub item: CollectedItem,
    /// Whether it may create a new entry: KOReader saved the document's
    /// properties. Established entries accept any sidecar with their identity.
    pub importable: bool,
}

#[derive(Default)]
pub(crate) struct Discovery {
    /// Documents KoShelf did not scan (any extension) but whose sidecar sits
    /// next to them.
    pub present: Vec<CollectedItem>,
    pub sidecars: Vec<Sidecar>,
    /// False when a sidecar could not be read; absence is then unknown.
    pub complete: bool,
}

/// Classify reading sidecars (`metadata.<ext>.lua` primary paths) against the
/// scanned documents.
pub(crate) fn discover<'a>(
    paths: impl IntoIterator<Item = &'a Path>,
    indices: &MetadataIndices,
    scanned: &HashSet<&Path>,
) -> Discovery {
    let mut discovery = Discovery {
        complete: indices.complete(),
        ..Default::default()
    };
    let mut seen = HashSet::new();
    let mut paths: Vec<_> = paths.into_iter().collect();
    paths.sort();
    for path in paths {
        let Some(path) = expected_document(path).filter(|(document, _)| {
            !scanned.contains(document.as_path()) && seen.insert(document.clone())
        }) else {
            continue;
        };
        match classify(&path.0, &path.1, &indices.location) {
            Ok(Some(Found::Present(item))) => discovery.present.push(item),
            Ok(Some(Found::Sidecar(sidecar))) => discovery.sidecars.push(sidecar),
            Ok(None) => {}
            Err(e) => {
                log::warn!("Cannot read {}: {e:#}", path.1.display());
                discovery.complete = false;
            }
        }
    }
    discovery
}

enum Found {
    Present(CollectedItem),
    Sidecar(Sidecar),
}

fn classify(
    document: &Path,
    sidecar: &Path,
    location: &MetadataLocation,
) -> anyhow::Result<Option<Found>> {
    let central = location
        .central_paths()
        .iter()
        .any(|root| document.starts_with(root));
    let format = LibraryItemFormat::from_document_path(document);
    let present = if central {
        false
    } else {
        match std::fs::metadata(document) {
            Ok(m) => m.is_file(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(e) => return Err(e.into()),
        }
    };
    if present {
        return Ok(Some(Found::Present(CollectedItem {
            path: document.to_path_buf(),
            format,
            kobo_hints: None,
            sidecar_checksum: None,
        })));
    }
    let Some(metadata) = read(sidecar)? else {
        return Ok(None);
    };
    let Some(checksum) = metadata
        .partial_md5_checksum
        .as_deref()
        .and_then(normalize_partial_md5)
    else {
        log::debug!(
            "Skipping {}: no valid partial_md5_checksum",
            sidecar.display()
        );
        return Ok(None);
    };
    if is_hash_path(document, location)
        && sdr_stem(sidecar).and_then(normalize_partial_md5).as_ref() != Some(&checksum)
    {
        log::warn!(
            "Skipping {}: folder does not match its checksum",
            sidecar.display()
        );
        return Ok(None);
    }
    let compound = is_hash_path(document, location)
        && metadata
            .doc_path
            .as_deref()
            .is_some_and(|p| p.to_lowercase().ends_with(".fb2.zip"))
        && document.extension().is_some_and(|ext| ext == "zip");
    let document = if compound {
        document.with_extension("fb2.zip")
    } else {
        document.to_path_buf()
    };
    let format = LibraryItemFormat::from_document_path(&document);
    Ok(Some(Found::Sidecar(Sidecar {
        importable: metadata.doc_props.is_some(),
        item: CollectedItem {
            path: document.to_path_buf(),
            format,
            kobo_hints: None,
            sidecar_checksum: Some(checksum),
        },
    })))
}

/// Parse the primary sidecar, or its backup when only that exists.
fn read(primary: &Path) -> anyhow::Result<Option<KoReaderMetadata>> {
    let backup = primary.with_file_name(format!(
        "{}.old",
        primary
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
    ));
    let mut invalid = None;
    for path in [primary, backup.as_path()] {
        match std::fs::read(path) {
            Ok(bytes) => match LuaParser::new().parse_bytes(path, &bytes) {
                Ok(metadata) => return Ok(Some(metadata)),
                Err(e) => {
                    invalid = Some(e);
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    if let Some(error) = invalid {
        return Err(error);
    }
    Ok(None)
}

fn sdr_stem(sidecar: &Path) -> Option<&str> {
    sidecar
        .parent()?
        .file_name()?
        .to_str()?
        .strip_suffix(".sdr")
}

fn is_hash_path(path: &Path, location: &MetadataLocation) -> bool {
    match location {
        MetadataLocation::HashDocSettings(root)
        | MetadataLocation::Central {
            hashdocsettings: root,
            ..
        } => path.starts_with(root),
        _ => false,
    }
}

/// Where the document of a sidecar would be: `<dir>/<stem>.sdr/metadata.<ext>.lua`
/// belongs to `<dir>/<stem>.<ext>`. Returns it with the sidecar's primary path.
pub(crate) fn expected_document(sidecar: &Path) -> Option<(PathBuf, PathBuf)> {
    let name = sidecar.file_name()?.to_str()?;
    let primary = sidecar.with_file_name(name.strip_suffix(".old").unwrap_or(name));
    let filename = LibraryItemFormat::book_filename_from_sidecar(
        sdr_stem(&primary)?,
        primary.file_name()?.to_str()?,
    )?;
    Some((primary.parent()?.parent()?.join(filename), primary))
}

/// Base metadata of a document KoShelf cannot parse, from KOReader's saved
/// `doc_props`: title, then the `doc_path` filename, then the document name.
pub(crate) fn book_info(metadata: Option<&KoReaderMetadata>, document: &Path) -> BookInfo {
    let props = metadata.and_then(|m| m.doc_props.as_ref());
    let text = |value: Option<&String>| {
        value
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    let lines = |value: Option<&String>| -> Vec<String> {
        value
            .map(|v| {
                v.lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    };
    let stem = |path: &Path| {
        let name = path.file_name()?.to_str()?;
        let format = LibraryItemFormat::from_document_path(path);
        let ext = format.extension_for_path(path);
        let name = match format {
            LibraryItemFormat::Fb2 if name.to_lowercase().ends_with(".fb2.zip") => {
                &name[..name.len() - ".fb2.zip".len()]
            }
            _ if !ext.is_empty() => &name[..name.len() - ext.len() - 1],
            _ => name,
        };
        Some(name.to_string()).filter(|n| !n.is_empty())
    };
    let doc_path = metadata.and_then(|m| m.doc_path.as_deref());
    let title = text(props.and_then(|p| p.title.as_ref()))
        .or_else(|| doc_path.and_then(|p| stem(Path::new(p.rsplit(['/', '\\']).next()?))))
        .or_else(|| stem(document))
        .unwrap_or_default();
    BookInfo {
        custom: None,
        title,
        authors: lines(props.and_then(|p| p.authors.as_ref())),
        description: text(props.and_then(|p| p.description.as_ref())),
        language: text(props.and_then(|p| p.language.as_ref())),
        publisher: None,
        identifiers: lines(props.and_then(|p| p.identifiers.as_ref()))
            .into_iter()
            .filter_map(|line| {
                let (scheme, value) = line.split_once(':')?;
                Some(Identifier::new(
                    scheme.trim().to_string(),
                    value.trim().to_string(),
                ))
            })
            .filter(|id| !id.scheme.is_empty() && !id.value.is_empty())
            .collect(),
        subjects: lines(props.and_then(|p| p.keywords.as_ref())),
        series: text(props.and_then(|p| p.series.as_ref())),
        series_number: text(props.and_then(|p| p.series_index.as_ref())),
        pages: metadata.and_then(|m| m.doc_pages),
        chapters: vec![],
        cover_data: None,
        cover_mime_type: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ID: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn discovers_adjacent_backups_compound_extensions_and_central_names() {
        let dir = tempfile::tempdir().unwrap();
        let doc = dir.path().join("doc");
        let hash = dir.path().join("hash");
        let paths = [
            doc.join("one/Book.sdr/metadata.pdf.lua"),
            doc.join("two/Book.sdr/metadata.pdf.lua.old"),
            doc.join("Book.fb2.sdr/metadata.zip.lua"),
            doc.join("NoExtension.sdr/metadata._.lua"),
            hash.join(format!("01/{ID}.sdr/metadata.pdf.lua")),
        ];
        for (n, path) in paths.iter().enumerate() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let id = if n == 4 {
                ID.into()
            } else {
                format!("{n:032x}")
            };
            std::fs::write(
                path,
                format!("return {{partial_md5_checksum='{id}', doc_props={{}}}} "),
            )
            .unwrap();
        }
        let location = MetadataLocation::Central {
            docsettings: doc,
            hashdocsettings: hash,
        };
        let indices = MetadataIndices::new(&location).unwrap();
        let discovered = discover(indices.sidecars(), &indices, &HashSet::new());
        assert!(discovered.complete);
        assert_eq!(discovered.sidecars.len(), 5);
        for sidecar in &discovered.sidecars {
            assert!(sidecar.importable);
            let item = &sidecar.item;
            let resolved = indices.resolve(
                &item.path,
                item.format,
                item.sidecar_checksum.as_deref().unwrap(),
                None,
            );
            assert!(!resolved.blocked);
            let custom = crate::source::custom::resolve(
                &item.path,
                item.sidecar_checksum.as_deref().unwrap(),
                resolved.selected.as_ref().map(|s| s.primary_path.as_path()),
                &indices,
                None,
            );
            assert!(custom.issues.is_empty(), "{:?}", custom.issues);
            assert!(resolved.selected.is_some(), "{}", item.path.display());
        }
        assert!(
            discovered
                .sidecars
                .iter()
                .any(|s| s.item.format == LibraryItemFormat::Fb2)
        );
        assert!(
            discovered
                .sidecars
                .iter()
                .any(|s| s.item.path.file_name().unwrap() == "NoExtension")
        );
    }

    #[test]
    fn eligibility_and_title_fallbacks() {
        let dir = tempfile::tempdir().unwrap();
        let document = dir.path().join("Fallback.pdf");
        let sidecar = dir.path().join("Fallback.sdr/metadata.pdf.lua");
        std::fs::create_dir_all(sidecar.parent().unwrap()).unwrap();
        for (body, expected) in [
            (format!("partial_md5_checksum='{ID}'"), Some(false)),
            ("partial_md5_checksum='bad', doc_props={}".into(), None),
            (
                format!("partial_md5_checksum='{ID}', doc_props={{}}"),
                Some(true),
            ),
        ] {
            std::fs::write(&sidecar, format!("return {{{body}}}")).unwrap();
            let found = classify(&document, &sidecar, &MetadataLocation::InBookFolder).unwrap();
            assert_eq!(
                found.map(|f| match f {
                    Found::Sidecar(s) => s.importable,
                    _ => panic!(),
                }),
                expected
            );
        }
        for (props, expected) in [
            (
                "doc_props={title='Title'},doc_path='/device/Name.pdf'",
                "Title",
            ),
            ("doc_props={},doc_path='/device/Name.pdf'", "Name"),
            ("doc_props={}", "Fallback"),
        ] {
            let parsed = LuaParser::new()
                .parse_bytes(&sidecar, format!("return {{{props}}}").as_bytes())
                .unwrap();
            assert_eq!(book_info(Some(&parsed), &document).title, expected);
        }
        let wrong_hash = dir
            .path()
            .join(format!("hash/00/{}.sdr/metadata.pdf.lua", "0".repeat(32)));
        std::fs::create_dir_all(wrong_hash.parent().unwrap()).unwrap();
        std::fs::write(
            &wrong_hash,
            format!("return {{partial_md5_checksum='{ID}',doc_props={{}}}}"),
        )
        .unwrap();
        let (document, _) = expected_document(&wrong_hash).unwrap();
        assert!(
            classify(
                &document,
                &wrong_hash,
                &MetadataLocation::HashDocSettings(dir.path().join("hash"))
            )
            .unwrap()
            .is_none()
        );
    }

    #[tokio::test]
    async fn ignores_syncthing_sources() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            ".stversions/Old.sdr/metadata.pdf.lua",
            ".stfolder/Old.sdr/metadata.pdf.lua",
            "Book.sdr/metadata.pdf.sync-conflict-copy.lua",
            "Book.sync-conflict-copy.pdf",
        ] {
            let path = dir.path().join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(
                path,
                format!("return {{partial_md5_checksum='{ID}',doc_props={{}}}}"),
            )
            .unwrap();
        }
        let scan =
            crate::source::scanner::scan_paths(&[dir.path().into()], &Default::default(), &[])
                .await;
        assert!(scan.complete);
        assert!(scan.items.is_empty());
        assert!(scan.sidecars.is_empty());
        let indices =
            MetadataIndices::new(&MetadataLocation::DocSettings(dir.path().into())).unwrap();
        assert_eq!(indices.sidecars().count(), 0);
    }
}
