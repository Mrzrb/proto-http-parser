//! Property-based tests for HTTP annotation extraction

use proto_http_parser_v2::*;
use proptest::prelude::*;

// Shared generators for all tests

// Generator for valid HTTP methods
fn http_method_generator() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("get".to_string()),
        Just("post".to_string()),
        Just("put".to_string()),
        Just("patch".to_string()),
        Just("delete".to_string()),
    ]
}

// Generator for valid path templates
fn path_template_generator() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("/users".to_string()),
        Just("/users/{id}".to_string()),
        Just("/users/{user_id}/posts".to_string()),
        Just("/users/{user_id}/posts/{post_id}".to_string()),
        Just("/api/v1/users/{id}".to_string()),
        Just("/organizations/{org_id}/users/{user_id}".to_string()),
    ]
}

// Generator for valid service names
fn service_name_generator() -> impl Strategy<Value = String> {
    "[A-Z][a-zA-Z0-9]*Service".prop_map(|s| s)
}

// Generator for valid method names
fn method_name_generator() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("GetUser".to_string()),
        Just("CreateUser".to_string()),
        Just("UpdateUser".to_string()),
        Just("DeleteUser".to_string()),
        Just("ListUsers".to_string()),
    ]
}
// Generator for valid type references
fn type_reference_generator() -> impl Strategy<Value = TypeReference> {
    prop_oneof![
        Just(TypeReference::new("GetUserRequest".to_string())),
        Just(TypeReference::new("CreateUserRequest".to_string())),
        Just(TypeReference::new("UpdateUserRequest".to_string())),
        Just(TypeReference::new("DeleteUserRequest".to_string())),
        Just(TypeReference::new("ListUsersRequest".to_string())),
    ]
}

// Generator for valid response type references
fn response_type_generator() -> impl Strategy<Value = TypeReference> {
    prop_oneof![
        Just(TypeReference::new("User".to_string())),
        Just(TypeReference::new("CreateUserResponse".to_string())),
        Just(TypeReference::new("UpdateUserResponse".to_string())),
        Just(TypeReference::with_package("Empty".to_string(), "google.protobuf".to_string())),
        Just(TypeReference::new("ListUsersResponse".to_string())),
    ]
}

// Generator for HTTP annotations
fn http_annotation_generator() -> impl Strategy<Value = HttpAnnotation> {
    (http_method_generator(), path_template_generator(), prop::option::of("[a-z_]*"))
        .prop_map(|(method_str, path, body)| {
            let method = match method_str.as_str() {
                "get" => HttpMethod::Get,
                "post" => HttpMethod::Post,
                "put" => HttpMethod::Put,
                "patch" => HttpMethod::Patch,
                "delete" => HttpMethod::Delete,
                _ => HttpMethod::Get,
            };
            
            HttpAnnotation {
                method,
                path,
                body,
                additional_bindings: Vec::new(),
            }
        })
}

// Generator for RPC methods with HTTP annotations
fn rpc_method_with_http_generator() -> impl Strategy<Value = RpcMethod> {
    (
        method_name_generator(),
        type_reference_generator(),
        response_type_generator(),
        http_annotation_generator(),
    ).prop_map(|(name, input_type, output_type, http_annotation)| {
        RpcMethod {
            name,
            input_type,
            output_type,
            options: Vec::new(),
            comments: Vec::new(),
            http_annotation: Some(http_annotation),
        }
    })
}

// Generator for services with HTTP methods
fn service_with_http_generator() -> impl Strategy<Value = Service> {
    (
        service_name_generator(),
        prop::collection::vec(rpc_method_with_http_generator(), 1..5),
    ).prop_map(|(name, methods)| {
        Service {
            name,
            methods,
            options: Vec::new(),
            comments: Vec::new(),
        }
    })
}

// Generator for proto files with HTTP services
fn proto_file_with_http_generator() -> impl Strategy<Value = ProtoFile> {
    (
        prop::option::of("[a-z][a-z0-9.]*"),
        prop::collection::vec(service_with_http_generator(), 1..3),
    ).prop_map(|(package, services)| {
        ProtoFile {
            syntax: ProtocolVersion::Proto3,
            package,
            imports: Vec::new(),
            options: Vec::new(),
            services,
            messages: Vec::new(),
            enums: Vec::new(),
        }
    })
}

