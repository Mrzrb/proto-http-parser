//! Author service implementation

use crate::generated::AuthorServiceService;
use crate::proto::{Author, ListAuthorsResponse, ListBooksResponse};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// In-memory author service implementation
#[derive(Clone)]
pub struct AuthorServiceImpl {
    authors: Arc<Mutex<HashMap<String, Author>>>,
}

impl AuthorServiceImpl {
    pub fn new() -> Self {
        let mut authors = HashMap::new();
        
        // Add some sample authors
        let author1 = Author {
            id: "author-1".to_string(),
            name: "Steve Klabnik".to_string(),
            email: Some("steve@example.com".to_string()),
            bio: Some("Steve Klabnik is a software developer and author, known for his work on the Rust programming language.".to_string()),
        };
        
        let author2 = Author {
            id: "author-2".to_string(),
            name: "Jim Blandy".to_string(),
            email: Some("jim@example.com".to_string()),
            bio: Some("Jim Blandy is a software engineer and co-author of Programming Rust.".to_string()),
        };
        
        authors.insert(author1.id.clone(), author1);
        authors.insert(author2.id.clone(), author2);
        
        Self {
            authors: Arc::new(Mutex::new(authors)),
        }
    }
}

#[async_trait::async_trait]
impl AuthorServiceService for AuthorServiceImpl {
    async fn get_author(&self, author_id: String) -> Result<Author, Box<dyn std::error::Error>> {
        let authors = self.authors.lock().unwrap();
        
        match authors.get(&author_id) {
            Some(author) => Ok(author.clone()),
            None => Err(format!("Author with id '{}' not found", author_id).into()),
        }
    }

    async fn create_author(&self, request: Author) -> Result<Author, Box<dyn std::error::Error>> {
        let mut authors = self.authors.lock().unwrap();
        
        let author = Author {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            email: request.email,
            bio: request.bio,
        };
        
        authors.insert(author.id.clone(), author.clone());
        Ok(author)
    }

    async fn update_author(&self, author_id: String, author: String) -> Result<Author, Box<dyn std::error::Error>> {
        let mut authors = self.authors.lock().unwrap();
        
        match authors.get_mut(&author_id) {
            Some(existing_author) => {
                // In a real implementation, you'd parse the author JSON and update fields
                existing_author.name = format!("Updated: {}", existing_author.name);
                Ok(existing_author.clone())
            }
            None => Err(format!("Author with id '{}' not found", author_id).into()),
        }
    }

    async fn list_authors(&self) -> Result<ListAuthorsResponse, Box<dyn std::error::Error>> {
        let authors = self.authors.lock().unwrap();
        
        let all_authors: Vec<Author> = authors.values().cloned().collect();
        
        Ok(ListAuthorsResponse {
            authors: all_authors,
            next_page_token: None,
        })
    }

    async fn get_books_by_author(&self, author_id: String) -> Result<ListBooksResponse, Box<dyn std::error::Error>> {
        // In a real implementation, this would query the book service
        // For this example, we'll return an empty list
        
        Ok(ListBooksResponse {
            books: Vec::new(),
            next_page_token: None,
        })
    }
}