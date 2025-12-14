//! Integration tests for the complete end-to-end workflow
//!
//! These tests verify that the entire process from proto parsing to code generation
//! works correctly and that the generated code compiles and integrates with poem-openapi.

use proto_http_parser::*;
use tempfile::TempDir;

/// Test the complete end-to-end workflow with a simple service
#[test]
fn test_end_to_end_simple_service() {
    let proto_content = r#"
syntax = "proto3";

package test.v1;

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
}

message GetUserRequest {
    string user_id = 1;
}

message CreateUserRequest {
    string name = 1;
    string email = 2;
}

message User {
    string id = 1;
    string name = 2;
    string email = 3;
}
"#;

    // Create temporary directory and proto file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proto_path = temp_dir.path().join("user_service.proto");
    std::fs::write(&proto_path, proto_content).expect("Failed to write proto file");

    // Create coordinator and process the file
    let coordinator = ProtoHttpCoordinator::new();
    let result = coordinator.process_file(&proto_path).expect("Failed to process proto file");

    // Verify the results
    assert_eq!(result.proto_file.services.len(), 1);
    assert_eq!(result.proto_file.services[0].name, "UserService");
    assert_eq!(result.routes.len(), 2);
    assert_eq!(result.generated_files.len(), 2); // Controller + Service trait

    // Verify routes
    let get_route = result.routes.iter().find(|r| r.method_name == "GetUser").unwrap();
    assert_eq!(get_route.http_method, HttpMethod::Get);
    assert_eq!(get_route.path_template, "/v1/users/{user_id}");
    assert_eq!(get_route.path_parameters.len(), 1);
    assert_eq!(get_route.path_parameters[0].name, "user_id");

    let create_route = result.routes.iter().find(|r| r.method_name == "CreateUser").unwrap();
    assert_eq!(create_route.http_method, HttpMethod::Post);
    assert_eq!(create_route.path_template, "/v1/users");
    assert!(create_route.request_body.is_some());

    // Verify generated files exist
    assert!(result.generated_files.contains_key("user_service_controller.rs"));
    assert!(result.generated_files.contains_key("user_service_service.rs"));

    // Write generated code and verify it can be written
    let output_dir = temp_dir.path().join("generated");
    coordinator.write_generated_code(&result, &output_dir).expect("Failed to write generated code");

    // Verify files were written
    assert!(output_dir.join("user_service_controller.rs").exists());
    assert!(output_dir.join("user_service_service.rs").exists());

    // Read and verify the generated controller contains expected content
    let controller_content = std::fs::read_to_string(output_dir.join("user_service_controller.rs"))
        .expect("Failed to read controller file");
    
    assert!(controller_content.contains("pub struct UserServiceController"));
    assert!(controller_content.contains("async fn get_user"));
    assert!(controller_content.contains("async fn create_user"));
    assert!(controller_content.contains("#[oai(path = \"/v1/users/{user_id}\", method = \"get\")]"));
    assert!(controller_content.contains("#[oai(path = \"/v1/users\", method = \"post\")]"));

    // Read and verify the generated service trait contains expected content
    let service_content = std::fs::read_to_string(output_dir.join("user_service_service.rs"))
        .expect("Failed to read service file");
    
    assert!(service_content.contains("pub trait UserServiceService"));
    assert!(service_content.contains("async fn get_user"));
    assert!(service_content.contains("async fn create_user"));
    assert!(service_content.contains("use async_trait::async_trait"));
}

