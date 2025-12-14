//! Build.rs integration examples

use proto_http_parser::*;
use std::path::PathBuf;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Proto HTTP Parser v2 - Build Integration Examples");
    
    // Create temporary directory and proto files for testing
    let temp_dir = tempfile::tempdir()?;
    let proto_dir = temp_dir.path().join("proto");
    std::fs::create_dir_all(&proto_dir)?;
    
    // Create sample proto files
    create_sample_proto_files(&proto_dir)?;
    
    // Example 1: Basic build integration
    println!("\n=== Example 1: Basic Build Integration ===");
    
    let output_dir = temp_dir.path().join("generated1");
    
    let result = BuildIntegration::new()
        .add_proto_file(proto_dir.join("user_service.proto"))
        .add_proto_file(proto_dir.join("product_service.proto"))
        .output_dir(&output_dir)
        .verbose(true)
        .generate()?;
    
    println!("✓ Basic build integration completed!");
    println!("  Generated {} files", result.file_count());
    println!("  Output directory: {}", result.output_dir.display());
    
    for file in &result.generated_files {
        println!("    - {}", file.file_name().unwrap().to_string_lossy());
    }
    
    // Example 2: Build integration with configuration file
    println!("\n=== Example 2: Build Integration with Config File ===");
    
    // Create a configuration file
    let config_path = temp_dir.path().join("build_config.toml");
    let config_content = r#"
[parser]
preserve_comments = true
strict_validation = true
max_import_depth = 10

[generator]
generate_service_traits = true
use_dependency_injection = true

[generator.formatting]
use_rustfmt = false
indent_size = 2
max_line_length = 100

[extractor]
infer_query_params = true
"#;
    std::fs::write(&config_path, config_content)?;
    
    let output_dir2 = temp_dir.path().join("generated2");
    
    let result2 = BuildIntegration::new()
        .with_config_file(&config_path)
        .add_proto_directory(&proto_dir)?
        .output_dir(&output_dir2)
        .verbose(true)
        .generate()?;
    
    println!("✓ Build integration with config file completed!");
    println!("  Generated {} files", result2.file_count());
    
    // Example 3: Build integration with environment variables
    println!("\n=== Example 3: Build Integration with Environment Variables ===");
    
    // Set environment variables
    std::env::set_var("PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS", "false");
    std::env::set_var("PROTO_HTTP_PARSER_USE_DEPENDENCY_INJECTION", "false");
    std::env::set_var("PROTO_HTTP_PARSER_INFER_QUERY_PARAMS", "false");
    
    let output_dir3 = temp_dir.path().join("generated3");
    
    let result3 = BuildIntegration::new()
        .with_env_config()
        .add_proto_glob(&format!("{}/*.proto", proto_dir.display()))?
        .output_dir(&output_dir3)
        .verbose(true)
        .generate()?;
    
    println!("✓ Build integration with environment variables completed!");
    println!("  Generated {} files", result3.file_count());
    
    // Clean up environment variables
    std::env::remove_var("PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS");
    std::env::remove_var("PROTO_HTTP_PARSER_USE_DEPENDENCY_INJECTION");
    std::env::remove_var("PROTO_HTTP_PARSER_INFER_QUERY_PARAMS");
    
    // Example 4: Build integration with custom configuration
    println!("\n=== Example 4: Build Integration with Custom Configuration ===");
    
    let output_dir4 = temp_dir.path().join("generated4");
    
    let result4 = BuildIntegration::new()
        .configure(|builder| {
            builder
                .preserve_comments(false)
                .generate_service_traits(true)
                .use_dependency_injection(true)
                .infer_query_params(true)
                .use_rustfmt(false)
                .indent_size(4)
                .max_line_length(120)
                .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
                .add_import("use chrono::{DateTime, Utc};")
        })
        .add_proto_directory(&proto_dir)?
        .output_dir(&output_dir4)
        .verbose(true)
        .generate()?;
    
    println!("✓ Build integration with custom configuration completed!");
    println!("  Generated {} files", result4.file_count());
    
    // Example 5: Build integration with OUT_DIR
    println!("\n=== Example 5: Build Integration with OUT_DIR ===");
    
    // Simulate OUT_DIR environment variable
    std::env::set_var("OUT_DIR", temp_dir.path().join("out"));
    
    let result5 = BuildIntegration::new()
        .add_proto_file(proto_dir.join("user_service.proto"))
        .output_dir_from_env("generated")
        .verbose(true)
        .generate()?;
    
    println!("✓ Build integration with OUT_DIR completed!");
    println!("  Generated {} files", result5.file_count());
    println!("  Output directory: {}", result5.output_dir.display());
    
    std::env::remove_var("OUT_DIR");
    
    // Example 6: Auto-configuration build integration
    println!("\n=== Example 6: Auto-configuration Build Integration ===");
    
    // Create a default config file
    let auto_config_path = temp_dir.path().join("proto-http-parser.toml");
    std::fs::write(&auto_config_path, config_content)?;
    
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&temp_dir)?;
    
    let output_dir6 = temp_dir.path().join("generated6");
    
    match BuildIntegration::with_auto_config() {
        Ok(integration) => {
            let result6 = integration
                .add_proto_directory(&proto_dir)?
                .output_dir(&output_dir6)
                .verbose(true)
                .generate()?;
            
            println!("✓ Auto-configuration build integration completed!");
            println!("  Generated {} files", result6.file_count());
        }
        Err(e) => {
            println!("✗ Auto-configuration failed: {}", e);
        }
    }
    
    std::env::set_current_dir(&original_dir)?;
    
    // Example 7: Error handling in build integration
    println!("\n=== Example 7: Error Handling ===");
    
    // Try to process a non-existent file
    let error_result = BuildIntegration::new()
        .add_proto_file("non_existent.proto")
        .output_dir(temp_dir.path().join("error_output"))
        .generate();
    
    match error_result {
        Ok(_) => println!("✗ Expected an error for non-existent file!"),
        Err(e) => println!("✓ Correctly handled error: {}", e),
    }
    
    println!("\n✓ All build integration examples completed!");
    
    Ok(())
}

