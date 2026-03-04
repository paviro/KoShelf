//! Asset management: directory creation, static file copying, and cover generation.

use super::SnapshotBuilder;
use crate::contracts::{mappers, site::SiteCapabilities};
use crate::models::{LibraryItem, StatisticsData};
use crate::runtime::ContractSnapshot;
use anyhow::{Context, Result};
use futures::future;
use include_dir::{Dir, File, include_dir};
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use std::fs;
use std::time::{Instant, SystemTime};

static FRONTEND_DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");

impl SnapshotBuilder {
    pub(crate) async fn create_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.output_dir)?;
        fs::create_dir_all(self.assets_dir())?;
        fs::create_dir_all(self.covers_dir())?;

        Ok(())
    }

    fn cleanup_removed_legacy_outputs(&self) -> Result<()> {
        for relative_dir in [
            "books",
            "comics",
            "statistics",
            "calendar",
            "recap",
            "assets/css",
            "assets/js",
        ] {
            let dir = self.output_dir.join(relative_dir);
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
            let file = self.output_dir.join(relative_file);
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

    fn write_embedded_frontend_file(&self, file: &File<'_>) -> Result<()> {
        let relative_path = file.path().to_string_lossy().replace('\\', "/");
        if relative_path.is_empty() {
            return Ok(());
        }

        let output_path = self.output_dir.join(&relative_path);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if relative_path == "index.html" {
            let server_mode = if self.is_internal_server {
                "internal"
            } else {
                "external"
            };
            let source = file
                .contents_utf8()
                .context("Embedded React index.html is not UTF-8")?;
            let injected = Self::inject_server_mode_script(source, server_mode);
            fs::write(&output_path, injected)?;
        } else {
            fs::write(&output_path, file.contents())?;
        }

        Ok(())
    }

    fn copy_embedded_frontend_dir(&self, dir: &Dir<'_>) -> Result<()> {
        for file in dir.files() {
            self.write_embedded_frontend_file(file)?;
        }

        for child in dir.dirs() {
            self.copy_embedded_frontend_dir(child)?;
        }

        Ok(())
    }

    fn sync_static_frontend_output(&self, stats_data: &Option<StatisticsData>) -> Result<()> {
        self.cleanup_removed_legacy_outputs()?;
        self.copy_embedded_frontend_dir(&FRONTEND_DIST)?;

        if stats_data.is_none() {
            let recap_assets_dir = self.recap_dir();
            if let Err(error) = fs::remove_dir_all(&recap_assets_dir)
                && error.kind() != std::io::ErrorKind::NotFound
            {
                return Err(error.into());
            }
        }

        Ok(())
    }

    pub(crate) fn compute_core_snapshot_data(
        &self,
        items: &[LibraryItem],
        stats_data: &Option<StatisticsData>,
        snapshot: &mut ContractSnapshot,
    ) -> Result<()> {
        if !self.is_internal_server {
            self.sync_static_frontend_output(stats_data)?;
        }

        let version = self.get_version();
        let generated_at = self.get_last_updated();
        let meta = mappers::build_meta(version.clone(), generated_at.clone());

        // New /data/locales.json contract payload.
        let locales_response =
            mappers::map_locales_response(meta.clone(), &self.translations.to_json_string())
                .context("Failed to build /data/locales.json")?;
        snapshot.locales = Some(locales_response);

        // New /data/site.json contract payload.
        let has_books = items.iter().any(|item| item.is_book());
        let has_comics = items.iter().any(|item| item.is_comic());
        let has_statistics = stats_data.is_some();
        let site_response = mappers::map_site_response(
            meta,
            &self.site_title,
            SiteCapabilities {
                has_books,
                has_comics,
                has_statistics,
                has_recap: has_statistics,
            },
        );
        snapshot.site = Some(site_response);

        // Always emit list contracts so API/static endpoints are present even when empty.
        let books: Vec<LibraryItem> = items
            .iter()
            .filter(|item| item.is_book())
            .cloned()
            .collect();
        let books_response = mappers::map_library_list_response(
            mappers::build_meta(version.clone(), generated_at.clone()),
            &books,
        );
        snapshot.books = Some(books_response);

        let comics: Vec<LibraryItem> = items
            .iter()
            .filter(|item| item.is_comic())
            .cloned()
            .collect();
        let comics_response =
            mappers::map_library_list_response(mappers::build_meta(version, generated_at), &comics);
        snapshot.comics = Some(comics_response);

        Ok(())
    }

    pub(crate) async fn generate_covers(&self, items: &[LibraryItem]) -> Result<()> {
        info!("Extracting and converting book covers...");
        let start = Instant::now();

        // Collect all cover generation tasks.
        let mut tasks = Vec::new();

        // Channel to track progress from spawned tasks
        let (progress_tx, progress_rx) = std::sync::mpsc::channel::<()>();

        for book in items {
            if let Some(ref cover_data) = book.book_info.cover_data {
                let cover_path = self.covers_dir().join(format!("{}.webp", book.id));
                let file_path = book.file_path.clone();
                let cover_data = cover_data.clone();

                let should_generate = match (fs::metadata(&file_path), fs::metadata(&cover_path)) {
                    (Ok(epub_meta), Ok(cover_meta)) => {
                        let epub_time = epub_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        let cover_time = cover_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        epub_time > cover_time
                    }
                    (Ok(_), Err(_)) => true, // Cover missing
                    _ => true,               // If we can't get metadata, be safe and regenerate
                };

                if should_generate {
                    let tx = progress_tx.clone();
                    // Spawn a task for each cover generation
                    let task = tokio::task::spawn_blocking(move || -> Result<()> {
                        let img = image::load_from_memory(&cover_data)
                            .context("Failed to load cover image")?;

                        // Resize to height of 600px while maintaining aspect ratio (skip if already small).
                        let resized = {
                            let (original_width, original_height) = (img.width(), img.height());
                            let target_height = 600;
                            if original_height > target_height {
                                let target_width =
                                    (original_width * target_height) / original_height;
                                img.resize(
                                    target_width,
                                    target_height,
                                    image::imageops::FilterType::CatmullRom,
                                )
                            } else {
                                img
                            }
                        };

                        // Convert to RGB8 format for WebP encoding
                        let rgb_img = resized.to_rgb8();

                        // Use webp crate with a faster config than defaults (method=4 by default).
                        let encoder =
                            webp::Encoder::from_rgb(&rgb_img, rgb_img.width(), rgb_img.height());
                        let mut config = webp::WebPConfig::new()
                            .map_err(|_| anyhow::anyhow!("Failed to create WebP config"))?;
                        config.lossless = 0;
                        config.quality = 50.0;
                        config.method = 1; // faster encoding; good enough for 600px covers
                        config.thread_level = 1; // allow libwebp internal threading

                        let webp_data = encoder
                            .encode_advanced(&config)
                            .map_err(|e| anyhow::anyhow!("Failed to encode WebP: {:?}", e))?;

                        fs::write(&cover_path, &*webp_data)
                            .with_context(|| format!("Failed to save cover: {:?}", cover_path))?;

                        // Signal progress
                        let _ = tx.send(());

                        Ok(())
                    });

                    tasks.push(task);
                }
            }
        }

        let total_covers = tasks.len();

        // Only show progress bar if there's actual work to do
        if total_covers > 0 {
            // Set up progress bar
            let pb = ProgressBar::new(total_covers as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg} {bar:30.cyan/blue} {pos}/{len}")
                    .unwrap()
                    .progress_chars("━╸─"),
            );
            pb.set_message("Extracting and converting covers:");

            // Drop our sender so the channel closes when all tasks complete
            drop(progress_tx);

            // Spawn a task to update progress bar as tasks complete
            let pb_clone = pb.clone();
            let progress_task = tokio::task::spawn_blocking(move || {
                while progress_rx.recv().is_ok() {
                    pb_clone.inc(1);
                }
            });

            // Wait for all cover generation tasks to complete
            let results = future::join_all(tasks).await;

            // Wait for progress tracking to finish
            let _ = progress_task.await;

            // Clear the progress bar
            pb.finish_and_clear();

            // Check for any errors
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
}