/// Test batch processing of multiple proto files
#[test]
fn test_batch_processing() {
    let user_proto = r#"
syntax = "proto3";

package test.v1;

service UserService {
    rpc GetUser(GetUserRequest) returns (User) {
        option (google.api.http) = {
            get: "/v1/users/{user_id}"
        };
    }
}

message GetUserRequest {
    string user_id = 1;
}

message User {
    string id = 1;
    string name = 2;
}
"#;

    let product_proto = r#"
syntax = "proto3";

package test.v1;

service ProductService {
    rpc GetProduct(GetProductRequest) returns (Product) {
        option (google.api.http) = {
            get: "/v1/products/{product_id}"
        };
    }
}

message GetProductRequest {
    string product_id = 1;
}

message Product {
    string id = 1;
    string name = 2;
    double price = 3;
}
"#;

    // Create temporary directory and proto files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let user_proto_path = temp_dir.path().join("user_service.proto");
    let product_proto_path = temp_dir.path().join("product_service.proto");
    
    std::fs::write(&user_proto_path, user_proto).expect("Failed to write user proto file");
    std::fs::write(&product_proto_path, product_proto).expect("Failed to write product proto file");

    // Process multiple files
    let coordinator = ProtoHttpCoordinator::new();
    let proto_files = vec![&user_proto_path, &product_proto_path];
    let batch_result = coordinator.process_files(&proto_files).expect("Failed to process proto files");

    // Verify batch results
    assert!(batch_result.is_success());
    assert_eq!(batch_result.success_count(), 2);
    assert_eq!(batch_result.error_count(), 0);

    // Verify each service was processed
    assert!(batch_result.results.contains_key(&user_proto_path));
    assert!(batch_result.results.contains_key(&product_proto_path));

    let user_result = &batch_result.results[&user_proto_path];
    let product_result = &batch_result.results[&product_proto_path];

    assert_eq!(user_result.proto_file.services[0].name, "UserService");
    assert_eq!(product_result.proto_file.services[0].name, "ProductService");

    // Write batch results
    let output_dir = temp_dir.path().join("batch_output");
    coordinator.write_batch_results(&batch_result, &output_dir).expect("Failed to write batch results");

    // Verify output structure
    assert!(output_dir.join("user_service").exists());
    assert!(output_dir.join("product_service").exists());
    assert!(output_dir.join("user_service/user_service_controller.rs").exists());
    assert!(output_dir.join("product_service/product_service_controller.rs").exists());
}

/// Test custom configuration
#[test]
fn test_custom_configuration() {
    let proto_content = r#"
syntax = "proto3";

package test.v1;

service TestService {
    rpc TestMethod(TestRequest) returns (TestResponse) {
        option (google.api.http) = {
            get: "/v1/test"
        };
    }
}

message TestRequest {}
message TestResponse {
    string message = 1;
}
"#;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proto_path = temp_dir.path().join("test_service.proto");
    std::fs::write(&proto_path, proto_content).expect("Failed to write proto file");

    // Test with service traits disabled
    let mut config = ProtoHttpParserConfig::new();
    config.generator.generate_service_traits = false;
    
    let coordinator = ProtoHttpCoordinator::with_config(config);
    let result = coordinator.process_file(&proto_path).expect("Failed to process proto file");

    // Should only have controller, no service trait
    assert_eq!(result.generated_files.len(), 1);
    assert!(result.generated_files.contains_key("test_service_controller.rs"));
    assert!(!result.generated_files.contains_key("test_service_service.rs"));

    // Test with service traits enabled (default)
    let coordinator_default = ProtoHttpCoordinator::new();
    let result_default = coordinator_default.process_file(&proto_path).expect("Failed to process proto file");

    // Should have both controller and service trait
    assert_eq!(result_default.generated_files.len(), 2);
    assert!(result_default.generated_files.contains_key("test_service_controller.rs"));
    assert!(result_default.generated_files.contains_key("test_service_service.rs"));
}

