use crate::i18n::Translations;
use crate::models::*;
use askama::Template;
use std::rc::Rc;

#[derive(Clone)]
pub struct NavItem {
    pub label: String,
    pub href: String,
    pub icon_svg: String,
    pub is_active: bool,
}

#[derive(Template)]
#[template(path = "book_list/book_list.html", whitespace = "minimize")]
pub struct IndexTemplate {
    pub site_title: String,
    /// Base path for detail pages (e.g. "/books/" or "/comics/").
    #[allow(dead_code)]
    pub details_base_path: String,
    pub reading_books: Vec<LibraryItem>,
    pub completed_books: Vec<LibraryItem>,
    pub abandoned_books: Vec<LibraryItem>,
    pub unread_books: Vec<LibraryItem>,
    pub version: String,
    pub last_updated: String,
    pub navbar_items: Vec<NavItem>,
    pub translation: Rc<Translations>,
}

#[derive(Template)]
#[template(path = "recap/recap_year.html", whitespace = "minimize")]
pub struct RecapTemplate {
    pub site_title: String,
    /// "all" | "books" | "comics"
    pub recap_scope: String,
    /// Whether we should show the Books/Comics filter UI (only when both exist).
    pub show_type_filter: bool,
    pub year: i32,
    pub available_years: Vec<i32>,
    pub prev_year: Option<i32>,
    pub next_year: Option<i32>,
    pub monthly: Vec<MonthRecap>,
    pub summary: YearlySummary,
    pub version: String,
    pub last_updated: String,
    pub navbar_items: Vec<NavItem>,
    pub translation: Rc<Translations>,
}

#[derive(Template)]
#[template(path = "recap/recap_empty.html", whitespace = "minimize")]
pub struct RecapEmptyTemplate {
    pub site_title: String,
    /// "all" | "books" | "comics"
    pub recap_scope: String,
    /// Whether we should show the Books/Comics filter UI (only when both exist).
    pub show_type_filter: bool,
    /// When present, show year + scope pickers and prev/next navigation.
    pub year: Option<i32>,
    pub available_years: Vec<i32>,
    pub prev_year: Option<i32>,
    pub next_year: Option<i32>,
    pub version: String,
    pub last_updated: String,
    pub navbar_items: Vec<NavItem>,
    pub translation: Rc<Translations>,
}

#[derive(Template)]
#[template(path = "book_details/book_details.html", whitespace = "minimize")]
pub struct BookTemplate {
    pub site_title: String,
    pub book: LibraryItem,
    pub book_stats: Option<StatBook>,
    pub session_stats: Option<BookSessionStats>,
    pub search_base_path: String,
    pub version: String,
    pub last_updated: String,
    pub navbar_items: Vec<NavItem>,
    pub translation: Rc<Translations>,
}

#[derive(Template)]
#[template(path = "book_details/book_details.md", escape = "none")]
pub struct BookMarkdownTemplate {
    pub book: LibraryItem,
    pub book_stats: Option<StatBook>,
    pub session_stats: Option<BookSessionStats>,
    pub version: String,
    pub last_updated: String,
}

#[derive(Template)]
#[template(path = "statistics/statistics.html", whitespace = "minimize")]
pub struct StatsTemplate {
    pub site_title: String,
    /// "all" | "books" | "comics"
    pub stats_scope: String,
    /// Whether we should show the Books/Comics filter UI (only when both exist).
    pub show_type_filter: bool,
    /// Base path for stats JSON (e.g. "/assets/json/statistics", "/assets/json/statistics/books")
    pub stats_json_base_path: String,
    pub reading_stats: ReadingStats,
    pub available_years: Vec<i32>,
    pub version: String,
    pub last_updated: String,
    pub navbar_items: Vec<NavItem>,
    pub translation: Rc<Translations>,
}

#[derive(Template)]
#[template(path = "calendar/calendar.html", whitespace = "minimize")]
pub struct CalendarTemplate {
    pub site_title: String,
    /// Whether we should show the Books/Comics filter UI (only when both exist).
    pub show_type_filter: bool,
    pub version: String,
    pub last_updated: String,
    pub navbar_items: Vec<NavItem>,
    pub translation: Rc<Translations>,
}