/// **Feature: proto-http-parser-v2, Property 2: HTTP annotation extraction completeness**
/// **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**
/// 
/// For any proto file containing google.api.http annotations, the system should correctly
/// extract all HTTP methods, path parameters, query parameters and request body configuration
#[cfg(test)]
mod http_annotation_extraction_tests {
    use super::*;
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        /// **Feature: proto-http-parser-v2, Property 2: HTTP annotation extraction completeness**
        /// **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**
        #[test]
        fn test_http_annotation_extraction_completeness(
            proto_file in proto_file_with_http_generator()
        ) {
            let extractor = GoogleApiHttpExtractor::new();
            let result = extractor.extract_routes(&proto_file);
            
            // The extraction should succeed for valid proto files
            prop_assert!(result.is_ok());
            
            let routes = result.unwrap();
            
            // Should extract routes for all HTTP-enabled methods
            let expected_route_count: usize = proto_file.services.iter()
                .map(|service| service.methods.iter()
                    .filter(|method| method.http_annotation.is_some())
                    .count())
                .sum();
            
            prop_assert_eq!(routes.len(), expected_route_count);
            
            // Each route should have correct service and method names
            for route in &routes {
                let service_exists = proto_file.services.iter()
                    .any(|service| service.name == route.service_name);
                prop_assert!(service_exists, "Route service name should match a service in the proto file");
                
                let method_exists = proto_file.services.iter()
                    .flat_map(|service| &service.methods)
                    .any(|method| method.name == route.method_name);
                prop_assert!(method_exists, "Route method name should match a method in the proto file");
            }
            
            // Each route should have a valid HTTP method
            for route in &routes {
                match &route.http_method {
                    HttpMethod::Get | HttpMethod::Post | HttpMethod::Put | 
                    HttpMethod::Patch | HttpMethod::Delete => {
                        // Valid standard methods
                    }
                    HttpMethod::Custom(_) => {
                        // Custom methods should only be allowed if configured
                        prop_assert!(false, "Custom HTTP methods should not be generated by default");
                    }
                }
            }
            
            // Each route should have a valid path template
            for route in &routes {
                prop_assert!(!route.path_template.is_empty(), "Path template should not be empty");
                prop_assert!(route.path_template.starts_with('/'), "Path template should start with '/'");
            }
            
            // Path parameters should be correctly extracted
            for route in &routes {
                let param_count = route.path_template.matches('{').count();
                prop_assert_eq!(route.path_parameters.len(), param_count, 
                    "Number of path parameters should match number of parameter placeholders");
                
                // Each path parameter should have a valid name
                for param in &route.path_parameters {
                    prop_assert!(!param.name.is_empty(), "Path parameter name should not be empty");
                    prop_assert!(param.required, "Path parameters should always be required");
                }
            }
            
            // Request body should be appropriate for HTTP method
            for route in &routes {
                match route.http_method {
                    HttpMethod::Get | HttpMethod::Delete => {
                        prop_assert!(route.request_body.is_none(), 
                            "GET and DELETE methods should not have request bodies");
                    }
                    HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch => {
                        // These methods may have request bodies
                        if let Some(body) = &route.request_body {
                            prop_assert_eq!(&body.content_type, "application/json", 
                                "Request body should use JSON content type");
                        }
                    }
                    HttpMethod::Custom(_) => {
                        // Custom methods are not expected in this test
                    }
                }
            }
        }
        
        /// Test that validation catches invalid annotations
        #[test]
        fn test_validation_catches_invalid_annotations(
            mut proto_file in proto_file_with_http_generator()
        ) {
            // Introduce invalid path templates
            for service in &mut proto_file.services {
                for method in &mut service.methods {
                    if let Some(ref mut annotation) = method.http_annotation {
                        // Create invalid path template (unmatched braces)
                        annotation.path = "/invalid/{unclosed".to_string();
                    }
                }
            }
            
            let extractor = GoogleApiHttpExtractor::new();
            let result = extractor.extract_routes(&proto_file);
            
            // Should fail validation for invalid path templates
            prop_assert!(result.is_err(), "Should reject invalid path templates");
        }
        