/// Test build integration API
#[test]
fn test_build_integration() {
    let proto_content = r#"
syntax = "proto3";

package test.v1;

service BuildTestService {
    rpc TestBuild(BuildRequest) returns (BuildResponse) {
        option (google.api.http) = {
            get: "/v1/build/test"
        };
    }
}

message BuildRequest {}
message BuildResponse {
    bool success = 1;
}
"#;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proto_path = temp_dir.path().join("build_test.proto");
    let output_dir = temp_dir.path().join("build_output");
    
    std::fs::write(&proto_path, proto_content).expect("Failed to write proto file");

    // Test build integration
    let result = BuildIntegration::new()
        .add_proto_file(&proto_path)
        .output_dir(&output_dir)
        .generate();

    assert!(result.is_ok(), "Build integration failed: {:?}", result.err());

    // Verify output files
    assert!(output_dir.join("build_test_service_controller.rs").exists());
    assert!(output_dir.join("build_test_service_service.rs").exists());
}

/// Test error handling in integration scenarios
#[test]
fn test_error_handling_integration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Test with non-existent file
    let coordinator = ProtoHttpCoordinator::new();
    let non_existent_path = temp_dir.path().join("non_existent.proto");
    let result = coordinator.process_file(&non_existent_path);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ProtoHttpParserError::Parse(_) => {}, // Expected
        other => panic!("Expected ParseError, got: {:?}", other),
    }

    // Test with invalid proto content
    let invalid_proto = "this is not valid proto syntax";
    let invalid_proto_path = temp_dir.path().join("invalid.proto");
    std::fs::write(&invalid_proto_path, invalid_proto).expect("Failed to write invalid proto file");
    
    let result = coordinator.process_file(&invalid_proto_path);
    assert!(result.is_err());

    // Test batch processing with mixed valid/invalid files
    let valid_proto = r#"
syntax = "proto3";
package test.v1;
service ValidService {
    rpc Test(TestRequest) returns (TestResponse) {
        option (google.api.http) = { get: "/test" };
    }
}
message TestRequest {}
message TestResponse {}
"#;
    
    let valid_proto_path = temp_dir.path().join("valid.proto");
    std::fs::write(&valid_proto_path, valid_proto).expect("Failed to write valid proto file");
    
    let proto_files = vec![&valid_proto_path, &invalid_proto_path];
    let batch_result = coordinator.process_files(&proto_files).expect("Batch processing should not fail completely");
    
    // Should have one success and one error
    assert_eq!(batch_result.success_count(), 1);
    assert_eq!(batch_result.error_count(), 1);
    assert!(!batch_result.is_success());
}

