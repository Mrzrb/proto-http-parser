# Plugin System Guide

The proto-http-parser-v2 library includes a flexible plugin system that allows you to extend its functionality with custom validators, code generators, formatters, and other extensions.

## Overview

The plugin system provides several extension points:

- **Proto Validators**: Validate Protocol Buffer files for custom rules
- **HTTP Validators**: Validate HTTP routes against custom patterns
- **Code Generators**: Generate custom code from proto services
- **Code Formatters**: Apply custom formatting rules to generated code
- **Error Reporters**: Customize error messages and suggestions

## Quick Start

### 1. Register a Plugin

```rust
use proto_http_parser::*;
use proto_http_parser::plugins::*;
use proto_http_parser::plugins::examples::*;

let mut coordinator = ProtoHttpCoordinator::new();

// Register a naming convention validator
let validator = NamingConventionValidator::new();
let config = PluginConfigBuilder::new()
    .enabled(true)
    .priority(10)
    .setting("service_pattern", "^[A-Z][a-zA-Z0-9]*Service$")
    .build();

coordinator.plugin_manager_mut()
    .register_proto_validator(validator, config)?;
```

### 2. Use Plugin Configuration Files

Create a `plugin_config.json` file:

```json
{
  "naming_convention_validator": {
    "enabled": true,
    "priority": 10,
    "settings": {
      "service_pattern": "^[A-Z][a-zA-Z0-9]*Service$",
      "method_pattern": "^[A-Z][a-zA-Z0-9]*$",
      "message_pattern": "^[A-Z][a-zA-Z0-9]*$"
    }
  },
  "rest_api_validator": {
    "enabled": true,
    "priority": 5,
    "settings": {
      "require_resource_paths": true,
      "allow_nested_resources": false,
      "max_path_depth": 3
    }
  }
}
```

Load the configuration:

```rust
coordinator.load_plugins_from_config("plugin_config.json")?;
```

## Built-in Example Plugins

### Naming Convention Validator

Validates that services, methods, and messages follow naming conventions:

```rust
let validator = NamingConventionValidator::new();
let config = PluginConfigBuilder::new()
    .setting("service_pattern", "^[A-Z][a-zA-Z0-9]*Service$")
    .setting("method_pattern", "^[A-Z][a-zA-Z0-9]*$")
    .setting("message_pattern", "^[A-Z][a-zA-Z0-9]*$")
    .build();
```

### REST API Validator

Validates HTTP routes against REST API best practices:

```rust
let validator = RestApiValidator::new();
let config = PluginConfigBuilder::new()
    .bool_setting("require_resource_paths", true)
    .bool_setting("allow_nested_resources", false)
    .number_setting("max_path_depth", 3)
    .build();
```

### Custom Code Formatter

Applies custom formatting rules to generated code:

```rust
let formatter = CustomCodeFormatter::new();
let config = PluginConfigBuilder::new()
    .number_setting("indent_size", 2)
    .bool_setting("use_tabs", false)
    .number_setting("max_line_length", 120)
    .build();
```

### Documentation Generator

Generates API documentation from proto files:

```rust
let generator = DocumentationGenerator::new();
let config = PluginConfigBuilder::new()
    .bool_setting("include_examples", true)
    .setting("output_format", "markdown")
    .build();
```

## Creating Custom Plugins

### 1. Implement the Plugin Trait

```rust
use proto_http_parser::plugins::*;

struct MyCustomValidator {
    name: String,
}

impl Plugin for MyCustomValidator {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "My custom validator plugin"
    }
    
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        // Initialize plugin with configuration
        Ok(())
    }
    
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::ProtoValidator]
    }
}
```

### 2. Implement Extension Point Traits

