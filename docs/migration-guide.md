# Migration Guide: v1 to v2

This guide helps you migrate from proto-http-parser v1 to v2. The v2 rewrite introduces significant architectural improvements, better error handling, and more flexible configuration.

## Table of Contents

- [Overview of Changes](#overview-of-changes)
- [Breaking Changes](#breaking-changes)
- [Migration Steps](#migration-steps)
- [Configuration Migration](#configuration-migration)
- [API Changes](#api-changes)
- [Generated Code Changes](#generated-code-changes)
- [Build Script Migration](#build-script-migration)
- [Common Migration Issues](#common-migration-issues)

## Overview of Changes

### What's New in v2

- **Modular Architecture**: Clear separation of parsing, validation, and code generation
- **Improved Error Handling**: Detailed error messages with line numbers and context
- **Flexible Configuration**: Multiple configuration sources (files, environment, code)
- **Plugin System**: Extensible architecture for custom validators and generators
- **Better Type Safety**: Stronger type system and validation
- **Template Engine**: Handlebars-based template system for customizable code generation
- **Dependency Injection**: Generated controllers support dependency injection pattern
- **Service Traits**: Separate trait generation for clean business logic separation

### What's Removed

- **String-based parsing**: Replaced with proper nom-based parser
- **Hardcoded templates**: Replaced with flexible template system
- **Monolithic API**: Split into focused traits and components
- **Limited configuration**: Replaced with comprehensive configuration system

## Breaking Changes

### 1. Main API Structure

**v1:**
```rust
use proto_http_parser::*;

let generator = Generator::new();
let result = generator.generate_from_file("service.proto")?;
```

**v2:**
```rust
use proto_http_parser_v2::*;

// Option 1: Use coordinator (recommended)
let coordinator = ProtoHttpCoordinator::new();
let result = coordinator.process_file("service.proto")?;

// Option 2: Use individual components
let parser = NomProtoParser::new();
let extractor = GoogleApiHttpExtractor::new();
let generator = PoemOpenApiGenerator::new();

let proto_file = parser.parse_file("service.proto")?;
let routes = extractor.extract_routes(&proto_file)?;
// ... generate code
```

### 2. Configuration Structure

**v1:**
```rust
let config = GeneratorConfig {
    preserve_comments: true,
    generate_traits: true,
    format_code: true,
    ..Default::default()
};
```

**v2:**
```rust
let config = ConfigBuilder::new()
    .preserve_comments(true)
    .generate_service_traits(true)
    .use_rustfmt(true)
    .build()?;
```

### 3. Generated Code Structure

**v1:** Generated monolithic controllers with embedded business logic

**v2:** Generated controllers with dependency injection and separate service traits

### 4. Error Types

**v1:**
```rust
enum GeneratorError {
    ParseError(String),
    ValidationError(String),
    IoError(std::io::Error),
}
```

**v2:**
```rust
enum ProtoHttpParserError {
    Parse(ParseError),
    Validation(ValidationError),
    CodeGeneration(CodeGenerationError),
    Template(TemplateError),
    Io(std::io::Error),
}
```

## Migration Steps

### Step 1: Update Dependencies

**Update Cargo.toml:**

```toml
# Remove v1 dependency
# proto-http-parser = "1.0"

# Add v2 dependency
[dependencies]
proto-http-parser-v2 = "2.0"

[build-dependencies]
proto-http-parser-v2 = "2.0"
```

### Step 2: Update Imports

**v1:**
```rust
use proto_http_parser::*;
```

**v2:**
```rust
use proto_http_parser_v2::*;
```

### Step 3: Migrate Build Scripts

**v1 build.rs:**
```rust
use proto_http_parser::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let generator = Generator::new();
    generator.generate_from_file("proto/service.proto", "src/generated")?;
    Ok(())
}
```

**v2 build.rs:**
```rust
use proto_http_parser_v2::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BuildIntegration::new()
        .add_proto_file("proto/service.proto")
        .output_dir("src/generated")
        .generate()?;
    
    Ok(())
}
```

### Step 4: Update Application Code

**v1 usage:**
```rust
mod generated;
use generated::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let controller = UserController::new();
    
    let app = poem::Route::new()
        .nest("/api", controller);
    
    // ... start server
}
```

**v2 usage:**
```rust
mod generated;
mod services;

use generated::*;
use services::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create service implementation
    let service = UserServiceImpl::new();
    
    // Create controller with dependency injection
    let controller = UserServiceController::new(service);
    
    let app = poem::Route::new()
        .nest("/api", controller);
    
    // ... start server
}
```

### Step 5: Implement Service Traits

In v2, you need to implement the generated service traits:

```rust
// src/services/user_service.rs
use crate::generated::*;
use poem::Result;

#[derive(Clone)]
pub struct UserServiceImpl {
    // Your service state
}

impl UserServiceImpl {
    pub fn new() -> Self {
        Self {
            // Initialize your service
        }
    }
}

#[async_trait::async_trait]
impl UserService for UserServiceImpl {
    async fn get_user(&self, user_id: String) -> Result<User> {
        // Implement your business logic here
        // This was previously embedded in the controller
        
        Ok(User {
            id: user_id,
            name: "John Doe".to_string(),
            // ... other fields
        })
    }
    
    async fn create_user(&self, request: CreateUserRequest) -> Result<User> {
        // Implement user creation logic
        Ok(User {
            id: uuid::Uuid::new_v4().to_string(),
            name: request.name,
            // ... other fields
        })
    }
}
```

## Configuration Migration

### Basic Configuration

**v1:**
```rust
let config = GeneratorConfig {
    preserve_comments: true,
    generate_traits: true,
    use_dependency_injection: true,
    format_code: true,
    indent_size: 4,
    max_line_length: 100,
    type_mappings: vec![
        ("google.protobuf.Timestamp".to_string(), "chrono::DateTime<chrono::Utc>".to_string()),
    ],
    additional_imports: vec![
        "use chrono::{DateTime, Utc};".to_string(),
    ],
};
```

**v2:**
```rust
let config = ConfigBuilder::new()
    .preserve_comments(true)
    .generate_service_traits(true)
    .use_dependency_injection(true)
    .use_rustfmt(true)
    .indent_size(4)
    .max_line_length(100)
    .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
    .add_import("use chrono::{DateTime, Utc};")
    .build()?;
```

### Configuration Files

**v1:** Limited configuration file support

**v2:** Full TOML configuration support

Create `proto-http-parser.toml`:
```toml
[parser]
preserve_comments = true
strict_validation = true
max_import_depth = 10

[generator]
generate_service_traits = true
use_dependency_injection = true

[generator.type_mappings]
"google.protobuf.Timestamp" = "chrono::DateTime<chrono::Utc>"

[generator.formatting]
indent_size = 4
max_line_length = 100
use_rustfmt = true
```

## API Changes

### Parsing

**v1:**
```rust
let generator = Generator::new();
let proto_file = generator.parse_file("service.proto")?;
```

**v2:**
```rust
let parser = NomProtoParser::new();
let proto_file = parser.parse_file("service.proto")?;
```

### HTTP Route Extraction

**v1:** Embedded in generator

**v2:** Separate extractor
```rust
let extractor = GoogleApiHttpExtractor::new();
let routes = extractor.extract_routes(&proto_file)?;
```

### Code Generation

**v1:**
```rust
let code = generator.generate_controller(&service)?;
```

**v2:**
```rust
let generator = PoemOpenApiGenerator::new();
let controller = generator.generate_controller(&service, &routes)?;
let service_trait = generator.generate_service_trait(&service, &routes)?;
```

## Generated Code Changes

### Controller Structure

**v1 Generated Controller:**
```rust
pub struct UserController {
    // Business logic embedded here
}

impl UserController {
    pub async fn get_user(&self, user_id: String) -> poem::Result<User> {
        // Business logic mixed with HTTP handling
        Ok(User { id: user_id, name: "John".to_string() })
    }
}
```

**v2 Generated Controller:**
```rust
pub struct UserServiceController<T: UserService> {
    service: T,
}

impl<T: UserService> UserServiceController<T> {
    pub fn new(service: T) -> Self {
        Self { service }
    }
}

#[OpenApi]
impl<T: UserService + Send + Sync> UserServiceController<T> {
    #[oai(path = "/v1/users/{user_id}", method = "get")]
    pub async fn get_user(&self, user_id: Path<String>) -> poem::Result<User> {
        self.service.get_user(user_id.0).await
    }
}
```

**v2 Generated Service Trait:**
```rust
#[async_trait::async_trait]
pub trait UserService {
    async fn get_user(&self, user_id: String) -> poem::Result<User>;
    async fn create_user(&self, request: CreateUserRequest) -> poem::Result<User>;
}
```

### Parameter Handling

**v1:** Manual parameter extraction

**v2:** Automatic parameter extraction with proper types
- `Path<T>` for path parameters
- `Query<T>` for query parameters  
- `Json<T>` for request bodies

## Build Script Migration

### Simple Migration

**v1:**
```rust
use proto_http_parser::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Generator::new()
        .with_config(config)
        .generate_from_file("proto/service.proto", "src/generated")?;
    Ok(())
}
```

**v2:**
```rust
use proto_http_parser_v2::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BuildIntegration::new()
        .configure(|builder| {
            builder
                .preserve_comments(true)
                .generate_service_traits(true)
                .use_dependency_injection(true)
        })
        .add_proto_file("proto/service.proto")
        .output_dir("src/generated")
        .generate()?;
    
    Ok(())
}
```

### Advanced Migration

**v1:**
```rust
let config = GeneratorConfig {
    preserve_comments: true,
    generate_traits: true,
    format_code: true,
    type_mappings: vec![
        ("google.protobuf.Timestamp".to_string(), "chrono::DateTime<Utc>".to_string()),
    ],
};

Generator::new()
    .with_config(config)
    .generate_from_directory("proto", "src/generated")?;
```

**v2:**
```rust
BuildIntegration::new()
    .with_config_file("proto-http-parser.toml")
    .add_proto_directory("proto")?
    .output_dir("src/generated")
    .verbose(true)
    .generate()?;
```

## Common Migration Issues

### Issue 1: Missing Service Implementations

**Problem:** v2 generates service traits that need to be implemented

**Solution:** Create service implementations
```rust
// Create src/services/mod.rs
pub mod user_service;
pub use user_service::UserServiceImpl;

// Create src/services/user_service.rs
use crate::generated::*;

#[derive(Clone)]
pub struct UserServiceImpl;

#[async_trait::async_trait]
impl UserService for UserServiceImpl {
    // Implement all trait methods
}
```

### Issue 2: Configuration Errors

**Problem:** v1 configuration doesn't work in v2

**Solution:** Use ConfigBuilder or configuration files
```rust
// Instead of struct initialization
let config = ConfigBuilder::new()
    .preserve_comments(true)
    .generate_service_traits(true)
    .build()?;
```

### Issue 3: Import Errors

**Problem:** Generated code has missing imports

**Solution:** Add type mappings and imports
```rust
let config = ConfigBuilder::new()
    .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<Utc>")
    .add_import("use chrono::{DateTime, Utc};")
    .build()?;
```

### Issue 4: Build Script Failures

**Problem:** Build script doesn't find proto files

**Solution:** Use proper path handling
```rust
BuildIntegration::new()
    .add_proto_directory("proto")?  // Use ? for error handling
    .output_dir("src/generated")
    .verbose(true)  // Enable debugging
    .generate()?;
```

### Issue 5: Controller Registration

**Problem:** Controllers need service instances in v2

**Solution:** Update server setup
```rust
// v1
let controller = UserController::new();

// v2
let service = UserServiceImpl::new();
let controller = UserServiceController::new(service);
```

## Migration Checklist

- [ ] Update Cargo.toml dependencies
- [ ] Update import statements
- [ ] Migrate build.rs script
- [ ] Create service trait implementations
- [ ] Update controller instantiation
- [ ] Migrate configuration (if using custom config)
- [ ] Update error handling
- [ ] Test generated code compilation
- [ ] Test runtime functionality
- [ ] Update documentation and examples

## Benefits After Migration

After migrating to v2, you'll gain:

1. **Better Separation of Concerns**: HTTP handling separated from business logic
2. **Improved Testability**: Service traits are easy to mock and test
3. **Better Error Messages**: Detailed parse and validation errors
4. **Flexible Configuration**: Multiple configuration sources and formats
5. **Extensibility**: Plugin system for custom functionality
6. **Type Safety**: Stronger type checking and validation
7. **Better Performance**: More efficient parsing with nom
8. **Future-Proof**: Modular architecture supports future enhancements

## Getting Help

If you encounter issues during migration:

1. Check the [examples](../examples/) directory for working code
2. Review the [API Reference](api-reference.md) for detailed documentation
3. Enable verbose output in build scripts for debugging
4. Compare your v1 and v2 configurations side-by-side
5. Test with simple proto files first, then add complexity

The migration effort is worthwhile for the improved architecture, better error handling, and enhanced flexibility that v2 provides.