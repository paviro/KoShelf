use anyhow::{Context, Result};
use clap::Parser;
use ipnet::IpNet;
use regex::Regex;
use std::path::PathBuf;

/// KoShelf — a reading companion powered by KOReader metadata.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to a TOML configuration file
    #[arg(short = 'c', long, env = "KOSHELF_CONFIG", global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum CliCommand {
    /// Start the web server (API + live data refresh).
    Serve(ServeArgs),

    /// Generate a static site.
    Export(ExportArgs),

    /// Set the authentication password.
    #[command(long_about = "Set the authentication password.\n\n\
        No-ops if a password is already set (use --overwrite to replace it).\n\
        If no password argument is provided, prompts interactively unless --random is used.")]
    SetPassword {
        /// Path to the data directory containing koshelf.sqlite.
        /// Falls back to KOSHELF_DATA_PATH env var or koshelf.toml when omitted.
        #[arg(long, env = "KOSHELF_DATA_PATH")]
        data_path: Option<PathBuf>,

        /// The password to set. If omitted, prompts interactively via terminal.
        /// Can also be provided via KOSHELF_PASSWORD env var.
        /// CLI flag takes precedence over env var; omit both for interactive mode.
        #[arg(long, env = "KOSHELF_PASSWORD")]
        password: Option<String>,

        /// Generate and set a random password (printed once to stderr).
        /// Mutually exclusive with --password / KOSHELF_PASSWORD.
        #[arg(long, default_value = "false", conflicts_with = "password")]
        random: bool,

        /// Overwrite an existing password. Also invalidates all sessions.
        #[arg(long, default_value = "false")]
        overwrite: bool,
    },

    /// List all supported UI languages and exit.
    ListLanguages,

    /// Print the GitHub repository URL and exit.
    Github,
}

/// Flags shared by `serve` and `export` subcommands.
#[derive(clap::Args, Debug, Clone)]
pub struct CommonArgs {
    // ── Library source ──────────────────────────────────────────
    /// Path(s) to folders containing ebooks (EPUB, FB2, MOBI) and/or comics (CBZ, CBR) with KoReader metadata.
    /// Can be specified multiple times. (optional if statistics_db is provided)
    #[arg(short = 'i', visible_short_alias = 'b', long, env = "KOSHELF_LIBRARY_PATH", alias = "books-path", action = clap::ArgAction::Append)]
    pub library_path: Vec<PathBuf>,

    /// Path to KOReader's docsettings folder (for users who store metadata separately). Requires --library-path. Mutually exclusive with --hashdocsettings-path.
    #[arg(long, env = "KOSHELF_DOCSETTINGS_PATH")]
    pub docsettings_path: Option<PathBuf>,

    /// Path to KOReader's hashdocsettings folder (for users who store metadata by hash). Requires --library-path. Mutually exclusive with --docsettings-path.
    #[arg(long, env = "KOSHELF_HASHDOCSETTINGS_PATH")]
    pub hashdocsettings_path: Option<PathBuf>,

    /// Path to the statistics.sqlite3 file for additional reading stats (optional if library_path is provided)
    #[arg(short, long, env = "KOSHELF_STATISTICS_DB")]
    pub statistics_db: Option<PathBuf>,

    /// Include unread books (EPUBs without KoReader metadata) in the generated site
    #[arg(long, env = "KOSHELF_INCLUDE_UNREAD", default_value = "false")]
    pub include_unread: bool,

    // ── Data ────────────────────────────────────────────────────
    /// Persistent runtime data directory for cache files (for example library.sqlite).
    #[arg(long, env = "KOSHELF_DATA_PATH", alias = "data-dir")]
    pub data_path: Option<PathBuf>,

    // ── Site display ────────────────────────────────────────────
    /// Site title
    #[arg(short, long, env = "KOSHELF_TITLE", default_value = "KoShelf")]
    pub title: String,

    /// Default server language for UI translations.
    /// Frontend language/region settings can override this per browser.
    /// Use full locale (e.g., en_US, de_DE) for correct date formatting. Use `list-languages` to see available options.
    #[arg(long, short = 'l', env = "KOSHELF_LANGUAGE", default_value = "en_US")]
    pub language: String,

