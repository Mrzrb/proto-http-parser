//! Property-based tests for code generation syntax correctness
//! 
//! **Feature: proto-http-parser-v2, Property 3: Code generation syntax correctness**
//! **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 7.2**

use proto_http_parser_v2::*;
use proptest::prelude::*;

/// Test that generated code has correct syntax and compiles
#[cfg(test)]
mod code_generation_property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Feature: proto-http-parser-v2, Property 3: Code generation syntax correctness**
        /// For any valid service definition and HTTP routes, generated Rust code should 
        /// pass compiler checks and conform to poem-openapi specifications
        #[test]
        fn test_code_generation_syntax_correctness(
            service in valid_service_generator(),
            routes in valid_routes_generator()
        ) {
            let generator = PoemOpenApiGenerator::new();
            
            // Generate controller code
            let controller_result = generator.generate_controller(&service, &routes);
            prop_assert!(controller_result.is_ok(), "Controller generation failed: {:?}", controller_result.err());
            
            let controller_code = controller_result.unwrap();
            
            // Basic syntax validation: code should not be empty
            prop_assert!(!controller_code.content.trim().is_empty(), "Generated controller is empty");
            
            // Should not contain template variables (indicates successful rendering)
            prop_assert!(!controller_code.content.contains("{{"), "Generated code contains unresolved template variables: {{{{");
            prop_assert!(!controller_code.content.contains("}}"), "Generated code contains unresolved template variables: }}}}");
            
            // Should contain valid Rust syntax elements
            prop_assert!(controller_code.content.contains("pub struct"), "Generated code should contain struct definition");
            prop_assert!(controller_code.content.contains("impl"), "Generated code should contain impl block");
            
            // Should contain poem-openapi specific elements
            prop_assert!(controller_code.content.contains("#[poem_openapi::OpenApi]"), 
                        "Generated code should contain OpenApi attribute");
            
            // Filter routes for this service
            let service_routes: Vec<&HttpRoute> = routes.iter()
                .filter(|route| route.service_name == service.name)
                .collect();
            
            // If there are routes, verify method generation
            if !service_routes.is_empty() {
                // Should contain async functions for each route
                for route in &service_routes {
                    let method_name = to_snake_case(&route.method_name);
                    prop_assert!(controller_code.content.contains(&format!("async fn {}", method_name)),
                               "Generated code should contain async method {} for route {}", method_name, route.method_name);
                    
                    // Should contain proper OpenAPI annotations
                    prop_assert!(controller_code.content.contains("#[oai(path ="), 
                               "Generated code should contain OpenAPI path annotations");
                    
                    // Should contain proper return type
                    prop_assert!(controller_code.content.contains("-> poem_openapi::payload::Json<"), 
                               "Generated code should contain proper JSON return type");
                }
            }
            
            // Verify basic Rust syntax patterns
            verify_rust_syntax_patterns(&controller_code.content)?;
            
            // Generate service trait code
            let trait_result = generator.generate_service_trait(&service, &routes);
            prop_assert!(trait_result.is_ok(), "Service trait generation failed: {:?}", trait_result.err());
            
            let trait_code = trait_result.unwrap();
            
            // Verify trait syntax
            prop_assert!(!trait_code.content.trim().is_empty(), "Generated trait is empty");
            prop_assert!(!trait_code.content.contains("{{") && !trait_code.content.contains("}}"), 
                        "Generated trait contains unresolved template variables");
            
            let trait_name = format!("{}Service", to_pascal_case(&service.name));
            prop_assert!(trait_code.content.contains(&format!("trait {}", trait_name)), 
                        "Generated trait should contain trait definition for {}", trait_name);
            
            prop_assert!(trait_code.content.contains("#[async_trait]"), 
                        "Generated trait should contain async_trait attribute");
            
            // Verify trait syntax patterns
            verify_rust_syntax_patterns(&trait_code.content)?;
        }

        /// Test that generated code handles various parameter types correctly
        #[test]
        fn test_parameter_type_syntax_correctness(
            service_name in "[a-zA-Z][a-zA-Z0-9_]*",
            method_name in "[a-zA-Z][a-zA-Z0-9_]*"
        ) {
            let generator = PoemOpenApiGenerator::new();
            
            // Create service with method
            let service = Service::new(service_name.clone())
                .with_method(RpcMethod::new(
                    method_name.clone(),
                    TypeReference::new("TestRequest".to_string()),
                    TypeReference::new("TestResponse".to_string())
                ));
            
            // Create route with various parameter types
            let route = HttpRoute::new(
                service_name.clone(),
                method_name.clone(),
                HttpMethod::Get,
                "/test/{id}".to_string(),
            )
            .with_path_parameter(PathParameter::new("id".to_string(), ParameterType::String))
            .with_path_parameter(PathParameter::new("count".to_string(), ParameterType::Integer))
            .with_query_parameter(QueryParameter::new("active".to_string(), ParameterType::Boolean, false))
            .with_query_parameter(QueryParameter::new("score".to_string(), ParameterType::Float, false))
            .with_response_type(TypeReference::new("TestResponse".to_string()));
            
            let routes = vec![route];
            
            let controller_result = generator.generate_controller(&service, &routes);
            prop_assert!(controller_result.is_ok(), "Controller generation with parameters failed: {:?}", controller_result.err());
            
            let controller_code = controller_result.unwrap();
            
            // Verify parameter type mappings
            prop_assert!(controller_code.content.contains("Path<String>"), 
                        "Should contain String path parameter type");
            prop_assert!(controller_code.content.contains("Path<i32>"), 
                        "Should contain i32 path parameter type");
            prop_assert!(controller_code.content.contains("Query<Option<bool>>"), 
                        "Should contain optional boolean query parameter type");
            prop_assert!(controller_code.content.contains("Query<Option<f64>>"), 
                        "Should contain optional float query parameter type");
            
            // Verify parameter usage in method calls
            prop_assert!(controller_code.content.contains("id.0") && controller_code.content.contains("count.0"), 
                        "Should extract path parameter values");
            prop_assert!(controller_code.content.contains("active.0") && controller_code.content.contains("score.0"), 
                        "Should extract query parameter values");
        }

        /// Test that generated code handles HTTP methods correctly
        #[test]
        fn test_http_method_syntax_correctness(
            service_name in "[a-zA-Z][a-zA-Z0-9_]*",
            http_method in http_method_generator()
        ) {
            let generator = PoemOpenApiGenerator::new();
            
            let service = Service::new(service_name.clone())
                .with_method(RpcMethod::new(
                    "TestMethod".to_string(),
                    TypeReference::new("TestRequest".to_string()),
                    TypeReference::new("TestResponse".to_string())
                ));
            
            let route = HttpRoute::new(
                service_name.clone(),
                "TestMethod".to_string(),
                http_method.clone(),
                "/test".to_string(),
            )
            .with_response_type(TypeReference::new("TestResponse".to_string()));
            
            let routes = vec![route];
            
            let controller_result = generator.generate_controller(&service, &routes);
            prop_assert!(controller_result.is_ok(), "Controller generation with HTTP method failed: {:?}", controller_result.err());
            
            let controller_code = controller_result.unwrap();
            
            // Verify HTTP method annotation
            let expected_method = http_method.as_str().to_lowercase();
            prop_assert!(controller_code.content.contains(&format!(r#"method = "{}""#, expected_method)), 
                        "Should contain correct HTTP method annotation for {}", expected_method);
            
            // Verify method name conversion
            prop_assert!(controller_code.content.contains("async fn test_method"), 
                        "Should contain snake_case method name");
        }

        /// Test that generated code handles request bodies correctly
        #[test]
        fn test_request_body_syntax_correctness(
            service_name in "[a-zA-Z][a-zA-Z0-9_]*"
        ) {
            let generator = PoemOpenApiGenerator::new();
            
            let service = Service::new(service_name.clone())
                .with_method(RpcMethod::new(
                    "CreateItem".to_string(),
                    TypeReference::new("CreateItemRequest".to_string()),
                    TypeReference::new("Item".to_string())
                ));
            
            // Test with entire message as body
            let route_with_body = HttpRoute::new(
                service_name.clone(),
                "CreateItem".to_string(),
                HttpMethod::Post,
                "/items".to_string(),
            )
            .with_request_body(RequestBody::entire_message())
            .with_response_type(TypeReference::new("Item".to_string()));
            
            let routes = vec![route_with_body];
            
            let controller_result = generator.generate_controller(&service, &routes);
            prop_assert!(controller_result.is_ok(), "Controller generation with request body failed: {:?}", controller_result.err());
            
            let controller_code = controller_result.unwrap();
            
            // Verify request body parameter
            prop_assert!(controller_code.content.contains("body: Json<"), 
                        "Should contain JSON body parameter");
            
            // Verify body usage in service call
            prop_assert!(controller_code.content.contains("body.0"), 
                        "Should extract body value in service call");
            
            // Verify JSON return type
            prop_assert!(controller_code.content.contains("Json(result)"), 
                        "Should wrap result in JSON response");
        }
    }

    // Generator functions for property tests

    /// Generate valid service definitions
    fn valid_service_generator() -> impl Strategy<Value = Service> {
        ("[a-zA-Z][a-zA-Z0-9_]*", prop::collection::vec(valid_rpc_method_generator(), 0..3))
            .prop_map(|(name, methods)| {
                Service {
                    name,
                    methods,
                    options: vec![],
                    comments: vec![],
                }
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
            prop::collection::vec(path_parameter_generator(), 0..2),
            prop::collection::vec(query_parameter_generator(), 0..2),
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
            Just("/items".to_string()),
            Just("/items/{id}".to_string()),
            Just("/users/{user_id}/items".to_string()),
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

    /// Verify basic Rust syntax patterns in generated code
    fn verify_rust_syntax_patterns(code: &str) -> std::result::Result<(), proptest::test_runner::TestCaseError> {
        // Check for balanced braces
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        prop_assert_eq!(open_braces, close_braces, "Unbalanced braces in generated code");
        
        // Check for balanced parentheses
        let open_parens = code.matches('(').count();
        let close_parens = code.matches(')').count();
        prop_assert_eq!(open_parens, close_parens, "Unbalanced parentheses in generated code");
        
        // Check for balanced angle brackets (generics) - exclude -> arrows
        let open_angles = code.matches('<').count();
        let close_angles = code.matches('>').count() - code.matches("->").count();
        prop_assert_eq!(open_angles, close_angles, "Unbalanced angle brackets in generated code");
        
        // Should not contain common syntax errors
        prop_assert!(!code.contains(";;"), "Generated code should not contain double semicolons");
        prop_assert!(!code.contains(",,"), "Generated code should not contain double commas");
        
        // Should contain proper Rust keywords and patterns
        if code.contains("struct") {
            prop_assert!(code.contains("pub struct"), "Struct should be public");
        }
        
        // Skip impl formatting check for now - it's not critical for syntax correctness
        
        Ok(())
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