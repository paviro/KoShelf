//! Media asset management: directory creation, cover generation/cleanup,
//! and static frontend synchronisation.

use crate::models::LibraryItem;
use anyhow::{Context, Result};
use futures::future;
use include_dir::{Dir, File, include_dir};
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

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

/// Generate WebP cover images for all library items that have cover data.
///
/// Covers are only regenerated when the source file is newer than the cached
/// cover, or when the cover file does not exist yet.
pub async fn generate_covers(items: &[LibraryItem], covers_dir: &Path) -> Result<()> {
    info!("Extracting and converting book covers...");
    let start = Instant::now();

    let mut tasks = Vec::new();
    let (progress_tx, progress_rx) = std::sync::mpsc::channel::<()>();

    for book in items {
        if let Some(ref cover_data) = book.book_info.cover_data {
            let cover_path = covers_dir.join(format!("{}.webp", book.id));
            let file_path = book.file_path.clone();
            let cover_data = cover_data.clone();

            let should_generate = match (fs::metadata(&file_path), fs::metadata(&cover_path)) {
                (Ok(epub_meta), Ok(cover_meta)) => {
                    let epub_time = epub_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    let cover_time = cover_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    epub_time > cover_time
                }
                (Ok(_), Err(_)) => true,
                _ => true,
            };

            if should_generate {
                let tx = progress_tx.clone();
                let task = tokio::task::spawn_blocking(move || -> Result<()> {
                    let img = image::load_from_memory(&cover_data)
                        .context("Failed to load cover image")?;

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
                    let encoder =
                        webp::Encoder::from_rgb(&rgb_img, rgb_img.width(), rgb_img.height());
                    let mut config = webp::WebPConfig::new()
                        .map_err(|_| anyhow::anyhow!("Failed to create WebP config"))?;
                    config.lossless = 0;
                    config.quality = 50.0;
                    config.method = 1;
                    config.thread_level = 1;

                    let webp_data = encoder
                        .encode_advanced(&config)
                        .map_err(|e| anyhow::anyhow!("Failed to encode WebP: {:?}", e))?;

                    fs::write(&cover_path, &*webp_data)
                        .with_context(|| format!("Failed to save cover: {:?}", cover_path))?;

                    let _ = tx.send(());
                    Ok(())
                });

                tasks.push(task);
            }
        }
    }

    let total_covers = tasks.len();

    if total_covers > 0 {
        let pb = ProgressBar::new(total_covers as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} {bar:30.cyan/blue} {pos}/{len}")
                .unwrap()
                .progress_chars("━╸─"),
        );
        pb.set_message("Extracting and converting covers:");

        drop(progress_tx);

        let pb_clone = pb.clone();
        let progress_task = tokio::task::spawn_blocking(move || {
            while progress_rx.recv().is_ok() {
                pb_clone.inc(1);
            }
        });

        let results = future::join_all(tasks).await;
        let _ = progress_task.await;
        pb.finish_and_clear();

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => return Err(e.context(format!("Failed to generate cover {}", i))),
                Err(e) => {
                    return Err(anyhow::Error::new(e).context(format!("Task {} panicked", i)));
                }
            }
        }

        let elapsed = start.elapsed();
        info!(
            "Extracted and converted {} covers in {:.1}s",
            total_covers,
            elapsed.as_secs_f64()
        );
    }

    Ok(())
}

/// Remove cover files for items that no longer exist in the library.
pub fn cleanup_stale_covers(items: &[LibraryItem], covers_dir: &Path) -> Result<()> {
    if !covers_dir.exists() {
        return Ok(());
    }

    let current_ids: HashSet<String> = items.iter().map(|b| b.id.clone()).collect();

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
