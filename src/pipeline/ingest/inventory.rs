//! Shared source discovery for full scans and affected-identity reconciliation.
use super::sources::{Source, SourceIndex};
use crate::{
    app::config::SiteConfig,
    source::{
        koreader::{calculate_partial_md5, merge::normalize_partial_md5},
        metadata::MetadataIndices,
        scanner::{CollectedItem, CollectionOptions, scan_paths},
        sidecars,
    },
    store::sqlite::repo::{ItemSource, LibraryRepository},
};
use anyhow::Result;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

pub(super) struct Inventory {
    pub documents: Vec<CollectedItem>,
    pub sidecars: Vec<CollectedItem>,
    pub complete: bool,
    pub indices: Arc<MetadataIndices>,
    pub sources: SourceIndex,
    affected: Option<HashSet<String>>,
}

impl Inventory {
    pub fn includes(&self, id: &str) -> bool {
        self.affected.as_ref().is_none_or(|ids| ids.contains(id))
    }
}

fn folders(item: &CollectedItem, checksum: &str, indices: &MetadataIndices) -> Vec<PathBuf> {
    let mut paths: Vec<_> = indices
        .candidates(&item.path, item.format, checksum)
        .into_iter()
        .flat_map(|(_, paths)| paths)
        .filter_map(|path| path.parent().map(Path::to_path_buf))
        .collect();
    paths.extend(indices.sdr_folders(&item.path).iter().cloned());
    // Custom hash folders can exist before their first reading sidecar.
    use crate::source::scanner::MetadataLocation;
    if let MetadataLocation::HashDocSettings(root)
    | MetadataLocation::Central {
        hashdocsettings: root,
        ..
    } = &indices.location
        && let Some(prefix) = checksum.get(..2)
    {
        paths.push(root.join(prefix).join(format!("{checksum}.sdr")));
    }
    paths.sort();
    paths.dedup();
    paths
}

fn known_source(row: &ItemSource, indices: &MetadataIndices) -> Source {
    let item = CollectedItem {
        path: row.file_path.clone().into(),
        format: row.format(),
        kobo_hints: None,
        sidecar_checksum: None,
    };
    let checksum = row.calculated_checksum.clone().unwrap_or_default();
    Source {
        folders: folders(&item, &checksum, indices),
        item,
        checksum,
        identity: Some(row.id.clone()),
    }
}

fn identify(
    item: CollectedItem,
    checksum: String,
    known: &[ItemSource],
    indices: &MetadataIndices,
) -> Source {
    let matching: Vec<_> = known
        .iter()
        .filter(|row| row.calculated_checksum.as_deref() == Some(&checksum))
        .collect();
    let established = if matching.len() == 1 {
        Some(matching[0].id.as_str())
    } else {
        None
    };
    let resolution = indices.resolve(&item.path, item.format, &checksum, established);
    let identity = if matching.len() > 1 || resolution.blocked {
        None
    } else {
        established
            .map(str::to_owned)
            .or_else(|| {
                resolution
                    .selected
                    .as_ref()
                    .and_then(|capture| capture.parse().ok())
                    .and_then(|metadata| {
                        metadata
                            .partial_md5_checksum
                            .as_deref()
                            .and_then(normalize_partial_md5)
                    })
            })
            .or_else(|| Some(checksum.clone()))
    };
    Source {
        folders: folders(&item, &checksum, indices),
        item,
        checksum,
        identity,
    }
}

