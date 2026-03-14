//! Static frontend synchronisation: version-checked copying of the embedded
//! React build to the static-export output directory.

use anyhow::{Context, Result};
use include_dir::{Dir, File, include_dir};
use log::info;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::ErrorKind;
use std::path::Path;
use std::sync::LazyLock;

static FRONTEND_DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");

static FRONTEND_VERSION: LazyLock<String> = LazyLock::new(|| {
    let mut hasher = DefaultHasher::new();
    hash_embedded_dir(&FRONTEND_DIST, &mut hasher);
    format!("{}-{:x}", env!("CARGO_PKG_VERSION"), hasher.finish())
});

fn hash_embedded_dir(dir: &Dir<'_>, hasher: &mut DefaultHasher) {
    for file in dir.files() {
        file.path().hash(hasher);
        file.contents().hash(hasher);
    }
    for child in dir.dirs() {
        hash_embedded_dir(child, hasher);
    }
}

/// In static-export mode, copy the embedded React frontend to the output
/// directory and clean up legacy output artifacts.
pub fn sync_static_frontend(output_dir: &Path, has_reading_data: bool) -> Result<()> {
    let version_file = output_dir.join(".version");
    let current_version = &*FRONTEND_VERSION;

    let version_matches = fs::read_to_string(&version_file)
        .map(|v| v.trim() == current_version)
        .unwrap_or(false);

    if !version_matches || !embedded_files_exist(output_dir, &FRONTEND_DIST) {
        cleanup_removed_legacy_outputs(output_dir)?;
        copy_embedded_frontend_dir(output_dir, &FRONTEND_DIST)?;
        fs::write(&version_file, current_version)?;
        info!("Static frontend updated (version {})", current_version);
    }

    if !has_reading_data {
        let recap_assets_dir = output_dir.join("assets").join("recap");
        if let Err(error) = fs::remove_dir_all(&recap_assets_dir)
            && error.kind() != ErrorKind::NotFound
        {
            return Err(error.into());
        }
    }

    Ok(())
}

fn cleanup_removed_legacy_outputs(output_dir: &Path) -> Result<()> {
    for relative_dir in [
        "books",
        "comics",
        "statistics",
        "calendar",
        "recap",
        "assets/css",
        "assets/js",
    ] {
        let dir = output_dir.join(relative_dir);
        if let Err(error) = fs::remove_dir_all(&dir)
            && error.kind() != ErrorKind::NotFound
        {
            return Err(error.into());
        }
    }

    for relative_file in [
        "404.html",
        "service-worker.js",
        "version.txt",
        "cache-manifest.json",
    ] {
        let file = output_dir.join(relative_file);
        if let Err(error) = fs::remove_file(&file)
            && error.kind() != ErrorKind::NotFound
        {
            return Err(error.into());
        }
    }

    Ok(())
}

fn inject_server_mode_script(index_html: &str, server_mode: &str) -> String {
    if index_html.contains("__KOSHELF_SERVER_MODE") {
        return index_html.to_string();
    }

    let script = format!(
        "<script>window.__KOSHELF_SERVER_MODE = '{}';</script>",
        server_mode
    );

    if index_html.contains("<head>") {
        return index_html.replacen("<head>", &format!("<head>\n        {}", script), 1);
    }

    if index_html.contains("<body") {
        return index_html.replacen("<body", &format!("{}\n    <body", script), 1);
    }

    format!("{}\n{}", script, index_html)
}

fn write_embedded_frontend_file(output_dir: &Path, file: &File<'_>) -> Result<()> {
    let relative_path = file.path().to_string_lossy().replace('\\', "/");
    if relative_path.is_empty() {
        return Ok(());
    }

    let output_path = output_dir.join(&relative_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if relative_path == "index.html" {
        let source = file
            .contents_utf8()
            .context("Embedded React index.html is not UTF-8")?;
        let injected = inject_server_mode_script(source, "external");
        fs::write(&output_path, injected)?;
    } else {
        fs::write(&output_path, file.contents())?;
    }

    Ok(())
}

fn embedded_files_exist(output_dir: &Path, dir: &Dir<'_>) -> bool {
    dir.files().all(|f| {
        let path = f.path().to_string_lossy().replace('\\', "/");
        !path.is_empty() && output_dir.join(&path).exists()
    }) && dir
        .dirs()
        .all(|child| embedded_files_exist(output_dir, child))
}

fn copy_embedded_frontend_dir(output_dir: &Path, dir: &Dir<'_>) -> Result<()> {
    for file in dir.files() {
        write_embedded_frontend_file(output_dir, file)?;
    }
    for child in dir.dirs() {
        copy_embedded_frontend_dir(output_dir, child)?;
    }
    Ok(())
}
