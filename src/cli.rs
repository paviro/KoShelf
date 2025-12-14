use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use std::path::PathBuf;

/// KoShelf CLI arguments.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
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

    /// Maximum value for heatmap color intensity scaling (e.g., "auto", "1h", "1h30m", "45min"). Values above this will still be shown but use the highest color intensity.
    #[arg(long, default_value = "auto", display_order = 10)]
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

    /// Minimum reading time per day to be counted in statistics (e.g., "15m", "1h"). (optional)
    #[arg(long, display_order = 14)]
    pub min_time_per_day: Option<String>,

    /// Include statistics for all books in the database, not just those in --books-path.
    /// By default, when --books-path is provided, statistics are filtered to only include
    /// books present in that directory. Use this flag to include all statistics.
    #[arg(long, default_value = "false", display_order = 15)]
    pub include_all_stats: bool,

    /// Language for UI translations. Use full locale (e.g., en_US, de_DE) for correct date formatting. Use --list-languages to see available options
    #[arg(long, short = 'l', default_value = "en_US", display_order = 16)]
    pub language: String,

    /// List all supported languages and exit
    #[arg(long, display_order = 17)]
    pub list_languages: bool,

    /// Print GitHub repository URL
    #[arg(long, display_order = 18)]
    pub github: bool,
}

/// Parse time format strings like "1h", "1h30m", "45min", "30s" into seconds.
///
/// Special case: "auto" returns `Ok(None)`.
pub fn parse_time_to_seconds(time_str: &str) -> Result<Option<u32>> {
    if time_str.eq_ignore_ascii_case("auto") {
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
    /// Validate CLI inputs that are independent of runtime mode.
    pub fn validate(&self) -> Result<()> {
        // Require at least one of library_path or statistics_db
        if self.library_path.is_empty() && self.statistics_db.is_none() {
            anyhow::bail!("Either --library-path or --statistics-db (or both) must be provided");
        }

        // Validate library paths if provided
        for library_path in &self.library_path {
            if !library_path.exists() {
                anyhow::bail!("Library path does not exist: {:?}", library_path);
            }
            if !library_path.is_dir() {
                anyhow::bail!("Library path is not a directory: {:?}", library_path);
            }
        }

        // Validate include-unread option
        if self.include_unread && self.library_path.is_empty() {
            anyhow::bail!("--include-unread can only be used when --library-path is provided");
        }

        // Validate docsettings-path and hashdocsettings-path options
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

        // Validate docsettings path if provided
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

        // Validate hashdocsettings path if provided
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

        // Validate port option
        if self.output.is_some() && self.port != 3000 {
            anyhow::bail!("--port can only be used in web server mode (without --output)");
        }

        // Validate statistics database if provided
        if let Some(ref stats_path) = self.statistics_db
            && !stats_path.exists()
        {
            anyhow::bail!("Statistics database does not exist: {:?}", stats_path);
        }

        // Validate heatmap scale max
        parse_time_to_seconds(&self.heatmap_scale_max).with_context(|| {
            format!(
                "Invalid heatmap-scale-max format: {}",
                self.heatmap_scale_max
            )
        })?;

        // Validate min time per day
        if let Some(ref min_time_str) = self.min_time_per_day {
            parse_time_to_seconds(min_time_str)
                .with_context(|| format!("Invalid min-time-per-day format: {}", min_time_str))?;
        }

        Ok(())
    }
}
