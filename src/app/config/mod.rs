pub mod cli;
pub mod file;
pub mod site;

pub use cli::{
    Cli, CliCommand, CommonArgs, ExportArgs, ServeArgs, parse_time_to_seconds,
    parse_trusted_proxy_nets,
};
pub use file::FileConfig;
pub use site::SiteConfig;

use clap::parser::ValueSource;
use std::path::PathBuf;

fn is_explicit_value_source(source: Option<ValueSource>) -> bool {
    matches!(
        source,
        Some(ValueSource::CommandLine | ValueSource::EnvVariable)
    )
}

/// Returns `true` when the user did NOT pass an explicit CLI/env value.
fn not_explicit(matches: &clap::ArgMatches, id: &str) -> bool {
    if is_explicit_value_source(matches.value_source(id)) {
        return false;
    }

    if id.contains('_') {
        let dashed = id.replace('_', "-");
        let has_dashed_id = matches.ids().any(|arg_id| arg_id.as_str() == dashed);
        if has_dashed_id && is_explicit_value_source(matches.value_source(&dashed)) {
            return false;
        }
    }

    true
}

/// Merge TOML config into CommonArgs fields not explicitly set via CLI/env.
fn merge_common_with_file_config(
    common: &mut CommonArgs,
    config: &FileConfig,
    matches: &clap::ArgMatches,
) {
    // ── library section ──────────────────────────────────────────
    if let Some(ref lib) = config.library {
        if let Some(ref paths) = lib.paths
            && not_explicit(matches, "library_path")
            && !paths.is_empty()
        {
            common.library_path = paths.clone();
        }
        if let Some(ref p) = lib.docsettings_path
            && not_explicit(matches, "docsettings_path")
        {
            common.docsettings_path = Some(p.clone());
        }
        if let Some(ref p) = lib.hashdocsettings_path
            && not_explicit(matches, "hashdocsettings_path")
        {
            common.hashdocsettings_path = Some(p.clone());
        }
        if let Some(ref p) = lib.statistics_db
            && not_explicit(matches, "statistics_db")
        {
            common.statistics_db = Some(p.clone());
        }
        if let Some(v) = lib.include_unread
            && not_explicit(matches, "include_unread")
        {
            common.include_unread = v;
        }
    }

    // ── koshelf section ──────────────────────────────────────────
    if let Some(ref ks) = config.koshelf {
        if let Some(ref v) = ks.title
            && not_explicit(matches, "title")
        {
            common.title = v.clone();
        }
        if let Some(ref v) = ks.language
            && not_explicit(matches, "language")
        {
            common.language = v.clone();
        }
        if let Some(ref v) = ks.timezone
            && not_explicit(matches, "timezone")
        {
            common.timezone = Some(v.clone());
        }
        if let Some(ref p) = ks.data_path
            && not_explicit(matches, "data_path")
        {
            common.data_path = Some(p.clone());
        }
    }

    // ── statistics section ───────────────────────────────────────
    if let Some(ref stats) = config.statistics {
        if let Some(ref v) = stats.heatmap_scale_max
            && not_explicit(matches, "heatmap_scale_max")
        {
            common.heatmap_scale_max = v.clone();
        }
        if let Some(ref v) = stats.day_start_time
            && not_explicit(matches, "day_start_time")
        {
            common.day_start_time = Some(v.clone());
        }
        if let Some(v) = stats.min_pages_per_day
            && not_explicit(matches, "min_pages_per_day")
        {
            common.min_pages_per_day = Some(v);
        }
        if let Some(ref v) = stats.min_time_per_day
            && not_explicit(matches, "min_time_per_day")
        {
            common.min_time_per_day = Some(v.clone());
        }
        if let Some(v) = stats.include_all_stats
            && not_explicit(matches, "include_all_stats")
        {
            common.include_all_stats = v;
        }
        if let Some(v) = stats.ignore_stable_page_metadata
            && not_explicit(matches, "ignore_stable_page_metadata")
        {
            common.ignore_stable_page_metadata = v;
        }
    }
}

/// Merge TOML config into ServeArgs (common + server sections).
pub fn merge_serve_with_file_config(
    args: &mut ServeArgs,
    config: &FileConfig,
    matches: &clap::ArgMatches,
) {
    merge_common_with_file_config(&mut args.common, config, matches);

    if let Some(ref srv) = config.server {
        if let Some(v) = srv.port
            && not_explicit(matches, "port")
        {
            args.port = v;
        }
        if let Some(v) = srv.enable_auth
            && not_explicit(matches, "enable_auth")
        {
            args.enable_auth = v;
        }
        if let Some(ref values) = srv.trusted_proxies
            && not_explicit(matches, "trusted_proxies")
            && !values.is_empty()
        {
            args.trusted_proxies = values.clone();
        }
    }
}

