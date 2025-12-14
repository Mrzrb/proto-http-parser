//! Example demonstrating the Handlebars template engine usage

use proto_http_parser_v2::*;
use std::collections::HashMap;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create a template engine
    let mut engine = HandlebarsTemplateEngine::new();
    
    // Create a sample service
    let service = Service::new("UserService".to_string())
        .with_method(RpcMethod::new(
            "GetUser".to_string(),
            TypeReference::new("GetUserRequest".to_string()),
            TypeReference::new("User".to_string()),
        ));
    
    // Create sample routes
    let routes = vec![
        HttpRoute::new(
            "UserService".to_string(),
            "GetUser".to_string(),
            HttpMethod::Get,
            "/users/{id}".to_string(),
        )
        .with_path_parameter(PathParameter::new("id".to_string(), ParameterType::String))
        .with_response_type(TypeReference::new("User".to_string())),
    ];
    
    // Create template context
    let mut custom_data = HashMap::new();
    custom_data.insert("imports".to_string(), TemplateValue::Array(vec![
        TemplateValue::String("std::collections::HashMap".to_string()),
        TemplateValue::String("serde::{Serialize, Deserialize}".to_string()),
    ]));
    
    let context = TemplateContext {
        service: service.clone(),
        routes: routes.clone(),
        custom_data,
    };
    
    // Render built-in templates
    println!("=== Controller Template ===");
    let controller = engine.render("controller", &context)?;
    println!("{}", controller);
    
    println!("\n=== Service Trait Template ===");
    let service_trait = engine.render("service_trait", &context)?;
    println!("{}", service_trait);
    
    println!("\n=== Imports Template ===");
    let imports = engine.render("imports", &context)?;
    println!("{}", imports);
    
    // Demonstrate custom template
    let custom_template = r#"
// Custom template for {{service.name}}
pub mod {{snake_case service.name}} {
    {{#each routes}}
    /// {{method_name}} handler
    pub async fn {{snake_case method_name}}() -> String {
        "{{pascal_case ../service.name}}::{{method_name}}".to_string()
    }
    {{/each}}
}
"#;
    
    engine.register_template("custom", custom_template)?;
    
    println!("\n=== Custom Template ===");
    let custom_result = engine.render("custom", &context)?;
    println!("{}", custom_result);
    
    // Demonstrate helper functions
    println!("\n=== Helper Functions Demo ===");
    let helper_template = r#"
Original: {{service.name}}
Snake Case: {{snake_case service.name}}
Camel Case: {{camel_case service.name}}
Pascal Case: {{pascal_case service.name}}
Type Mapping: {{map_type "string"}} -> {{map_type "int32"}} -> {{map_type "google.protobuf.Timestamp"}}
"#;
    
    engine.register_template("helpers", helper_template)?;
    let helpers_result = engine.render("helpers", &context)?;
    println!("{}", helpers_result);
    
    Ok(())
}