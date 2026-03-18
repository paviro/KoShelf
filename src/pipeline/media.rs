//! Media asset management: directory creation, cover generation/cleanup.

use anyhow::{Context, Result, bail};
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
    pub files_dir: PathBuf,
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
    let files_dir = assets_dir.join("files");
    let recap_dir = assets_dir.join("recap");
    MediaDirs {
        output_dir: output_dir.to_path_buf(),
        assets_dir,
        covers_dir,
        files_dir,
        recap_dir,
    }
}

/// Create the required output directories for media assets.
pub fn create_media_directories(dirs: &MediaDirs) -> Result<()> {
    fs::create_dir_all(&dirs.output_dir)?;
    fs::create_dir_all(&dirs.assets_dir)?;
    fs::create_dir_all(&dirs.covers_dir)?;
    fs::create_dir_all(&dirs.files_dir)?;
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

    ensure_plain_directory(covers_dir)?;

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

// ── Item file symlinks ──────────────────────────────────────────────────

/// Build a safe `{id}.{format}` basename for item files.
///
/// Returns `None` when the inputs are not canonical/supported.
pub fn item_file_basename(item_id: &str, format: &str) -> Option<String> {
    if !is_canonical_item_id(item_id) {
        return None;
    }

    let normalized_format = format.trim().to_ascii_lowercase();
    if !matches!(
        normalized_format.as_str(),
        "epub" | "fb2" | "mobi" | "cbz" | "cbr"
    ) {
        return None;
    }

    Some(format!("{}.{}", item_id, normalized_format))
}

/// Refuse to operate on symlinked media directories to avoid accidental
/// writes/deletes outside the configured output path.
pub fn ensure_plain_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    // Guard against symlinked ancestors like `output/assets -> /some/other/path`.
    // In that case, operating on `output/assets/files` would escape the intended tree.
    for ancestor in path.ancestors().skip(1) {
        if ancestor.as_os_str().is_empty() || !ancestor.exists() {
            continue;
        }

        let meta = fs::symlink_metadata(ancestor).with_context(|| {
            format!("Failed to inspect media directory ancestor {:?}", ancestor)
        })?;

        if meta.file_type().is_symlink() {
            bail!(
                "Refusing to operate under symlinked media directory ancestor {:?}",
                ancestor
            );
        }
    }

    let meta = fs::symlink_metadata(path)
        .with_context(|| format!("Failed to inspect media directory {:?}", path))?;

    if meta.file_type().is_symlink() {
        bail!(
            "Refusing to operate on symlinked media directory {:?}",
            path
        );
    }

    if !meta.is_dir() {
        bail!(
            "Expected media directory but found non-directory {:?}",
            path
        );
    }

    Ok(())
}

/// Create or update a symlink `files_dir/{id}.{format}` → `source_path`.
///
/// If a symlink already exists but points to a different target, it is replaced.
pub fn sync_item_file_symlink(
    item_id: &str,
    format: &str,
    source_path: &Path,
    files_dir: &Path,
) -> Result<()> {
    ensure_plain_directory(files_dir)?;

    let basename = item_file_basename(item_id, format).with_context(|| {
        format!(
            "Invalid item file target for id '{}' and format '{}'",
            item_id, format
        )
    })?;

    let link_path = files_dir.join(basename);

    if link_path.exists() {
        if link_path.is_symlink() {
            match fs::read_link(&link_path) {
                Ok(existing_target) if existing_target == source_path => return Ok(()),
                _ => {
                    fs::remove_file(&link_path).with_context(|| {
                        format!("Failed to remove stale file symlink {:?}", link_path)
                    })?;
                }
            }
        } else {
            fs::remove_file(&link_path)
                .with_context(|| format!("Failed to remove existing file {:?}", link_path))?;
        }
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(source_path, &link_path)
        .with_context(|| format!("Failed to create file symlink {:?}", link_path))?;

    #[cfg(not(unix))]
    {
        // On non-Unix platforms, fall back to a hard copy.
        fs::copy(source_path, &link_path)
            .with_context(|| format!("Failed to copy item file to {:?}", link_path))?;
    }

    Ok(())
}

/// Remove all item file entries for a given item ID from `files_dir`.
///
/// This removes any `{id}.*` file/symlink variants regardless of extension.
pub fn remove_item_files_by_id(item_id: &str, files_dir: &Path) -> Result<()> {
    if !files_dir.exists() {
        return Ok(());
    }

    ensure_plain_directory(files_dir)?;

    if !is_canonical_item_id(item_id) {
        log::warn!(
            "Skipping file cleanup for non-canonical item id: {}",
            item_id
        );
        return Ok(());
    }

    for entry in fs::read_dir(files_dir)?.flatten() {
        let path = entry.path();
        if !path.is_file() && !path.is_symlink() {
            continue;
        }
        if path.file_stem().and_then(|n| n.to_str()) == Some(item_id)
            && let Err(e) = fs::remove_file(&path)
        {
            log::warn!("Failed to remove item file {:?}: {}", path, e);
        }
    }

    Ok(())
}

/// Remove file symlinks whose stem (ID) is not in the given set.
pub fn cleanup_stale_files_by_ids(current_ids: &HashSet<String>, files_dir: &Path) -> Result<()> {
    if !files_dir.exists() {
        return Ok(());
    }

    ensure_plain_directory(files_dir)?;

    let entries = fs::read_dir(files_dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() && !path.is_symlink() {
            continue;
        }
        if let Some(file_stem) = path.file_stem().and_then(|n| n.to_str())
            && !current_ids.contains(file_stem)
        {
            info!("Removing stale file symlink: {:?}", path);
            if let Err(e) = fs::remove_file(&path) {
                log::warn!("Failed to remove stale file symlink {:?}: {}", path, e);
            }
        }
    }

    Ok(())
}

pub(crate) fn is_canonical_item_id(item_id: &str) -> bool {
    item_id.len() == 32 && item_id.bytes().all(|b| b.is_ascii_hexdigit())
}
