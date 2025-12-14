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

    // Handle --github flag
    if cli.github {
        println!("https://github.com/paviro/KOShelf");
        return Ok(());
    }

    // Handle --list-languages flag
    if cli.list_languages {
        println!("{}", koshelf::i18n::list_supported_languages());
        return Ok(());
    }
    run(cli).await
}
