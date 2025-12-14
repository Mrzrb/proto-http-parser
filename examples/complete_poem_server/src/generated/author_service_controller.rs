
use poem_openapi::{OpenApi, payload::Json, param::Path, param::Query};
use std::sync::Arc;
// Import types from proto module using relative path from generated directory
use super::super::proto::{Book, Author, DeleteBookResponse, ListBooksResponse, SearchBooksResponse, ListAuthorsResponse};
use super::AuthorServiceService;

/// AuthorService controller generated from Protocol Buffer service
#[derive(Clone)]
pub struct AuthorServiceController<T: AuthorServiceService> {
    service: Arc<T>,
}

impl<T: AuthorServiceService> AuthorServiceController<T> {
    /// Create a new controller with the given service implementation
    pub fn new(service: T) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}

#[poem_openapi::OpenApi]
impl<T: AuthorServiceService + Send + Sync + 'static> AuthorServiceController<T> {
    /// GetAuthor endpoint
    #[oai(path = "/v1/authors/{author_id}", method = "get")]
    async fn get_author(
        &self,
        author_id: Path<String>,
    ) -> poem_openapi::payload::Json<Author> {
        let result = self.service.get_author(
            author_id.0,
        ).await.unwrap();
        
        Json(result)
    }

    /// CreateAuthor endpoint
    #[oai(path = "/v1/authors", method = "post")]
    async fn create_author(
        &self,
        body: Json<Author>,
    ) -> poem_openapi::payload::Json<Author> {
        let result = self.service.create_author(
            body.0,
        ).await.unwrap();
        
        Json(result)
    }

    /// UpdateAuthor endpoint
    #[oai(path = "/v1/authors/{author.id}", method = "put")]
    async fn update_author(
        &self,
        author_id: Path<String>,
        body: Json<String>,
    ) -> poem_openapi::payload::Json<Author> {
        let result = self.service.update_author(
            author_id.0,
            body.0,
        ).await.unwrap();
        
        Json(result)
    }

    /// ListAuthors endpoint
    #[oai(path = "/v1/authors", method = "get")]
    async fn list_authors(
        &self,
    ) -> poem_openapi::payload::Json<ListAuthorsResponse> {
        let result = self.service.list_authors(
        ).await.unwrap();
        
        Json(result)
    }

    /// GetBooksByAuthor endpoint
    #[oai(path = "/v1/authors/{author_id}/books", method = "get")]
    async fn get_books_by_author(
        &self,
        author_id: Path<String>,
    ) -> poem_openapi::payload::Json<ListBooksResponse> {
        let result = self.service.get_books_by_author(
            author_id.0,
        ).await.unwrap();
        
        Json(result)
    }

}
