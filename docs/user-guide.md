# User Guide

This guide provides step-by-step instructions for using proto-http-parser-v2 to generate poem-openapi controllers from Protocol Buffer files.

## Table of Contents

- [Getting Started](#getting-started)
- [Basic Usage](#basic-usage)
- [Configuration](#configuration)
- [Build Integration](#build-integration)
- [Generated Code](#generated-code)
- [Advanced Features](#advanced-features)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Getting Started

### Installation

Add proto-http-parser-v2 to your `Cargo.toml`:

```toml
[dependencies]
proto-http-parser-v2 = "2.0.0"

# For build.rs integration
[build-dependencies]
proto-http-parser-v2 = "2.0.0"
```

### Prerequisites

- Rust 1.70 or later
- Protocol Buffer files with `google.api.http` annotations
- Basic familiarity with poem and poem-openapi

### Quick Start

1. **Create a proto file** with HTTP annotations:

```protobuf
syntax = "proto3";

package hello.v1;

import "google/api/annotations.proto";

service GreeterService {
    rpc SayHello(HelloRequest) returns (HelloResponse) {
        option (google.api.http) = {
            post: "/v1/hello"
            body: "*"
        };
    }
}

message HelloRequest {
    string name = 1;
}

message HelloResponse {
    string message = 1;
}
```

2. **Generate controllers** in your build.rs:

```rust
// build.rs
use proto_http_parser::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BuildIntegration::new()
        .add_proto_file("proto/hello.proto")
        .output_dir("src/generated")
        .generate()?;
    
    Ok(())
}
```

3. **Use the generated code** in your application:

```rust
// src/main.rs
mod generated;

use generated::*;
use poem::{listener::TcpListener, Route, Server};
use poem_openapi::OpenApiService;

#[derive(Default)]
struct GreeterServiceImpl;

#[async_trait::async_trait]
impl GreeterService for GreeterServiceImpl {
    async fn say_hello(&self, request: HelloRequest) -> poem::Result<HelloResponse> {
        Ok(HelloResponse {
            message: format!("Hello, {}!", request.name),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = GreeterServiceImpl::default();
    let controller = GreeterServiceController::new(service);
    
    let api_service = OpenApiService::new(controller, "Hello API", "1.0.0");
    
    let app = Route::new()
        .nest("/api", api_service)
        .nest("/docs", api_service.swagger_ui());
    
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await?;
    
    Ok(())
}
```

## Basic Usage

### Parsing Proto Files

```rust
use proto_http_parser::*;

// Create a parser
let parser = NomProtoParser::new();

// Parse from file
let proto_file = parser.parse_file("service.proto")?;

// Parse from string content
let proto_content = r#"
syntax = "proto3";
service MyService {
    rpc GetData(Request) returns (Response);
}
message Request {}
message Response {}
"#;
let proto_file = parser.parse_content(proto_content)?;
```

### Extracting HTTP Routes

```rust
use proto_http_parser::*;

// Create an extractor
let extractor = GoogleApiHttpExtractor::new();

// Extract routes from parsed proto file
let routes = extractor.extract_routes(&proto_file)?;

// Examine the routes
for route in &routes {
    println!("Route: {} {} -> {}::{}", 
        route.http_method.as_str(),
        route.path_template,
        route.service_name,
        route.method_name
    );
}
```

### Generating Code

```rust
use proto_http_parser::*;

// Create a generator
let generator = PoemOpenApiGenerator::new();

// Generate for each service
for service in &proto_file.services {
    // Filter routes for this service
    let service_routes: Vec<_> = routes.iter()
        .filter(|r| r.service_name == service.name)
        .cloned()
        .collect();
    
    // Generate controller
    let controller = generator.generate_controller(service, &service_routes)?;
    
    // Generate service trait
    let service_trait = generator.generate_service_trait(service, &service_routes)?;
    
    // Save to files
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

### Using the Coordinator

For simpler usage, use the `ProtoHttpCoordinator`:

```rust
use proto_http_parser::*;

// Create coordinator with default configuration
let coordinator = ProtoHttpCoordinator::new();

// Process a proto file
let result = coordinator.process_file("service.proto")?;

// Access generated code
for (service_name, controller) in result.controllers {
    println!("Generated controller for {}", service_name);
    std::fs::write(
        format!("{}_controller.rs", service_name.to_lowercase()),
        controller.content
    )?;
}

for (service_name, trait_code) in result.service_traits {
    println!("Generated trait for {}", service_name);
    std::fs::write(
        format!("{}_service.rs", service_name.to_lowercase()),
        trait_code.content
    )?;
}
```

## Configuration

### Using ConfigBuilder

```rust
use proto_http_parser::*;

let config = ConfigBuilder::new()
    // Parser settings
    .preserve_comments(true)
    .strict_validation(true)
    .max_import_depth(10)
    
    // Generator settings
    .generate_service_traits(true)
    .use_dependency_injection(true)
    .infer_query_params(true)
    
    // Formatting settings
    .use_rustfmt(true)
    .indent_size(4)
    .max_line_length(100)
    
    // Type mappings
    .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
    .add_type_mapping("google.protobuf.Duration", "std::time::Duration")
    
    // Additional imports
    .add_import("use chrono::{DateTime, Utc};")
    .add_import("use serde::{Deserialize, Serialize};")
    
    .build()?;

// Use with coordinator
let coordinator = ProtoHttpCoordinator::with_config(config);
```

### Configuration Files

Create a `proto-http-parser.toml` file:

```toml
[parser]
preserve_comments = true
strict_validation = true
max_import_depth = 10
include_paths = ["./proto", "./third_party/googleapis"]

[extractor]
infer_query_params = true
validate_http_methods = true
allow_custom_methods = false
common_query_params = ["page", "limit", "sort", "filter"]

[generator]
generate_service_traits = true
use_dependency_injection = true
target_framework = "PoemOpenApi"
additional_imports = [
    "use chrono::{DateTime, Utc};",
    "use serde::{Deserialize, Serialize};"
]

[generator.type_mappings]
"google.protobuf.Timestamp" = "chrono::DateTime<chrono::Utc>"
"google.protobuf.Duration" = "std::time::Duration"

[generator.formatting]
indent_style = "Spaces"
indent_size = 4
max_line_length = 100
use_rustfmt = true

[template]
use_builtin_templates = true
```

Load the configuration:

```rust
use proto_http_parser::*;

// Load from specific file
let config = ProtoHttpParserConfig::from_file("my-config.toml")?;

// Auto-load from standard locations
let config = ProtoHttpParserConfig::load()?;

// Use with coordinator
let coordinator = ProtoHttpCoordinator::with_config(config);
```

### Environment Variables

Configure using environment variables:

```bash
export PROTO_HTTP_PARSER_PRESERVE_COMMENTS=true
export PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS=true
export PROTO_HTTP_PARSER_USE_DEPENDENCY_INJECTION=true
export PROTO_HTTP_PARSER_INFER_QUERY_PARAMS=true
export PROTO_HTTP_PARSER_USE_RUSTFMT=true
```

```rust
use proto_http_parser::*;

let config = ProtoHttpParserConfig::from_env()?;
let coordinator = ProtoHttpCoordinator::with_config(config);
```

## Build Integration

### Basic Build Script

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

### Advanced Build Script

```rust
// build.rs
use proto_http_parser::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BuildIntegration::new()
        // Configuration
        .configure(|builder| {
            builder
                .preserve_comments(true)
                .generate_service_traits(true)
                .use_dependency_injection(true)
                .infer_query_params(true)
                .use_rustfmt(true)
                .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
        })
        // Input files
        .add_proto_directory("proto")?
        .add_proto_glob("schemas/**/*.proto")?
        // Output
        .output_dir("src/generated")
        .verbose(true)
        .generate()?;
    
    println!("cargo:rerun-if-changed=proto/");
    println!("cargo:rerun-if-changed=schemas/");
    
    Ok(())
}
```

### With Configuration File

```rust
// build.rs
use proto_http_parser::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BuildIntegration::new()
        .with_config_file("build-config.toml")
        .add_proto_directory("proto")?
        .output_dir("src/generated")
        .generate()?;
    
    Ok(())
}
```

## Generated Code

### Controller Structure

Generated controllers follow this pattern:

```rust
use poem_openapi::{OpenApi, ApiResponse, Object};
use poem::{Result, web::Path, web::Query, web::Json};

#[derive(Default)]
pub struct MyServiceController<T: MyServiceService> {
    service: T,
}

impl<T: MyServiceService> MyServiceController<T> {
    pub fn new(service: T) -> Self {
        Self { service }
    }
}

#[OpenApi]
impl<T: MyServiceService + Send + Sync> MyServiceController<T> {
    #[oai(path = "/v1/items/{item_id}", method = "get")]
    pub async fn get_item(
        &self,
        item_id: Path<String>,
    ) -> Result<GetItemResponse> {
        self.service.get_item(item_id.0).await
    }
    
    #[oai(path = "/v1/items", method = "post")]
    pub async fn create_item(
        &self,
        body: Json<CreateItemRequest>,
    ) -> Result<Item> {
        self.service.create_item(body.0).await
    }
}
```

### Service Trait Structure

Generated service traits follow this pattern:

```rust
use poem::Result;

#[async_trait::async_trait]
pub trait MyServiceService {
    async fn get_item(&self, item_id: String) -> Result<GetItemResponse>;
    async fn create_item(&self, request: CreateItemRequest) -> Result<Item>;
}
```

### Parameter Handling

The generator automatically handles different parameter types:

- **Path Parameters**: `Path<T>` for URL path segments
- **Query Parameters**: `Query<QueryStruct>` for query strings
- **Request Body**: `Json<T>` for JSON request bodies
- **Headers**: Custom header extraction (if configured)

### Type Mappings

Common Protocol Buffer types are mapped to Rust types:

| Proto Type | Default Rust Type | With chrono mapping |
|------------|-------------------|-------------------|
| `string` | `String` | `String` |
| `int32` | `i32` | `i32` |
| `int64` | `i64` | `i64` |
| `double` | `f64` | `f64` |
| `bool` | `bool` | `bool` |
| `google.protobuf.Timestamp` | `prost_types::Timestamp` | `chrono::DateTime<Utc>` |
| `google.protobuf.Duration` | `prost_types::Duration` | `std::time::Duration` |

## Advanced Features

### Custom Templates

You can provide custom templates for code generation:

```rust
let custom_template = r#"
// Custom controller template
use poem_openapi::OpenApi;

pub struct {{service_name}}Controller {
    // Custom implementation
}

#[OpenApi]
impl {{service_name}}Controller {
{{#each methods}}
    #[oai(path = "{{path}}", method = "{{http_method}}")]
    pub async fn {{snake_case name}}(&self) -> poem::Result<String> {
        Ok("Custom response".to_string())
    }
{{/each}}
}
"#;

let config = ConfigBuilder::new()
    .use_builtin_templates(false)
    .add_custom_template("controller", custom_template)
    .build()?;
```

### Plugin System

Extend functionality with plugins:

```rust
use proto_http_parser::plugins::*;

let mut coordinator = ProtoHttpCoordinator::new();

// Add a naming convention validator
let validator = NamingConventionValidator::new();
let plugin_config = PluginConfigBuilder::new()
    .enabled(true)
    .setting("service_pattern", "^[A-Z][a-zA-Z0-9]*Service$")
    .build();

coordinator.plugin_manager_mut()
    .register_proto_validator(validator, plugin_config)?;
```

### Batch Processing

Process multiple proto files efficiently:

```rust
use proto_http_parser::*;
use std::path::Path;

let coordinator = ProtoHttpCoordinator::new();
let proto_files = vec![
    "proto/user.proto",
    "proto/product.proto",
    "proto/order.proto",
];

for proto_file in proto_files {
    let result = coordinator.process_file(proto_file)?;
    
    // Save generated code
    for (service_name, controller) in result.controllers {
        let filename = format!("src/generated/{}_controller.rs", 
            service_name.to_lowercase());
        std::fs::write(filename, controller.content)?;
    }
}
```

## Best Practices

### Project Organization

```
my-project/
├── proto/                     # Protocol Buffer definitions
│   ├── user.proto
│   ├── product.proto
│   └── google/               # Third-party proto files
│       └── api/
├── src/
│   ├── generated/            # Generated controllers (by build.rs)
│   ├── services/             # Service implementations
│   │   ├── user_service.rs
│   │   └── product_service.rs
│   └── main.rs
├── build.rs                  # Code generation
├── proto-http-parser.toml    # Configuration
└── Cargo.toml
```

### Configuration Management

1. **Use configuration files** for project-specific settings
2. **Use environment variables** for deployment-specific settings
3. **Use builder pattern** for programmatic configuration

### Error Handling

```rust
use proto_http_parser::*;

match coordinator.process_file("service.proto") {
    Ok(result) => {
        // Handle success
    }
    Err(ProtoHttpParserError::Parse(parse_error)) => {
        eprintln!("Parse error: {}", parse_error);
        // Handle parse errors specifically
    }
    Err(ProtoHttpParserError::Validation(validation_error)) => {
        eprintln!("Validation error: {}", validation_error);
        // Handle validation errors specifically
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

### Service Implementation

```rust
use poem::Result;
use crate::generated::*;

#[derive(Clone)]
pub struct UserServiceImpl {
    // Your service state
}

#[async_trait::async_trait]
impl UserService for UserServiceImpl {
    async fn get_user(&self, user_id: String) -> Result<User> {
        // Implement your business logic
        // Return appropriate errors using poem::Error
        
        if user_id.is_empty() {
            return Err(poem::Error::from_string(
                "User ID is required",
                poem::http::StatusCode::BAD_REQUEST,
            ));
        }
        
        // Your implementation here
        Ok(User {
            id: user_id,
            name: "John Doe".to_string(),
            // ... other fields
        })
    }
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use poem::test::TestClient;
    
    #[tokio::test]
    async fn test_get_user() {
        let service = UserServiceImpl::new();
        let controller = UserServiceController::new(service);
        
        let app = poem::Route::new().nest("/api", controller);
        let client = TestClient::new(app);
        
        let response = client.get("/api/v1/users/123").send().await;
        response.assert_status_is_ok();
        
        let user: User = response.json().await.value().deserialize();
        assert_eq!(user.id, "123");
    }
}
```

## Troubleshooting

### Common Issues

#### Build Errors

**Problem**: Build fails with "proto file not found"
```
error: proto file not found: service.proto
```

**Solution**: Check file paths and ensure proto files exist
```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Add debug output
    println!("cargo:warning=Looking for proto files in: {:?}", 
        std::env::current_dir()?);
    
    BuildIntegration::new()
        .add_proto_file("proto/service.proto")  // Check this path
        .verbose(true)  // Enable verbose output
        .generate()?;
    
    Ok(())
}
```

#### Parse Errors

**Problem**: Parse error with HTTP annotations
```
Parse error: Syntax error at line 15, column 20: unexpected token 'custom'
```

**Solution**: Check HTTP annotation syntax
```protobuf
// Correct syntax
option (google.api.http) = {
    get: "/v1/users/{user_id}"
};

// Incorrect syntax
option (google.api.http) = {
    custom: "/v1/users/{user_id}"  // 'custom' is not a valid HTTP method
};
```

#### Import Issues

**Problem**: Import not found errors
```
Import not found: google/api/annotations.proto
```

**Solution**: Add include paths for third-party proto files
```rust
let config = ConfigBuilder::new()
    .add_include_path("./third_party/googleapis")
    .build()?;
```

#### Generated Code Issues

**Problem**: Generated code doesn't compile
```
error[E0433]: failed to resolve: use of undeclared type `DateTime`
```

**Solution**: Add required type mappings and imports
```rust
let config = ConfigBuilder::new()
    .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
    .add_import("use chrono::{DateTime, Utc};")
    .build()?;
```

### Debug Mode

Enable verbose output for debugging:

```rust
// build.rs
BuildIntegration::new()
    .verbose(true)
    .generate()?;
```

Set environment variable for detailed logging:
```bash
export PROTO_HTTP_PARSER_LOG=debug
```

### Getting Help

1. **Check the examples** in the `examples/` directory
2. **Review the API documentation** for detailed method signatures
3. **Enable verbose output** to see what the library is doing
4. **Check configuration** - many issues are configuration-related
5. **Validate proto files** independently using `protoc`

### Performance Tips

1. **Use build.rs** for code generation instead of runtime generation
2. **Limit import depth** to avoid deep recursion
3. **Disable rustfmt** during development for faster builds
4. **Use specific proto files** instead of glob patterns when possible

```rust
let config = ConfigBuilder::new()
    .max_import_depth(5)        // Limit recursion
    .use_rustfmt(false)         // Faster builds
    .preserve_comments(false)   // Skip comment processing
    .build()?;
```