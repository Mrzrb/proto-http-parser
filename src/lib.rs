//! Proto HTTP Parser v2
//! 
//! A complete rewrite of the Protocol Buffer HTTP annotation parser with a modular
//! architecture for generating high-quality poem-openapi controller code.
//!
//! # Core Features
//! 
//! - **Powerful Parsing**: Uses nom parser combinators for robust proto file parsing
//! - **HTTP Annotation Support**: Full support for google.api.http annotations
//! - **Code Generation**: Generates poem-openapi controllers with dependency injection
//! - **Template System**: Flexible Handlebars-based template engine
//! - **Comprehensive Validation**: Detailed error reporting and validation
//! - **Extensible Architecture**: Plugin system for custom extensions
//!
//! # Architecture
//!
//! The library is built around four core traits:
//! - [`ProtoParser`] - Parses Protocol Buffer files
//! - [`HttpAnnotationExtractor`] - Extracts HTTP annotations
//! - [`CodeGenerator`] - Generates Rust code
//! - [`TemplateEngine`] - Renders templates
//!
//! # Example Usage
//!
//! ## Simple Usage with Coordinator
//!
//! ```rust,no_run
//! use proto_http_parser::*;
//! use std::path::Path;
//! 
//! // Create coordinator with default configuration
//! let coordinator = ProtoHttpCoordinator::new();
//! 
//! // Process a single proto file
//! let result = coordinator.process_file("service.proto")?;
//! 
//! // Write generated code to output directory
//! coordinator.write_generated_code(&result, "src/generated")?;
//! 
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Batch Processing
//!
//! ```rust,no_run
//! use proto_http_parser::*;
//! 
//! let coordinator = ProtoHttpCoordinator::new();
//! 
//! // Process multiple files
//! let proto_files = ["user.proto", "product.proto", "order.proto"];
//! let batch_result = coordinator.process_files(&proto_files)?;
//! 
//! // Write all generated code
//! coordinator.write_batch_results(&batch_result, "src/generated")?;
//! 
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Build.rs Integration
//!
//! ```rust,no_run
//! // In build.rs
//! use proto_http_parser::BuildIntegration;
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     BuildIntegration::new()
//!         .add_proto_directory("proto")?
//!         .output_dir("src/generated")
//!         .generate()?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Manual Component Usage
//!
//! ```rust,no_run
//! use proto_http_parser::*;
//! use std::path::Path;
//! 
//! // Create individual components
//! let parser = NomProtoParser::new();
//! let extractor = GoogleApiHttpExtractor::new();
//! let generator = PoemOpenApiGenerator::new();
//! 
//! // Parse proto file
//! let proto_file = parser.parse_file(Path::new("service.proto"))?;
//! 
//! // Extract HTTP routes
//! let routes = extractor.extract_routes(&proto_file)?;
//! 
//! // Generate controller code
//! for service in &proto_file.services {
//!     let controller = generator.generate_controller(service, &routes)?;
//!     let service_trait = generator.generate_service_trait(service, &routes)?;
//!     
//!     // Write generated code to files
//!     std::fs::write(format!("{}_controller.rs", service.name), controller.content)?;
//!     std::fs::write(format!("{}_service.rs", service.name), service_trait.content)?;
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

// Core trait definitions and data structures
pub mod core;

// Parser implementation using nom
pub mod parser;

// HTTP annotation extraction
pub mod extractor;

// Code generation engine
pub mod generator;

// Template system
pub mod templates;

// Configuration management
pub mod config;

// Error types and handling
pub mod errors;

// Validation engine
pub mod validation;

// Error reporting system
pub mod error_reporter;

// Utility functions
pub mod utils;

// Main coordinator
pub mod coordinator;

// Plugin system
pub mod plugins;

// Re-export core types for convenience
pub use core::*;
pub use parser::NomProtoParser;
pub use extractor::GoogleApiHttpExtractor;
pub use generator::PoemOpenApiGenerator;
pub use templates::HandlebarsTemplateEngine;
pub use validation::ValidationEngine;
pub use error_reporter::ErrorReporter;
pub use coordinator::{ProtoHttpCoordinator, ProcessResult, BatchProcessResult};

// Re-export utility functions
pub use utils::*;

// Re-export build integration for convenience
pub use coordinator::build_integration::BuildIntegration;


/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type alias for the library
pub type Result<T> = std::result::Result<T, ProtoHttpParserError>;