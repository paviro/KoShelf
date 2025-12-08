use anyhow::{Context, Result};
use std::path::Path;

// Embed SVG templates at compile time
const STORY_TEMPLATE: &str = include_str!("../assets/share_story.svg");
const SQUARE_TEMPLATE: &str = include_str!("../assets/share_square.svg");
const BANNER_TEMPLATE: &str = include_str!("../assets/share_banner.svg");

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
    /// Story format - 1080×1920 (9:16 vertical)
    Story,
    /// Square format - 1080×1080 (1:1)
    Square,
    /// Banner format - 1200×630 (horizontal)
    Banner,
}

impl ShareFormat {
    /// Get the dimensions for this format
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            ShareFormat::Story => (1260, 2240),
            ShareFormat::Square => (2160, 2160),
            ShareFormat::Banner => (2400, 1260),
        }
    }

    /// Get the PNG filename for this format
    pub fn filename(&self) -> &'static str {
        match self {
            ShareFormat::Story => "share_story.png",
            ShareFormat::Square => "share_square.png",
            ShareFormat::Banner => "share_banner.png",
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
    let (width, height) = format.dimensions();
    
    // Generate SVG content by replacing placeholders in template
    let svg_content = fill_template(data, format);
    
    // Set up font database with system fonts
    let mut fontdb = resvg::usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    
    // Parse SVG with usvg
    let options = resvg::usvg::Options {
        fontdb: std::sync::Arc::new(fontdb),
        ..Default::default()
    };
    
    let tree = resvg::usvg::Tree::from_str(&svg_content, &options)
        .context("Failed to parse SVG")?;
    
    // Create pixmap for rendering
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .context("Failed to create pixmap")?;
    
    // Render SVG to pixmap
    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
    
    // Encode as PNG and write to file
    let png_data = pixmap.encode_png()
        .context("Failed to encode PNG")?;
    
    std::fs::write(output_path, png_data)
        .context("Failed to write PNG file")?;
    
    // Also generate SVG file
    let svg_path = output_path.with_extension("svg");
    generate_share_svg(data, format, &svg_path)?;
    
    Ok(())
}

/// Generate a share SVG file from the given data and format
pub fn generate_share_svg(
    data: &ShareImageData,
    format: ShareFormat,
    output_path: &Path,
) -> Result<()> {
    let svg_content = fill_template(data, format);
    
    std::fs::write(output_path, svg_content)
        .context("Failed to write SVG file")?;
    
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
