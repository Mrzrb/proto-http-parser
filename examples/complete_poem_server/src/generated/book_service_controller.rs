
use poem_openapi::{OpenApi, payload::Json, param::Path, param::Query};
use std::sync::Arc;
// Import types from proto module using relative path from generated directory
use super::super::proto::{Book, Author, DeleteBookResponse, ListBooksResponse, SearchBooksResponse, ListAuthorsResponse};
use super::BookServiceService;

/// BookService controller generated from Protocol Buffer service
#[derive(Clone)]
pub struct BookServiceController<T: BookServiceService> {
    service: Arc<T>,
}

impl<T: BookServiceService> BookServiceController<T> {
    /// Create a new controller with the given service implementation
    pub fn new(service: T) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}

#[poem_openapi::OpenApi]
impl<T: BookServiceService + Send + Sync + 'static> BookServiceController<T> {
    /// GetBook endpoint
    #[oai(path = "/v1/books/{book_id}", method = "get")]
    async fn get_book(
        &self,
        book_id: Path<String>,
    ) -> poem_openapi::payload::Json<Book> {
        let result = self.service.get_book(
            book_id.0,
        ).await.unwrap();
        
        Json(result)
    }

    /// CreateBook endpoint
    #[oai(path = "/v1/books", method = "post")]
    async fn create_book(
        &self,
        body: Json<Book>,
    ) -> poem_openapi::payload::Json<Book> {
        let result = self.service.create_book(
            body.0,
        ).await.unwrap();
        
        Json(result)
    }

    /// UpdateBook endpoint
    #[oai(path = "/v1/books/{book.id}", method = "put")]
    async fn update_book(
        &self,
        book_id: Path<String>,
        body: Json<String>,
    ) -> poem_openapi::payload::Json<Book> {
        let result = self.service.update_book(
            book_id.0,
            body.0,
        ).await.unwrap();
        
        Json(result)
    }

    /// DeleteBook endpoint
    #[oai(path = "/v1/books/{book_id}", method = "delete")]
    async fn delete_book(
        &self,
        book_id: Path<String>,
    ) -> poem_openapi::payload::Json<DeleteBookResponse> {
        let result = self.service.delete_book(
            book_id.0,
        ).await.unwrap();
        
        Json(result)
    }

    /// ListBooks endpoint
    #[oai(path = "/v1/books", method = "get")]
    async fn list_books(
        &self,
    ) -> poem_openapi::payload::Json<ListBooksResponse> {
        let result = self.service.list_books(
        ).await.unwrap();
        
        Json(result)
    }

    /// SearchBooks endpoint
    #[oai(path = "/v1/books/search", method = "get")]
    async fn search_books(
        &self,
    ) -> poem_openapi::payload::Json<SearchBooksResponse> {
        let result = self.service.search_books(
        ).await.unwrap();
        
        Json(result)
    }

}
