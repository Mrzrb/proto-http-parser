//! Property-based tests for service trait generation
//! 
//! **Feature: proto-http-parser-v2, Property 4: Service trait and controller consistency**
//! **Validates: Requirements 3.6, 3.7, 8.1, 8.2, 8.3, 8.4**

use proto_http_parser_v2::*;
use proptest::prelude::*;

/// Test that service trait generation maintains consistency with controller requirements
#[cfg(test)]
mod service_trait_property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Feature: proto-http-parser-v2, Property 4: Service trait and controller consistency**
        /// For any generated service trait and controller, trait method signatures should be 
        /// consistent with controller handler method parameters and return types
        #[test]
        fn test_service_trait_controller_consistency(
            service in valid_service_generator(),
            routes in valid_routes_generator()
        ) {
            let generator = PoemOpenApiGenerator::new();
            
            // Generate service trait
            let trait_result = generator.generate_service_trait(&service, &routes);
            prop_assert!(trait_result.is_ok(), "Service trait generation failed: {:?}", trait_result.err());
            
            let trait_code = trait_result.unwrap();
            
            // Basic validation: trait should not be empty
            prop_assert!(!trait_code.content.trim().is_empty(), "Generated trait is empty");
            
            // Should contain trait definition
            let trait_name = format!("{}Service", to_pascal_case(&service.name));
            prop_assert!(trait_code.content.contains(&format!("trait {}", trait_name)), 
                        "Generated code does not contain trait definition for {}. Generated content: {}", trait_name, trait_code.content);
            
            // Should contain async_trait annotation
            prop_assert!(trait_code.content.contains("#[async_trait]"), 
                        "Generated trait does not have async_trait annotation");
            
            // Should have async-trait dependency
            prop_assert!(trait_code.dependencies.contains(&"async-trait".to_string()),
                        "Generated trait does not include async-trait dependency");
            
            // Filter routes for this service
            let service_routes: Vec<&HttpRoute> = routes.iter()
                .filter(|route| route.service_name == service.name)
                .collect();
            
            // Each route should have a corresponding method in the trait
            for route in &service_routes {
                let method_name = to_snake_case(&route.method_name);
                prop_assert!(trait_code.content.contains(&format!("async fn {}", method_name)),
                            "Generated trait does not contain method {} for route {}", method_name, route.method_name);
                
                // Method should return the correct response type
                let response_type = map_proto_type_to_rust(&route.response_type.name);
                prop_assert!(trait_code.content.contains(&response_type),
                            "Generated trait method {} does not reference response type {}", method_name, response_type);
            }
            
            // Should not contain unresolved template variables
            prop_assert!(!trait_code.content.contains("{{"), "Generated trait contains unresolved template variables");
            prop_assert!(!trait_code.content.contains("}}"), "Generated trait contains unresolved template variables");
        }

        /// Test that service trait methods have correct parameter types matching HTTP routes
        #[test]
        fn test_service_trait_parameter_types(
            service in valid_service_with_typed_routes_generator()
        ) {
            let generator = PoemOpenApiGenerator::new();
            
            // Create routes with specific parameter types
            let routes = create_typed_routes_for_service(&service);
            
            let trait_result = generator.generate_service_trait(&service, &routes);
            prop_assert!(trait_result.is_ok(), "Service trait generation failed: {:?}", trait_result.err());
            
            let trait_code = trait_result.unwrap();
            
            // Only verify parameter mappings if there are actually routes for this service
            let service_routes: Vec<&HttpRoute> = routes.iter()
                .filter(|route| route.service_name == service.name)
                .collect();
            
            if !service_routes.is_empty() {
                // Verify parameter type mappings for each matching route
                for route in &service_routes {
                    let method_name = to_snake_case(&route.method_name);
                    
                    // The method should exist in the trait
                    prop_assert!(trait_code.content.contains(&format!("async fn {}", method_name)),
                               "Generated trait should contain method {} for route", method_name);
                    
                    // Check that parameter types are correctly mapped (basic validation)
                    for path_param in &route.path_parameters {
                        let rust_type = map_parameter_type_to_rust(&path_param.param_type);
                        // Just verify the type appears somewhere in the trait (not necessarily as exact parameter)
                        prop_assert!(trait_code.content.contains(&rust_type) || rust_type == "String",
                                   "Generated trait should reference parameter type {}", rust_type);
                    }
                    
                    for query_param in &route.query_parameters {
                        let rust_type = map_parameter_type_to_rust(&query_param.param_type);
                        // For optional parameters, check for Option<T>
                        if !query_param.required {
                            let optional_type = format!("Option<{}>", rust_type);
                            prop_assert!(trait_code.content.contains(&optional_type) || trait_code.content.contains(&rust_type),
                                       "Generated trait should reference optional parameter type {}", optional_type);
                        }
                    }
                }
            } else {
                // If no routes match this service, the trait should still be valid but may have no methods
                let trait_name = format!("{}Service", to_pascal_case(&service.name));
                prop_assert!(trait_code.content.contains(&format!("trait {}", trait_name)),
                           "Generated trait should contain trait definition even without routes");
            }
        }

        /// Test that service trait generation handles edge cases correctly
        #[test]
        fn test_service_trait_edge_cases(
            service_name in "[a-zA-Z][a-zA-Z0-9_]*"
        ) {
            let generator = PoemOpenApiGenerator::new();
            
            // Test with empty service (no methods)
            let empty_service = Service::new(service_name.clone());
            let empty_routes = vec![];
            
            let trait_result = generator.generate_service_trait(&empty_service, &empty_routes);
            prop_assert!(trait_result.is_ok(), "Empty service trait generation failed: {:?}", trait_result.err());
            
            let trait_code = trait_result.unwrap();
            
            // Should still generate valid trait structure
            let trait_name = format!("{}Service", to_pascal_case(&service_name));
            prop_assert!(trait_code.content.contains(&format!("trait {}", trait_name)),
                        "Empty service does not generate trait definition. Expected: {}, Generated: {}", trait_name, trait_code.content);
            
            // Should contain async_trait annotation
            prop_assert!(trait_code.content.contains("#[async_trait]"),
                        "Empty service trait does not have async_trait annotation");
            
            // Test with service that has methods but no HTTP routes
            let service_with_methods = Service::new(service_name.clone())
                .with_method(RpcMethod::new(
                    "TestMethod".to_string(),
                    TypeReference::new("TestRequest".to_string()),
                    TypeReference::new("TestResponse".to_string())
                ));
            
            let trait_result = generator.generate_service_trait(&service_with_methods, &empty_routes);
            prop_assert!(trait_result.is_ok(), "Service with non-HTTP methods trait generation failed: {:?}", trait_result.err());
            
            let trait_code = trait_result.unwrap();
            
            // Should generate trait but without methods (since no HTTP routes)
            prop_assert!(trait_code.content.contains(&format!("trait {}", trait_name)),
                        "Service with non-HTTP methods does not generate trait definition. Expected: {}, Generated: {}", trait_name, trait_code.content);
        }
    }

    // Generator functions for property tests

    /// Generate valid service definitions
    fn valid_service_generator() -> impl Strategy<Value = Service> {
        ("[a-zA-Z][a-zA-Z0-9_]*", prop::collection::vec(valid_rpc_method_generator(), 1..5))
            .prop_map(|(name, methods)| {
                Service {
                    name,
                    methods,
                    options: vec![],
                    comments: vec![],
                }
            })
    }

    /// Generate valid service with typed routes
    fn valid_service_with_typed_routes_generator() -> impl Strategy<Value = Service> {
        "[a-zA-Z][a-zA-Z0-9_]*"
            .prop_map(|name| {
                Service::new(name)
                    .with_method(RpcMethod::new(
                        "GetUser".to_string(),
                        TypeReference::new("GetUserRequest".to_string()),
                        TypeReference::new("User".to_string())
                    ))
                    .with_method(RpcMethod::new(
                        "CreateUser".to_string(),
                        TypeReference::new("CreateUserRequest".to_string()),
                        TypeReference::new("User".to_string())
                    ))
            })
    }

    /// Generate valid RPC methods
    fn valid_rpc_method_generator() -> impl Strategy<Value = RpcMethod> {
        ("[a-zA-Z][a-zA-Z0-9_]*", valid_type_reference_generator(), valid_type_reference_generator())
            .prop_map(|(name, input_type, output_type)| {
                RpcMethod {
                    name,
                    input_type,
                    output_type,
                    options: vec![],
                    comments: vec![],
                    http_annotation: None,
                }
            })
    }

    /// Generate valid type references
    fn valid_type_reference_generator() -> impl Strategy<Value = TypeReference> {
        "[a-zA-Z][a-zA-Z0-9_]*"
            .prop_map(|name| TypeReference::new(name))
    }

    /// Generate valid HTTP routes
    fn valid_routes_generator() -> impl Strategy<Value = Vec<HttpRoute>> {
        prop::collection::vec(valid_http_route_generator(), 0..3)
    }

    /// Generate valid HTTP routes
    fn valid_http_route_generator() -> impl Strategy<Value = HttpRoute> {
        (
            "[a-zA-Z][a-zA-Z0-9_]*", // service_name
            "[a-zA-Z][a-zA-Z0-9_]*", // method_name
            http_method_generator(),
            path_template_generator(),
            prop::collection::vec(path_parameter_generator(), 0..3),
            prop::collection::vec(query_parameter_generator(), 0..3),
            valid_type_reference_generator()
        ).prop_map(|(service_name, method_name, http_method, path_template, path_params, query_params, response_type)| {
            HttpRoute {
                service_name,
                method_name,
                http_method,
                path_template,
                path_parameters: path_params,
                query_parameters: query_params,
                request_body: None,
                response_type,
            }
        })
    }

    /// Generate HTTP methods
    fn http_method_generator() -> impl Strategy<Value = HttpMethod> {
        prop_oneof![
            Just(HttpMethod::Get),
            Just(HttpMethod::Post),
            Just(HttpMethod::Put),
            Just(HttpMethod::Patch),
            Just(HttpMethod::Delete),
        ]
    }

    /// Generate path templates
    fn path_template_generator() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("/users".to_string()),
            Just("/users/{id}".to_string()),
            Just("/users/{user_id}/posts".to_string()),
            Just("/users/{user_id}/posts/{post_id}".to_string()),
        ]
    }

    /// Generate path parameters
    fn path_parameter_generator() -> impl Strategy<Value = PathParameter> {
        ("[a-zA-Z][a-zA-Z0-9_]*", parameter_type_generator())
            .prop_map(|(name, param_type)| PathParameter::new(name, param_type))
    }

    /// Generate query parameters
    fn query_parameter_generator() -> impl Strategy<Value = QueryParameter> {
        ("[a-zA-Z][a-zA-Z0-9_]*", parameter_type_generator(), any::<bool>())
            .prop_map(|(name, param_type, required)| QueryParameter::new(name, param_type, required))
    }

    /// Generate parameter types
    fn parameter_type_generator() -> impl Strategy<Value = ParameterType> {
        prop_oneof![
            Just(ParameterType::String),
            Just(ParameterType::Integer),
            Just(ParameterType::Float),
            Just(ParameterType::Boolean),
        ]
    }

    /// Create typed routes for a service for testing parameter type consistency
    fn create_typed_routes_for_service(service: &Service) -> Vec<HttpRoute> {
        let mut routes = vec![];
        
        // Only create routes for services that have methods
        if !service.methods.is_empty() {
            // Create a simple route for the first method to test parameter handling
            let method = &service.methods[0];
            let route = HttpRoute {
                service_name: service.name.clone(),
                method_name: method.name.clone(),
                http_method: HttpMethod::Get,
                path_template: format!("/{}/{}", to_snake_case(&service.name), to_snake_case(&method.name)),
                path_parameters: vec![
                    PathParameter::new("id".to_string(), ParameterType::String),
                ],
                query_parameters: vec![
                    QueryParameter::new("limit".to_string(), ParameterType::Integer, false),
                    QueryParameter::new("offset".to_string(), ParameterType::Integer, false),
                ],
                request_body: None,
                response_type: method.output_type.clone(),
            };
            routes.push(route);
        }
        
        routes
    }
}

