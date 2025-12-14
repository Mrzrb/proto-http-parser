//! Service implementations for the bookstore API

pub mod book_service;
pub mod author_service;

pub use book_service::BookServiceImpl;
pub use author_service::AuthorServiceImpl;