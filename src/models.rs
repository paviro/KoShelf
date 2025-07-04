use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::Datelike;
use std::collections::{HashMap, BTreeMap};

/// Sanitizes HTML content to only allow safe tags while removing styles and dangerous elements
pub fn sanitize_html(html: &str) -> String {
    use ammonia::Builder;
    
    let allowed_tags: &[&str] = &[
        "p", "br", "h1", "h2", "h3", "h4", "h5", "h6",
        "ul", "ol", "li", "strong", "em", "b", "i",
        "blockquote", "pre", "code", "div", "span"
    ];
    
    let empty_attrs: &[&str] = &[];
    
    Builder::default()
        .add_tags(allowed_tags)
        .add_generic_attributes(empty_attrs)  // Remove all attributes including style
        .clean(html)
        .to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub scheme: String,
    pub value: String,
}

impl Identifier {
    pub fn new(scheme: String, value: String) -> Self {
        Self { scheme, value }
    }
    
    /// Get a display-friendly version of the scheme name
    pub fn display_scheme(&self) -> String {
        match self.scheme.to_lowercase().as_str() {
            "isbn" => "ISBN".to_string(),
            "google" => "Google Books".to_string(),
            "amazon" => "Amazon".to_string(),
            "asin" => "Amazon".to_string(),
            "mobi-asin" => "Amazon".to_string(),
            "goodreads" => "Goodreads".to_string(),
            "doi" => "DOI".to_string(),
            "kobo" => "Kobo".to_string(),
            "oclc" => "WorldCat".to_string(),
            "lccn" => "Library of Congress".to_string(),
            _ => self.scheme.clone(),
        }
    }
    
    /// Get the URL for this identifier if it can be linked
    pub fn url(&self) -> Option<String> {
        match self.scheme.to_lowercase().as_str() {
            "isbn" => Some(format!("https://www.worldcat.org/isbn/{}", self.value)),
            "google" => Some(format!("https://books.google.com/books?id={}", self.value)),
            "amazon" | "asin" | "mobi-asin" => Some(format!("https://www.amazon.com/dp/{}", self.value)),
            "goodreads" => Some(format!("https://www.goodreads.com/book/show/{}", self.value)),
            "doi" => Some(format!("https://doi.org/{}", self.value)),
            "kobo" => Some(format!("https://www.kobo.com/ebook/{}", self.value)),
            "oclc" => Some(format!("https://www.worldcat.org/oclc/{}", self.value)),
            _ => None,
        }
    }
    
    /// Check if this identifier can be linked to an external service
    pub fn is_linkable(&self) -> bool {
        self.url().is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub epub_info: EpubInfo,
    pub koreader_metadata: Option<KoReaderMetadata>,
    pub epub_path: PathBuf,
}

impl Book {
    pub fn status(&self) -> BookStatus {
        self.koreader_metadata
            .as_ref()
            .and_then(|m| m.summary.as_ref())
            .map(|s| s.status.clone())
            .unwrap_or(BookStatus::Unknown)
    }
    
    pub fn rating(&self) -> Option<u32> {
        self.koreader_metadata
            .as_ref()
            .and_then(|m| m.summary.as_ref())
            .and_then(|s| s.rating)
    }
    
    pub fn star_display(&self) -> [bool; 5] {
        let rating = self.rating().unwrap_or(0);
        [
            rating >= 1,
            rating >= 2,
            rating >= 3,
            rating >= 4,
            rating >= 5,
        ]
    }
    
    pub fn review_note(&self) -> Option<&String> {
        self.koreader_metadata
            .as_ref()
            .and_then(|m| m.summary.as_ref())
            .and_then(|s| s.note.as_ref())
    }
    
    pub fn progress_percentage(&self) -> Option<f64> {
        self.koreader_metadata
            .as_ref()
            .and_then(|m| m.percent_finished)
    }
    
