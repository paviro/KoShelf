use anyhow::{Context, Result};
use std::path::Path;
use std::sync::{Arc, LazyLock};

const WEBP_QUALITY: f32 = 85.0;
const WEBP_METHOD: i32 = 1;

// Embed SVG templates at compile time
const STORY_TEMPLATE: &str = include_str!("../../assets/share_story.svg");
const SQUARE_TEMPLATE: &str = include_str!("../../assets/share_square.svg");
const BANNER_TEMPLATE: &str = include_str!("../../assets/share_banner.svg");

// Embed fonts at compile time for cross-platform consistency
const FONT_REGULAR: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Gelasio-Regular.ttf"));
const FONT_ITALIC: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Gelasio-Italic.ttf"));

/// Cached font database - initialized once and reused across all share image generations.
static FONT_DATABASE: LazyLock<Arc<resvg::usvg::fontdb::Database>> = LazyLock::new(|| {
    let mut fontdb = resvg::usvg::fontdb::Database::new();

    // Load embedded fonts - these are bundled at compile time
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

    // Generate SVG content by replacing placeholders in template
    let svg_content = fill_template(data, format);

    // Parse SVG with usvg using cached font database
    let options = resvg::usvg::Options {
        fontdb: FONT_DATABASE.clone(),
        ..Default::default()
    };

    let tree =
        resvg::usvg::Tree::from_str(&svg_content, &options).context("Failed to parse SVG")?;

    // Create pixmap for rendering
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(width, height).context("Failed to create pixmap")?;

    // Render SVG to pixmap
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    // Encode as WebP and write to file
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

    std::fs::write(output_path, &*webp_data).context("Failed to write WebP file")?;

    // Also generate SVG file (reuse the parsed tree instead of parsing again)
    let svg_path = output_path.with_extension("svg");
    let svg_output = tree.to_string(&resvg::usvg::WriteOptions::default());
    std::fs::write(svg_path, svg_output).context("Failed to write SVG file")?;

    log::debug!("Finished creating share image: {:?}", output_path);

    Ok(())
}

/// Generate a share SVG file from the given data and format
/// Text is converted to paths for font-independent rendering
pub fn generate_share_svg(
    data: &ShareImageData,
    format: ShareFormat,
    output_path: &Path,
) -> Result<()> {
    let svg_content = fill_template(data, format);

    // Parse SVG with usvg using cached font database
    let options = resvg::usvg::Options {
        fontdb: FONT_DATABASE.clone(),
        ..Default::default()
    };

    let tree = resvg::usvg::Tree::from_str(&svg_content, &options)
        .context("Failed to parse SVG for path conversion")?;

    // Write the processed SVG (with text converted to paths)
    let svg_output = tree.to_string(&resvg::usvg::WriteOptions::default());

    std::fs::write(output_path, svg_output).context("Failed to write SVG file")?;

    Ok(())
}

/// Fill the SVG template with actual data
fn fill_template(data: &ShareImageData, format: ShareFormat) -> String {
    let template = format.template();
    let reading_time = format_reading_time(data.reading_time_hours, data.reading_time_days);
    let best_month = data.best_month.as_deref().unwrap_or("-");

    template
        .replace("{{YEAR}}", &data.year.to_string())
        .replace("{{BOOKS}}", &data.books_read.to_string())
        .replace("{{READING_TIME}}", &reading_time)
        .replace("{{ACTIVE_DAYS}}", &data.active_days.to_string())
        .replace("{{ACTIVE_PCT}}", &data.active_days_percentage.to_string())
        .replace("{{STREAK}}", &data.longest_streak.to_string())
        .replace("{{BEST_MONTH}}", best_month)
}

/// Format reading time display
fn format_reading_time(hours: u32, days: u32) -> String {
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else {
        format!("{}h", hours)
    }
}
