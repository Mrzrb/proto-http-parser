//! Custom configuration example demonstrating advanced configuration options

use proto_http_parser_v2::*;
use std::collections::HashMap;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Proto HTTP Parser v2 - Custom Configuration Example");
    
    // Example proto content for testing configurations
    let proto_content = r#"
syntax = "proto3";

package api.v1;

import "google/api/annotations.proto";
import "google/protobuf/timestamp.proto";
import "google/protobuf/duration.proto";

service ConfigTestService {
    rpc GetResource(GetResourceRequest) returns (Resource) {
        option (google.api.http) = {
            get: "/v1/resources/{resource_id}"
        };
    }
    
    rpc CreateResource(CreateResourceRequest) returns (Resource) {
        option (google.api.http) = {
            post: "/v1/resources"
            body: "*"
        };
    }
    
    rpc UpdateResource(UpdateResourceRequest) returns (Resource) {
        option (google.api.http) = {
            put: "/v1/resources/{resource.id}"
            body: "resource"
        };
    }
    
    rpc ListResources(ListResourcesRequest) returns (ListResourcesResponse) {
        option (google.api.http) = {
            get: "/v1/resources"
        };
    }
}

message GetResourceRequest {
    string resource_id = 1;
}

message CreateResourceRequest {
    Resource resource = 1;
}

message UpdateResourceRequest {
    Resource resource = 1;
}

message ListResourcesRequest {
    int32 page_size = 1;
    string page_token = 2;
    string filter = 3;
    string sort_by = 4;
}

message ListResourcesResponse {
    repeated Resource resources = 1;
    string next_page_token = 2;
    int32 total_count = 3;
}

message Resource {
    string id = 1;
    string name = 2;
    string description = 3;
    ResourceType type = 4;
    google.protobuf.Timestamp created_at = 5;
    google.protobuf.Timestamp updated_at = 6;
    google.protobuf.Duration ttl = 7;
}

enum ResourceType {
    RESOURCE_TYPE_UNSPECIFIED = 0;
    RESOURCE_TYPE_DOCUMENT = 1;
    RESOURCE_TYPE_IMAGE = 2;
    RESOURCE_TYPE_VIDEO = 3;
}
"#;

    // Example 1: Basic custom configuration
    println!("\n=== Example 1: Basic Custom Configuration ===");
    
    let basic_config = ConfigBuilder::new()
        .preserve_comments(true)
        .strict_validation(false)
        .max_import_depth(20)
        .generate_service_traits(true)
        .use_dependency_injection(true)
        .infer_query_params(true)
        .build()?;
    
    println!("✓ Basic configuration created");
    println!("  Preserve comments: {}", basic_config.parser.preserve_comments);
    println!("  Generate service traits: {}", basic_config.generator.generate_service_traits);
    println!("  Use dependency injection: {}", basic_config.generator.use_dependency_injection);
    
    // Test with basic configuration
    let coordinator = ProtoHttpCoordinator::with_config(basic_config.clone());
    let result = coordinator.process_content(proto_content)?;
    
    println!("  Generated {} files", 
        result.generated_files.len());
    
    // Example 2: Advanced formatting configuration
    println!("\n=== Example 2: Advanced Formatting Configuration ===");
    
    let formatting_config = ConfigBuilder::new()
        .indent_size(2)
        .max_line_length(80)
        .use_rustfmt(false)  // Disable rustfmt to see raw formatting
        .build()?;
    
    println!("✓ Formatting configuration created");
    println!("  Indent size: {}", formatting_config.generator.formatting.indent_size);
    println!("  Max line length: {}", formatting_config.generator.formatting.max_line_length);
    println!("  Use rustfmt: {}", formatting_config.generator.formatting.use_rustfmt);
    
    let coordinator = ProtoHttpCoordinator::with_config(formatting_config);
    let result = coordinator.process_content(proto_content)?;
    
    // Show formatting differences
    if let Some(controller) = result.generated_files.values().next() {
        println!("\n  Sample formatted code (first 10 lines):");
        for (i, line) in controller.content.lines().take(10).enumerate() {
            println!("    {:2}: {}", i + 1, line);
        }
    }
    
    // Example 3: Type mapping configuration
    println!("\n=== Example 3: Type Mapping Configuration ===");
    
    let type_mapping_config = ConfigBuilder::new()
        .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
        .add_type_mapping("google.protobuf.Duration", "chrono::Duration")
        .add_type_mapping("string", "String")
        .add_import("use chrono::{DateTime, Duration, Utc};")
        .add_import("use serde::{Deserialize, Serialize};")
        .build()?;
    
    println!("✓ Type mapping configuration created");
    println!("  Type mappings: {}", type_mapping_config.generator.type_mappings.len());
    println!("  Additional imports: {}", type_mapping_config.generator.additional_imports.len());
    
    for (proto_type, rust_type) in &type_mapping_config.generator.type_mappings {
        println!("    {} -> {}", proto_type, rust_type);
    }
    
    let coordinator = ProtoHttpCoordinator::with_config(type_mapping_config);
    let result = coordinator.process_content(proto_content)?;
    
    // Check if type mappings were applied
    if let Some(controller) = result.generated_files.values().next() {
        let has_chrono = controller.content.contains("chrono::");
        let has_datetime = controller.content.contains("DateTime<Utc>");
        println!("  Applied chrono types: {}", has_chrono || has_datetime);
    }
    
    // Example 4: Query parameter inference configuration
    println!("\n=== Example 4: Query Parameter Configuration ===");
    
    let query_config = ConfigBuilder::new()
        .infer_query_params(true)
        .build()?;
    
    println!("✓ Query parameter configuration created");
    println!("  Infer query params: {}", query_config.extractor.infer_query_params);
    println!("  Query parameter inference enabled");
    
    let coordinator = ProtoHttpCoordinator::with_config(query_config);
    let result = coordinator.process_content(proto_content)?;
    
    // Check query parameter inference
    let service_routes: Vec<_> = result.routes.iter()
        .filter(|r| r.service_name == "ConfigTestService")
        .collect();
    if !service_routes.is_empty() {
        for route in &service_routes {
            if route.method_name == "ListResources" {
                println!("  ListResources query params: {:?}", 
                    route.query_parameters.iter()
                        .map(|p| &p.name)
                        .collect::<Vec<_>>()
                );
            }
        }
    }
    
    // Example 5: Template customization
    println!("\n=== Example 5: Template Customization ===");
    
    // Create custom template content
    let custom_controller_template = r#"
