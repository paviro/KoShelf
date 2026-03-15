pub mod cli;
pub mod file;
pub mod site;

pub use cli::{Cli, parse_time_to_seconds};
pub use file::FileConfig;
pub use site::SiteConfig;

/// Merge values from a TOML config file into CLI fields.
///
/// Only overrides fields that were NOT explicitly set on the command line.
/// Precedence: CLI explicit args > config file > clap defaults.
pub fn merge_with_file_config(cli: &mut Cli, config: &FileConfig, matches: &clap::ArgMatches) {
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
            cli.library_path = paths.clone();
        }
        if let Some(ref p) = lib.docsettings_path
            && not_explicit(matches, "docsettings_path")
        {
            cli.docsettings_path = Some(p.clone());
        }
        if let Some(ref p) = lib.hashdocsettings_path
            && not_explicit(matches, "hashdocsettings_path")
        {
            cli.hashdocsettings_path = Some(p.clone());
        }
        if let Some(ref p) = lib.statistics_db
            && not_explicit(matches, "statistics_db")
        {
            cli.statistics_db = Some(p.clone());
        }
        if let Some(v) = lib.include_unread
            && not_explicit(matches, "include_unread")
        {
            cli.include_unread = v;
        }
    }

    // ── koshelf section ──────────────────────────────────────────
    if let Some(ref ks) = config.koshelf {
        if let Some(ref v) = ks.title
            && not_explicit(matches, "title")
        {
            cli.title = v.clone();
        }
        if let Some(ref v) = ks.language
            && not_explicit(matches, "language")
        {
            cli.language = v.clone();
        }
        if let Some(ref v) = ks.timezone
            && not_explicit(matches, "timezone")
        {
            cli.timezone = Some(v.clone());
        }
        if let Some(ref p) = ks.data_path
            && not_explicit(matches, "data_path")
        {
            cli.data_path = Some(p.clone());
        }
    }

    // ── server section ───────────────────────────────────────────
    if let Some(ref srv) = config.server
        && let Some(v) = srv.port
        && not_explicit(matches, "port")
    {
        cli.port = v;
    }

    // ── output section ───────────────────────────────────────────
    if let Some(ref out) = config.output {
        if let Some(ref p) = out.path
            && not_explicit(matches, "output")
        {
            cli.output = Some(p.clone());
        }
        if let Some(v) = out.watch
            && not_explicit(matches, "watch")
        {
            cli.watch = v;
        }
    }

    // ── statistics section ───────────────────────────────────────
    if let Some(ref stats) = config.statistics {
        if let Some(ref v) = stats.heatmap_scale_max
            && not_explicit(matches, "heatmap_scale_max")
        {
            cli.heatmap_scale_max = v.clone();
        }
        if let Some(ref v) = stats.day_start_time
            && not_explicit(matches, "day_start_time")
        {
            cli.day_start_time = Some(v.clone());
        }
        if let Some(v) = stats.min_pages_per_day
            && not_explicit(matches, "min_pages_per_day")
        {
            cli.min_pages_per_day = Some(v);
        }
        if let Some(ref v) = stats.min_time_per_day
            && not_explicit(matches, "min_time_per_day")
        {
            cli.min_time_per_day = Some(v.clone());
        }
        if let Some(v) = stats.include_all_stats
            && not_explicit(matches, "include_all_stats")
        {
            cli.include_all_stats = v;
        }
        if let Some(v) = stats.ignore_stable_page_metadata
            && not_explicit(matches, "ignore_stable_page_metadata")
        {
            cli.ignore_stable_page_metadata = v;
        }
    }
}
