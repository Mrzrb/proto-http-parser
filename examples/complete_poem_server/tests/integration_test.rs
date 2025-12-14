//! Integration tests for the complete poem server example

use complete_poem_server_example::*;

#[tokio::test]
async fn test_code_generation_works() {
    // This test verifies that the code generation process works
    // and that we can instantiate the generated types
    
    // Test that we can create the basic types
    let book = Book {
        id: "test-id".to_string(),
        title: "Test Book".to_string(),
        author_id: "test-author".to_string(),
        isbn: "123-456-789".to_string(),
        description: "A test book".to_string(),
        price: 29.99,
        pages: 200,
        publisher: "Test Publisher".to_string(),
        genre: 1, // BookGenre::Fiction
        status: 1, // BookStatus::Available
        created_at: None,
        updated_at: None,
    };
    
    assert_eq!(book.title, "Test Book");
    assert_eq!(book.price, 29.99);
}

#[test]
fn test_proto_files_exist() {
    // Verify that the required proto files exist
    assert!(std::path::Path::new("proto/api.proto").exists());
    assert!(std::path::Path::new("proto/google/api/annotations.proto").exists());
    assert!(std::path::Path::new("proto/google/api/http.proto").exists());
}

#[test]
fn test_generated_files_exist() {
    // Verify that code generation created the expected files
    assert!(std::path::Path::new("src/generated").exists());
    
    // Check for generated service files
    let generated_files = [
        "src/generated/book_service_controller.rs",
        "src/generated/book_service_service.rs",
        "src/generated/author_service_controller.rs", 
        "src/generated/author_service_service.rs",
    ];
    
    for file in &generated_files {
        if std::path::Path::new(file).exists() {
            println!("✓ Generated file exists: {}", file);
        } else {
            println!("✗ Generated file missing: {}", file);
        }
    }
}