/// A batch shares its metadata index. Publication still verifies fresh bytes.
/// Only confirmed absence permits lifecycle changes; an unreadable inventory
/// preserves previous availability in both storage modes.
pub(super) async fn collect(
    config: &SiteConfig,
    repo: &LibraryRepository,
    events: Option<&HashSet<PathBuf>>,
) -> Result<Inventory> {
    repo.configure_metadata(&config.metadata_location);
    *repo
        .runtime_config
        .write()
        .expect("runtime configuration lock") = Some(config.clone());
    let indices = Arc::new(repo.metadata_indices()?);
    let known = repo.item_sources().await?;
    let previous = repo
        .source_index
        .lock()
        .map_err(|_| anyhow::anyhow!("source index lock poisoned"))?
        .clone();
    let mut associations: Vec<_> = previous.all().cloned().collect();
    associations.extend(known.iter().map(|row| known_source(row, &indices)));
    // Saved provenance also identifies deleted central folders no longer in the index.
    for fp in repo.load_all_fingerprints().await? {
        if let Some(folder) = fp
            .metadata_path
            .as_deref()
            .and_then(|p| Path::new(p).parent())
        {
            for source in associations
                .iter_mut()
                .filter(|s| s.identity.as_deref() == Some(&fp.item_id))
            {
                source.folders.push(folder.to_path_buf());
            }
        }
    }
    if let Some(store) = &repo.bundles {
        for bundle in store.records()? {
            if let Some(folder) = bundle.record.source.as_ref().and_then(|s| s.path.parent()) {
                for source in associations
                    .iter_mut()
                    .filter(|s| s.identity.as_deref() == Some(&bundle.record.id))
                {
                    source.folders.push(folder.to_path_buf());
                }
            }
        }
    }
    let events = events
        .map(|paths| {
            paths
                .iter()
                .map(std::path::absolute)
                .collect::<std::io::Result<HashSet<_>>>()
        })
        .transpose()?;
    let mut affected = HashSet::new();
    let mut requested = HashSet::new();
    if let Some(events) = &events {
        requested.extend(events.iter().cloned());
        for source in &associations {
            if events
                .iter()
                .any(|event| SourceIndex::affects(source, event))
            {
                requested.insert(source.item.path.clone());
                requested.extend(source.folders.iter().cloned());
                affected.extend(source.identity.iter().cloned());
            }
        }
        // Unknown document extensions are eligible only through a sidecar.
        for event in events {
            if let Some(stem) = event.file_stem().and_then(|s| s.to_str()) {
                requested.insert(event.with_file_name(format!("{stem}.sdr")));
            }
        }
    } else {
        requested.extend(
            config
                .library_paths
                .iter()
                .map(std::path::absolute)
                .collect::<std::io::Result<Vec<_>>>()?,
        );
        requested.extend(
            indices
                .location
                .central_paths()
                .into_iter()
                .map(Path::to_path_buf),
        );
    }
    let excluded = repo
        .bundles
        .as_ref()
        .map(|s| std::path::absolute(&s.root))
        .transpose()?
        .into_iter()
        .collect::<Vec<_>>();
    let mut complete = indices.complete();
    // A missing root is not proof that its contents were deleted.
    for root in &config.library_paths {
        if crate::source::scanner::library_root_metadata(root).is_err() {
            complete = false;
        }
    }
    let mut visited = HashSet::new();
    let mut items = HashMap::new();
    let mut sidecar_paths = HashSet::new();
    let mut sources = SourceIndex::default();
    loop {
        if events.is_some() {
            for related in associations.iter().filter(|source| {
                source
                    .identity
                    .as_ref()
                    .is_some_and(|id| affected.contains(id))
            }) {
                if !visited.contains(&related.item.path) {
                    requested.insert(related.item.path.clone());
                }
                requested.extend(
                    related
                        .folders
                        .iter()
                        .filter(|p| !visited.contains(*p))
                        .cloned(),
                );
            }
        }
        let mut roots = Vec::new();
        for path in requested.drain() {
            if !visited.insert(path.clone()) {
                continue;
            }
            if crate::shelf::models::is_ignored_path(&path)
                || excluded.iter().any(|root| path.starts_with(root))
            {
                continue;
            }
            match std::fs::metadata(&path) {
                Ok(_) => roots.push(path),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => complete = false,
            }
        }
        if !roots.is_empty() {
            let scan = scan_paths(
                &roots,
                &CollectionOptions {
                    kobo_db_path: config.kobo_db_path.clone(),
                },
                &excluded,
            )
            .await;
            complete &= scan.complete;
            // Central roots contain metadata, never original documents.
            for item in scan.items {
                if !indices
                    .location
                    .central_paths()
                    .iter()
                    .any(|root| item.path.starts_with(root))
                {
                    items.insert(item.path.clone(), item);
                }
            }
            sidecar_paths.extend(scan.sidecars);
        }
        // Include known arbitrary/extensionless documents when their event is selected.
        for source in &associations {
            if (events.is_none() || visited.contains(&source.item.path))
                && config
                    .library_paths
                    .iter()
                    .filter_map(|root| std::path::absolute(root).ok())
                    .any(|root| source.item.path.starts_with(root))
                && source.item.path.is_file()
                && !indices
                    .location
                    .central_paths()
                    .iter()
                    .any(|root| source.item.path.starts_with(root))
            {
                let mut item = source.item.clone();
                item.sidecar_checksum = None;
                items.entry(item.path.clone()).or_insert(item);
            }
        }
        let discovered = sidecars::discover(
            sidecar_paths.iter().map(PathBuf::as_path),
            &indices,
            &items
                .values()
                .filter(|i| i.sidecar_checksum.is_none())
                .map(|i| i.path.as_path())
                .collect(),
        );
        complete &= discovered.complete;
        for item in discovered.present {
            items.insert(item.path.clone(), item);
        }
        for sidecar in discovered.sidecars {
            if sidecar.importable
                || known.iter().any(|row| {
                    row.calculated_checksum == sidecar.item.sidecar_checksum
                        || row.file_path == sidecar.item.path.to_string_lossy()
                })
            {
                items
                    .entry(sidecar.item.path.clone())
                    .or_insert(sidecar.item);
            }
        }
        // Established identities may omit optional properties/checksum during updates.
        for row in &known {
            let path = PathBuf::from(&row.file_path);
            if items
                .get(&path)
                .is_some_and(|item| item.sidecar_checksum.is_none())
                || (events.is_some() && !affected.contains(&row.id))
            {
                continue;
            }
            let Some(checksum) = row.calculated_checksum.as_deref() else {
                continue;
            };
            let resolution = indices.resolve(&path, row.format(), checksum, Some(&row.id));
            if resolution.blocked
                || (resolution.selected.is_none() && !resolution.issues.is_empty())
            {
                complete = false;
            } else if resolution.selected.is_some() {
                items.insert(
                    path.clone(),
                    CollectedItem {
                        path,
                        format: row.format(),
                        kobo_hints: None,
                        sidecar_checksum: Some(checksum.to_owned()),
                    },
                );
            }
        }
        for item in items.values() {
            if sources.get(&item.path).is_some() {
                continue;
            }
            let checksum = match item
                .sidecar_checksum
                .clone()
                .map_or_else(|| calculate_partial_md5(&item.path), Ok)
            {
                Ok(checksum) => checksum,
                Err(e) => {
                    log::warn!("Cannot inspect {}: {e:#}", item.path.display());
                    complete = false;
                    continue;
                }
            };
            let source = identify(item.clone(), checksum, &known, &indices);
            affected.extend(source.identity.iter().cloned());
            // Conflicting sources must not cause either established identity to disappear.
            affected.extend(
                known
                    .iter()
                    .filter(|row| row.calculated_checksum.as_deref() == Some(&source.checksum))
                    .map(|row| row.id.clone()),
            );
            if events.is_some() {
                for related in associations.iter().filter(|other| {
                    other.checksum == source.checksum
                        || other
                            .identity
                            .as_ref()
                            .is_some_and(|id| affected.contains(id))
                }) {
                    if !visited.contains(&related.item.path) {
                        requested.insert(related.item.path.clone());
                    }
                    requested.extend(
                        related
                            .folders
                            .iter()
                            .filter(|p| !visited.contains(*p))
                            .cloned(),
                    );
                }
            }
            sources.insert(source);
        }
        if requested.is_empty() {
            break;
        }
    }
    let mut cache = if events.is_some() {
        previous
    } else {
        SourceIndex::default()
    };
    let stale: Vec<_> = cache
        .all()
        .filter(|source| {
            events.is_none()
                || affected
                    .iter()
                    .any(|id| source.identity.as_ref() == Some(id))
                || visited.contains(&source.item.path)
        })
        .map(|source| source.item.path.clone())
        .collect();
    if complete {
        for path in stale {
            cache.remove(&path);
        }
    }
    for source in sources.all() {
        cache.insert(source.clone());
    }
    *repo
        .source_index
        .lock()
        .map_err(|_| anyhow::anyhow!("source index lock poisoned"))? = cache;
    // One transition policy for both storage modes and both kinds of scans.
    if !complete {
        items.retain(|path, item| {
            let identity = sources
                .get(path)
                .and_then(|source| source.identity.as_deref());
            known
                .iter()
                .find(|row| Path::new(&row.file_path) == path || identity == Some(row.id.as_str()))
                .is_none_or(|row| {
                    !row.archived
                        && (row.document_state == crate::shelf::models::DocumentState::Present)
                            == item.sidecar_checksum.is_none()
                })
        });
    }
    let (mut documents, mut sidecars): (Vec<_>, Vec<_>) = items
        .into_values()
        .partition(|item| item.sidecar_checksum.is_none());
    documents.sort_by(|a, b| a.path.cmp(&b.path));
    sidecars.sort_by_key(|item| {
        (
            !known
                .iter()
                .any(|row| Path::new(&row.file_path) == item.path),
            item.path.clone(),
        )
    });
    Ok(Inventory {
        documents,
        sidecars,
        complete,
        indices,
        sources,
        affected: events.map(|_| affected),
    })
}
