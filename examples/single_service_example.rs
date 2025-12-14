//! Single service example demonstrating basic usage with one service

use proto_http_parser::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Proto HTTP Parser v2 - Single Service Example");
    
    // Create a simple proto file with one service
    let proto_content = r#"
syntax = "proto3";

package user.v1;

import "google/api/annotations.proto";

// User management service
service UserService {
    // Get a user by ID
    rpc GetUser(GetUserRequest) returns (GetUserResponse) {
        option (google.api.http) = {
            get: "/v1/users/{user_id}"
        };
    }
    
    // Create a new user
    rpc CreateUser(CreateUserRequest) returns (User) {
        option (google.api.http) = {
            post: "/v1/users"
            body: "*"
        };
    }
    
    // Update an existing user
    rpc UpdateUser(UpdateUserRequest) returns (User) {
        option (google.api.http) = {
            put: "/v1/users/{user.id}"
            body: "user"
        };
    }
    
    // Delete a user
    rpc DeleteUser(DeleteUserRequest) returns (DeleteUserResponse) {
        option (google.api.http) = {
            delete: "/v1/users/{user_id}"
        };
    }
    
    // List users with pagination
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
    User user = 1;
}

message DeleteUserRequest {
    string user_id = 1;
}

message ListUsersRequest {
    int32 page_size = 1;
    string page_token = 2;
    string filter = 3;
}

message GetUserResponse {
    User user = 1;
}

message DeleteUserResponse {
    bool success = 1;
}

message ListUsersResponse {
    repeated User users = 1;
    string next_page_token = 2;
    int32 total_count = 3;
}

message User {
    string id = 1;
    string name = 2;
    string email = 3;
    Status status = 4;
    google.protobuf.Timestamp created_at = 5;
    google.protobuf.Timestamp updated_at = 6;
}

enum Status {
    STATUS_UNSPECIFIED = 0;
    STATUS_ACTIVE = 1;
    STATUS_INACTIVE = 2;
    STATUS_SUSPENDED = 3;
}
"#;

    // Step 1: Parse the proto file
    println!("\n=== Step 1: Parsing Protocol Buffer File ===");
    let parser = NomProtoParser::new();
    let proto_file = parser.parse_content(proto_content)?;
    
    println!("✓ Successfully parsed proto file!");
    println!("  Package: {}", proto_file.package.as_deref().unwrap_or("(none)"));
    println!("  Services: {}", proto_file.services.len());
    println!("  Messages: {}", proto_file.messages.len());
    println!("  Enums: {}", proto_file.enums.len());
    
    // Step 2: Extract HTTP routes
    println!("\n=== Step 2: Extracting HTTP Routes ===");
    let extractor = GoogleApiHttpExtractor::new();
    let routes = extractor.extract_routes(&proto_file)?;
    
    println!("✓ Successfully extracted HTTP routes!");
    println!("  Total routes: {}", routes.len());
    
    for route in &routes {
        println!("  {} {} -> {}::{}", 
            route.http_method.as_str(),
            route.path_template,
            route.service_name,
            route.method_name
        );
        
        if !route.path_parameters.is_empty() {
            println!("    Path params: {:?}", 
                route.path_parameters.iter()
                    .map(|p| &p.name)
                    .collect::<Vec<_>>()
            );
        }
        
        if !route.query_parameters.is_empty() {
            println!("    Query params: {:?}", 
                route.query_parameters.iter()
                    .map(|p| &p.name)
                    .collect::<Vec<_>>()
            );
        }
    }
    
    // Step 3: Generate code
    println!("\n=== Step 3: Generating Code ===");
    let generator = PoemOpenApiGenerator::new();
    
    // Generate for the single service
    let service = &proto_file.services[0];
    let service_routes: Vec<_> = routes.iter()
        .filter(|r| r.service_name == service.name)
        .cloned()
        .collect();
    
    // Generate controller
    let controller_code = generator.generate_controller(service, &service_routes)?;
    println!("✓ Generated controller code ({} lines)", 
        controller_code.content.lines().count());
    
    // Generate service trait
    let trait_code = generator.generate_service_trait(service, &service_routes)?;
    println!("✓ Generated service trait ({} lines)", 
        trait_code.content.lines().count());
    
    // Step 4: Display generated code samples
    println!("\n=== Step 4: Generated Code Preview ===");
    
    println!("\n--- Controller Code (first 20 lines) ---");
    for (i, line) in controller_code.content.lines().take(20).enumerate() {
        println!("{:3}: {}", i + 1, line);
    }
    if controller_code.content.lines().count() > 20 {
        println!("... ({} more lines)", controller_code.content.lines().count() - 20);
    }
    
    println!("\n--- Service Trait Code (first 15 lines) ---");
    for (i, line) in trait_code.content.lines().take(15).enumerate() {
        println!("{:3}: {}", i + 1, line);
    }
    if trait_code.content.lines().count() > 15 {
        println!("... ({} more lines)", trait_code.content.lines().count() - 15);
    }
    
    // Step 5: Save generated code to files
    println!("\n=== Step 5: Saving Generated Code ===");
    
    let temp_dir = tempfile::tempdir()?;
    let controller_path = temp_dir.path().join("user_service_controller.rs");
    let trait_path = temp_dir.path().join("user_service_trait.rs");
    
    std::fs::write(&controller_path, &controller_code.content)?;
    std::fs::write(&trait_path, &trait_code.content)?;
    
    println!("✓ Saved controller to: {}", controller_path.display());
    println!("✓ Saved service trait to: {}", trait_path.display());
    
    // Step 6: Validation and analysis
    println!("\n=== Step 6: Code Analysis ===");
    
    // Analyze the generated code
    let controller_imports = controller_code.imports.len();
    let trait_imports = trait_code.imports.len();
    
    println!("Controller analysis:");
    println!("  Imports: {}", controller_imports);
    println!("  Dependencies: {:?}", controller_code.dependencies);
    
    println!("Service trait analysis:");
    println!("  Imports: {}", trait_imports);
    println!("  Dependencies: {:?}", trait_code.dependencies);
    
    // Count methods in generated code
    let controller_methods = controller_code.content.matches("pub async fn").count();
    let trait_methods = trait_code.content.matches("async fn").count();
    
    println!("  Controller methods: {}", controller_methods);
    println!("  Trait methods: {}", trait_methods);
    
    println!("\n✓ Single service example completed successfully!");
    println!("  Generated {} controller methods and {} trait methods", 
        controller_methods, trait_methods);
    
    Ok(())
}