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
#[template(path = "index.html")]
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
#[template(path = "book.html")]
pub struct BookTemplate {
    pub site_title: String,
    pub book: Book,
} 