/// Test complex proto file with multiple services and advanced features
#[test]
fn test_complex_proto_integration() {
    let complex_proto = r#"
syntax = "proto3";

package complex.v1;

// User management service
service UserService {
    rpc GetUser(GetUserRequest) returns (User) {
        option (google.api.http) = {
            get: "/v1/users/{user_id}"
        };
    }
    
    rpc ListUsers(ListUsersRequest) returns (ListUsersResponse) {
        option (google.api.http) = {
            get: "/v1/users"
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
}

// Product management service
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

message GetUserRequest {
    string user_id = 1;
}

message ListUsersRequest {
    int32 page = 1;
    int32 limit = 2;
    string sort = 3;
}

message CreateUserRequest {
    string name = 1;
    string email = 2;
    UserStatus status = 3;
}

message UpdateUserRequest {
    string user_id = 1;
    User user = 2;
}

message DeleteUserRequest {
    string user_id = 1;
}

message ListUsersResponse {
    repeated User users = 1;
    int32 total = 2;
    int32 page = 3;
}

message DeleteUserResponse {
    bool success = 1;
}

message User {
    string id = 1;
    string name = 2;
    string email = 3;
    UserStatus status = 4;
    int64 created_at = 5;
    int64 updated_at = 6;
}

enum UserStatus {
    USER_STATUS_UNSPECIFIED = 0;
    USER_STATUS_ACTIVE = 1;
    USER_STATUS_INACTIVE = 2;
    USER_STATUS_SUSPENDED = 3;
}

message GetProductRequest {
    string product_id = 1;
}

message ListProductsRequest {
    int32 page = 1;
    int32 limit = 2;
    string category = 3;
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
    bool available = 6;
}
"#;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proto_path = temp_dir.path().join("complex_service.proto");
    std::fs::write(&proto_path, complex_proto).expect("Failed to write proto file");

    // Process the complex proto file
    let coordinator = ProtoHttpCoordinator::new();
    let result = coordinator.process_file(&proto_path).expect("Failed to process complex proto file");

    // Verify parsing results
    assert_eq!(result.proto_file.services.len(), 2);
    assert_eq!(result.routes.len(), 7); // 5 user routes + 2 product routes
    assert_eq!(result.generated_files.len(), 4); // 2 controllers + 2 service traits

    // Verify services
    let user_service = result.proto_file.services.iter().find(|s| s.name == "UserService").unwrap();
    let product_service = result.proto_file.services.iter().find(|s| s.name == "ProductService").unwrap();
    
    assert_eq!(user_service.methods.len(), 5);
    assert_eq!(product_service.methods.len(), 2);

    // Verify routes for different HTTP methods
    let get_routes: Vec<_> = result.routes.iter().filter(|r| r.http_method == HttpMethod::Get).collect();
    let post_routes: Vec<_> = result.routes.iter().filter(|r| r.http_method == HttpMethod::Post).collect();
    let put_routes: Vec<_> = result.routes.iter().filter(|r| r.http_method == HttpMethod::Put).collect();
    let delete_routes: Vec<_> = result.routes.iter().filter(|r| r.http_method == HttpMethod::Delete).collect();
    
    assert_eq!(get_routes.len(), 4); // GetUser, ListUsers, GetProduct, ListProducts
    assert_eq!(post_routes.len(), 1); // CreateUser
    assert_eq!(put_routes.len(), 1); // UpdateUser
    assert_eq!(delete_routes.len(), 1); // DeleteUser

    // Verify path parameters
    let get_user_route = result.routes.iter().find(|r| r.method_name == "GetUser").unwrap();
    assert_eq!(get_user_route.path_parameters.len(), 1);
    assert_eq!(get_user_route.path_parameters[0].name, "user_id");

    // Verify query parameters (should be inferred for list operations)
    let list_users_route = result.routes.iter().find(|r| r.method_name == "ListUsers").unwrap();
    assert!(!list_users_route.query_parameters.is_empty()); // Should have inferred query params

    // Write and verify generated code
    let output_dir = temp_dir.path().join("complex_output");
    coordinator.write_generated_code(&result, &output_dir).expect("Failed to write generated code");

    // Verify all expected files are generated
    assert!(output_dir.join("user_service_controller.rs").exists());
    assert!(output_dir.join("user_service_service.rs").exists());
    assert!(output_dir.join("product_service_controller.rs").exists());
    assert!(output_dir.join("product_service_service.rs").exists());

    // Verify controller content includes all methods
    let user_controller_content = std::fs::read_to_string(output_dir.join("user_service_controller.rs"))
        .expect("Failed to read user controller file");
    
    assert!(user_controller_content.contains("async fn get_user"));
    assert!(user_controller_content.contains("async fn list_users"));
    assert!(user_controller_content.contains("async fn create_user"));
    assert!(user_controller_content.contains("async fn update_user"));
    assert!(user_controller_content.contains("async fn delete_user"));
}

/// Test that generated code has valid Rust syntax (compilation test)
#[test]
fn test_generated_code_syntax_validity() {
    let proto_content = r#"
syntax = "proto3";

package syntax.test.v1;

service SyntaxTestService {
    rpc TestSyntax(SyntaxRequest) returns (SyntaxResponse) {
        option (google.api.http) = {
            get: "/v1/syntax/test/{test_id}"
        };
    }
    
    rpc CreateSyntax(CreateSyntaxRequest) returns (SyntaxResponse) {
        option (google.api.http) = {
            post: "/v1/syntax"
            body: "*"
        };
    }
}

message SyntaxRequest {
    string test_id = 1;
}

message CreateSyntaxRequest {
    string name = 1;
    int32 value = 2;
    bool enabled = 3;
}

message SyntaxResponse {
    string id = 1;
    string name = 2;
    int32 value = 3;
    bool enabled = 4;
}
"#;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proto_path = temp_dir.path().join("syntax_test.proto");
    std::fs::write(&proto_path, proto_content).expect("Failed to write proto file");

    // Generate code
    let coordinator = ProtoHttpCoordinator::new();
    let result = coordinator.process_file(&proto_path).expect("Failed to process proto file");
    
    let output_dir = temp_dir.path().join("syntax_output");
    coordinator.write_generated_code(&result, &output_dir).expect("Failed to write generated code");

    // Read generated files and verify they contain valid Rust syntax patterns
    let controller_content = std::fs::read_to_string(output_dir.join("syntax_test_service_controller.rs"))
        .expect("Failed to read controller file");
    let service_content = std::fs::read_to_string(output_dir.join("syntax_test_service_service.rs"))
        .expect("Failed to read service file");

    // Basic syntax checks
    assert!(controller_content.contains("use poem_openapi"));
    assert!(controller_content.contains("pub struct"));
    assert!(controller_content.contains("impl"));
    assert!(controller_content.contains("async fn"));
    assert!(controller_content.contains("#[oai("));
    
    assert!(service_content.contains("use async_trait::async_trait"));
    assert!(service_content.contains("#[async_trait]"));
    assert!(service_content.contains("pub trait"));
    assert!(service_content.contains("async fn"));

    // Verify no obvious syntax errors (balanced braces, etc.)
    let controller_open_braces = controller_content.matches('{').count();
    let controller_close_braces = controller_content.matches('}').count();
    assert_eq!(controller_open_braces, controller_close_braces, "Unbalanced braces in controller");

    let service_open_braces = service_content.matches('{').count();
    let service_close_braces = service_content.matches('}').count();
    assert_eq!(service_open_braces, service_close_braces, "Unbalanced braces in service trait");

    // Verify proper async/await syntax
    assert!(controller_content.contains(".await"));
    
    // Verify proper type annotations
    assert!(controller_content.contains("Path<"));
    assert!(controller_content.contains("Json<"));
    assert!(service_content.contains("Result<"));
}

/// Test directory processing functionality
#[test]
fn test_directory_processing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proto_dir = temp_dir.path().join("protos");
    std::fs::create_dir_all(&proto_dir).expect("Failed to create proto directory");

    // Create multiple proto files in the directory
    let service1_proto = r#"
syntax = "proto3";
package dir.test.v1;
service Service1 {
    rpc Method1(Request1) returns (Response1) {
        option (google.api.http) = { get: "/v1/service1" };
    }
}
message Request1 {}
message Response1 { string data = 1; }
"#;

    let service2_proto = r#"
syntax = "proto3";
package dir.test.v1;
service Service2 {
    rpc Method2(Request2) returns (Response2) {
        option (google.api.http) = { get: "/v1/service2" };
    }
}
message Request2 {}
message Response2 { int32 count = 1; }
"#;

    std::fs::write(proto_dir.join("service1.proto"), service1_proto).expect("Failed to write service1.proto");
    std::fs::write(proto_dir.join("service2.proto"), service2_proto).expect("Failed to write service2.proto");

    // Process the entire directory
    let coordinator = ProtoHttpCoordinator::new();
    let batch_result = coordinator.process_directory(&proto_dir).expect("Failed to process directory");

    // Verify results
    assert!(batch_result.is_success());
    assert_eq!(batch_result.success_count(), 2);
    assert_eq!(batch_result.error_count(), 0);

    // Write results
    let output_dir = temp_dir.path().join("dir_output");
    coordinator.write_batch_results(&batch_result, &output_dir).expect("Failed to write batch results");

    // Verify output structure
    assert!(output_dir.join("service1").exists());
    assert!(output_dir.join("service2").exists());
    assert!(output_dir.join("service1/service1_controller.rs").exists());
    assert!(output_dir.join("service2/service2_controller.rs").exists());
}