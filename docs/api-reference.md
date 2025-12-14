# API Reference

This document provides a comprehensive reference for the proto-http-parser-v2 library API.

## Table of Contents

- [Core Traits](#core-traits)
- [Data Structures](#data-structures)
- [Configuration](#configuration)
- [Error Handling](#error-handling)
- [Utilities](#utilities)
- [Build Integration](#build-integration)

## Core Traits

### ProtoParser

The `ProtoParser` trait defines the interface for parsing Protocol Buffer files.

```rust
pub trait ProtoParser {
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn parse_file(&self, path: &Path) -> Result<ProtoFile, Self::Error>;
    fn parse_content(&self, content: &str) -> Result<ProtoFile, Self::Error>;
    fn parse_with_imports(&self, path: &Path, import_paths: &[PathBuf]) -> Result<ProtoFile, Self::Error>;
}
```

#### Methods

- **`parse_file`**: Parse a Protocol Buffer file from disk
- **`parse_content`**: Parse Protocol Buffer content from a string
- **`parse_with_imports`**: Parse a file with custom import paths

#### Implementation: NomProtoParser

```rust
use proto_http_parser::*;

let parser = NomProtoParser::new();
let proto_file = parser.parse_file("service.proto")?;
```

### HttpAnnotationExtractor

The `HttpAnnotationExtractor` trait extracts HTTP annotations from parsed proto files.

```rust
pub trait HttpAnnotationExtractor {
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn extract_routes(&self, proto_file: &ProtoFile) -> Result<Vec<HttpRoute>, Self::Error>;
    fn validate_annotations(&self, routes: &[HttpRoute]) -> Result<(), Self::Error>;
}
```

#### Methods

- **`extract_routes`**: Extract HTTP routes from a proto file
- **`validate_annotations`**: Validate extracted HTTP annotations

#### Implementation: GoogleApiHttpExtractor

```rust
use proto_http_parser::*;

let extractor = GoogleApiHttpExtractor::new();
let routes = extractor.extract_routes(&proto_file)?;
```

### CodeGenerator

The `CodeGenerator` trait generates Rust code from proto services and HTTP routes.

```rust
pub trait CodeGenerator {
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn generate_controller(&self, service: &Service, routes: &[HttpRoute]) -> Result<GeneratedCode, Self::Error>;
    fn generate_service_trait(&self, service: &Service, routes: &[HttpRoute]) -> Result<GeneratedCode, Self::Error>;
}
```

#### Methods

- **`generate_controller`**: Generate a poem-openapi controller
- **`generate_service_trait`**: Generate a service trait interface

#### Implementation: PoemOpenApiGenerator

```rust
use proto_http_parser::*;

let generator = PoemOpenApiGenerator::new();
let controller = generator.generate_controller(&service, &routes)?;
let trait_code = generator.generate_service_trait(&service, &routes)?;
```

### TemplateEngine

The `TemplateEngine` trait provides template rendering capabilities.

```rust
pub trait TemplateEngine {
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn render(&self, template_name: &str, context: &TemplateContext) -> Result<String, Self::Error>;
    fn register_template(&mut self, name: &str, content: &str) -> Result<(), Self::Error>;
    fn register_helper(&mut self, name: &str, helper: Box<dyn TemplateHelper>) -> Result<(), Self::Error>;
}
```

#### Methods

- **`render`**: Render a template with context data
- **`register_template`**: Register a custom template
- **`register_helper`**: Register a template helper function

#### Implementation: HandlebarsTemplateEngine

```rust
use proto_http_parser::*;

let mut engine = HandlebarsTemplateEngine::new();
engine.register_template("custom", "Hello {{name}}!")?;
let result = engine.render("custom", &context)?;
```

## Data Structures

### ProtoFile

Represents a parsed Protocol Buffer file.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ProtoFile {
    pub syntax: ProtocolVersion,
    pub package: Option<String>,
    pub imports: Vec<Import>,
    pub options: Vec<Option>,
    pub services: Vec<Service>,
    pub messages: Vec<Message>,
    pub enums: Vec<Enum>,
}
```

#### Fields

- **`syntax`**: Protocol Buffer syntax version (proto2 or proto3)
- **`package`**: Package name if specified
- **`imports`**: List of imported files
- **`options`**: File-level options
- **`services`**: Service definitions
- **`messages`**: Message type definitions
- **`enums`**: Enum type definitions

### Service

Represents a gRPC service definition.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Service {
    pub name: String,
    pub methods: Vec<RpcMethod>,
    pub options: Vec<Option>,
    pub comments: Vec<Comment>,
}
```

#### Fields

- **`name`**: Service name
- **`methods`**: RPC method definitions
- **`options`**: Service-level options
- **`comments`**: Associated comments

### RpcMethod

Represents an RPC method within a service.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct RpcMethod {
    pub name: String,
    pub input_type: TypeReference,
    pub output_type: TypeReference,
    pub options: Vec<Option>,
    pub comments: Vec<Comment>,
    pub http_annotation: Option<HttpAnnotation>,
}
```

#### Fields

- **`name`**: Method name
- **`input_type`**: Request message type
- **`output_type`**: Response message type
- **`options`**: Method-level options
- **`comments`**: Associated comments
- **`http_annotation`**: HTTP annotation if present

### HttpRoute

Represents an extracted HTTP route.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct HttpRoute {
    pub service_name: String,
    pub method_name: String,
    pub http_method: HttpMethod,
    pub path_template: String,
    pub path_parameters: Vec<PathParameter>,
    pub query_parameters: Vec<QueryParameter>,
    pub request_body: Option<RequestBody>,
    pub response_type: TypeReference,
}
```

#### Fields

- **`service_name`**: Name of the service
- **`method_name`**: Name of the RPC method
- **`http_method`**: HTTP method (GET, POST, etc.)
- **`path_template`**: URL path template
- **`path_parameters`**: Extracted path parameters
- **`query_parameters`**: Inferred query parameters
- **`request_body`**: Request body configuration
- **`response_type`**: Response type reference

### GeneratedCode

Represents generated code output.

```rust
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    pub content: String,
    pub imports: Vec<String>,
    pub dependencies: Vec<String>,
}
```

#### Fields

- **`content`**: Generated code content
- **`imports`**: Required import statements
- **`dependencies`**: Required dependencies

## Configuration

### ProtoHttpParserConfig

Main configuration structure for the library.

```rust
#[derive(Debug, Clone)]
pub struct ProtoHttpParserConfig {
    pub parser: ParserConfig,
    pub extractor: ExtractorConfig,
    pub generator: GeneratorConfig,
    pub template: TemplateConfig,
}
```

#### Loading Configuration

```rust
// From file
let config = ProtoHttpParserConfig::from_file("config.toml")?;

// From environment variables
let config = ProtoHttpParserConfig::from_env()?;

// Auto-load (searches for config files)
let config = ProtoHttpParserConfig::load()?;
```

### ConfigBuilder

Builder pattern for creating configurations programmatically.

```rust
let config = ConfigBuilder::new()
    .preserve_comments(true)
    .generate_service_traits(true)
    .use_dependency_injection(true)
    .infer_query_params(true)
    .use_rustfmt(true)
    .indent_size(4)
    .max_line_length(100)
    .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
    .add_import("use chrono::{DateTime, Utc};")
    .build()?;
```

#### Common Methods

- **`preserve_comments(bool)`**: Whether to preserve proto comments
- **`strict_validation(bool)`**: Enable strict validation
- **`max_import_depth(usize)`**: Maximum import recursion depth
- **`generate_service_traits(bool)`**: Generate service trait interfaces
- **`use_dependency_injection(bool)`**: Use dependency injection pattern
- **`infer_query_params(bool)`**: Automatically infer query parameters
- **`use_rustfmt(bool)`**: Format generated code with rustfmt
- **`indent_size(usize)`**: Indentation size for generated code
- **`max_line_length(usize)`**: Maximum line length
- **`add_type_mapping(proto_type, rust_type)`**: Add custom type mapping
- **`add_import(import)`**: Add additional import statement

## Error Handling

### ProtoHttpParserError

Main error type for the library.

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtoHttpParserError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Code generation error: {0}")]
    CodeGeneration(#[from] CodeGenerationError),
    
    #[error("Template error: {0}")]
    Template(#[from] TemplateError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### ParseError

Errors that occur during proto file parsing.

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Syntax error at line {line}, column {column}: {message}")]
    Syntax { line: usize, column: usize, message: String },
    
    #[error("Unexpected token '{token}' at line {line}, expected {expected}")]
    UnexpectedToken { token: String, line: usize, expected: String },
    
    #[error("Import not found: {import_path}")]
    ImportNotFound { import_path: String },
    
    #[error("Circular import detected: {cycle}")]
    CircularImport { cycle: Vec<String> },
}
```

### ValidationError

Errors that occur during validation.

```rust
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid HTTP annotation: {message}")]
    InvalidHttpAnnotation { message: String, line: usize },
    
    #[error("Undefined type: {type_name}")]
    UndefinedType { type_name: String },
    
    #[error("Conflicting HTTP routes: {route1} and {route2}")]
    ConflictingRoutes { route1: String, route2: String },
}
```

## Utilities

### ProtoHttpCoordinator

High-level coordinator for processing proto files.

```rust
use proto_http_parser::*;

// With default configuration
let coordinator = ProtoHttpCoordinator::new();

// With custom configuration
let coordinator = ProtoHttpCoordinator::with_config(config);

// Process a file
let result = coordinator.process_file("service.proto")?;

// Process content
let result = coordinator.process_content(proto_content)?;
```

#### ProcessResult

```rust
#[derive(Debug)]
pub struct ProcessResult {
    pub proto_file: ProtoFile,
    pub routes: HashMap<String, Vec<HttpRoute>>,
    pub controllers: HashMap<String, GeneratedCode>,
    pub service_traits: HashMap<String, GeneratedCode>,
}
```

### Type Conversion Utilities

```rust
use proto_http_parser::utils::*;

// Convert to snake_case
let snake = to_snake_case("MyServiceName"); // "my_service_name"

// Convert to camel_case
let camel = to_camel_case("my_field_name"); // "myFieldName"

// Convert to PascalCase
let pascal = to_pascal_case("my_type_name"); // "MyTypeName"

// Validate identifier
let is_valid = is_valid_identifier("valid_name"); // true
```

## Build Integration

### BuildIntegration

Convenient API for build.rs integration.

```rust
// build.rs
use proto_http_parser::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BuildIntegration::new()
        .add_proto_file("proto/service.proto")
        .output_dir("src/generated")
        .generate()?;
    
    Ok(())
}
```

#### Methods

- **`new()`**: Create a new build integration instance
- **`with_config(config)`**: Use custom configuration
- **`with_config_file(path)`**: Load configuration from file
- **`with_env_config()`**: Use environment variable configuration
- **`add_proto_file(path)`**: Add a single proto file
- **`add_proto_directory(path)`**: Add all proto files in a directory
- **`add_proto_glob(pattern)`**: Add proto files matching a glob pattern
- **`output_dir(path)`**: Set output directory for generated code
- **`output_dir_from_env(subdir)`**: Use OUT_DIR with subdirectory
- **`verbose(bool)`**: Enable verbose output
- **`generate()`**: Generate the code

#### Advanced Usage

```rust
// build.rs
use proto_http_parser::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BuildIntegration::new()
        .configure(|builder| {
            builder
                .preserve_comments(false)
                .generate_service_traits(true)
                .use_dependency_injection(true)
                .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
        })
        .add_proto_directory("proto")?
        .output_dir("src/generated")
        .verbose(true)
        .generate()?;
    
    Ok(())
}
```

## Examples

### Basic Usage

```rust
use proto_http_parser::*;

// Parse proto file
let parser = NomProtoParser::new();
let proto_file = parser.parse_file("service.proto")?;

// Extract HTTP routes
let extractor = GoogleApiHttpExtractor::new();
let routes = extractor.extract_routes(&proto_file)?;

// Generate code
let generator = PoemOpenApiGenerator::new();
for service in &proto_file.services {
    let service_routes: Vec<_> = routes.iter()
        .filter(|r| r.service_name == service.name)
        .cloned()
        .collect();
    
    let controller = generator.generate_controller(service, &service_routes)?;
    let service_trait = generator.generate_service_trait(service, &service_routes)?;
    
    // Save generated code
    std::fs::write(
        format!("{}_controller.rs", service.name.to_lowercase()),
        controller.content
    )?;
    std::fs::write(
        format!("{}_service.rs", service.name.to_lowercase()),
        service_trait.content
    )?;
}
```

### Using Coordinator

```rust
use proto_http_parser::*;

let coordinator = ProtoHttpCoordinator::new();
let result = coordinator.process_file("service.proto")?;

// Access generated code
for (service_name, controller) in &result.controllers {
    println!("Generated controller for {}: {} lines", 
        service_name, controller.content.lines().count());
}

for (service_name, trait_code) in &result.service_traits {
    println!("Generated trait for {}: {} lines", 
        service_name, trait_code.content.lines().count());
}
```

### Custom Configuration

```rust
use proto_http_parser::*;

let config = ConfigBuilder::new()
    .preserve_comments(true)
    .generate_service_traits(true)
    .use_dependency_injection(true)
    .infer_query_params(true)
    .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
    .add_type_mapping("google.protobuf.Duration", "std::time::Duration")
    .build()?;

let coordinator = ProtoHttpCoordinator::with_config(config);
let result = coordinator.process_file("service.proto")?;
```

## Version Compatibility

- **Rust**: Minimum version 1.70+
- **Edition**: 2021
- **poem**: 3.1+
- **poem-openapi**: 5.1+

## Feature Flags

Currently, the library does not use feature flags, but all functionality is available by default.

## Thread Safety

All core types are thread-safe and can be used across multiple threads:

- `ProtoFile`, `Service`, `HttpRoute`: `Send + Sync`
- `ProtoHttpParserConfig`: `Send + Sync`
- `NomProtoParser`, `GoogleApiHttpExtractor`, `PoemOpenApiGenerator`: `Send + Sync`

## Performance Considerations

- **Parsing**: Uses nom parser combinators for efficient parsing
- **Memory**: Structures use `String` and `Vec` for owned data
- **Caching**: No built-in caching; implement at application level if needed
- **Parallel Processing**: Safe to process multiple files in parallel