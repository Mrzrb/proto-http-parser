//! Example demonstrating the main coordinator usage

use proto_http_parser_v2::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Proto HTTP Parser v2 - Coordinator Usage Example");
    
    // Example proto content with HTTP annotations
    let proto_content = r#"
syntax = "proto3";

package example.v1;

// import "google/api/annotations.proto";

service UserService {
    rpc GetUser(GetUserRequest) returns (GetUserResponse) {
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

message GetUserResponse {
    User user = 1;
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

    // Write the proto content to a temporary file
    let temp_dir = tempfile::tempdir()?;
    let proto_path = temp_dir.path().join("user_service.proto");
    std::fs::write(&proto_path, proto_content)?;
    
    println!("✓ Created temporary proto file: {}", proto_path.display());
    
    // Example 1: Basic coordinator usage
    println!("\n=== Example 1: Basic Coordinator Usage ===");
    
    let coordinator = ProtoHttpCoordinator::new();
    
    // Process the proto file
    let result = coordinator.process_file(&proto_path)?;
    
    println!("✓ Successfully processed proto file!");
    println!("  Services: {}", result.proto_file.services.len());
    println!("  HTTP Routes: {}", result.routes.len());
    println!("  Generated Files: {}", result.generated_files.len());
    
    // Display route information
    for route in &result.routes {
        println!("  Route: {} {} -> {}", 
            route.http_method.as_str(), 
            route.path_template,
            route.method_name
        );
    }
    
    // Display generated files
    for (filename, _) in &result.generated_files {
        println!("  Generated: {}", filename);
    }
    
    // Example 2: Write generated code to output directory
    println!("\n=== Example 2: Writing Generated Code ===");
    
    let output_dir = temp_dir.path().join("generated");
    coordinator.write_generated_code(&result, &output_dir)?;
    
    println!("✓ Generated code written to: {}", output_dir.display());
    
    // List generated files
    for entry in std::fs::read_dir(&output_dir)? {
        let entry = entry?;
        let file_size = entry.metadata()?.len();
        println!("  {} ({} bytes)", entry.file_name().to_string_lossy(), file_size);
    }
    
    // Example 3: Custom configuration
    println!("\n=== Example 3: Custom Configuration ===");
    
    let mut config = ProtoHttpParserConfig::new();
    config.generator.generate_service_traits = true;
    config.generator.use_dependency_injection = true;
    config.extractor.infer_query_params = true;
    
    let custom_coordinator = ProtoHttpCoordinator::with_config(config);
    let custom_result = custom_coordinator.process_file(&proto_path)?;
    
    println!("✓ Processed with custom configuration!");
    println!("  Generated Files: {}", custom_result.generated_files.len());
    
    // Example 4: Batch processing (simulate multiple files)
    println!("\n=== Example 4: Batch Processing ===");
    
    // Create another proto file
    let product_proto_content = r#"
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
}

message GetProductRequest {
    string product_id = 1;
}

message ListProductsRequest {
    int32 page = 1;
    int32 limit = 2;
}

message ListProductsResponse {
    repeated Product products = 1;
}

message Product {
    string id = 1;
    string name = 2;
    string description = 3;
    double price = 4;
}
"#;
    
    let product_proto_path = temp_dir.path().join("product_service.proto");
    std::fs::write(&product_proto_path, product_proto_content)?;
    
    // Process multiple files
    let proto_files = vec![&proto_path, &product_proto_path];
    let batch_result = coordinator.process_files(&proto_files)?;
    
    println!("✓ Batch processing completed!");
    println!("  Successful: {}", batch_result.success_count());
    println!("  Errors: {}", batch_result.error_count());
    
    if batch_result.is_success() {
        let batch_output_dir = temp_dir.path().join("batch_generated");
        coordinator.write_batch_results(&batch_result, &batch_output_dir)?;
        println!("  Batch results written to: {}", batch_output_dir.display());
    }
    
    // Example 5: Build integration API
    println!("\n=== Example 5: Build Integration API ===");
    
    let build_output_dir = temp_dir.path().join("build_generated");
    
    // Simulate build.rs usage
    let build_result = BuildIntegration::new()
        .add_proto_file(&proto_path)
        .add_proto_file(&product_proto_path)
        .output_dir(&build_output_dir)
        .generate();
    
    match build_result {
        Ok(build_result) => {
            println!("✓ Build integration successful!");
            println!("  Output directory: {}", build_output_dir.display());
            println!("  Generated files: {}", build_result.generated_files.len());
        }
        Err(e) => {
            println!("✗ Build integration failed: {}", e);
        }
    }
    
    println!("\n✓ All examples completed successfully!");
    
    Ok(())
}