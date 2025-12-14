# Configuration System

The Proto HTTP Parser v2 library provides a comprehensive configuration system that supports multiple configuration sources and flexible customization options.

## Configuration Sources

The configuration system supports loading configuration from multiple sources with the following precedence (highest to lowest):

1. **Environment Variables** (highest priority)
2. **Configuration Files**
3. **Code-based Configuration**
4. **Default Values** (lowest priority)

## Configuration Structure

The main configuration is organized into several sections:

### Parser Configuration (`parser`)

Controls how Protocol Buffer files are parsed:

```toml
[parser]
preserve_comments = true          # Whether to preserve comments from proto files
strict_validation = true          # Enable strict syntax validation
max_import_depth = 10            # Maximum depth for recursive imports
include_paths = ["./proto", "./vendor/proto"]  # Paths to search for imports
```

### Extractor Configuration (`extractor`)

Controls HTTP annotation extraction:

```toml
[extractor]
infer_query_params = true         # Automatically infer common query parameters
validate_http_methods = true      # Validate HTTP method compatibility
allow_custom_methods = false      # Allow custom HTTP methods beyond standard ones
common_query_params = ["page", "limit", "sort", "filter"]  # Common query parameter names
```

### Generator Configuration (`generator`)

Controls code generation behavior:

```toml
[generator]
generate_service_traits = true   # Generate service trait interfaces
use_dependency_injection = true  # Use dependency injection pattern
target_framework = "PoemOpenApi" # Target framework (currently only PoemOpenApi)
additional_imports = [           # Additional imports to include in generated code
    "use serde::{Deserialize, Serialize};"
]

[generator.type_mappings]        # Custom type mappings
"google.protobuf.Timestamp" = "chrono::DateTime<chrono::Utc>"
"google.protobuf.Duration" = "chrono::Duration"

[generator.formatting]           # Code formatting options
indent_style = "Spaces"          # "Spaces" or "Tabs"
indent_size = 4                  # Number of spaces/tabs for indentation
max_line_length = 100           # Maximum line length
use_rustfmt = true              # Format generated code with rustfmt
```

### Template Configuration (`template`)

Controls template engine behavior:

```toml
[template]
use_builtin_templates = true     # Use built-in templates
template_dir = "./templates"     # Custom template directory (optional)

[template.template_overrides]    # Override specific templates
"controller" = "custom_controller.hbs"

[template.helpers]               # Custom template helpers
snake_case = { helper_type = "SnakeCase" }
```

## Loading Configuration

### Automatic Loading

The easiest way to load configuration is using the automatic loader:

```rust
use proto_http_parser_v2::*;

// Automatically loads from default locations and environment
let config = ProtoHttpParserConfig::load()?;
```

This will search for configuration files in the following order:
1. `proto-http-parser.toml`
2. `.proto-http-parser.toml`
3. `config/proto-http-parser.toml`

### From Specific File

```rust
let config = ProtoHttpParserConfig::from_file("my-config.toml")?;
```

### From Environment Variables

```rust
let config = ProtoHttpParserConfig::from_env()?;
```

### Using Configuration Builder

```rust
let config = ConfigBuilder::new()
    .preserve_comments(true)
    .strict_validation(false)
    .max_import_depth(15)
    .add_include_path("./proto")
    .generate_service_traits(true)
    .use_dependency_injection(true)
    .infer_query_params(true)
    .use_rustfmt(true)
    .indent_size(2)
    .max_line_length(120)
    .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
    .build()?;
```

## Environment Variables

All configuration options can be overridden using environment variables with the `PROTO_HTTP_PARSER_` prefix:

| Environment Variable | Configuration Path | Type | Example |
|---------------------|-------------------|------|---------|
| `PROTO_HTTP_PARSER_PRESERVE_COMMENTS` | `parser.preserve_comments` | boolean | `true` |
| `PROTO_HTTP_PARSER_STRICT_VALIDATION` | `parser.strict_validation` | boolean | `false` |
| `PROTO_HTTP_PARSER_MAX_IMPORT_DEPTH` | `parser.max_import_depth` | integer | `15` |
| `PROTO_HTTP_PARSER_INCLUDE_PATHS` | `parser.include_paths` | colon-separated paths | `./proto:./vendor` |
| `PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS` | `generator.generate_service_traits` | boolean | `true` |
| `PROTO_HTTP_PARSER_USE_DEPENDENCY_INJECTION` | `generator.use_dependency_injection` | boolean | `false` |
| `PROTO_HTTP_PARSER_INFER_QUERY_PARAMS` | `extractor.infer_query_params` | boolean | `true` |
| `PROTO_HTTP_PARSER_VALIDATE_HTTP_METHODS` | `extractor.validate_http_methods` | boolean | `false` |
| `PROTO_HTTP_PARSER_USE_RUSTFMT` | `generator.formatting.use_rustfmt` | boolean | `true` |

## Configuration Validation

The configuration system includes comprehensive validation:

### Parser Validation
- `max_import_depth` must be between 1 and 100
- All `include_paths` must exist

### Generator Validation
- `indent_size` must be greater than 0 and not exceed 8
- `max_line_length` must be at least 50 characters

### Example Validation Errors

