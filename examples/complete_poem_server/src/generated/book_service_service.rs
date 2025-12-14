
use async_trait::async_trait;
// Import types from proto module using relative path from generated directory
use super::super::proto::{Book, Author, DeleteBookResponse, ListBooksResponse, SearchBooksResponse, ListAuthorsResponse};

/// Service trait for BookService
/// 
/// Implement this trait to provide business logic for the BookService service.
/// The generated controller will delegate to your implementation.
#[async_trait]
pub trait BookServiceService {
    /// GetBook operation
    async fn get_book(
        &self,
        book_id: String,
    ) -> Result<Book, Box<dyn std::error::Error>>;

    /// CreateBook operation
    async fn create_book(
        &self,
        request: Book,
    ) -> Result<Book, Box<dyn std::error::Error>>;

    /// UpdateBook operation
    async fn update_book(
        &self,
        book_id: String,
        book: String,
    ) -> Result<Book, Box<dyn std::error::Error>>;

    /// DeleteBook operation
    async fn delete_book(
        &self,
        book_id: String,
    ) -> Result<DeleteBookResponse, Box<dyn std::error::Error>>;

    /// ListBooks operation
    async fn list_books(
        &self,
    ) -> Result<ListBooksResponse, Box<dyn std::error::Error>>;

    /// SearchBooks operation
    async fn search_books(
        &self,
    ) -> Result<SearchBooksResponse, Box<dyn std::error::Error>>;

}
