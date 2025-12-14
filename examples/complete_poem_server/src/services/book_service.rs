//! Book service implementation

use crate::generated::BookServiceService;
use crate::proto::{Book, DeleteBookResponse, ListBooksResponse, SearchBooksResponse};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// In-memory book service implementation
#[derive(Clone)]
pub struct BookServiceImpl {
    books: Arc<Mutex<HashMap<String, Book>>>,
}

impl BookServiceImpl {
    pub fn new() -> Self {
        let mut books = HashMap::new();
        
        // Add some sample books
        let book1 = Book {
            id: "book-1".to_string(),
            title: "The Rust Programming Language".to_string(),
            author_id: "author-1".to_string(),
            published_date: Some("2023-01-01T00:00:00Z".to_string()),
            isbn: Some("978-1593278281".to_string()),
            description: Some("A comprehensive guide to Rust programming".to_string()),
        };
        
        let book2 = Book {
            id: "book-2".to_string(),
            title: "Programming Rust".to_string(),
            author_id: "author-2".to_string(),
            published_date: Some("2023-01-02T00:00:00Z".to_string()),
            isbn: Some("978-1491927281".to_string()),
            description: Some("Fast, safe systems development".to_string()),
        };
        
        books.insert(book1.id.clone(), book1);
        books.insert(book2.id.clone(), book2);
        
        Self {
            books: Arc::new(Mutex::new(books)),
        }
    }
}

#[async_trait::async_trait]
impl BookServiceService for BookServiceImpl {
    async fn get_book(&self, book_id: String) -> Result<Book, Box<dyn std::error::Error>> {
        let books = self.books.lock().unwrap();
        
        match books.get(&book_id) {
            Some(book) => Ok(book.clone()),
            None => Err(format!("Book with id '{}' not found", book_id).into()),
        }
    }

    async fn create_book(&self, request: Book) -> Result<Book, Box<dyn std::error::Error>> {
        let mut books = self.books.lock().unwrap();
        
        let book = Book {
            id: Uuid::new_v4().to_string(),
            title: request.title,
            author_id: request.author_id,
            published_date: Some(chrono::Utc::now().to_rfc3339()),
            isbn: request.isbn,
            description: request.description,
        };
        
        books.insert(book.id.clone(), book.clone());
        Ok(book)
    }

    async fn update_book(&self, book_id: String, book: String) -> Result<Book, Box<dyn std::error::Error>> {
        let mut books = self.books.lock().unwrap();
        
        match books.get_mut(&book_id) {
            Some(existing_book) => {
                // In a real implementation, you'd parse the book JSON and update fields
                existing_book.title = format!("Updated: {}", existing_book.title);
                Ok(existing_book.clone())
            }
            None => Err(format!("Book with id '{}' not found", book_id).into()),
        }
    }

    async fn delete_book(&self, book_id: String) -> Result<DeleteBookResponse, Box<dyn std::error::Error>> {
        let mut books = self.books.lock().unwrap();
        
        match books.remove(&book_id) {
            Some(_) => Ok(DeleteBookResponse {
                success: true,
                message: Some(format!("Book '{}' deleted successfully", book_id)),
            }),
            None => Err(format!("Book with id '{}' not found", book_id).into()),
        }
    }



    async fn list_books(&self) -> Result<ListBooksResponse, Box<dyn std::error::Error>> {
        let books = self.books.lock().unwrap();
        
        let mut all_books: Vec<Book> = books.values().cloned().collect();
        all_books.sort_by(|a, b| a.title.cmp(&b.title));
        
        Ok(ListBooksResponse {
            books: all_books,
            next_page_token: None,
        })
    }

    async fn search_books(&self) -> Result<SearchBooksResponse, Box<dyn std::error::Error>> {
        let books = self.books.lock().unwrap();
        
        let all_books: Vec<Book> = books.values().cloned().collect();
        let total_count = all_books.len() as i32;
        
        Ok(SearchBooksResponse {
            books: all_books,
            total_count,
        })
    }
}