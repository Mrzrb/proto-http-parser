# Proto HTTP Parser v2 - Project Structure

## Overview

This document describes the project structure and organization of Proto HTTP Parser v2.

## Directory Structure

```
proto-http-parser-v2/
├── src/                    # Main library source code
│   ├── lib.rs             # Library entry point and re-exports
│   ├── core.rs            # Core trait definitions
│   │   ├── data.rs        # Data structures (ProtoFile, Service, etc.)
│   │   ├── config.rs      # Configuration types
│   │   └── errors.rs      # Error types and handling
│   ├── parser.rs          # nom-based Protocol Buffer parser
│   ├── extractor.rs       # HTTP annotation extractor
│   ├── generator.rs       # Code generator for poem-openapi
│   ├── templates.rs       # Handlebars template engine
│   ├── config.rs          # Configuration re-exports
│   ├── errors.rs          # Error re-exports
│   └── utils.rs           # Utility functions
├── examples/              # Usage examples
│   └── basic_usage.rs     # Basic usage demonstration
├── Cargo.toml            # Package configuration
├── README.md             # Project documentation
└── STRUCTURE.md          # This file
```

## Core Components

### 1. Core Traits (`src/core.rs`)

Defines the fundamental abstractions:

- `ProtoParser` - Protocol Buffer file parsing
- `HttpAnnotationExtractor` - HTTP annotation extraction
- `CodeGenerator` - Code generation
- `TemplateEngine` - Template rendering
- `TemplateHelper` - Template helper functions

### 2. Data Structures (`src/core/data.rs`)

Core data types representing parsed Protocol Buffer content:

- `ProtoFile` - Complete proto file representation
- `Service` - Service definition
- `RpcMethod` - RPC method with HTTP annotations
- `HttpRoute` - Structured HTTP route information
- `Message`, `Field`, `Enum` - Protocol Buffer type definitions

### 3. Configuration (`src/core/config.rs`)

Configuration types for all components:

- `ProtoHttpParserConfig` - Main configuration
- `ParserConfig` - Parser-specific settings
- `ExtractorConfig` - HTTP extraction settings
- `GeneratorConfig` - Code generation settings
- `TemplateConfig` - Template engine settings

### 4. Error Handling (`src/core/errors.rs`)

Comprehensive error types:

- `ProtoHttpParserError` - Main error type
- `ParseError` - Parsing errors
- `ValidationError` - Validation errors
- `CodeGenerationError` - Code generation errors
- `TemplateError` - Template errors

## Implementation Status

### ✅ Completed

- [x] Project structure and core interfaces
- [x] Core trait definitions
- [x] Data structure definitions
- [x] Configuration system
- [x] Error handling system
- [x] Utility functions
- [x] Basic project setup
- [x] Unit tests for utilities
- [x] Documentation and examples

### 🚧 In Progress

- [ ] nom-based Protocol Buffer parser implementation
- [ ] HTTP annotation extraction logic
- [ ] Code generation templates
- [ ] Template engine helpers

### 📋 Planned

- [ ] Property-based testing
- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] Plugin system
- [ ] Advanced examples

## Dependencies

### Core Dependencies

- `nom` - Parser combinators for Protocol Buffer parsing
- `handlebars` - Template engine for code generation
- `thiserror` - Error handling and custom error types
- `serde` - Serialization support
- `regex` - Regular expression support
- `chrono` - Date/time handling
- `toml` - Configuration file support

### Development Dependencies

- `proptest` - Property-based testing
- `tempfile` - Temporary file handling for tests
- `tokio` - Async runtime for testing
- `criterion` - Benchmarking framework
- `pretty_assertions` - Enhanced test assertions

## Usage

See `examples/basic_usage.rs` for a complete usage example.

```rust
use proto_http_parser_v2::*;

// Create components
let parser = NomProtoParser::new();
let extractor = GoogleApiHttpExtractor::new();
let generator = PoemOpenApiGenerator::new();

// Use the library...
```

## Testing

Run all tests:

```bash
cargo test
```

Run specific test module:

```bash
cargo test utils
```

Run examples:

```bash
cargo run --example basic_usage
```

## Building

Build the library:

```bash
cargo build
```

Build with optimizations:

```bash
cargo build --release
```

Check code without building:

```bash
cargo check
```