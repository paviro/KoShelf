use askama::Template;
use crate::models::*;

#[derive(Clone)]
pub struct NavItem {
    pub label: String,
    pub href: String,
    pub icon_svg: String,
    pub is_active: bool,
}

#[derive(Template)]
#[template(path = "book_list/book_list.html")]
pub struct IndexTemplate {
    pub site_title: String,
    pub reading_books: Vec<Book>,
    pub completed_books: Vec<Book>,
    pub unread_books: Vec<Book>,
    pub version: String,
    pub last_updated: String,
    pub navbar_items: Vec<NavItem>,
}

#[derive(Template)]
#[template(path = "book_details/book_details.html")]
pub struct BookTemplate {
    pub site_title: String,
    pub book: Book,
    pub book_stats: Option<StatBook>,
    pub session_stats: Option<BookSessionStats>,
    pub version: String,
    pub last_updated: String,
    pub navbar_items: Vec<NavItem>,
}

#[derive(Template)]
#[template(path = "book_details/book_details.md", escape = "none")]
pub struct BookMarkdownTemplate {
    pub book: Book,
    pub book_stats: Option<StatBook>,
    pub session_stats: Option<BookSessionStats>,
    pub version: String,
    pub last_updated: String,
}

#[derive(Template)]
#[template(path = "statistics/statistics.html")]
pub struct StatsTemplate {
    pub site_title: String,
    pub reading_stats: ReadingStats,
    pub available_years: Vec<i32>,
    pub version: String,
    pub last_updated: String,
    pub navbar_items: Vec<NavItem>,
}

#[derive(Template)]
#[template(path = "calendar/calendar.html")]
pub struct CalendarTemplate {
    pub site_title: String,
    pub version: String,
    pub last_updated: String,
    pub navbar_items: Vec<NavItem>,
} 