        /// Test that conflicting routes are detected
        #[test]
        fn test_conflicting_routes_detection(
            service_name in service_name_generator(),
            method_name1 in method_name_generator(),
            method_name2 in method_name_generator(),
            path_template in path_template_generator(),
        ) {
            // Create two methods with the same HTTP method and path
            let method1 = RpcMethod {
                name: method_name1,
                input_type: TypeReference::new("Request1".to_string()),
                output_type: TypeReference::new("Response1".to_string()),
                options: Vec::new(),
                comments: Vec::new(),
                http_annotation: Some(HttpAnnotation {
                    method: HttpMethod::Get,
                    path: path_template.clone(),
                    body: None,
                    additional_bindings: Vec::new(),
                }),
            };
            
            let method2 = RpcMethod {
                name: method_name2,
                input_type: TypeReference::new("Request2".to_string()),
                output_type: TypeReference::new("Response2".to_string()),
                options: Vec::new(),
                comments: Vec::new(),
                http_annotation: Some(HttpAnnotation {
                    method: HttpMethod::Get,
                    path: path_template,
                    body: None,
                    additional_bindings: Vec::new(),
                }),
            };
            
            let service = Service {
                name: service_name,
                methods: vec![method1, method2],
                options: Vec::new(),
                comments: Vec::new(),
            };
            
            let proto_file = ProtoFile {
                syntax: ProtocolVersion::Proto3,
                package: None,
                imports: Vec::new(),
                options: Vec::new(),
                services: vec![service],
                messages: Vec::new(),
                enums: Vec::new(),
            };
            
            let extractor = GoogleApiHttpExtractor::new();
            let routes_result = extractor.extract_routes(&proto_file);
            
            if let Ok(routes) = routes_result {
                let validation_result = extractor.validate_annotations(&routes);
                // Should detect conflicting routes
                prop_assert!(validation_result.is_err(), "Should detect conflicting routes");
            }
        }
    }
}

/// **Feature: proto-http-parser-v2, Property 7: HTTP pattern mapping correctness**
/// **Validates: Requirements 7.3**
/// 
/// For any HTTP annotation pattern, the system should generate correct poem-openapi 
/// route definitions and parameter mappings
#[cfg(test)]
mod http_pattern_mapping_tests {
    use super::*;
    
