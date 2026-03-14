#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use anyhow::{Context, Result};
use clap::{CommandFactory, FromArgMatches};
use koshelf::app::config::FileConfig;
use koshelf::{Cli, run};
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
        None if default_config_path.exists() => {
            // Default path exists: load it
            log::info!("Loading config from {}", default_config_path.display());
            Some(FileConfig::load(default_config_path)?)
        }
        None => None,
    };

    if let Some(ref fc) = file_config {
        cli.merge_with_file_config(fc, &matches);
    }

    if cli.github {
        println!("https://github.com/paviro/KOShelf");
        return Ok(());
    }

    if cli.list_languages {
        println!("{}", koshelf::i18n::list_supported_languages());
        return Ok(());
    }
    run(cli).await
}