    /// Timezone to interpret timestamps (IANA name, e.g., "Australia/Sydney"). Defaults to system local timezone.
    #[arg(long, env = "KOSHELF_TIMEZONE")]
    pub timezone: Option<String>,

    // ── Statistics tuning ───────────────────────────────────────
    /// Maximum value for heatmap color intensity scaling (e.g., "auto", "1h", "1h30m", "45min"). Values above this will still be shown but use the highest color intensity. Default is "2h".
    #[arg(long, env = "KOSHELF_HEATMAP_SCALE_MAX", default_value = "2h")]
    pub heatmap_scale_max: String,

    /// Logical day start time (HH:MM). Defaults to 00:00.
    #[arg(long, env = "KOSHELF_DAY_START_TIME", value_name = "HH:MM")]
    pub day_start_time: Option<String>,

    /// Minimum pages read per day to be counted in statistics (optional)
    #[arg(long, env = "KOSHELF_MIN_PAGES_PER_DAY")]
    pub min_pages_per_day: Option<u32>,

    /// Minimum reading time per day to be counted in statistics (e.g., "30s", "15m", "1h", "off").
    /// Default is "30s". Use "off" to disable this filter.
    #[arg(long, env = "KOSHELF_MIN_TIME_PER_DAY", default_value = "30s")]
    pub min_time_per_day: Option<String>,

    /// Include statistics for all books in the database, not just those in --library-path.
    /// By default, when --library-path is provided, statistics are filtered to only include
    /// books present in that directory. Use this flag to include all statistics.
    #[arg(long, env = "KOSHELF_INCLUDE_ALL_STATS", default_value = "false")]
    pub include_all_stats: bool,

    /// Ignore KOReader stable page metadata for page totals and page-based stats scaling.
    /// By default, stable page metadata is used when available.
    #[arg(
        long,
        env = "KOSHELF_IGNORE_STABLE_PAGE_METADATA",
        default_value = "false"
    )]
    pub ignore_stable_page_metadata: bool,
}

/// Arguments for the `serve` subcommand.
#[derive(clap::Args, Debug, Clone)]
pub struct ServeArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Port for web server (default: 3000)
    #[arg(short, long, env = "KOSHELF_PORT", default_value = "3000")]
    pub port: u16,

    /// Enable password authentication.
    /// On first run, generates a random password and prints it to stderr.
    #[arg(long, env = "KOSHELF_ENABLE_AUTH", default_value = "false")]
    pub enable_auth: bool,

    /// Trusted reverse proxy IPs/CIDRs allowed to provide Forwarded/X-Forwarded-* headers.
    /// Repeat the flag or pass comma-separated values.
    #[arg(long, env = "KOSHELF_TRUSTED_PROXIES", value_delimiter = ',', action = clap::ArgAction::Append)]
    pub trusted_proxies: Vec<String>,
}

/// Arguments for the `export` subcommand.
#[derive(clap::Args, Debug, Clone)]
pub struct ExportArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Output directory for the generated static site.
    #[arg(env = "KOSHELF_OUTPUT")]
    pub output: Option<PathBuf>,

    /// Re-export on library changes.
    #[arg(short, long, env = "KOSHELF_WATCH", default_value = "false")]
    pub watch: bool,
}

/// Parse time format strings like "1h", "1h30m", "45min", "30s" into seconds.
///
/// Special cases: "auto" and "off" return `Ok(None)`.
pub fn parse_time_to_seconds(time_str: &str) -> Result<Option<u32>> {
    if time_str.eq_ignore_ascii_case("auto") || time_str.eq_ignore_ascii_case("off") {
        return Ok(None);
    }

    let re = Regex::new(r"(?i)(\d+)(h|m|min|s)")?;
    let mut total_seconds: u32 = 0;
    let mut matched_any = false;

    for cap in re.captures_iter(time_str) {
        matched_any = true;
        let value: u32 = cap[1].parse()?;
        let unit = &cap[2].to_lowercase();

        match unit.as_str() {
            "h" => total_seconds += value * 3600,
            "m" | "min" => total_seconds += value * 60,
            "s" => total_seconds += value,
            _ => anyhow::bail!("Unknown time unit: {}", unit),
        }
    }

    if !matched_any {
        anyhow::bail!("Invalid time format: {}", time_str);
    }
    if total_seconds == 0 {
        anyhow::bail!("Time cannot be zero: {}", time_str);
    }

    Ok(Some(total_seconds))
}

