
use async_trait::async_trait;
// Import types from proto module using relative path from generated directory
use super::super::proto::{Book, Author, DeleteBookResponse, ListBooksResponse, SearchBooksResponse, ListAuthorsResponse};

/// Service trait for AuthorService
/// 
/// Implement this trait to provide business logic for the AuthorService service.
/// The generated controller will delegate to your implementation.
#[async_trait]
pub trait AuthorServiceService {
    /// GetAuthor operation
    async fn get_author(
        &self,
        author_id: String,
    ) -> Result<Author, Box<dyn std::error::Error>>;

    /// CreateAuthor operation
    async fn create_author(
        &self,
        request: Author,
    ) -> Result<Author, Box<dyn std::error::Error>>;

    /// UpdateAuthor operation
    async fn update_author(
        &self,
        author_id: String,
        author: String,
    ) -> Result<Author, Box<dyn std::error::Error>>;

    /// ListAuthors operation
    async fn list_authors(
        &self,
    ) -> Result<ListAuthorsResponse, Box<dyn std::error::Error>>;

    /// GetBooksByAuthor operation
    async fn get_books_by_author(
        &self,
        author_id: String,
    ) -> Result<ListBooksResponse, Box<dyn std::error::Error>>;

}