    // Generator for complex path templates with various parameter patterns
    fn complex_path_template_generator() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("/users/{user_id}".to_string()),
            Just("/users/{user_id}/posts/{post_id}".to_string()),
            Just("/organizations/{org_id}/users/{user_id}".to_string()),
            Just("/api/v1/users/{id}/profile".to_string()),
            Just("/users/{user_id}/posts/{post_id}/comments/{comment_id}".to_string()),
            Just("/projects/{project_id}/issues/{issue_number}".to_string()),
        ]
    }
    
    // Generator for HTTP methods with body requirements
    fn http_method_with_body_generator() -> impl Strategy<Value = (HttpMethod, bool)> {
        prop_oneof![
            Just((HttpMethod::Get, false)),
            Just((HttpMethod::Delete, false)),
            Just((HttpMethod::Post, true)),
            Just((HttpMethod::Put, true)),
            Just((HttpMethod::Patch, true)),
        ]
    }
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        /// **Feature: proto-http-parser-v2, Property 7: HTTP pattern mapping correctness**
        /// **Validates: Requirements 7.3**
        #[test]
        fn test_http_pattern_mapping_correctness(
            path_template in complex_path_template_generator(),
            (http_method, should_have_body) in http_method_with_body_generator(),
            service_name in service_name_generator(),
            method_name in method_name_generator(),
        ) {
            // Create an RPC method with HTTP annotation
            let http_annotation = HttpAnnotation {
                method: http_method.clone(),
                path: path_template.clone(),
                body: if should_have_body { Some("*".to_string()) } else { None },
                additional_bindings: Vec::new(),
            };
            
            let rpc_method = RpcMethod {
                name: method_name.clone(),
                input_type: TypeReference::new("TestRequest".to_string()),
                output_type: TypeReference::new("TestResponse".to_string()),
                options: Vec::new(),
                comments: Vec::new(),
                http_annotation: Some(http_annotation),
            };
            
            let service = Service {
                name: service_name.clone(),
                methods: vec![rpc_method],
                options: Vec::new(),
                comments: Vec::new(),
            };
            
            let proto_file = ProtoFile {
                syntax: ProtocolVersion::Proto3,
                package: None,
                imports: Vec::new(),
                options: Vec::new(),
                services: vec![service],
                messages: Vec::new(),
                enums: Vec::new(),
            };
            
            let extractor = GoogleApiHttpExtractor::new();
            let result = extractor.extract_routes(&proto_file);
            
            prop_assert!(result.is_ok(), "Route extraction should succeed for valid patterns");
            
            let routes = result.unwrap();
            prop_assert_eq!(routes.len(), 1, "Should extract exactly one route");
            
            let route = &routes[0];
            
            // Verify basic route properties
            prop_assert_eq!(&route.service_name, &service_name, "Service name should match");
            prop_assert_eq!(&route.method_name, &method_name, "Method name should match");
            prop_assert_eq!(&route.http_method, &http_method, "HTTP method should match");
            prop_assert_eq!(&route.path_template, &path_template, "Path template should match");
            
            // Verify path parameter extraction correctness
            let expected_param_count = path_template.matches('{').count();
            prop_assert_eq!(route.path_parameters.len(), expected_param_count, 
                "Number of extracted path parameters should match template placeholders");
            
            // Verify each path parameter is correctly extracted
            let param_regex = regex::Regex::new(r"\{([^}]+)\}").unwrap();
            let expected_params: Vec<&str> = param_regex.captures_iter(&path_template)
                .map(|cap| cap.get(1).unwrap().as_str())
                .collect();
            
            for (i, expected_param) in expected_params.iter().enumerate() {
                prop_assert!(i < route.path_parameters.len(), 
                    "Should have parameter at index {}", i);
                prop_assert_eq!(&route.path_parameters[i].name, expected_param, 
                    "Parameter name should match template");
                prop_assert!(route.path_parameters[i].required, 
                    "Path parameters should always be required");
            }
            
            // Verify request body mapping correctness
            match &http_method {
                HttpMethod::Get | HttpMethod::Delete => {
                    prop_assert!(route.request_body.is_none(), 
                        "GET/DELETE methods should not have request body");
                }
                HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch => {
                    if should_have_body {
                        prop_assert!(route.request_body.is_some(), 
                            "POST/PUT/PATCH methods should have request body when specified");
                        
                        let body = route.request_body.as_ref().unwrap();
                        prop_assert!(body.is_entire_message, 
                            "Body with '*' should map to entire message");
                        prop_assert_eq!(&body.content_type, "application/json", 
                            "Should use JSON content type");
                    }
                }
                HttpMethod::Custom(_) => {
                    // Custom methods are not expected in this test
                }
            }
            
            // Verify query parameter inference
            prop_assert!(!route.query_parameters.is_empty(), 
                "Should infer common query parameters");
            
            // Verify common query parameters are present
            let param_names: Vec<&str> = route.query_parameters.iter()
                .map(|p| p.name.as_str())
                .collect();
            
            let expected_common_params = ["page", "limit", "offset", "sort", "order", "filter", "search"];
            for expected_param in &expected_common_params {
                prop_assert!(param_names.contains(expected_param), 
                    "Should include common query parameter: {}", expected_param);
            }
            
            // All query parameters should be optional
            for param in &route.query_parameters {
                prop_assert!(!param.required, 
                    "Query parameters should be optional by default");
            }
        }
        
        /// Test parameter type inference correctness
        #[test]
        fn test_parameter_type_inference(
            param_name in "[a-z_]+(_id|_count|_size|_limit|_enabled|_active|_rate|_ratio)?",
        ) {
            let extractor = GoogleApiHttpExtractor::new();
            let inferred_type = extractor.infer_parameter_type(&param_name);
            
            // Verify type inference rules
            if param_name.ends_with("_id") || param_name == "id" {
                prop_assert_eq!(inferred_type, ParameterType::String, 
                    "ID parameters should be inferred as String");
            } else if param_name.contains("count") || param_name.contains("size") || 
                      param_name.contains("limit") {
                prop_assert_eq!(inferred_type, ParameterType::Integer, 
                    "Count/size/limit parameters should be inferred as Integer");
            } else if param_name.contains("rate") || param_name.contains("ratio") {
                prop_assert_eq!(inferred_type, ParameterType::Float, 
                    "Rate/ratio parameters should be inferred as Float");
            } else if param_name.contains("enabled") || param_name.contains("active") {
                prop_assert_eq!(inferred_type, ParameterType::Boolean, 
                    "Enabled/active parameters should be inferred as Boolean");
            } else {
                prop_assert_eq!(inferred_type, ParameterType::String, 
                    "Default parameter type should be String");
            }
        }
        
        /// Test path template validation correctness
        #[test]
        fn test_path_template_validation(
            valid_path in "(/[a-zA-Z0-9_-]+|/\\{[a-zA-Z_][a-zA-Z0-9_]*\\})+",
        ) {
            let extractor = GoogleApiHttpExtractor::new();
            let result = extractor.validate_path_template(&valid_path);
            
            prop_assert!(result.is_ok(), 
                "Valid path templates should pass validation: {}", valid_path);
        }
        
        /// Test invalid path template rejection
        #[test]
        fn test_invalid_path_template_rejection(
            invalid_path in prop_oneof![
                Just("".to_string()),                    // Empty path
                Just("no-leading-slash".to_string()),     // No leading slash
                Just("/unclosed{param".to_string()),      // Unclosed brace
                Just("/extra}brace".to_string()),         // Extra closing brace
                Just("/nested{param{inner}}".to_string()), // Nested braces
            ]
        ) {
            let extractor = GoogleApiHttpExtractor::new();
            let result = extractor.validate_path_template(&invalid_path);
            
            prop_assert!(result.is_err(), 
                "Invalid path templates should be rejected: {}", invalid_path);
        }
        
        /// Test route conflict detection accuracy
        #[test]
        fn test_route_conflict_detection_accuracy(
            method1 in http_method_generator(),
            method2 in http_method_generator(),
            path1 in path_template_generator(),
            path2 in path_template_generator(),
        ) {
            let route1 = HttpRoute::new(
                "TestService".to_string(),
                "Method1".to_string(),
                match method1.as_str() {
                    "get" => HttpMethod::Get,
                    "post" => HttpMethod::Post,
                    "put" => HttpMethod::Put,
                    "patch" => HttpMethod::Patch,
                    "delete" => HttpMethod::Delete,
                    _ => HttpMethod::Get,
                },
                path1.clone(),
            );
            
            let route2 = HttpRoute::new(
                "TestService".to_string(),
                "Method2".to_string(),
                match method2.as_str() {
                    "get" => HttpMethod::Get,
                    "post" => HttpMethod::Post,
                    "put" => HttpMethod::Put,
                    "patch" => HttpMethod::Patch,
                    "delete" => HttpMethod::Delete,
                    _ => HttpMethod::Get,
                },
                path2.clone(),
            );
            
            let routes = vec![route1, route2];
            let extractor = GoogleApiHttpExtractor::new();
            let result = extractor.validate_annotations(&routes);
            
            // Should detect conflict if and only if method and path are identical
            let should_conflict = method1 == method2 && path1 == path2;
            
            if should_conflict {
                prop_assert!(result.is_err(), 
                    "Should detect conflict for identical method and path");
            } else {
                prop_assert!(result.is_ok(), 
                    "Should not detect conflict for different method or path");
            }
        }
    }
}