use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::fs;
use std::path::Path;
use std::sync::{Arc, LazyLock};

const WEBP_QUALITY: f32 = 85.0;
const WEBP_METHOD: i32 = 1;

// Embed SVG templates at compile time
const STORY_TEMPLATE: &str = include_str!("../../../assets/share_story.svg");
const SQUARE_TEMPLATE: &str = include_str!("../../../assets/share_square.svg");
const BANNER_TEMPLATE: &str = include_str!("../../../assets/share_banner.svg");

// Embed fonts at compile time for cross-platform consistency
const FONT_REGULAR: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/frontend/node_modules/@expo-google-fonts/gelasio/400Regular/Gelasio_400Regular.ttf"
));
const FONT_ITALIC: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/frontend/node_modules/@expo-google-fonts/gelasio/400Regular_Italic/Gelasio_400Regular_Italic.ttf"
));

/// Cached font database - initialized once and reused across all share image generations.
static FONT_DATABASE: LazyLock<Arc<resvg::usvg::fontdb::Database>> = LazyLock::new(|| {
    let mut fontdb = resvg::usvg::fontdb::Database::new();

    fontdb.load_font_data(FONT_REGULAR.to_vec());
    fontdb.load_font_data(FONT_ITALIC.to_vec());

    Arc::new(fontdb)
});

/// Data needed to generate a share image
#[derive(Debug, Clone)]
pub struct ShareImageData {
    pub year: i32,
    pub books_read: u32,
    pub reading_time_hours: u32,
    pub reading_time_days: u32,
    pub active_days: u32,
    pub active_days_percentage: u8,
    pub longest_streak: u32,
    pub best_month: Option<String>,
}

impl ShareImageData {
    /// Compute a deterministic content fingerprint for change detection.
    ///
    /// All fields that influence the rendered image are included so that
    /// any data change produces a different fingerprint.
    pub fn fingerprint(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}:{}:{}",
            self.books_read,
            self.reading_time_hours,
            self.reading_time_days,
            self.active_days,
            self.active_days_percentage,
            self.longest_streak,
            self.best_month.as_deref().unwrap_or(""),
        )
    }
}

/// Available share image formats
#[derive(Debug, Clone, Copy)]
pub enum ShareFormat {
    Story,
    Square,
    Banner,
}

impl ShareFormat {
    /// Get the dimensions for this format
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            ShareFormat::Story => (1260, 2240),
            ShareFormat::Square => (1500, 1500),
            ShareFormat::Banner => (2400, 1260),
        }
    }

    /// Get the WebP filename for this format
    pub fn filename(&self) -> &'static str {
        match self {
            ShareFormat::Story => "share_story.webp",
            ShareFormat::Square => "share_square.webp",
            ShareFormat::Banner => "share_banner.webp",
        }
    }

    /// Get the SVG template for this format
    fn template(&self) -> &'static str {
        match self {
            ShareFormat::Story => STORY_TEMPLATE,
            ShareFormat::Square => SQUARE_TEMPLATE,
            ShareFormat::Banner => BANNER_TEMPLATE,
        }
    }
}

/// Generate a share image from the given data and format
pub fn generate_share_image(
    data: &ShareImageData,
    format: ShareFormat,
    output_path: &Path,
) -> Result<()> {
    log::debug!("Generating share image: {:?}", output_path);
    let (width, height) = format.dimensions();

    let svg_content = fill_template(data, format);

    let options = resvg::usvg::Options {
        fontdb: FONT_DATABASE.clone(),
        ..Default::default()
    };

    let tree =
        resvg::usvg::Tree::from_str(&svg_content, &options).context("Failed to parse SVG")?;

    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(width, height).context("Failed to create pixmap")?;

    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    let encoder = webp::Encoder::from_rgba(pixmap.data(), width, height);
    let mut config =
        webp::WebPConfig::new().map_err(|_| anyhow::anyhow!("Failed to create WebP config"))?;
    config.lossless = 0;
    config.quality = WEBP_QUALITY;
    config.method = WEBP_METHOD;
    config.thread_level = 1;
    config.alpha_compression = 1;
    config.alpha_filtering = 1;
    config.alpha_quality = WEBP_QUALITY.round() as i32;

    let webp_data = encoder
        .encode_advanced(&config)
        .map_err(|e| anyhow::anyhow!("Failed to encode WebP: {:?}", e))?;

    fs::write(output_path, &*webp_data).context("Failed to write WebP file")?;

    // Reuse the parsed tree for the SVG output instead of re-parsing.
    let svg_path = output_path.with_extension("svg");
    let svg_output = tree.to_string(&resvg::usvg::WriteOptions::default());
    fs::write(svg_path, svg_output).context("Failed to write SVG file")?;

    log::debug!("Finished creating share image: {:?}", output_path);

    Ok(())
}

/// Fill the SVG template with actual data
fn fill_template(data: &ShareImageData, format: ShareFormat) -> String {
    let template = format.template();
    let reading_time = format_reading_time(data.reading_time_hours, data.reading_time_days);
    let best_month = best_month_display(data.best_month.as_deref());

    template
        .replace("{{YEAR}}", &data.year.to_string())
        .replace("{{BOOKS}}", &data.books_read.to_string())
        .replace("{{READING_TIME}}", &reading_time)
        .replace("{{ACTIVE_DAYS}}", &data.active_days.to_string())
        .replace("{{ACTIVE_PCT}}", &data.active_days_percentage.to_string())
        .replace("{{STREAK}}", &data.longest_streak.to_string())
        .replace("{{BEST_MONTH}}", &best_month)
}

fn best_month_display(best_month: Option<&str>) -> String {
    let Some(best_month) = best_month else {
        return "-".to_string();
    };

    NaiveDate::parse_from_str(&format!("{}-01", best_month), "%Y-%m-%d")
        .map(|date| date.format("%B").to_string())
        .unwrap_or_else(|_| best_month.to_string())
}

/// Format reading time display
fn format_reading_time(hours: u32, days: u32) -> String {
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else {
        format!("{}h", hours)
    }
}

#[cfg(test)]
mod tests {
    use super::best_month_display;

    #[test]
    fn best_month_display_formats_month_keys_as_english_month_names() {
        assert_eq!(best_month_display(Some("2026-03")), "March");
    }

    #[test]
    fn best_month_display_preserves_non_month_values() {
        assert_eq!(best_month_display(Some("March")), "March");
        assert_eq!(best_month_display(None), "-");
    }
}
