use super::file::FileConfig;
use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use std::path::PathBuf;

/// KoShelf CLI arguments.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to a TOML configuration file
    #[arg(short = 'c', long, display_order = 0)]
    pub config: Option<PathBuf>,

    /// Path(s) to folders containing ebooks (EPUB, FB2, MOBI) and/or comics (CBZ, CBR) with KoReader metadata.
    /// Can be specified multiple times. (optional if statistics_db is provided)
    #[arg(short = 'i', visible_short_alias = 'b', long, alias = "books-path", display_order = 1, action = clap::ArgAction::Append)]
    pub library_path: Vec<PathBuf>,

    /// Path to KOReader's docsettings folder (for users who store metadata separately). Requires --books-path. Mutually exclusive with --hashdocsettings-path.
    #[arg(long, display_order = 2)]
    pub docsettings_path: Option<PathBuf>,

    /// Path to KOReader's hashdocsettings folder (for users who store metadata by hash). Requires --books-path. Mutually exclusive with --docsettings-path.
    #[arg(long, display_order = 3)]
    pub hashdocsettings_path: Option<PathBuf>,

    /// Path to the statistics.sqlite3 file for additional reading stats (optional if books_path is provided)
    #[arg(short, long, display_order = 4)]
    pub statistics_db: Option<PathBuf>,

    /// Output directory for the generated static site (if not provided, starts web server with file watching)
    #[arg(short, long, display_order = 5)]
    pub output: Option<PathBuf>,

    /// Port for web server mode (default: 3000)
    #[arg(short, long, default_value = "3000", display_order = 6)]
    pub port: u16,

    /// Enable file watching with static output (requires --output)
    #[arg(short, long, default_value = "false", display_order = 7)]
    pub watch: bool,

    /// Site title
    #[arg(short, long, default_value = "KoShelf", display_order = 8)]
    pub title: String,

    /// Include unread books (EPUBs without KoReader metadata) in the generated site
    #[arg(long, default_value = "false", display_order = 9)]
    pub include_unread: bool,

    /// Maximum value for heatmap color intensity scaling (e.g., "auto", "1h", "1h30m", "45min"). Values above this will still be shown but use the highest color intensity. Default is "2h".
    #[arg(long, default_value = "2h", display_order = 10)]
    pub heatmap_scale_max: String,

    /// Timezone to interpret timestamps (IANA name, e.g., "Australia/Sydney"). Defaults to system local timezone.
    #[arg(long, display_order = 11)]
    pub timezone: Option<String>,

    /// Logical day start time (HH:MM). Defaults to 00:00.
    #[arg(long, value_name = "HH:MM", display_order = 12)]
    pub day_start_time: Option<String>,

    /// Minimum pages read per day to be counted in statistics (optional)
    #[arg(long, display_order = 13)]
    pub min_pages_per_day: Option<u32>,

    /// Minimum reading time per day to be counted in statistics (e.g., "30s", "15m", "1h", "off").
    /// Default is "30s". Use "off" to disable this filter.
    #[arg(long, default_value = "30s", display_order = 14)]
    pub min_time_per_day: Option<String>,

    /// Include statistics for all books in the database, not just those in --books-path.
    /// By default, when --books-path is provided, statistics are filtered to only include
    /// books present in that directory. Use this flag to include all statistics.
    #[arg(long, default_value = "false", display_order = 15)]
    pub include_all_stats: bool,

    /// Default server language for UI translations.
    /// Frontend language/region settings can override this per browser.
    /// Use full locale (e.g., en_US, de_DE) for correct date formatting. Use --list-languages to see available options.
    #[arg(long, short = 'l', default_value = "en_US", display_order = 16)]
    pub language: String,

    /// List all supported languages and exit
    #[arg(long, display_order = 17)]
    pub list_languages: bool,

    /// Print GitHub repository URL
    #[arg(long, display_order = 18)]
    pub github: bool,

    /// Ignore KOReader stable page metadata for page totals and page-based stats scaling.
    /// By default, stable page metadata is used when available.
    #[arg(long, default_value = "false", display_order = 19)]
    pub ignore_stable_page_metadata: bool,

    /// Persistent runtime data directory for cache files (for example library.sqlite).
    /// Used for runtime library DB and media cache persistence in serve mode.
    #[arg(long, alias = "data-dir", display_order = 20)]
    pub data_path: Option<PathBuf>,
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