    pub fn progress_percentage_display(&self) -> u32 {
        self.progress_percentage()
            .map(|p| (p * 100.0).round() as u32)
            .unwrap_or(0)
    }
    
    pub fn annotation_count(&self) -> usize {
        self.koreader_metadata
            .as_ref()
            .map(|m| m.annotations.len())
            .unwrap_or(0)
    }
    
    pub fn bookmark_count(&self) -> usize {
        self.koreader_metadata
            .as_ref()
            .map(|m| m.annotations.iter().filter(|a| a.is_bookmark()).count())
            .unwrap_or(0)
    }
    
    pub fn highlight_count(&self) -> usize {
        self.koreader_metadata
            .as_ref()
            .map(|m| m.annotations.iter().filter(|a| a.is_highlight()).count())
            .unwrap_or(0)
    }
    
    /// Get language, preferring EPUB metadata over KoReader metadata
    pub fn language(&self) -> Option<&String> {
        self.epub_info.language.as_ref()
            .or_else(|| {
                self.koreader_metadata
                    .as_ref()
                    .and_then(|m| m.text_lang.as_ref())
            })
    }
    
    /// Get publisher from EPUB metadata
    pub fn publisher(&self) -> Option<&String> {
        self.epub_info.publisher.as_ref()
    }
    
    /// Get all identifiers from EPUB metadata, filtered to only include those with proper display schemes
    pub fn identifiers(&self) -> Vec<&Identifier> {
        self.epub_info.identifiers.iter()
            .filter(|id| id.display_scheme() != id.scheme)
            .collect()
    }
    
    /// Get subjects/genres from EPUB metadata
    pub fn subjects(&self) -> &Vec<String> {
        &self.epub_info.subjects
    }
    
    /// Get a formatted display string for subjects/genres
    pub fn subjects_display(&self) -> Option<String> {
        if self.epub_info.subjects.is_empty() {
            None
        } else {
            Some(self.epub_info.subjects.join(", "))
        }
    }
    
    /// Get series information from EPUB metadata
    pub fn series(&self) -> Option<&String> {
        self.epub_info.series.as_ref()
    }
    
    /// Get series number from EPUB metadata
    pub fn series_number(&self) -> Option<&String> {
        self.epub_info.series_number.as_ref()
    }
    
