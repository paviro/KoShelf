#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use anyhow::{Context, Result};
use clap::{CommandFactory, FromArgMatches};
use koshelf::app::config::{
    CliCommand, FileConfig, merge_export_with_file_config, merge_serve_with_file_config,
    merge_set_password_data_path,
};
use koshelf::{Cli, dispatch};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let matches = Cli::command().get_matches();
    let mut cli = Cli::from_arg_matches(&matches)?;

    // ── Load config file ─────────────────────────────────────────────
    let config_path_explicit = cli.config.clone();
    let default_config_path = Path::new("koshelf.toml");

    let file_config = match config_path_explicit {
        Some(ref path) => {
            // Explicit --config: must exist
            Some(
                FileConfig::load(path)
                    .with_context(|| format!("Failed to load config file: {:?}", path))?,
            )
        }
        None if default_config_path.exists() => Some(FileConfig::load(default_config_path)?),
        None => None,
    };

    if let Some(ref fc) = file_config
        && let Some((_, sub_matches)) = matches.subcommand()
    {
        match cli.command {
            CliCommand::Serve(ref mut args) => {
                merge_serve_with_file_config(args, fc, sub_matches);
            }
            CliCommand::Export(ref mut args) => {
                merge_export_with_file_config(args, fc, sub_matches);
            }
            CliCommand::SetPassword {
                ref mut data_path, ..
            } => {
                merge_set_password_data_path(data_path, fc, sub_matches);
            }
            _ => {}
        }
    }

    dispatch(cli.command).await
}
