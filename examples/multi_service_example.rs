//! Multi-service example demonstrating batch generation from multiple services

use proto_http_parser::*;
use std::collections::HashMap;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Proto HTTP Parser v2 - Multi-Service Example");
    
    // Create a proto file with multiple services
    let proto_content = r#"
syntax = "proto3";

package ecommerce.v1;

import "google/api/annotations.proto";

// User management service
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
    
    rpc ListUsers(ListUsersRequest) returns (ListUsersResponse) {
        option (google.api.http) = {
            get: "/v1/users"
        };
    }
}

// Product catalog service
service ProductService {
    rpc GetProduct(GetProductRequest) returns (Product) {
        option (google.api.http) = {
            get: "/v1/products/{product_id}"
        };
    }
    
    rpc CreateProduct(CreateProductRequest) returns (Product) {
        option (google.api.http) = {
            post: "/v1/products"
            body: "*"
        };
    }
    
    rpc UpdateProduct(UpdateProductRequest) returns (Product) {
        option (google.api.http) = {
            put: "/v1/products/{product.id}"
            body: "product"
        };
    }
    
    rpc DeleteProduct(DeleteProductRequest) returns (DeleteProductResponse) {
        option (google.api.http) = {
            delete: "/v1/products/{product_id}"
        };
    }
    
    rpc ListProducts(ListProductsRequest) returns (ListProductsResponse) {
        option (google.api.http) = {
            get: "/v1/products"
        };
    }
    
    rpc SearchProducts(SearchProductsRequest) returns (SearchProductsResponse) {
        option (google.api.http) = {
            get: "/v1/products/search"
        };
    }
}

// Order management service
service OrderService {
    rpc GetOrder(GetOrderRequest) returns (Order) {
        option (google.api.http) = {
            get: "/v1/orders/{order_id}"
        };
    }
    
    rpc CreateOrder(CreateOrderRequest) returns (Order) {
        option (google.api.http) = {
            post: "/v1/orders"
            body: "*"
        };
    }
    
    rpc UpdateOrderStatus(UpdateOrderStatusRequest) returns (Order) {
        option (google.api.http) = {
            patch: "/v1/orders/{order_id}/status"
            body: "*"
        };
    }
    
    rpc ListOrders(ListOrdersRequest) returns (ListOrdersResponse) {
        option (google.api.http) = {
            get: "/v1/orders"
        };
    }
    
    rpc GetOrdersByUser(GetOrdersByUserRequest) returns (ListOrdersResponse) {
        option (google.api.http) = {
            get: "/v1/users/{user_id}/orders"
        };
    }
}

// User messages
message GetUserRequest {
    string user_id = 1;
}

message CreateUserRequest {
    string name = 1;
    string email = 2;
}

message ListUsersRequest {
    int32 page_size = 1;
    string page_token = 2;
}

message ListUsersResponse {
    repeated User users = 1;
    string next_page_token = 2;
}

message User {
    string id = 1;
    string name = 2;
    string email = 3;
    UserStatus status = 4;
}

enum UserStatus {
    USER_STATUS_UNSPECIFIED = 0;
    USER_STATUS_ACTIVE = 1;
    USER_STATUS_INACTIVE = 2;
}

// Product messages
message GetProductRequest {
    string product_id = 1;
}

message CreateProductRequest {
    string name = 1;
    string description = 2;
    double price = 3;
    string category_id = 4;
}

message UpdateProductRequest {
    Product product = 1;
}

message DeleteProductRequest {
    string product_id = 1;
}

message DeleteProductResponse {
    bool success = 1;
}

message ListProductsRequest {
    int32 page_size = 1;
    string page_token = 2;
    string category_id = 3;
}

message ListProductsResponse {
    repeated Product products = 1;
    string next_page_token = 2;
}

message SearchProductsRequest {
    string query = 1;
    int32 page_size = 2;
    string page_token = 3;
    repeated string categories = 4;
    double min_price = 5;
    double max_price = 6;
}

message SearchProductsResponse {
    repeated Product products = 1;
    string next_page_token = 2;
    int32 total_count = 3;
}

message Product {
    string id = 1;
    string name = 2;
    string description = 3;
    double price = 4;
    string category_id = 5;
    ProductStatus status = 6;
}

enum ProductStatus {
    PRODUCT_STATUS_UNSPECIFIED = 0;
    PRODUCT_STATUS_AVAILABLE = 1;
    PRODUCT_STATUS_OUT_OF_STOCK = 2;
    PRODUCT_STATUS_DISCONTINUED = 3;
}

// Order messages
message GetOrderRequest {
    string order_id = 1;
}

message CreateOrderRequest {
    string user_id = 1;
    repeated OrderItem items = 2;
    Address shipping_address = 3;
}

message UpdateOrderStatusRequest {
    string order_id = 1;
    OrderStatus status = 2;
}

message ListOrdersRequest {
    int32 page_size = 1;
    string page_token = 2;
    OrderStatus status = 3;
}

message GetOrdersByUserRequest {
    string user_id = 1;
    int32 page_size = 2;
    string page_token = 3;
}