pub fn parse_trusted_proxy_nets(entries: &[String]) -> Result<Vec<IpNet>> {
    entries
        .iter()
        .map(|entry| {
            entry.parse::<IpNet>().with_context(|| {
                format!(
                    "Invalid trusted proxy entry '{}'. Expected IP or CIDR, for example 127.0.0.1/32",
                    entry
                )
            })
        })
        .collect()
}

impl CommonArgs {
    pub fn validate(&self) -> Result<()> {
        if self.library_path.is_empty() && self.statistics_db.is_none() {
            anyhow::bail!("Either --library-path or --statistics-db (or both) must be provided");
        }

        for library_path in &self.library_path {
            if !library_path.exists() {
                anyhow::bail!("Library path does not exist: {:?}", library_path);
            }
            if !library_path.is_dir() {
                anyhow::bail!("Library path is not a directory: {:?}", library_path);
            }
        }

        if self.include_unread && self.library_path.is_empty() {
            anyhow::bail!("--include-unread can only be used when --library-path is provided");
        }

        if self.docsettings_path.is_some() && self.hashdocsettings_path.is_some() {
            anyhow::bail!(
                "--docsettings-path and --hashdocsettings-path are mutually exclusive. Please use only one."
            );
        }

        if self.docsettings_path.is_some() && self.library_path.is_empty() {
            anyhow::bail!("--docsettings-path requires --library-path to be provided");
        }

        if self.hashdocsettings_path.is_some() && self.library_path.is_empty() {
            anyhow::bail!("--hashdocsettings-path requires --library-path to be provided");
        }

        if let Some(ref docsettings_path) = self.docsettings_path {
            if !docsettings_path.exists() {
                anyhow::bail!("Docsettings path does not exist: {:?}", docsettings_path);
            }
            if !docsettings_path.is_dir() {
                anyhow::bail!(
                    "Docsettings path is not a directory: {:?}",
                    docsettings_path
                );
            }
        }

        if let Some(ref hashdocsettings_path) = self.hashdocsettings_path {
            if !hashdocsettings_path.exists() {
                anyhow::bail!(
                    "Hashdocsettings path does not exist: {:?}",
                    hashdocsettings_path
                );
            }
            if !hashdocsettings_path.is_dir() {
                anyhow::bail!(
                    "Hashdocsettings path is not a directory: {:?}",
                    hashdocsettings_path
                );
            }
        }

        if let Some(ref stats_path) = self.statistics_db
            && !stats_path.exists()
        {
            anyhow::bail!("Statistics database does not exist: {:?}", stats_path);
        }

        if let Some(ref data_path) = self.data_path
            && data_path.exists()
            && !data_path.is_dir()
        {
            anyhow::bail!("Data directory path is not a directory: {:?}", data_path);
        }

        parse_time_to_seconds(&self.heatmap_scale_max).with_context(|| {
            format!(
                "Invalid heatmap-scale-max format: {}",
                self.heatmap_scale_max
            )
        })?;

        if let Some(ref min_time_str) = self.min_time_per_day {
            parse_time_to_seconds(min_time_str)
                .with_context(|| format!("Invalid min-time-per-day format: {}", min_time_str))?;
        }

        Ok(())
    }
}

impl ServeArgs {
    pub fn validate(&self) -> Result<()> {
        self.common.validate()?;

        if self.common.data_path.is_none() {
            anyhow::bail!(
                "--data-path is required in serve mode for persistent data storage. \
                 Provide a directory path where KoShelf can store its database and cache files."
            );
        }

        parse_trusted_proxy_nets(&self.trusted_proxies)?;

        Ok(())
    }
}