impl Cli {
    /// Merge values from a TOML config file into CLI fields.
    ///
    /// Only overrides fields that were NOT explicitly set on the command line.
    /// Precedence: CLI explicit args > config file > clap defaults.
    pub fn merge_with_file_config(&mut self, config: &FileConfig, matches: &clap::ArgMatches) {
        use clap::parser::ValueSource;

        /// Returns `true` when the user did NOT pass an explicit CLI flag.
        fn not_explicit(matches: &clap::ArgMatches, id: &str) -> bool {
            matches.value_source(id) != Some(ValueSource::CommandLine)
        }

        // ── library section ──────────────────────────────────────────
        if let Some(ref lib) = config.library {
            if let Some(ref paths) = lib.paths
                && not_explicit(matches, "library_path")
                && !paths.is_empty()
            {
                self.library_path = paths.clone();
            }
            if let Some(ref p) = lib.docsettings_path
                && not_explicit(matches, "docsettings_path")
            {
                self.docsettings_path = Some(p.clone());
            }
            if let Some(ref p) = lib.hashdocsettings_path
                && not_explicit(matches, "hashdocsettings_path")
            {
                self.hashdocsettings_path = Some(p.clone());
            }
            if let Some(ref p) = lib.statistics_db
                && not_explicit(matches, "statistics_db")
            {
                self.statistics_db = Some(p.clone());
            }
            if let Some(v) = lib.include_unread
                && not_explicit(matches, "include_unread")
            {
                self.include_unread = v;
            }
        }

        // ── koshelf section ──────────────────────────────────────────
        if let Some(ref ks) = config.koshelf {
            if let Some(ref v) = ks.title
                && not_explicit(matches, "title")
            {
                self.title = v.clone();
            }
            if let Some(ref v) = ks.language
                && not_explicit(matches, "language")
            {
                self.language = v.clone();
            }
            if let Some(ref v) = ks.timezone
                && not_explicit(matches, "timezone")
            {
                self.timezone = Some(v.clone());
            }
            if let Some(ref p) = ks.data_path
                && not_explicit(matches, "data_path")
            {
                self.data_path = Some(p.clone());
            }
        }

        // ── server section ───────────────────────────────────────────
        if let Some(ref srv) = config.server
            && let Some(v) = srv.port
            && not_explicit(matches, "port")
        {
            self.port = v;
        }

        // ── output section ───────────────────────────────────────────
        if let Some(ref out) = config.output {
            if let Some(ref p) = out.path
                && not_explicit(matches, "output")
            {
                self.output = Some(p.clone());
            }
            if let Some(v) = out.watch
                && not_explicit(matches, "watch")
            {
                self.watch = v;
            }
        }

        // ── statistics section ───────────────────────────────────────
        if let Some(ref stats) = config.statistics {
            if let Some(ref v) = stats.heatmap_scale_max
                && not_explicit(matches, "heatmap_scale_max")
            {
                self.heatmap_scale_max = v.clone();
            }
            if let Some(ref v) = stats.day_start_time
                && not_explicit(matches, "day_start_time")
            {
                self.day_start_time = Some(v.clone());
            }
            if let Some(v) = stats.min_pages_per_day
                && not_explicit(matches, "min_pages_per_day")
            {
                self.min_pages_per_day = Some(v);
            }
            if let Some(ref v) = stats.min_time_per_day
                && not_explicit(matches, "min_time_per_day")
            {
                self.min_time_per_day = Some(v.clone());
            }
            if let Some(v) = stats.include_all_stats
                && not_explicit(matches, "include_all_stats")
            {
                self.include_all_stats = v;
            }
            if let Some(v) = stats.ignore_stable_page_metadata
                && not_explicit(matches, "ignore_stable_page_metadata")
            {
                self.ignore_stable_page_metadata = v;
            }
        }
    }

    /// Validate CLI inputs that are independent of runtime mode.
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

        if self.output.is_some() && self.port != 3000 {
            anyhow::bail!("--port can only be used in web server mode (without --output)");
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

#[cfg(test)]
mod tests {
    use super::parse_time_to_seconds;

    #[test]
    fn parse_time_off_alias_maps_to_none() {
        assert_eq!(parse_time_to_seconds("off").unwrap(), None);
    }

    #[test]
    fn parse_time_auto_maps_to_none() {
        assert_eq!(parse_time_to_seconds("auto").unwrap(), None);
    }
}
