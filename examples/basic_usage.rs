//! Basic usage example for proto-http-parser-v2

use proto_http_parser::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Proto HTTP Parser v2 - Basic Usage Example");
    
    // Example proto content
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
}

message GetUserRequest {
    string user_id = 1;
}

message CreateUserRequest {
    User user = 1;
}

message GetUserResponse {
    User user = 1;
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

    // Create parser
    let parser = NomProtoParser::new();
    
    // Parse the proto content
    println!("Parsing Protocol Buffer content...");
    let proto_file = parser.parse_content(proto_content)?;
    
    // Display parsed information
    println!("✓ Successfully parsed proto file!");
    println!("  Syntax: {:?}", proto_file.syntax);
    println!("  Package: {:?}", proto_file.package);
    println!("  Imports: {} files", proto_file.imports.len());
    println!("  Services: {} services", proto_file.services.len());
    println!("  Messages: {} messages", proto_file.messages.len());
    println!("  Enums: {} enums", proto_file.enums.len());
    
    // Display service information
    for service in &proto_file.services {
        println!("\nService: {}", service.name);
        for method in &service.methods {
            println!("  Method: {} ({} -> {})", 
                method.name, 
                method.input_type.name, 
                method.output_type.name
            );
            
            if let Some(ref http_annotation) = method.http_annotation {
                println!("    HTTP: {} {}", 
                    http_annotation.method.as_str(), 
                    http_annotation.path
                );
                if let Some(ref body) = http_annotation.body {
                    println!("    Body: {}", body);
                }
            }
        }
    }
    
    // Display message information
    println!("\nMessages:");
    for message in &proto_file.messages {
        println!("  {}: {} fields", message.name, message.fields.len());
        for field in &message.fields {
            println!("    {}: {:?} (field {})", 
                field.name, 
                field.field_type, 
                field.number
            );
        }
    }
    
    // Display enum information
    println!("\nEnums:");
    for enum_def in &proto_file.enums {
        println!("  {}: {} values", enum_def.name, enum_def.values.len());
        for value in &enum_def.values {
            println!("    {} = {}", value.name, value.number);
        }
    }
    
    println!("\n✓ Parser demonstration complete!");
    
    Ok(())
}