/// Merge TOML config into ExportArgs (common + output sections).
pub fn merge_export_with_file_config(
    args: &mut ExportArgs,
    config: &FileConfig,
    matches: &clap::ArgMatches,
) {
    merge_common_with_file_config(&mut args.common, config, matches);

    if let Some(ref out) = config.output {
        if let Some(ref p) = out.path
            && not_explicit(matches, "output")
        {
            args.output = Some(p.clone());
        }
        if let Some(v) = out.include_files
            && not_explicit(matches, "include_files")
        {
            args.include_files = v;
        }
        if let Some(v) = out.watch
            && not_explicit(matches, "watch")
        {
            args.watch = v;
        }
    }
}

/// Merge TOML config data_path into SetPassword command when not explicitly set.
pub fn merge_set_password_data_path(
    data_path: &mut Option<PathBuf>,
    config: &FileConfig,
    matches: &clap::ArgMatches,
) {
    if let Some(ref ks) = config.koshelf
        && let Some(ref p) = ks.data_path
        && not_explicit(matches, "data_path")
    {
        *data_path = Some(p.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::{merge_export_with_file_config, merge_serve_with_file_config};
    use crate::app::config::cli::{Cli, CliCommand};
    use crate::app::config::file::{FileConfig, KoshelfSection};
    use clap::{CommandFactory, FromArgMatches};
    use std::path::PathBuf;

    #[test]
    fn cli_data_path_wins_over_file_config() {
        let matches = Cli::command()
            .try_get_matches_from([
                "koshelf",
                "serve",
                "--data-path",
                "/runtime/from-cli",
                "--library-path",
                "/library",
            ])
            .expect("CLI args should parse");

        let mut cli = Cli::from_arg_matches(&matches).expect("CLI should convert from matches");
        let file_config = FileConfig {
            koshelf: Some(KoshelfSection {
                data_path: Some(PathBuf::from("/runtime/from-config")),
                ..KoshelfSection::default()
            }),
            ..FileConfig::default()
        };

        let (_, sub_matches) = matches.subcommand().unwrap();
        let CliCommand::Serve(ref mut args) = cli.command else {
            panic!("expected serve command");
        };
        merge_serve_with_file_config(args, &file_config, sub_matches);

        assert_eq!(
            args.common.data_path,
            Some(PathBuf::from("/runtime/from-cli")),
            "explicit CLI value should not be overridden by config file"
        );
    }

    #[test]
    fn file_config_data_path_is_used_when_cli_not_explicit() {
        let matches = Cli::command()
            .try_get_matches_from(["koshelf", "serve", "--library-path", "/library"])
            .expect("CLI args should parse");

        let mut cli = Cli::from_arg_matches(&matches).expect("CLI should convert from matches");
        let file_config = FileConfig {
            koshelf: Some(KoshelfSection {
                data_path: Some(PathBuf::from("/runtime/from-config")),
                ..KoshelfSection::default()
            }),
            ..FileConfig::default()
        };

        let (_, sub_matches) = matches.subcommand().unwrap();
        let CliCommand::Serve(ref mut args) = cli.command else {
            panic!("expected serve command");
        };
        merge_serve_with_file_config(args, &file_config, sub_matches);

        assert_eq!(
            args.common.data_path,
            Some(PathBuf::from("/runtime/from-config")),
            "config file should provide fallback value"
        );
    }

    #[test]
    fn export_output_from_file_config() {
        let matches = Cli::command()
            .try_get_matches_from(["koshelf", "export", "--library-path", "/library"])
            .expect("CLI args should parse");

        let mut cli = Cli::from_arg_matches(&matches).expect("CLI should convert from matches");
        let file_config = FileConfig {
            output: Some(crate::app::config::file::OutputSection {
                path: Some(PathBuf::from("/output/from-config")),
                include_files: None,
                watch: None,
            }),
            ..FileConfig::default()
        };

        let (_, sub_matches) = matches.subcommand().unwrap();
        let CliCommand::Export(ref mut args) = cli.command else {
            panic!("expected export command");
        };
        merge_export_with_file_config(args, &file_config, sub_matches);

        assert_eq!(
            args.output,
            Some(PathBuf::from("/output/from-config")),
            "config file should provide output path"
        );
    }
}