```rust
impl ProtoValidatorPlugin for MyCustomValidator {
    fn validate_proto_file(&self, proto_file: &ProtoFile) -> Result<Vec<ValidationError>, PluginError> {
        let mut errors = Vec::new();
        
        // Add your custom validation logic here
        for service in &proto_file.services {
            if service.name.contains("Test") {
                errors.push(ValidationError::InvalidHttpAnnotation {
                    message: format!("Service name '{}' should not contain 'Test'", service.name),
                    line: 0,
                });
            }
        }
        
        Ok(errors)
    }
}
```

### 3. Register Your Plugin

```rust
let validator = MyCustomValidator {
    name: "my_custom_validator".to_string(),
};

let config = PluginConfigBuilder::new()
    .enabled(true)
    .priority(5)
    .build();

coordinator.plugin_manager_mut()
    .register_proto_validator(validator, config)?;
```

## Plugin Configuration

### Configuration Builder

Use the `PluginConfigBuilder` for programmatic configuration:

```rust
let config = PluginConfigBuilder::new()
    .enabled(true)
    .priority(10)
    .setting("string_key", "string_value")
    .bool_setting("bool_key", true)
    .number_setting("number_key", 42)
    .json_setting("complex_key", serde_json::json!({"nested": "value"}))
    .build();
```

### Environment Variables

Configure plugins using environment variables:

```bash
export PROTO_HTTP_PARSER_PLUGIN_NAMING_VALIDATOR_SERVICE_PATTERN="^[A-Z][a-zA-Z0-9]*Service$"
export PROTO_HTTP_PARSER_PLUGIN_NAMING_VALIDATOR_ENABLED="true"
export PROTO_HTTP_PARSER_PLUGIN_NAMING_VALIDATOR_PRIORITY="10"
```

### Configuration Precedence

Configuration sources are merged with the following precedence:
1. File configuration (lowest priority)
2. Environment variables
3. Programmatic configuration (highest priority)

## Plugin Capabilities

### ProtoValidator

Validates Protocol Buffer files:

```rust
fn validate_proto_file(&self, proto_file: &ProtoFile) -> Result<Vec<ValidationError>, PluginError>;
```

### HttpValidator

Validates HTTP routes:

```rust
fn validate_http_routes(&self, routes: &[HttpRoute]) -> Result<Vec<ValidationError>, PluginError>;
```

### CodeGenerator

Generates custom code:

```rust
fn generate_code(&self, service: &Service, routes: &[HttpRoute]) -> Result<GeneratedCode, PluginError>;
fn file_extension(&self) -> &str;
fn filename_pattern(&self) -> &str;
```

### CodeFormatter

Formats generated code:

```rust
fn format_code(&self, code: &str, language: &str) -> Result<String, PluginError>;
fn supported_languages(&self) -> Vec<String>;
```

### ErrorReporter

Customizes error reporting:

```rust
fn format_error(&self, error: &ProtoHttpParserError) -> Result<String, PluginError>;
fn suggest_fix(&self, error: &ProtoHttpParserError) -> Result<Option<String>, PluginError>;
```

## Best Practices

1. **Plugin Naming**: Use descriptive names with underscores (e.g., `naming_convention_validator`)
2. **Error Handling**: Provide clear error messages with context
3. **Configuration**: Support reasonable defaults and validate configuration values
4. **Performance**: Keep validation logic efficient for large proto files
5. **Compatibility**: Check library version compatibility in `is_compatible()`
6. **Documentation**: Document plugin capabilities and configuration options

## Integration with Coordinator

The plugin system is integrated with the main `ProtoHttpCoordinator`:

```rust
// Process a file with plugins
let result = coordinator.process_file("service.proto")?;

// Plugins are automatically invoked during:
// 1. Proto file validation
// 2. HTTP route validation  
// 3. Code generation (if applicable)
// 4. Code formatting (if applicable)
```

## Example: Complete Custom Plugin

See `examples/plugin_usage.rs` for a complete example demonstrating:
- Plugin registration
- Configuration management
- Validation with multiple plugins
- Code generation and formatting
- Error handling and reporting

The plugin system provides a powerful way to extend proto-http-parser-v2 for your specific needs while maintaining clean separation of concerns and configurability.