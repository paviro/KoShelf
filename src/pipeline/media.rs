//! Media asset management: directory creation, cover generation/cleanup.

use anyhow::{Context, Result};
use log::info;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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