```rust
// This will fail validation
let invalid_config = ConfigBuilder::new()
    .max_import_depth(0)  // Error: must be > 0
    .build();

match invalid_config {
    Err(ConfigError::ValidationError { field, message }) => {
        println!("Validation failed for {}: {}", field, message);
    }
    _ => {}
}
```

## Build.rs Integration

The configuration system integrates seamlessly with build.rs scripts:

### Basic Usage

```rust
// build.rs
use proto_http_parser_v2::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BuildIntegration::new()
        .add_proto_file("proto/user.proto")
        .output_dir("src/generated")
        .generate()?;
    
    Ok(())
}
```

### With Configuration File

```rust
// build.rs
use proto_http_parser_v2::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BuildIntegration::new()
        .with_config_file("build-config.toml")
        .add_proto_directory("proto")?
        .output_dir("src/generated")
        .verbose(true)
        .generate()?;
    
    Ok(())
}
```

### With Environment Variables

```rust
// build.rs
use proto_http_parser_v2::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BuildIntegration::new()
        .with_env_config()
        .add_proto_glob("proto/*.proto")?
        .output_dir_from_env("generated")  // Uses OUT_DIR/generated
        .generate()?;
    
    Ok(())
}
```

### With Custom Configuration

```rust
// build.rs
use proto_http_parser_v2::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BuildIntegration::new()
        .configure(|builder| {
            builder
                .preserve_comments(false)
                .generate_service_traits(true)
                .use_dependency_injection(true)
                .use_rustfmt(false)
                .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
        })
        .add_proto_directory("proto")?
        .output_dir("src/generated")
        .verbose(true)
        .generate()?;
    
    Ok(())
}
```

### Auto-Configuration

```rust
// build.rs
use proto_http_parser_v2::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Automatically loads configuration from files and environment
    BuildIntegration::with_auto_config()?
        .add_proto_directory("proto")?
        .output_dir("src/generated")
        .generate()?;
    
    Ok(())
}
```

## Configuration Merging

Configurations can be merged from multiple sources:

```rust
let base_config = ConfigBuilder::new()
    .preserve_comments(true)
    .max_import_depth(5)
    .build()?;

let file_config = ProtoHttpParserConfig::from_file("override.toml")?;

let mut final_config = base_config;
final_config.merge(file_config);  // file_config takes precedence
```

## Best Practices

### 1. Use Configuration Files for Project Settings

Create a `proto-http-parser.toml` file in your project root:

```toml
[parser]
include_paths = ["./proto", "./third_party/googleapis"]
preserve_comments = true

[generator]
generate_service_traits = true
use_dependency_injection = true

[generator.formatting]
indent_size = 4
max_line_length = 100
```

### 2. Use Environment Variables for CI/CD

Set environment variables in your CI/CD pipeline:

```bash
export PROTO_HTTP_PARSER_USE_RUSTFMT=false
export PROTO_HTTP_PARSER_STRICT_VALIDATION=true
```

### 3. Use Builder Pattern for Programmatic Configuration

```rust
let config = ConfigBuilder::new()
    .preserve_comments(cfg!(debug_assertions))  // Only in debug builds
    .use_rustfmt(!cfg!(feature = "fast-build")) // Skip rustfmt for fast builds
    .build()?;
```

### 4. Validate Configuration Early

```rust
// In build.rs or main application
let config = ProtoHttpParserConfig::load()?;
config.validate()?;  // Fail fast if configuration is invalid
```

### 5. Use Type Mappings for Better Integration

```toml
[generator.type_mappings]
"google.protobuf.Timestamp" = "chrono::DateTime<chrono::Utc>"
"google.protobuf.Duration" = "std::time::Duration"
"google.protobuf.Any" = "serde_json::Value"
```

## Error Handling

The configuration system provides detailed error messages:

```rust
match ProtoHttpParserConfig::from_file("config.toml") {
    Ok(config) => { /* use config */ }
    Err(ConfigError::FileError { path, error }) => {
        eprintln!("Failed to read config file {}: {}", path.display(), error);
    }
    Err(ConfigError::ParseError { source, error }) => {
        eprintln!("Failed to parse config from {}: {}", source, error);
    }
    Err(ConfigError::ValidationError { field, message }) => {
        eprintln!("Invalid configuration for {}: {}", field, message);
    }
    Err(e) => {
        eprintln!("Configuration error: {}", e);
    }
}
```

## Migration from v1

If you're migrating from v1, here's how the configuration maps:

| v1 Configuration | v2 Configuration |
|-----------------|------------------|
| `GeneratorConfig::preserve_comments` | `parser.preserve_comments` |
| `GeneratorConfig::generate_traits` | `generator.generate_service_traits` |
| `GeneratorConfig::use_dependency_injection` | `generator.use_dependency_injection` |
| `GeneratorConfig::format_code` | `generator.formatting.use_rustfmt` |

Example migration:

```rust
// v1
let config = GeneratorConfig {
    preserve_comments: true,
    generate_traits: true,
    use_dependency_injection: true,
    format_code: true,
    ..Default::default()
};

// v2
let config = ConfigBuilder::new()
    .preserve_comments(true)
    .generate_service_traits(true)
    .use_dependency_injection(true)
    .use_rustfmt(true)
    .build()?;
```