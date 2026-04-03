mod export;
mod licenses;
mod serve;
mod set_password;

use crate::app::config::{Cli, CliCommand};
use anyhow::Result;
use clap::CommandFactory;

/// Format a validation error as a clap-style error with usage line and `--help` hint,
/// then exit. This keeps our post-parse validation consistent with clap's own output.
fn exit_validation_error(subcommand: &str, error: anyhow::Error) -> ! {
    Cli::command()
        .find_subcommand(subcommand)
        .expect("known subcommand")
        .clone()
        .error(
            clap::error::ErrorKind::MissingRequiredArgument,
            error.to_string(),
        )
        .exit()
}

/// Dispatch the parsed CLI command to the appropriate handler.
///
/// `src/main.rs` is responsible for logging init, Clap argument parsing, and config file merging.
pub async fn dispatch(command: CliCommand) -> Result<()> {
    match command {
        CliCommand::Serve(args) => serve::serve(args).await,
        CliCommand::Export(args) => export::export(args).await,
        CliCommand::SetPassword {
            data_path,
            password,
            random,
            overwrite,
        } => {
            let Some(resolved_data_path) = data_path else {
                exit_validation_error(
                    "set-password",
                    anyhow::anyhow!(
                        "set-password requires a data path. Provide --data-path, \
                         set KOSHELF_DATA_PATH, or configure koshelf.data_path in your config file"
                    ),
                );
            };
            set_password::set_password(resolved_data_path, password, random, overwrite).await
        }
        CliCommand::ListLanguages => {
            println!("{}", crate::i18n::list_supported_languages());
            Ok(())
        }
        CliCommand::Licenses { dependency } => licenses::print_licenses(dependency),
        CliCommand::Github => {
            println!("https://github.com/paviro/KOShelf");
            Ok(())
        }
    }
}