//! Custom controller template for {{service_name}}
//! Generated with custom formatting

use poem_openapi::{OpenApi, ApiResponse, Object};
use poem::{Result, web::Path, web::Query, web::Json};
{{#each imports}}
{{{this}}}
{{/each}}

/// {{service_name}} controller with custom template
#[derive(Default)]
pub struct {{service_name}}Controller<T: {{service_name}}Service> {
    service: T,
}

impl<T: {{service_name}}Service> {{service_name}}Controller<T> {
    pub fn new(service: T) -> Self {
        Self { service }
    }
}

#[OpenApi]
impl<T: {{service_name}}Service + Send + Sync> {{service_name}}Controller<T> {
{{#each methods}}
    /// {{description}}
    /// Custom method: {{name}}
    #[oai(path = "{{path}}", method = "{{http_method}}")]
    pub async fn {{snake_case name}}(
        &self,
        {{#each path_params}}
        {{snake_case name}}: Path<{{rust_type}}>,
        {{/each}}
        {{#if has_query_params}}
        query: Query<{{query_struct_name}}>,
        {{/if}}
        {{#if has_body}}
        body: Json<{{body_type}}>,
        {{/if}}
    ) -> Result<{{response_type}}> {
        // Custom implementation with service delegation
        self.service.{{snake_case name}}(
            {{#each path_params}}
            {{snake_case name}}.0,
            {{/each}}
            {{#if has_query_params}}
            query.0,
            {{/if}}
            {{#if has_body}}
            body.0,
            {{/if}}
        ).await
    }
{{/each}}
}
"#;

    let template_config = ConfigBuilder::new()
        .use_builtin_templates(false)
        .build()?;
    
    println!("✓ Template customization configuration created");
    println!("  Use builtin templates: {}", template_config.template.use_builtin_templates);
    println!("  Custom templates: {}", template_config.template.template_overrides.len());
    
    let coordinator = ProtoHttpCoordinator::with_config(template_config);
    let result = coordinator.process_content(proto_content)?;
    
    if let Some(controller) = result.generated_files.values().next() {
        println!("\n  Custom template output (first 15 lines):");
        for (i, line) in controller.content.lines().take(15).enumerate() {
            println!("    {:2}: {}", i + 1, line);
        }
    }
    
    // Example 6: Validation configuration
    println!("\n=== Example 6: Validation Configuration ===");
    
    let validation_config = ConfigBuilder::new()
        .strict_validation(true)
        .build()?;
    
    println!("✓ Validation configuration created");
    println!("  Strict validation: {}", validation_config.parser.strict_validation);
    println!("  Validation enabled");
    
    // Test with invalid proto content
    let invalid_proto = r#"
syntax = "proto3";
service InvalidService {
    rpc BadMethod(Request) returns (Response) {
        option (google.api.http) = {
            custom: "/invalid"  // This should fail validation
        };
    }
}
message Request {}
message Response {}
"#;
    
    let coordinator = ProtoHttpCoordinator::with_config(validation_config);
    match coordinator.process_content(invalid_proto) {
        Ok(_) => println!("  ✗ Expected validation to fail"),
        Err(e) => println!("  ✓ Validation correctly failed: {}", e),
    }
    
    // Example 7: Performance configuration
    println!("\n=== Example 7: Performance Configuration ===");
    
    let performance_config = ConfigBuilder::new()
        .max_import_depth(5)  // Limit import depth for performance
        .preserve_comments(false)  // Skip comments for faster parsing
        .use_rustfmt(false)  // Skip rustfmt for faster generation
        .build()?;
    
    println!("✓ Performance configuration created");
    println!("  Max import depth: {}", performance_config.parser.max_import_depth);
    println!("  Preserve comments: {}", performance_config.parser.preserve_comments);
    println!("  Use rustfmt: {}", performance_config.generator.formatting.use_rustfmt);
    
    // Measure generation time
    let start = std::time::Instant::now();
    let coordinator = ProtoHttpCoordinator::with_config(performance_config);
    let result = coordinator.process_content(proto_content)?;
    let duration = start.elapsed();
    
    println!("  Generation time: {:?}", duration);
    println!("  Generated {} files", result.generated_files.len());
    
    // Example 8: Configuration from file
    println!("\n=== Example 8: Configuration from File ===");
    
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("custom_config.toml");
    
    let config_content = r#"
[parser]
preserve_comments = false
strict_validation = true
max_import_depth = 8
include_paths = ["./proto", "./third_party"]

[extractor]
infer_query_params = true
validate_http_methods = true
allow_custom_methods = false
common_query_params = ["page", "size", "sort", "filter", "search"]

[generator]
generate_service_traits = true
use_dependency_injection = true
target_framework = "PoemOpenApi"
additional_imports = [
    "use chrono::{DateTime, Utc};",
    "use serde::{Deserialize, Serialize};",
    "use uuid::Uuid;"
]

[generator.type_mappings]
"google.protobuf.Timestamp" = "DateTime<Utc>"
"google.protobuf.Duration" = "std::time::Duration"
"string" = "String"

[generator.formatting]
indent_style = "Spaces"
indent_size = 2
max_line_length = 100
use_rustfmt = true

[template]
use_builtin_templates = true

[template.template_overrides]

[template.helpers]
"#;
    
    std::fs::write(&config_path, config_content)?;
    
    let file_config = ProtoHttpParserConfig::from_file(&config_path)?;
    println!("✓ Configuration loaded from file");
    println!("  Config file: {}", config_path.display());
    println!("  Type mappings: {}", file_config.generator.type_mappings.len());
    println!("  Additional imports: {}", file_config.generator.additional_imports.len());
    
    let coordinator = ProtoHttpCoordinator::with_config(file_config);
    let result = coordinator.process_content(proto_content)?;
    
    println!("  Generated with file config: {} files", result.generated_files.len());
    
    // Example 9: Configuration merging
    println!("\n=== Example 9: Configuration Merging ===");
    
    let base_config = ConfigBuilder::new()
        .preserve_comments(true)
        .generate_service_traits(false)
        .indent_size(4)
        .build()?;
    
    let override_config = ConfigBuilder::new()
        .generate_service_traits(true)  // Override this setting
        .use_dependency_injection(true)  // Add this setting
        .build()?;
    
    let mut merged_config = base_config.clone();
    merged_config.merge(override_config);
    
    println!("✓ Configuration merging completed");
    println!("  Base preserve comments: {}", base_config.parser.preserve_comments);
    println!("  Base generate traits: {}", base_config.generator.generate_service_traits);
    println!("  Merged preserve comments: {}", merged_config.parser.preserve_comments);
    println!("  Merged generate traits: {}", merged_config.generator.generate_service_traits);
    println!("  Merged dependency injection: {}", merged_config.generator.use_dependency_injection);
    
    // Example 10: Environment variable configuration
    println!("\n=== Example 10: Environment Variable Configuration ===");
    
    // Set environment variables
    std::env::set_var("PROTO_HTTP_PARSER_PRESERVE_COMMENTS", "false");
    std::env::set_var("PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS", "true");
    std::env::set_var("PROTO_HTTP_PARSER_INDENT_SIZE", "2");
    std::env::set_var("PROTO_HTTP_PARSER_MAX_LINE_LENGTH", "120");
    std::env::set_var("PROTO_HTTP_PARSER_USE_RUSTFMT", "false");
    
    let env_config = ProtoHttpParserConfig::from_env()?;
    println!("✓ Configuration loaded from environment");
    println!("  Preserve comments: {}", env_config.parser.preserve_comments);
    println!("  Generate service traits: {}", env_config.generator.generate_service_traits);
    println!("  Indent size: {}", env_config.generator.formatting.indent_size);
    println!("  Max line length: {}", env_config.generator.formatting.max_line_length);
    
    // Clean up environment variables
    std::env::remove_var("PROTO_HTTP_PARSER_PRESERVE_COMMENTS");
    std::env::remove_var("PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS");
    std::env::remove_var("PROTO_HTTP_PARSER_INDENT_SIZE");
    std::env::remove_var("PROTO_HTTP_PARSER_MAX_LINE_LENGTH");
    std::env::remove_var("PROTO_HTTP_PARSER_USE_RUSTFMT");
    
    let coordinator = ProtoHttpCoordinator::with_config(env_config);
    let result = coordinator.process_content(proto_content)?;
    
    println!("  Generated with env config: {} files", result.generated_files.len());
    
    println!("\n✓ Custom configuration examples completed successfully!");
    println!("  Demonstrated {} different configuration approaches", 10);
    
    Ok(())
}