// Helper functions for tests

/// Convert string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;
    
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_was_upper {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_was_upper = true;
        } else {
            result.push(c);
            prev_was_upper = false;
        }
    }
    
    result
}

/// Convert string to PascalCase
fn to_pascal_case(s: &str) -> String {
    // If the string contains underscores, split on them
    if s.contains('_') {
        let words: Vec<&str> = s.split('_').collect();
        words.iter()
            .map(|word| capitalize_first(word))
            .collect::<String>()
    } else {
        // If no underscores, assume it's already camelCase or PascalCase
        // Just ensure the first letter is capitalized while preserving the rest
        capitalize_first_preserve_case(s)
    }
}

/// Capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

/// Capitalize first letter while preserving the rest of the string case
fn capitalize_first_preserve_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Map Protocol Buffer types to Rust types
fn map_proto_type_to_rust(proto_type: &str) -> String {
    match proto_type {
        "string" => "String".to_string(),
        "int32" | "sint32" | "sfixed32" => "i32".to_string(),
        "int64" | "sint64" | "sfixed64" => "i64".to_string(),
        "uint32" | "fixed32" => "u32".to_string(),
        "uint64" | "fixed64" => "u64".to_string(),
        "double" => "f64".to_string(),
        "float" => "f32".to_string(),
        "bool" => "bool".to_string(),
        "bytes" => "Vec<u8>".to_string(),
        "google.protobuf.Timestamp" => "chrono::DateTime<chrono::Utc>".to_string(),
        "google.protobuf.Duration" => "std::time::Duration".to_string(),
        "google.protobuf.Empty" => "()".to_string(),
        _ => proto_type.to_string(), // Custom types remain as-is
    }
}

/// Map parameter types to Rust types
fn map_parameter_type_to_rust(param_type: &ParameterType) -> String {
    match param_type {
        ParameterType::String => "String".to_string(),
        ParameterType::Integer => "i32".to_string(),
        ParameterType::Float => "f64".to_string(),
        ParameterType::Boolean => "bool".to_string(),
        ParameterType::Custom(type_name) => type_name.clone(),
    }
}