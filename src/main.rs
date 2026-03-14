#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use anyhow::Result;
use clap::Parser;
use koshelf::{Cli, run};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();
    let cli = Cli::parse();

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
