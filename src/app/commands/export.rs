use crate::app::bootstrap::initialize_pipeline;
use crate::app::config::ExportArgs;
use crate::pipeline::export::{ExportConfig, export_data_files};
use crate::pipeline::frontend;
use crate::pipeline::watcher::FileWatcher;
use anyhow::{Context, Result};
use log::info;

pub(crate) async fn export(args: ExportArgs) -> Result<()> {
    if let Err(e) = args.validate() {
        super::exit_validation_error("export", e);
    }

    let output_dir = args
        .output
        .clone()
        .context("Output directory is required for export")?;

    let state = initialize_pipeline(
        &args.common,
        output_dir.clone(),
        false,
        false,
        false,
        args.include_files,
    )
    .await?;

    // ── Sync static frontend ─────────────────────────────────────────
    frontend::sync_static_frontend(&state.config.output_dir, state.has_reading_data)?;

    // ── Export data files ────────────────────────────────────────────
    let export_config = ExportConfig {
        site_title: state.config.site_title.clone(),
        language: state.config.language.clone(),
        include_files: state.config.include_files,
    };
    export_data_files(
        &output_dir.join("data"),
        &output_dir,
        &state.repo,
        state.reading_data.as_ref(),
        &export_config,
    )
    .await?;

    if args.watch {
        info!("Watching library changes to refresh static shell/assets and /data export.");
        let file_watcher = FileWatcher::new(state.config, None, None, None, Some(state.repo), None);
        if let Err(e) = file_watcher.run().await {
            log::error!("File watcher error: {}", e);
        }
    } else {
        info!("Static export completed.");
    }

    Ok(())
}