    /// Get formatted series display (e.g., "Series Name #1")
    pub fn series_display(&self) -> Option<String> {
        match (self.series(), self.series_number()) {
            (Some(series), Some(number)) => Some(format!("{} #{}", series, number)),
            (Some(series), None) => Some(series.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubInfo {
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub identifiers: Vec<Identifier>, // Changed from single identifier to vector
    pub subjects: Vec<String>, // Genres/subjects/tags
    pub series: Option<String>,
    pub series_number: Option<String>,
    pub cover_data: Option<Vec<u8>>,
    pub cover_mime_type: Option<String>,
}

impl EpubInfo {
    pub fn sanitized_description(&self) -> Option<String> {
        self.description.as_ref().map(|desc| sanitize_html(desc))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoReaderMetadata {
    pub annotations: Vec<Annotation>,
    pub doc_pages: Option<u32>,
    pub doc_path: Option<String>,
    pub doc_props: Option<DocProps>,
    pub partial_md5_checksum: Option<String>,
    pub percent_finished: Option<f64>,
    pub stats: Option<Stats>,
    pub summary: Option<Summary>,
    pub text_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub chapter: Option<String>,
    pub datetime: Option<String>,
    pub pageno: Option<u32>,
    #[serde(skip_serializing)]
    pub pos0: Option<String>,
    #[serde(skip_serializing)]
    pub pos1: Option<String>,
    pub text: String,
    pub note: Option<String>,
}

impl Annotation {
    pub fn formatted_datetime(&self) -> Option<String> {
        self.datetime.as_ref().and_then(|dt| {
            // Parse "2025-06-17 23:48:38" format
            chrono::NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|ndt| ndt.format("%B %d, %Y at %I:%M %p").to_string())
        })
    }
    
    /// Returns true if this annotation is a bookmark (no pos0/pos1), false if it's a highlight/quote
    pub fn is_bookmark(&self) -> bool {
        self.pos0.is_none() && self.pos1.is_none()
    }
    
    /// Returns true if this annotation is a highlight/quote (has pos0/pos1), false if it's a bookmark
    pub fn is_highlight(&self) -> bool {
        !self.is_bookmark()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocProps {
    pub authors: Option<String>,
    pub description: Option<String>,
    pub identifiers: Option<String>,
    pub keywords: Option<String>,
    pub language: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub authors: Option<String>,
    pub highlights: Option<u32>,
    pub language: Option<String>,
    pub notes: Option<u32>,
    pub pages: Option<u32>,
    pub series: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub modified: Option<String>,
    pub note: Option<String>,
    pub rating: Option<u32>,
    pub status: BookStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BookStatus {
    Reading,
    Complete,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for BookStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookStatus::Reading => write!(f, "reading"),
            BookStatus::Complete => write!(f, "complete"),
            BookStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Data structure representing a book entry from the statistics database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatBook {
    #[serde(skip_serializing)]
    pub id: i64,
    #[serde(skip_serializing)]
    pub title: String,
    #[serde(skip_serializing)]
    pub authors: String,
    pub notes: Option<i64>,
    pub last_open: Option<i64>,
    pub highlights: Option<i64>,
    pub pages: Option<i64>,
    #[serde(skip_serializing)]
    pub md5: String,
    pub total_read_time: Option<i64>,
    #[serde(skip_serializing)]
    pub total_read_pages: Option<i64>,
}

/// Additional statistics calculated for a book from its reading sessions
#[derive(Debug, Clone, Serialize)]
pub struct BookSessionStats {
    pub session_count: i64,
    pub average_session_duration: Option<i64>, // in seconds
    pub longest_session_duration: Option<i64>, // in seconds
    pub last_read_date: Option<String>,
    pub reading_speed: Option<f64>, // pages per hour
}

/// Data structure representing a page stat entry from the statistics database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageStat {
    pub id_book: i64,
    pub page: i64,
    pub start_time: i64,
    pub duration: i64,
}

/// Main container for KoReader statistics data
#[derive(Debug, Clone)]
pub struct StatisticsData {
    pub books: Vec<StatBook>,
    pub page_stats: Vec<PageStat>,
    pub stats_by_md5: HashMap<String, StatBook>,
}

/// Streak information with date ranges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakInfo {
    pub days: i64,
    pub start_date: Option<String>,  // ISO format yyyy-mm-dd
    pub end_date: Option<String>,    // ISO format yyyy-mm-dd
}

impl StreakInfo {
    pub fn new(days: i64, start_date: Option<String>, end_date: Option<String>) -> Self {
        Self {
            days,
            start_date,
            end_date,
        }
    }
    
    /// Format the date range for display
    pub fn date_range_display(&self) -> Option<String> {
        match (&self.start_date, &self.end_date) {
            (Some(start), Some(end)) => {
                if start == end {
                    Some(format!("{}", Self::format_date_display(start)))
                } else {
                    Some(format!("{} - {}", 
                        Self::format_date_display(start),
                        Self::format_date_display(end)))
                }
            },
            (Some(start), None) => Some(format!("{} - now", Self::format_date_display(start))),
            _ => None,
        }
    }
    
    /// Format a date string for display (convert from YYYY-MM-DD to more readable format)
    fn format_date_display(date_str: &str) -> String {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let current_year = chrono::Utc::now().year();
            let date_year = date.year();
            
            if date_year == current_year {
                date.format("%b %d").to_string()
            } else {
                date.format("%b %d %Y").to_string()
            }
        } else {
            date_str.to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingStats {
    // Overall stats
    pub total_read_time: i64,          // seconds
    pub total_page_reads: i64,
    pub longest_read_time_in_day: i64, // seconds
    pub most_pages_in_day: i64,
    
    // Session stats (across all books)
    pub average_session_duration: Option<i64>, // seconds
    pub longest_session_duration: Option<i64>, // seconds
    
    // Streak stats
    pub longest_streak: StreakInfo,
    pub current_streak: StreakInfo,
    
    // Weekly stats
    pub weeks: Vec<WeeklyStats>,
    
    // Daily activity data for heatmap
    pub daily_activity: Vec<DailyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyStats {
    pub start_date: String,            // ISO format yyyy-mm-dd
    pub end_date: String,              // ISO format yyyy-mm-dd
    pub read_time: i64,                // seconds
    pub pages_read: i64,
    pub avg_pages_per_day: f64,
    pub avg_read_time_per_day: f64,    // seconds
    pub longest_session_duration: Option<i64>, // seconds
    pub average_session_duration: Option<i64>, // seconds
}

/// Daily reading stats for the activity heatmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,              // ISO format yyyy-mm-dd
    pub read_time: i64,            // seconds
    pub pages_read: i64,
}

/// Calendar event representing a book reading session (optimized structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub start: String,             // ISO date: yyyy-mm-dd
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,       // ISO date: yyyy-mm-dd (optional, for single-day events)
    pub total_read_time: i64,      // Total seconds read for this book
    pub total_pages_read: i64,     // Total pages read for this book
    pub book_id: String,           // Reference to book metadata
}

/// Book metadata for calendar events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarBook {
    pub title: String,
    pub authors: Vec<String>,
    pub color: String,             // Color for the event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book_path: Option<String>, // Relative path to the book detail page, if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book_cover: Option<String>, // Relative path to the book cover image, if available
}

/// Complete calendar data structure with optimized format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarMonthData {
    pub events: Vec<CalendarEvent>,
    pub books: BTreeMap<String, CalendarBook>,
    pub stats: MonthlyStats,
}

/// Map of "YYYY-MM" to its monthly calendar data payload
pub type CalendarMonths = BTreeMap<String, CalendarMonthData>;

impl CalendarEvent {
    /// Create a new calendar event for a book's reading period
    pub fn new(
        start_date: String,
        end_date: Option<String>,
        total_read_time: i64,
        total_pages_read: i64,
        book_id: String,
    ) -> Self {
        Self {
            start: start_date,
            end: end_date,
            total_read_time,
            total_pages_read,
            book_id,
        }
    }
}

impl CalendarBook {
    /// Create a new calendar book metadata entry
    pub fn new(
        title: String,
        authors: Vec<String>,
        book_path: Option<String>,
        book_cover: Option<String>,
    ) -> Self {
        // Generate a consistent color based on book title
        let color = Self::generate_color(&title);
        
        Self {
            title,
            authors,
            color,
            book_path,
            book_cover,
        }
    }
    
    /// Generate a consistent color for a book based on its title
    fn generate_color(title: &str) -> String {
        // Define a set of pleasant colors for calendar events
        let colors = [
            "#3B82F6", // Blue
            "#10B981", // Green  
            "#F59E0B", // Yellow
            "#EF4444", // Red
            "#8B5CF6", // Purple
            "#F97316", // Orange
            "#06B6D4", // Cyan
            "#84CC16", // Lime
            "#EC4899", // Pink
            "#6366F1", // Indigo
        ];
        
        // Simple hash function to get consistent color based on title
        let mut hash: u32 = 0;
        for byte in title.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        
        let index = (hash as usize) % colors.len();
        colors[index].to_string()
    }
}

/// Pre-calculated monthly reading statistics for the calendar view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyStats {
    pub books_read: usize,   // Number of unique books read in the month
    pub pages_read: i64,     // Total pages read in the month
    pub time_read: i64,      // Total time read in seconds in the month
    pub days_read_pct: u8,   // Percentage of days in the month with any reading activity (0-100)
} 