impl ExportArgs {
    pub fn validate(&self) -> Result<()> {
        self.common.validate()?;

        if self.output.is_none() {
            anyhow::bail!(
                "Output directory is required. Provide it as a positional argument, \
                 via KOSHELF_OUTPUT env var, or as [output].path in your config file."
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, CliCommand, parse_time_to_seconds};
    use clap::{CommandFactory, FromArgMatches};
    use std::path::PathBuf;

    #[test]
    fn parse_time_off_alias_maps_to_none() {
        assert_eq!(parse_time_to_seconds("off").unwrap(), None);
    }

    #[test]
    fn parse_time_auto_maps_to_none() {
        assert_eq!(parse_time_to_seconds("auto").unwrap(), None);
    }

    #[test]
    fn set_password_random_conflicts_with_password() {
        let matches = Cli::command().try_get_matches_from([
            "koshelf",
            "set-password",
            "--random",
            "--password",
            "explicit-pass",
        ]);

        assert!(
            matches.is_err(),
            "--random and --password should be mutually exclusive"
        );
    }

    #[test]
    fn set_password_random_parses() {
        let matches = Cli::command()
            .try_get_matches_from(["koshelf", "set-password", "--random"])
            .expect("CLI args should parse");

        let cli = Cli::from_arg_matches(&matches).expect("CLI should convert from matches");

        let CliCommand::SetPassword {
            password,
            random,
            overwrite,
            ..
        } = cli.command
        else {
            panic!("expected set-password command")
        };

        assert_eq!(password, None);
        assert!(random);
        assert!(!overwrite);
    }

    #[test]
    fn bare_koshelf_shows_help_error() {
        let result = Cli::command().try_get_matches_from(["koshelf"]);
        assert!(
            result.is_err(),
            "bare koshelf without subcommand should fail"
        );
    }

    #[test]
    fn serve_parses_common_and_specific_flags() {
        let matches = Cli::command()
            .try_get_matches_from([
                "koshelf",
                "serve",
                "--library-path",
                "/lib",
                "--data-path",
                "/data",
                "--port",
                "8080",
            ])
            .expect("CLI args should parse");

        let cli = Cli::from_arg_matches(&matches).expect("CLI should convert from matches");

        let CliCommand::Serve(args) = cli.command else {
            panic!("expected serve command")
        };

        assert_eq!(args.common.library_path, vec![PathBuf::from("/lib")]);
        assert_eq!(args.common.data_path, Some(PathBuf::from("/data")));
        assert_eq!(args.port, 8080);
    }

    #[test]
    fn export_parses_positional_output() {
        let matches = Cli::command()
            .try_get_matches_from(["koshelf", "export", "/out", "--library-path", "/lib"])
            .expect("CLI args should parse");

        let cli = Cli::from_arg_matches(&matches).expect("CLI should convert from matches");

        let CliCommand::Export(args) = cli.command else {
            panic!("expected export command")
        };

        assert_eq!(args.output, Some(PathBuf::from("/out")));
        assert_eq!(args.common.library_path, vec![PathBuf::from("/lib")]);
    }

    #[test]
    fn serve_rejects_export_flags() {
        let result = Cli::command().try_get_matches_from([
            "koshelf",
            "serve",
            "--library-path",
            "/lib",
            "--watch",
        ]);

        assert!(result.is_err(), "--watch should not be valid for serve");
    }

    #[test]
    fn export_rejects_serve_flags() {
        let result = Cli::command().try_get_matches_from([
            "koshelf",
            "export",
            "/out",
            "--library-path",
            "/lib",
            "--port",
            "8080",
        ]);

        assert!(result.is_err(), "--port should not be valid for export");
    }

    #[test]
    fn common_flags_work_in_both_subcommands() {
        for subcmd in ["serve", "export"] {
            let mut args = vec![
                "koshelf",
                subcmd,
                "--library-path",
                "/lib",
                "--title",
                "My Shelf",
                "--language",
                "de_DE",
            ];
            if subcmd == "serve" {
                args.extend(["--data-path", "/data"]);
            } else {
                args.push("/out");
            }
            let result = Cli::command().try_get_matches_from(args);
            assert!(
                result.is_ok(),
                "common flags should work with {subcmd} subcommand"
            );
        }
    }
}
