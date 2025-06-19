use askama::Template;
use crate::models::*;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub site_title: String,
    pub reading_books: Vec<Book>,
    pub completed_books: Vec<Book>,
    pub unread_books: Vec<Book>,
    pub version: String,
    pub last_updated: String,
}

#[derive(Template)]
#[template(path = "book.html")]
pub struct BookTemplate {
    pub site_title: String,
    pub book: Book,
} 