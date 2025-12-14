//! Complete Poem server example using generated controllers

use poem::{listener::TcpListener, Route, Server};
use poem_openapi::OpenApiService;
use tracing::{info, Level};
use tracing_subscriber;

use complete_poem_server_example::generated::*;
use complete_poem_server_example::services::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Bookstore API server...");

    // Create service implementations
    let book_service = BookServiceImpl::new();
    let author_service = AuthorServiceImpl::new();

    // Create controllers with dependency injection
    let book_controller = BookServiceController::new(book_service);
    let author_controller = AuthorServiceController::new(author_service);

    // Create OpenAPI service
    let api_service = OpenApiService::new(
        (book_controller, author_controller),
        "Bookstore API",
        "1.0.0"
    )
    .server("http://localhost:3000")
    .description("A complete bookstore API built with proto-http-parser-v2");

    // Create routes
    let app = Route::new()
        .nest("/api", api_service.clone())
        .nest("/docs", api_service.swagger_ui())
        .nest("/spec", api_service.spec_endpoint());

    info!("Server starting on http://localhost:3000");
    info!("API documentation available at http://localhost:3000/docs");
    info!("OpenAPI spec available at http://localhost:3000/spec");

    // Start the server
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await?;

    Ok(())
}