message ListOrdersResponse {
    repeated Order orders = 1;
    string next_page_token = 2;
}

message Order {
    string id = 1;
    string user_id = 2;
    repeated OrderItem items = 3;
    double total_amount = 4;
    OrderStatus status = 5;
    Address shipping_address = 6;
    google.protobuf.Timestamp created_at = 7;
    google.protobuf.Timestamp updated_at = 8;
}

message OrderItem {
    string product_id = 1;
    int32 quantity = 2;
    double unit_price = 3;
}

message Address {
    string street = 1;
    string city = 2;
    string state = 3;
    string postal_code = 4;
    string country = 5;
}

enum OrderStatus {
    ORDER_STATUS_UNSPECIFIED = 0;
    ORDER_STATUS_PENDING = 1;
    ORDER_STATUS_CONFIRMED = 2;
    ORDER_STATUS_SHIPPED = 3;
    ORDER_STATUS_DELIVERED = 4;
    ORDER_STATUS_CANCELLED = 5;
}
"#;

    // Step 1: Parse the proto file
    println!("\n=== Step 1: Parsing Multi-Service Proto File ===");
    let parser = NomProtoParser::new();
    let proto_file = parser.parse_content(proto_content)?;
    
    println!("✓ Successfully parsed proto file!");
    println!("  Package: {}", proto_file.package.as_deref().unwrap_or("(none)"));
    println!("  Services: {}", proto_file.services.len());
    println!("  Messages: {}", proto_file.messages.len());
    println!("  Enums: {}", proto_file.enums.len());
    
    // Display service information
    for service in &proto_file.services {
        println!("  Service '{}': {} methods", service.name, service.methods.len());
    }
    
    // Step 2: Extract HTTP routes for all services
    println!("\n=== Step 2: Extracting HTTP Routes ===");
    let extractor = GoogleApiHttpExtractor::new();
    let all_routes = extractor.extract_routes(&proto_file)?;
    
    println!("✓ Successfully extracted HTTP routes!");
    println!("  Total routes: {}", all_routes.len());
    
    // Group routes by service
    let mut routes_by_service: HashMap<String, Vec<HttpRoute>> = HashMap::new();
    for route in all_routes {
        routes_by_service
            .entry(route.service_name.clone())
            .or_insert_with(Vec::new)
            .push(route);
    }
    
    // Display routes by service
    for (service_name, routes) in &routes_by_service {
        println!("\n  Service '{}' routes:", service_name);
        for route in routes {
            println!("    {} {} -> {}", 
                route.http_method.as_str(),
                route.path_template,
                route.method_name
            );
        }
    }
    
    // Step 3: Generate code for all services
    println!("\n=== Step 3: Batch Code Generation ===");
    let generator = PoemOpenApiGenerator::new();
    
    let mut generated_controllers = HashMap::new();
    let mut generated_traits = HashMap::new();
    
    for service in &proto_file.services {
        let service_routes = routes_by_service
            .get(&service.name)
            .cloned()
            .unwrap_or_default();
        
        println!("  Generating code for service '{}'...", service.name);
        
        // Generate controller
        let controller_code = generator.generate_controller(service, &service_routes)?;
        generated_controllers.insert(service.name.clone(), controller_code);
        
        // Generate service trait
        let trait_code = generator.generate_service_trait(service, &service_routes)?;
        generated_traits.insert(service.name.clone(), trait_code);
        
        println!("    ✓ Controller: {} lines", 
            generated_controllers[&service.name].content.lines().count());
        println!("    ✓ Trait: {} lines", 
            generated_traits[&service.name].content.lines().count());
    }
    
    // Step 4: Save all generated code
    println!("\n=== Step 4: Saving Generated Code ===");
    
    let temp_dir = tempfile::tempdir()?;
    let controllers_dir = temp_dir.path().join("controllers");
    let traits_dir = temp_dir.path().join("traits");
    
    std::fs::create_dir_all(&controllers_dir)?;
    std::fs::create_dir_all(&traits_dir)?;
    
    for (service_name, controller_code) in &generated_controllers {
        let filename = format!("{}_controller.rs", service_name.to_lowercase());
        let path = controllers_dir.join(&filename);
        std::fs::write(&path, &controller_code.content)?;
        println!("  ✓ Saved controller: {}", path.display());
    }
    
    for (service_name, trait_code) in &generated_traits {
        let filename = format!("{}_service.rs", service_name.to_lowercase());
        let path = traits_dir.join(&filename);
        std::fs::write(&path, &trait_code.content)?;
        println!("  ✓ Saved trait: {}", path.display());
    }
    
    // Step 5: Generate a combined module file
    println!("\n=== Step 5: Generating Module Files ===");
    
    let mut mod_file_content = String::new();
    mod_file_content.push_str("//! Generated API modules\n\n");
    
    // Controllers module
    mod_file_content.push_str("pub mod controllers {\n");
    for service_name in generated_controllers.keys() {
        let module_name = service_name.to_lowercase();
        mod_file_content.push_str(&format!("    pub mod {};\n", module_name));
    }
    mod_file_content.push_str("}\n\n");
    
    // Services module
    mod_file_content.push_str("pub mod services {\n");
    for service_name in generated_traits.keys() {
        let module_name = service_name.to_lowercase();
        mod_file_content.push_str(&format!("    pub mod {};\n", module_name));
    }
    mod_file_content.push_str("}\n\n");
    
    // Re-exports
    mod_file_content.push_str("// Re-exports for convenience\n");
    for service_name in generated_controllers.keys() {
        let module_name = service_name.to_lowercase();
        mod_file_content.push_str(&format!(
            "pub use controllers::{}::{}Controller;\n", 
            module_name, service_name
        ));
        mod_file_content.push_str(&format!(
            "pub use services::{}::{}Service;\n", 
            module_name, service_name
        ));
    }
    
    let mod_file_path = temp_dir.path().join("mod.rs");
    std::fs::write(&mod_file_path, &mod_file_content)?;
    println!("  ✓ Generated module file: {}", mod_file_path.display());
    
    // Step 6: Generate a main server file example
    println!("\n=== Step 6: Generating Server Example ===");
    
    let mut server_content = String::new();
    server_content.push_str("//! Example server using all generated services\n\n");
    server_content.push_str("use poem::Route;\n");
    server_content.push_str("use poem_openapi::OpenApiService;\n\n");
    
    // Import all controllers
    for service_name in generated_controllers.keys() {
        server_content.push_str(&format!("use crate::{}Controller;\n", service_name));
    }
    
    server_content.push_str("\npub fn create_api_service() -> OpenApiService<impl poem::Endpoint, ()> {\n");
    server_content.push_str("    OpenApiService::new(\n");
    server_content.push_str("        (\n");
    
    // Add all controllers
    for service_name in generated_controllers.keys() {
        server_content.push_str(&format!("            {}Controller::new(),\n", service_name));
    }
    
    server_content.push_str("        ),\n");
    server_content.push_str("        \"E-commerce API\",\n");
    server_content.push_str("        \"1.0.0\"\n");
    server_content.push_str("    )\n");
    server_content.push_str("    .server(\"http://localhost:3000\")\n");
    server_content.push_str("}\n\n");
    
    server_content.push_str("pub fn create_routes() -> Route {\n");
    server_content.push_str("    let api_service = create_api_service();\n");
    server_content.push_str("    Route::new()\n");
    server_content.push_str("        .nest(\"/api\", api_service)\n");
    server_content.push_str("        .nest(\"/docs\", api_service.swagger_ui())\n");
    server_content.push_str("}\n");
    
    let server_file_path = temp_dir.path().join("server.rs");
    std::fs::write(&server_file_path, &server_content)?;
    println!("  ✓ Generated server example: {}", server_file_path.display());
    
    // Step 7: Analysis and statistics
    println!("\n=== Step 7: Generation Statistics ===");
    
    let mut total_controller_lines = 0;
    let mut total_trait_lines = 0;
    let mut total_methods = 0;
    
    for (service_name, controller_code) in &generated_controllers {
        let controller_lines = controller_code.content.lines().count();
        let trait_lines = generated_traits[service_name].content.lines().count();
        let methods = controller_code.content.matches("pub async fn").count();
        
        total_controller_lines += controller_lines;
        total_trait_lines += trait_lines;
        total_methods += methods;
        
        println!("  Service '{}':", service_name);
        println!("    Controller: {} lines, {} methods", controller_lines, methods);
        println!("    Trait: {} lines", trait_lines);
    }
    
    println!("\n  Total Statistics:");
    println!("    Services: {}", proto_file.services.len());
    println!("    HTTP Routes: {}", routes_by_service.values().map(|v| v.len()).sum::<usize>());
    println!("    Controller Lines: {}", total_controller_lines);
    println!("    Trait Lines: {}", total_trait_lines);
    println!("    Generated Methods: {}", total_methods);
    
    // Step 8: Validate generated code structure
    println!("\n=== Step 8: Code Validation ===");
    
    let mut validation_errors = 0;
    
    for (service_name, controller_code) in &generated_controllers {
        // Check for required imports
        if !controller_code.content.contains("use poem_openapi") {
            println!("  ✗ Missing poem_openapi import in {}", service_name);
            validation_errors += 1;
        }
        
        // Check for controller struct
        if !controller_code.content.contains(&format!("pub struct {}Controller", service_name)) {
            println!("  ✗ Missing controller struct in {}", service_name);
            validation_errors += 1;
        }
        
        // Check trait code
        let trait_code = &generated_traits[service_name];
        if !trait_code.content.contains(&format!("pub trait {}Service", service_name)) {
            println!("  ✗ Missing service trait in {}", service_name);
            validation_errors += 1;
        }
    }
    
    if validation_errors == 0 {
        println!("  ✓ All generated code passed validation!");
    } else {
        println!("  ✗ Found {} validation errors", validation_errors);
    }
    
    println!("\n✓ Multi-service example completed successfully!");
    println!("  Generated code for {} services with {} total HTTP endpoints", 
        proto_file.services.len(), 
        routes_by_service.values().map(|v| v.len()).sum::<usize>()
    );
    
    Ok(())
}