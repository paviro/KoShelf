//! Media asset management: directory creation, cover generation/cleanup,
//! and static frontend synchronisation.

use anyhow::{Context, Result};
use include_dir::{Dir, File, include_dir};
use log::info;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

static FRONTEND_DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");

// ── Directory helpers ───────────────────────────────────────────────────

/// Resolved media asset directories.
pub struct MediaDirs {
    pub output_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub covers_dir: PathBuf,
    pub recap_dir: PathBuf,
}

/// Compute media directories based on run mode.
///
/// In serve mode (`is_internal_server = true`) assets live directly in
/// `output_dir`; in static-export mode they live under `output_dir/assets/`.
pub fn resolve_media_dirs(output_dir: &Path, is_internal_server: bool) -> MediaDirs {
    let assets_dir = if is_internal_server {
        output_dir.to_path_buf()
    } else {
        output_dir.join("assets")
    };
    let covers_dir = assets_dir.join("covers");
    let recap_dir = assets_dir.join("recap");
    MediaDirs {
        output_dir: output_dir.to_path_buf(),
        assets_dir,
        covers_dir,
        recap_dir,
    }
}

/// Create the required output directories for media assets.
pub fn create_media_directories(dirs: &MediaDirs) -> Result<()> {
    fs::create_dir_all(&dirs.output_dir)?;
    fs::create_dir_all(&dirs.assets_dir)?;
    fs::create_dir_all(&dirs.covers_dir)?;
    Ok(())
}

// ── Cover generation ────────────────────────────────────────────────────

/// Encode raw cover bytes to WebP and write to disk.
///
/// Loads the image, resizes to 600px max height, encodes as WebP at quality 50,
/// and writes the result to `cover_path`.
pub fn encode_cover_to_disk(cover_data: &[u8], cover_path: &Path) -> Result<()> {
    let img = image::load_from_memory(cover_data).context("Failed to load cover image")?;

    let resized = {
        let (original_width, original_height) = (img.width(), img.height());
        let target_height = 600;
        if original_height > target_height {
            let target_width = (original_width * target_height) / original_height;
            img.resize(
                target_width,
                target_height,
                image::imageops::FilterType::CatmullRom,
            )
        } else {
            img
        }
    };

    let rgb_img = resized.to_rgb8();
    let encoder = webp::Encoder::from_rgb(&rgb_img, rgb_img.width(), rgb_img.height());
    let mut config =
        webp::WebPConfig::new().map_err(|_| anyhow::anyhow!("Failed to create WebP config"))?;
    config.lossless = 0;
    config.quality = 50.0;
    config.method = 1;
    config.thread_level = 1;

    let webp_data = encoder
        .encode_advanced(&config)
        .map_err(|e| anyhow::anyhow!("Failed to encode WebP: {:?}", e))?;

    fs::write(cover_path, &*webp_data)
        .with_context(|| format!("Failed to save cover: {:?}", cover_path))?;

    Ok(())
}

/// Check whether a cover file needs (re)generation based on file modification times.
pub fn cover_needs_generation(source_path: &Path, cover_path: &Path) -> bool {
    match (fs::metadata(source_path), fs::metadata(cover_path)) {
        (Ok(src_meta), Ok(cover_meta)) => {
            let src_time = src_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let cover_time = cover_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            src_time > cover_time
        }
        (Ok(_), Err(_)) => true,
        _ => true,
    }
}

/// Remove cover files whose IDs are not in the given set.
pub fn cleanup_stale_covers_by_ids(current_ids: &HashSet<String>, covers_dir: &Path) -> Result<()> {
    if !covers_dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(covers_dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(file_stem) = path.file_stem().and_then(|n| n.to_str())
            && !current_ids.contains(file_stem)
        {
            info!("Removing stale cover: {:?}", path);
            if let Err(e) = fs::remove_file(&path) {
                log::warn!("Failed to remove stale cover {:?}: {}", path, e);
            }
        }
    }

    Ok(())
}

// ── Static frontend synchronisation ─────────────────────────────────────

/// In static-export mode, copy the embedded React frontend to the output
/// directory and clean up legacy output artifacts.
pub fn sync_static_frontend(output_dir: &Path, has_reading_data: bool) -> Result<()> {
    cleanup_removed_legacy_outputs(output_dir)?;
    copy_embedded_frontend_dir(output_dir, &FRONTEND_DIST)?;

    if !has_reading_data {
        let recap_assets_dir = output_dir.join("assets").join("recap");
        if let Err(error) = fs::remove_dir_all(&recap_assets_dir)
            && error.kind() != std::io::ErrorKind::NotFound
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
            && error.kind() != std::io::ErrorKind::NotFound
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
            && error.kind() != std::io::ErrorKind::NotFound
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

fn copy_embedded_frontend_dir(output_dir: &Path, dir: &Dir<'_>) -> Result<()> {
    for file in dir.files() {
        write_embedded_frontend_file(output_dir, file)?;
    }
    for child in dir.dirs() {
        copy_embedded_frontend_dir(output_dir, child)?;
    }
    Ok(())
}
