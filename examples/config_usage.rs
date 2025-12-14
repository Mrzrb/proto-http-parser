//! Configuration system usage examples

use proto_http_parser::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Proto HTTP Parser v2 - Configuration Usage Examples");
    
    // Example 1: Using the configuration builder
    println!("\n=== Example 1: Configuration Builder ===");
    
    // Create temporary directories for the example
    let temp_dir = tempfile::tempdir()?;
    let proto_dir = temp_dir.path().join("proto");
    let third_party_dir = temp_dir.path().join("third_party/proto");
    std::fs::create_dir_all(&proto_dir)?;
    std::fs::create_dir_all(&third_party_dir)?;
    
    let config = ConfigBuilder::new()
        .preserve_comments(true)
        .strict_validation(true)
        .max_import_depth(10)
        .add_include_path(&proto_dir)
        .add_include_path(&third_party_dir)
        .generate_service_traits(true)
        .use_dependency_injection(true)
        .infer_query_params(true)
        .use_rustfmt(true)
        .indent_size(4)
        .max_line_length(100)
        .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
        .add_import("use chrono::{DateTime, Utc};")
        .build()?;
    
    println!("✓ Configuration built successfully!");
    println!("  Preserve comments: {}", config.parser.preserve_comments);
    println!("  Generate service traits: {}", config.generator.generate_service_traits);
    println!("  Include paths: {} directories", config.parser.include_paths.len());
    println!("  Type mappings: {} mappings", config.generator.type_mappings.len());
    
    // Example 2: Loading configuration from file
    println!("\n=== Example 2: Configuration from File ===");
    
    // Use the same temp directory
    let config_path = temp_dir.path().join("proto-http-parser.toml");
    
    let config_content = r#"
[parser]
preserve_comments = true
strict_validation = false
max_import_depth = 15
include_paths = ["."]

[extractor]
infer_query_params = true
validate_http_methods = true
allow_custom_methods = false
common_query_params = ["page", "limit", "sort", "filter"]

[generator]
generate_service_traits = true
use_dependency_injection = true
target_framework = "PoemOpenApi"
additional_imports = ["use serde::{Deserialize, Serialize};"]

[generator.formatting]
indent_style = "Spaces"
indent_size = 2
max_line_length = 120
use_rustfmt = true

[template]
use_builtin_templates = true

[template.template_overrides]

[template.helpers]

[generator.type_mappings]
"google.protobuf.Timestamp" = "chrono::DateTime<chrono::Utc>"
"google.protobuf.Duration" = "chrono::Duration"
"#;
    
    std::fs::write(&config_path, config_content)?;
    
    let file_config = ProtoHttpParserConfig::from_file(&config_path)?;
    println!("✓ Configuration loaded from file!");
    println!("  Max import depth: {}", file_config.parser.max_import_depth);
    println!("  Indent size: {}", file_config.generator.formatting.indent_size);
    println!("  Max line length: {}", file_config.generator.formatting.max_line_length);
    
    // Example 3: Environment variable configuration
    println!("\n=== Example 3: Environment Variables ===");
    
    // Set some environment variables
    std::env::set_var("PROTO_HTTP_PARSER_PRESERVE_COMMENTS", "false");
    std::env::set_var("PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS", "false");
    std::env::set_var("PROTO_HTTP_PARSER_MAX_IMPORT_DEPTH", "20");
    std::env::set_var("PROTO_HTTP_PARSER_USE_RUSTFMT", "false");
    
    let env_config = ProtoHttpParserConfig::from_env()?;
    println!("✓ Configuration loaded from environment!");
    println!("  Preserve comments: {}", env_config.parser.preserve_comments);
    println!("  Generate service traits: {}", env_config.generator.generate_service_traits);
    println!("  Max import depth: {}", env_config.parser.max_import_depth);
    println!("  Use rustfmt: {}", env_config.generator.formatting.use_rustfmt);
    
    // Clean up environment variables
    std::env::remove_var("PROTO_HTTP_PARSER_PRESERVE_COMMENTS");
    std::env::remove_var("PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS");
    std::env::remove_var("PROTO_HTTP_PARSER_MAX_IMPORT_DEPTH");
    std::env::remove_var("PROTO_HTTP_PARSER_USE_RUSTFMT");
    
    // Example 4: Configuration merging
    println!("\n=== Example 4: Configuration Merging ===");
    
    let base_config = ConfigBuilder::new()
        .preserve_comments(true)
        .max_import_depth(5)
        .build()?;
    
    let override_config = ConfigBuilder::new()
        .max_import_depth(15)
        .generate_service_traits(false)
        .build()?;
    
    let mut merged_config = base_config.clone();
    merged_config.merge(override_config);
    
    println!("✓ Configuration merged successfully!");
    println!("  Original max import depth: {}", base_config.parser.max_import_depth);
    println!("  Merged max import depth: {}", merged_config.parser.max_import_depth);
    println!("  Preserve comments (unchanged): {}", merged_config.parser.preserve_comments);
    
    // Example 5: Configuration validation
    println!("\n=== Example 5: Configuration Validation ===");
    
    // Try to create an invalid configuration
    let invalid_config_result = ConfigBuilder::new()
        .max_import_depth(0)  // Invalid: must be > 0
        .build();
    
    match invalid_config_result {
        Ok(_) => println!("✗ Expected validation to fail!"),
        Err(e) => println!("✓ Validation correctly failed: {}", e),
    }
    
    // Try another invalid configuration
    let invalid_config_result2 = ConfigBuilder::new()
        .indent_size(0)  // Invalid: must be > 0
        .build();
    
    match invalid_config_result2 {
        Ok(_) => println!("✗ Expected validation to fail!"),
        Err(e) => println!("✓ Validation correctly failed: {}", e),
    }
    
    // Example 6: Saving configuration to file
    println!("\n=== Example 6: Saving Configuration ===");
    
    let save_config = ConfigBuilder::new()
        .preserve_comments(false)
        .max_import_depth(8)
        .generate_service_traits(true)
        .use_dependency_injection(false)
        .infer_query_params(true)
        .indent_size(2)
        .max_line_length(80)
        .build()?;
    
    let save_path = temp_dir.path().join("saved_config.toml");
    save_config.to_file(&save_path)?;
    
    println!("✓ Configuration saved to: {}", save_path.display());
    
    // Verify by loading it back
    let loaded_config = ProtoHttpParserConfig::from_file(&save_path)?;
    println!("✓ Configuration loaded back successfully!");
    println!("  Max import depth: {}", loaded_config.parser.max_import_depth);
    println!("  Indent size: {}", loaded_config.generator.formatting.indent_size);
    
    // Example 7: Auto-loading configuration
    println!("\n=== Example 7: Auto-loading Configuration ===");
    
    // Create a config file in the current directory
    let auto_config_path = temp_dir.path().join("proto-http-parser.toml");
    save_config.to_file(&auto_config_path)?;
    
    // Change to the temp directory to test auto-loading
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&temp_dir)?;
    
    match ProtoHttpParserConfig::load() {
        Ok(auto_config) => {
            println!("✓ Configuration auto-loaded successfully!");
            println!("  Max import depth: {}", auto_config.parser.max_import_depth);
        }
        Err(e) => {
            println!("✗ Auto-loading failed: {}", e);
        }
    }
    
    // Restore original directory
    std::env::set_current_dir(&original_dir)?;
    
    println!("\n✓ All configuration examples completed successfully!");
    
    Ok(())
}