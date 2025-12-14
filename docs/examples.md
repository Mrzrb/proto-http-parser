# Examples Guide

This document provides a comprehensive guide to all examples included with proto-http-parser-v2.

## Table of Contents

- [Basic Examples](#basic-examples)
- [Configuration Examples](#configuration-examples)
- [Advanced Examples](#advanced-examples)
- [Complete Applications](#complete-applications)
- [Performance Examples](#performance-examples)
- [Running Examples](#running-examples)

## Basic Examples

### 1. Basic Usage (`examples/basic_usage.rs`)

**Purpose**: Demonstrates the fundamental usage of the library with a simple proto file.

**What it shows**:
- Parsing a Protocol Buffer file
- Extracting HTTP routes
- Displaying parsed information
- Basic error handling

**Key concepts**:
- `NomProtoParser` for parsing
- `GoogleApiHttpExtractor` for HTTP route extraction
- Basic proto file structure with HTTP annotations

**Run with**:
```bash
cargo run --example basic_usage
```

### 2. Single Service Example (`examples/single_service_example.rs`)

**Purpose**: Complete workflow for processing a single service with multiple HTTP endpoints.

**What it shows**:
- Full CRUD operations (Create, Read, Update, Delete, List)
- Different HTTP methods (GET, POST, PUT, DELETE)
- Path parameters and query parameters
- Request body handling
- Code generation and file output

**Key concepts**:
- Complete HTTP service definition
- Path parameter extraction (`{user_id}`)
- Request body configuration (`body: "*"`)
- Generated code structure

**Run with**:
```bash
cargo run --example single_service_example
```

### 3. Multi-Service Example (`examples/multi_service_example.rs`)

**Purpose**: Demonstrates batch processing of multiple services in a single proto file.

**What it shows**:
- Multiple services in one proto file
- Batch code generation
- Service organization and module structure
- Cross-service relationships
- Statistics and analysis

**Key concepts**:
- Multi-service architecture
- Batch processing workflows
- Generated code organization
- Module file generation

**Run with**:
```bash
cargo run --example multi_service_example
```

## Configuration Examples

### 4. Configuration Usage (`examples/config_usage.rs`)

**Purpose**: Comprehensive demonstration of the configuration system.

**What it shows**:
- Configuration builder pattern
- Loading from files
- Environment variable configuration
- Configuration merging
- Validation and error handling

**Key concepts**:
- `ConfigBuilder` usage
- TOML configuration files
- Environment variable mapping
- Configuration precedence

**Run with**:
```bash
cargo run --example config_usage
```

### 5. Custom Configuration (`examples/custom_configuration_example.rs`)

**Purpose**: Advanced configuration scenarios and customization options.

**What it shows**:
- Type mappings for Protocol Buffer types
- Custom template usage
- Query parameter inference
- Formatting options
- Performance tuning

**Key concepts**:
- Type mapping (`google.protobuf.Timestamp` → `chrono::DateTime<Utc>`)
- Template customization
- Performance optimization
- Advanced validation settings

**Run with**:
```bash
cargo run --example custom_configuration_example
```

## Advanced Examples

### 6. Coordinator Usage (`examples/coordinator_usage.rs`)

**Purpose**: Using the high-level coordinator for simplified processing.

**What it shows**:
- `ProtoHttpCoordinator` usage
- Simplified API for common tasks
- Error handling and recovery
- Result processing

**Key concepts**:
- High-level API abstraction
- Simplified error handling
- Batch processing with coordinator

**Run with**:
```bash
cargo run --example coordinator_usage
```

### 7. Template Usage (`examples/template_usage.rs`)

**Purpose**: Custom template system usage and customization.

**What it shows**:
- Custom template creation
- Template helper functions
- Template inheritance
- Advanced code generation patterns

**Key concepts**:
- Handlebars template engine
- Custom template helpers
- Template context data
- Code generation customization

**Run with**:
```bash
cargo run --example template_usage
```

### 8. Plugin Usage (`examples/plugin_usage.rs`)

**Purpose**: Extending functionality with the plugin system.

**What it shows**:
- Plugin registration and configuration
- Custom validators
- Plugin lifecycle management
- Error handling in plugins

**Key concepts**:
- Plugin architecture
- Custom validation rules
- Extensibility patterns
- Plugin configuration

**Run with**:
```bash
cargo run --example plugin_usage
```

## Build Integration Examples

### 9. Build Integration (`examples/build_integration.rs`)

**Purpose**: Integration with Rust build scripts (build.rs).

**What it shows**:
- `BuildIntegration` API usage
- Automatic code generation during build
- File watching and regeneration
- Build script best practices

**Key concepts**:
- Build-time code generation
- `cargo:rerun-if-changed` directives
- Output directory management
- Build script error handling

**Run with**:
```bash
cargo run --example build_integration
```

### 10. Sample Build Script (`examples/sample_build.rs`)

**Purpose**: Complete example of a build.rs script.

**What it shows**:
- Real-world build script structure
- Configuration loading in build scripts
- Multiple proto file handling
- Build optimization

**Key concepts**:
- Production build script patterns
- Configuration management in builds
- Performance considerations
- Error handling in builds

**Run with**:
```bash
cargo run --example sample_build
```

## Complete Applications

### 11. Complete Poem Server (`examples/complete_poem_server/`)

**Purpose**: Full working web server using generated controllers.

**What it includes**:
- Complete Cargo project structure
- Protocol Buffer definitions
- Generated controllers and service traits
- Service implementations
- Server setup with poem and poem-openapi
- API documentation with Swagger UI

**Key features**:
- Dependency injection pattern
- Service trait implementations
- HTTP endpoint handling
- OpenAPI documentation
- Error handling and validation

**Run with**:
```bash
cd examples/complete_poem_server
cargo run
```

**Access**:
- Server: http://localhost:3000
- API Docs: http://localhost:3000/docs
- OpenAPI Spec: http://localhost:3000/spec

## Performance Examples

### 12. Performance Example (`examples/performance_example.rs`)

**Purpose**: Performance analysis and optimization techniques.

**What it shows**:
- Performance measurement and profiling
- Configuration optimization for speed
- Memory usage analysis
- Scaling behavior analysis
- Component-level performance breakdown

**Key concepts**:
- Performance benchmarking
- Optimization strategies
- Memory efficiency
- Scaling analysis

**Run with**:
```bash
cargo run --example performance_example
```

### 13. Benchmarks (`benches/parsing_benchmarks.rs`)

**Purpose**: Comprehensive performance benchmarks using Criterion.

**What it measures**:
- Parsing performance
- HTTP extraction performance
- Code generation performance
- End-to-end processing performance
- Memory usage patterns

**Run with**:
```bash
cargo bench
```

## Running Examples

### Prerequisites

Ensure you have the required dependencies:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone <repository-url>
cd proto-http-parser-v2
```

### Running Individual Examples

```bash
# Basic examples
cargo run --example basic_usage
cargo run --example single_service_example
cargo run --example multi_service_example

# Configuration examples
cargo run --example config_usage
cargo run --example custom_configuration_example

# Advanced examples
cargo run --example coordinator_usage
cargo run --example template_usage
cargo run --example plugin_usage

# Build integration examples
cargo run --example build_integration
cargo run --example sample_build

# Performance examples
cargo run --example performance_example
```

### Running the Complete Server Example

```bash
cd examples/complete_poem_server
cargo build  # This runs build.rs to generate code
cargo run    # Start the server
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench parsing
cargo bench code_generation
cargo bench end_to_end

# Generate benchmark report
cargo bench -- --output-format html
```

## Example Output

### Basic Usage Example Output

```
Proto HTTP Parser v2 - Basic Usage Example
Parsing Protocol Buffer content...
✓ Successfully parsed proto file!
  Syntax: Proto3
  Package: Some("example.v1")
  Imports: 1 files
  Services: 1 services
  Messages: 4 messages
  Enums: 1 enums

Service: GreeterService
  Method: SayHello (HelloRequest -> HelloResponse)
    HTTP: POST /v1/hello
    Body: *

Messages:
  HelloRequest: 1 fields
    name: String (field 1)
  HelloResponse: 1 fields
    message: String (field 1)

✓ Parser demonstration complete!
```

### Performance Example Output

```
Proto HTTP Parser v2 - Performance Example
Generated test proto with 2847 lines

=== Example 1: Baseline Performance ===
Baseline processing time: 45.2ms
Generated 50 controllers and 50 traits

=== Example 2: Speed-Optimized Configuration ===
Optimized processing time: 23.1ms
Speed improvement: 1.96x

=== Component Performance Breakdown ===
  Parsing: 12.3ms (27.2%)
  HTTP extraction: 8.7ms (19.2%)
  Code generation: 24.2ms (53.5%)

✓ Performance analysis completed!
```

## Learning Path

### For Beginners

1. Start with `basic_usage.rs` to understand core concepts
2. Try `single_service_example.rs` for a complete workflow
3. Explore `config_usage.rs` for configuration options
4. Run the `complete_poem_server` example for a real application

### For Advanced Users

1. Study `custom_configuration_example.rs` for advanced configuration
2. Explore `template_usage.rs` for code generation customization
3. Try `plugin_usage.rs` for extensibility patterns
4. Analyze `performance_example.rs` for optimization techniques

### For Integration

1. Review `build_integration.rs` for build script patterns
2. Study `sample_build.rs` for production build scripts
3. Examine the `complete_poem_server` project structure
4. Run benchmarks to understand performance characteristics

## Common Use Cases

### API Development

- Use `single_service_example.rs` as a template for REST APIs
- Follow the `complete_poem_server` pattern for full applications
- Configure type mappings for Protocol Buffer well-known types

### Code Generation Pipeline

- Use `build_integration.rs` patterns in your build.rs
- Configure performance optimizations for large proto files
- Set up proper error handling and validation

### Custom Requirements

- Extend with plugins using `plugin_usage.rs` patterns
- Customize templates using `template_usage.rs` examples
- Configure advanced settings with `custom_configuration_example.rs`

## Troubleshooting Examples

If examples fail to run:

1. **Check Rust version**: Ensure Rust 1.70+ is installed
2. **Update dependencies**: Run `cargo update`
3. **Clean build**: Run `cargo clean` then `cargo build`
4. **Check proto files**: Ensure proto files exist and are valid
5. **Enable verbose output**: Add `.verbose(true)` to see detailed logs

## Contributing Examples

To contribute new examples:

1. Create a new file in `examples/`
2. Follow the existing naming convention
3. Include comprehensive comments and documentation
4. Add the example to this guide
5. Test the example thoroughly
6. Submit a pull request

Examples should demonstrate specific features or use cases and include clear explanations of what they show and how to use them.