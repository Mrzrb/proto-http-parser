# Proto HTTP Parser v2

A complete rewrite of the Protocol Buffer HTTP annotation parser with a modular architecture for generating high-quality poem-openapi controller code.

## Features

- **Powerful Parsing**: Uses nom parser combinators for robust proto file parsing
- **HTTP Annotation Support**: Full support for google.api.http annotations
- **Code Generation**: Generates poem-openapi controllers with dependency injection
- **Template System**: Flexible Handlebars-based template engine
- **Comprehensive Validation**: Detailed error reporting and validation
- **Extensible Architecture**: Plugin system for custom extensions
- **Service Traits**: Clean separation between HTTP handling and business logic
- **Type Safety**: Full type safety from proto definitions to HTTP handlers
- **Build Integration**: Seamless integration with Rust build scripts

## Quick Start

### 1. Add Dependency

```toml
[dependencies]
proto-http-parser-v2 = "2.0.0"

[build-dependencies]
proto-http-parser-v2 = "2.0.0"
```

### 2. Create Proto File

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

### 3. Generate Code in build.rs

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

### 4. Implement Service

```rust
// src/main.rs
mod generated;
use generated::*;

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
    
    let api_service = poem_openapi::OpenApiService::new(controller, "Hello API", "1.0.0");
    
    let app = poem::Route::new()
        .nest("/api", api_service)
        .nest("/docs", api_service.swagger_ui());
    
    poem::Server::new(poem::listener::TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await?;
    
    Ok(())
}
```

## Architecture

The library is built around core traits that provide clean separation of concerns:

- **`ProtoParser`** - Parses Protocol Buffer files using nom combinators
- **`HttpAnnotationExtractor`** - Extracts and validates HTTP annotations
- **`CodeGenerator`** - Generates poem-openapi controllers and service traits
- **`TemplateEngine`** - Renders customizable code templates
- **`ProtoHttpCoordinator`** - High-level API that orchestrates the entire pipeline

## Documentation

- **[User Guide](docs/user-guide.md)** - Comprehensive usage guide
- **[API Reference](docs/api-reference.md)** - Complete API documentation
- **[Configuration Guide](docs/configuration.md)** - Configuration system documentation
- **[Migration Guide](docs/migration-guide.md)** - Migration from v1 to v2
- **[Examples Guide](docs/examples.md)** - All examples with explanations
- **[Plugin System](docs/plugin-system.md)** - Extensibility and plugins

## Examples

### Basic Examples

- **[Basic Usage](examples/basic_usage.rs)** - Simple parsing and code generation
- **[Single Service](examples/single_service_example.rs)** - Complete CRUD service example
- **[Multi Service](examples/multi_service_example.rs)** - Multiple services in one proto file

### Configuration Examples

- **[Configuration Usage](examples/config_usage.rs)** - Configuration system demonstration
- **[Custom Configuration](examples/custom_configuration_example.rs)** - Advanced configuration options

### Advanced Examples

- **[Template Usage](examples/template_usage.rs)** - Custom template system
- **[Plugin Usage](examples/plugin_usage.rs)** - Plugin system and extensions
- **[Performance Example](examples/performance_example.rs)** - Performance analysis and optimization

### Complete Applications

- **[Complete Poem Server](examples/complete_poem_server/)** - Full working web server with:
  - Generated controllers and service traits
  - Dependency injection pattern
  - OpenAPI documentation
  - Service implementations
  - Build script integration

Run the complete server example:

```bash
cd examples/complete_poem_server
cargo run
# Visit http://localhost:3000/docs for API documentation
```

## Key Features

### Generated Code Structure

The library generates two main components:

1. **Controllers**: HTTP request handlers with poem-openapi integration
2. **Service Traits**: Business logic interfaces for clean separation of concerns

```rust
// Generated controller (handles HTTP)
pub struct UserServiceController<T: UserService> {
    service: T,
}

// Generated service trait (business logic interface)
#[async_trait::async_trait]
pub trait UserService {
    async fn get_user(&self, user_id: String) -> poem::Result<User>;
    async fn create_user(&self, request: CreateUserRequest) -> poem::Result<User>;
}
```

### Dependency Injection

Controllers use dependency injection to separate HTTP handling from business logic:

```rust
// Your service implementation
struct UserServiceImpl { /* ... */ }

#[async_trait::async_trait]
impl UserService for UserServiceImpl {
    // Implement business logic
}

// Inject service into controller
let service = UserServiceImpl::new();
let controller = UserServiceController::new(service);
```

### Type Safety

Full type safety from Protocol Buffer definitions to HTTP handlers:

- Path parameters: `Path<String>`
- Query parameters: `Query<QueryStruct>`
- Request bodies: `Json<RequestType>`
- Responses: Proper HTTP status codes and types

### Configuration System

Flexible configuration supporting multiple sources:

```rust
// Programmatic configuration
let config = ConfigBuilder::new()
    .generate_service_traits(true)
    .use_dependency_injection(true)
    .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<Utc>")
    .build()?;

// File-based configuration (proto-http-parser.toml)
let config = ProtoHttpParserConfig::load()?;

// Environment variable configuration
let config = ProtoHttpParserConfig::from_env()?;
```

## Performance

The library is designed for performance:

- **Efficient Parsing**: nom parser combinators for fast, memory-efficient parsing
- **Build-time Generation**: Code generation happens at build time, not runtime
- **Configurable Optimization**: Disable features like rustfmt and comment preservation for faster builds
- **Parallel Processing**: Safe to process multiple files in parallel

Run benchmarks:

```bash
cargo bench
```

## Plugin System

Extend functionality with plugins:

```rust
// Register custom validators
coordinator.plugin_manager_mut()
    .register_proto_validator(MyValidator::new(), config)?;

// Custom code generators
coordinator.plugin_manager_mut()
    .register_code_generator(MyGenerator::new(), config)?;
```

## Migration from v1

See the [Migration Guide](docs/migration-guide.md) for detailed migration instructions from proto-http-parser v1.

Key changes in v2:
- Modular architecture with trait-based design
- Dependency injection pattern for generated controllers
- Separate service trait generation
- Comprehensive configuration system
- Plugin system for extensibility
- Better error handling and validation

## Contributing

Contributions are welcome! Please see our contributing guidelines and:

1. Check existing examples and documentation
2. Add tests for new features
3. Update documentation as needed
4. Follow the existing code style

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Built with [nom](https://github.com/Geal/nom) for parsing
- Uses [Handlebars](https://github.com/sunng87/handlebars-rust) for templating
- Integrates with [poem](https://github.com/poem-web/poem) and [poem-openapi](https://github.com/poem-web/poem)
- Inspired by Protocol Buffer HTTP annotations and gRPC-Gateway