/// Create sample proto files for testing
fn create_sample_proto_files(proto_dir: &std::path::Path) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let user_service_proto = r#"
syntax = "proto3";

package example.v1;

service UserService {
    rpc GetUser(GetUserRequest) returns (User) {
        option (google.api.http) = {
            get: "/v1/users/{user_id}"
        };
    }
    
    rpc CreateUser(CreateUserRequest) returns (User) {
        option (google.api.http) = {
            post: "/v1/users"
            body: "*"
        };
    }
    
    rpc UpdateUser(UpdateUserRequest) returns (User) {
        option (google.api.http) = {
            put: "/v1/users/{user_id}"
            body: "user"
        };
    }
    
    rpc DeleteUser(DeleteUserRequest) returns (DeleteUserResponse) {
        option (google.api.http) = {
            delete: "/v1/users/{user_id}"
        };
    }
    
    rpc ListUsers(ListUsersRequest) returns (ListUsersResponse) {
        option (google.api.http) = {
            get: "/v1/users"
        };
    }
}

message GetUserRequest {
    string user_id = 1;
}

message CreateUserRequest {
    User user = 1;
}

message UpdateUserRequest {
    string user_id = 1;
    User user = 2;
}

message DeleteUserRequest {
    string user_id = 1;
}

message ListUsersRequest {
    int32 page = 1;
    int32 limit = 2;
    string sort = 3;
}

message DeleteUserResponse {
    bool success = 1;
}

message ListUsersResponse {
    repeated User users = 1;
    int32 total = 2;
}

message User {
    string id = 1;
    string name = 2;
    string email = 3;
    Status status = 4;
}

enum Status {
    STATUS_UNSPECIFIED = 0;
    STATUS_ACTIVE = 1;
    STATUS_INACTIVE = 2;
}
"#;

    let product_service_proto = r#"
syntax = "proto3";

package example.v1;

service ProductService {
    rpc GetProduct(GetProductRequest) returns (Product) {
        option (google.api.http) = {
            get: "/v1/products/{product_id}"
        };
    }
    
    rpc ListProducts(ListProductsRequest) returns (ListProductsResponse) {
        option (google.api.http) = {
            get: "/v1/products"
        };
    }
    
    rpc CreateProduct(CreateProductRequest) returns (Product) {
        option (google.api.http) = {
            post: "/v1/products"
            body: "*"
        };
    }
}

message GetProductRequest {
    string product_id = 1;
}

message ListProductsRequest {
    int32 page = 1;
    int32 limit = 2;
    string category = 3;
}

message CreateProductRequest {
    Product product = 1;
}

message ListProductsResponse {
    repeated Product products = 1;
    int32 total = 2;
}

message Product {
    string id = 1;
    string name = 2;
    string description = 3;
    double price = 4;
    string category = 5;
}
"#;

    std::fs::write(proto_dir.join("user_service.proto"), user_service_proto)?;
    std::fs::write(proto_dir.join("product_service.proto"), product_service_proto)?;
    
    Ok(())
}