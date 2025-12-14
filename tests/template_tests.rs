//! Property-based tests for the template engine
//! 
//! **Feature: proto-http-parser-v2, Property 6: Template rendering consistency**
//! **Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5, 7.5**

use proto_http_parser_v2::*;
use proptest::prelude::*;
use std::collections::HashMap;

/// Test that template rendering is consistent and handles all valid inputs correctly
#[cfg(test)]
mod template_property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Feature: proto-http-parser-v2, Property 6: Template rendering consistency**
        /// For any valid template and context data, the template engine should generate 
        /// well-formed code and correctly handle variable substitution and conditional logic
        #[test]
        fn test_template_rendering_consistency(
            service in valid_service_generator(),
            routes in valid_routes_generator(),
            custom_data in valid_custom_data_generator()
        ) {
            let mut engine = HandlebarsTemplateEngine::new();
            
            // Create template context
            let mut context_data = custom_data;
            context_data.insert("imports".to_string(), TemplateValue::Array(vec![
                TemplateValue::String("std::collections::HashMap".to_string()),
                TemplateValue::String("serde::{Serialize, Deserialize}".to_string()),
            ]));
            
            let context = TemplateContext {
                service: service.clone(),
                routes: routes.clone(),
                custom_data: context_data,
            };
            
            // Test built-in templates (excluding method template which needs route context)
            let templates = vec!["controller", "service_trait", "imports"];
            
            for template_name in templates {
                let result = engine.render(template_name, &context);
                
                // Template should render successfully
                prop_assert!(result.is_ok(), "Template '{}' failed to render: {:?}", template_name, result.err());
                
                let rendered = result.unwrap();
                
                // Rendered content should not be empty
                prop_assert!(!rendered.trim().is_empty(), "Template '{}' produced empty output", template_name);
                
                // Should contain service name in some form (except for imports template)
                if template_name != "imports" {
                    let service_name_variations = vec![
                        service.name.clone(),
                        to_snake_case(&service.name),
                        to_pascal_case(&service.name),
                    ];
                    
                    let contains_service_name = service_name_variations.iter()
                        .any(|name| rendered.contains(name));
                    prop_assert!(contains_service_name, "Template '{}' does not contain service name", template_name);
                }
                
                // Should not contain unresolved template variables
                prop_assert!(!rendered.contains("{{"), "Template '{}' contains unresolved variables", template_name);
                prop_assert!(!rendered.contains("}}"), "Template '{}' contains unresolved variables", template_name);
            }
        }

        /// Test that custom templates can be registered and rendered correctly
        #[test]
        fn test_custom_template_registration(
            template_content in valid_template_generator(),
            service in valid_service_generator(),
            routes in valid_routes_generator()
        ) {
            let mut engine = HandlebarsTemplateEngine::new();
            
            // Register custom template
            let template_name = "custom_test";
            let register_result = engine.register_template(template_name, &template_content);
            prop_assert!(register_result.is_ok(), "Failed to register template: {:?}", register_result.err());
            
            // Create context
            let context = TemplateContext {
                service,
                routes,
                custom_data: HashMap::new(),
            };
            
            // Render template
            let render_result = engine.render(template_name, &context);
            prop_assert!(render_result.is_ok(), "Failed to render custom template: {:?}", render_result.err());
        }

        /// Test that template helpers work correctly
        #[test]
        fn test_template_helpers(
            input_string in "[a-zA-Z_][a-zA-Z0-9_]*"
        ) {
            let mut engine = HandlebarsTemplateEngine::new();
            
            // Test snake_case helper
            let snake_template = format!("{{{{snake_case '{}'}}}}", input_string);
            engine.register_template("snake_test", &snake_template).unwrap();
            
            let context = TemplateContext {
                service: Service::new("TestService".to_string()),
                routes: vec![],
                custom_data: HashMap::new(),
            };
            
            let result = engine.render("snake_test", &context);
            prop_assert!(result.is_ok(), "snake_case helper failed: {:?}", result.err());
            
            let rendered = result.unwrap();
            let expected = to_snake_case(&input_string);
            prop_assert_eq!(rendered.trim(), expected, "snake_case helper produced incorrect output");
            
            // Test camel_case helper
            let camel_template = format!("{{{{camel_case '{}'}}}}", input_string);
            engine.register_template("camel_test", &camel_template).unwrap();
            
            let result = engine.render("camel_test", &context);
            prop_assert!(result.is_ok(), "camel_case helper failed: {:?}", result.err());
            
            let rendered = result.unwrap();
            let expected = to_camel_case(&input_string);
            prop_assert_eq!(rendered.trim(), expected, "camel_case helper produced incorrect output");
        }

        /// Test that type mapping helper works correctly
        #[test]
        fn test_type_mapping_helper(
            proto_type in proto_type_generator()
        ) {
            let mut engine = HandlebarsTemplateEngine::new();
            
            let template = format!("{{{{map_type '{}'}}}}", proto_type);
            engine.register_template("type_test", &template).unwrap();
            
            let context = TemplateContext {
                service: Service::new("TestService".to_string()),
                routes: vec![],
                custom_data: HashMap::new(),
            };
            
            let result = engine.render("type_test", &context);
            prop_assert!(result.is_ok(), "map_type helper failed: {:?}", result.err());
            
            let rendered = result.unwrap().trim().to_string();
            
            // Should map to a valid Rust type
            let valid_rust_types = vec![
                "String", "i32", "i64", "u32", "u64", "f32", "f64", "bool", "Vec<u8>",
                "chrono::DateTime<chrono::Utc>", "std::time::Duration", "()"
            ];
            
            let is_valid_type = valid_rust_types.contains(&rendered.as_str()) || 
                               rendered.chars().all(|c| c.is_alphanumeric() || c == ':' || c == '<' || c == '>');
            
            prop_assert!(is_valid_type, "Type mapping produced invalid Rust type: {}", rendered);
        }
    }

    // Generator functions for property tests

    /// Generate valid service definitions
    fn valid_service_generator() -> impl Strategy<Value = Service> {
        ("[a-zA-Z][a-zA-Z0-9]*", prop::collection::vec(valid_rpc_method_generator(), 1..5))
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
        ("[a-zA-Z][a-zA-Z0-9]*", valid_type_reference_generator(), valid_type_reference_generator())
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
        "[a-zA-Z][a-zA-Z0-9]*"
            .prop_map(|name| TypeReference::new(name))
    }

    /// Generate valid HTTP routes
    fn valid_routes_generator() -> impl Strategy<Value = Vec<HttpRoute>> {
        prop::collection::vec(valid_http_route_generator(), 0..3)
    }

    /// Generate valid HTTP routes
    fn valid_http_route_generator() -> impl Strategy<Value = HttpRoute> {
        (
            "[a-zA-Z][a-zA-Z0-9]*", // service_name
            "[a-zA-Z][a-zA-Z0-9]*", // method_name
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

    /// Generate valid custom data for template context
    fn valid_custom_data_generator() -> impl Strategy<Value = HashMap<String, TemplateValue>> {
        prop::collection::hash_map(
            "[a-zA-Z][a-zA-Z0-9_]*",
            template_value_generator(),
            0..5
        )
    }

    /// Generate template values
    fn template_value_generator() -> impl Strategy<Value = TemplateValue> {
        prop_oneof![
            "[a-zA-Z0-9 ]*".prop_map(TemplateValue::String),
            any::<f64>().prop_map(TemplateValue::Number),
            any::<bool>().prop_map(TemplateValue::Boolean),
            Just(TemplateValue::Null),
        ]
    }

    /// Generate valid template content
    fn valid_template_generator() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("Hello {{service.name}}!".to_string()),
            Just("Service: {{pascal_case service.name}}".to_string()),
            Just("{{#each routes}}Route: {{method_name}}{{/each}}".to_string()),
            Just("{{snake_case service.name}}_controller".to_string()),
        ]
    }

    /// Generate Protocol Buffer type names
    fn proto_type_generator() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("string".to_string()),
            Just("int32".to_string()),
            Just("int64".to_string()),
            Just("uint32".to_string()),
            Just("uint64".to_string()),
            Just("double".to_string()),
            Just("float".to_string()),
            Just("bool".to_string()),
            Just("bytes".to_string()),
            Just("google.protobuf.Timestamp".to_string()),
            Just("google.protobuf.Duration".to_string()),
            Just("google.protobuf.Empty".to_string()),
        ]
    }
}

// Helper functions for tests (re-implementing from templates.rs for testing)

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

/// Convert string to camelCase
fn to_camel_case(s: &str) -> String {
    let words: Vec<&str> = s.split('_').collect();
    let mut result = String::new();
    
    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            result.push_str(&word.to_lowercase());
        } else {
            result.push_str(&capitalize_first(word));
        }
    }
    
    result
}

/// Convert string to PascalCase
fn to_pascal_case(s: &str) -> String {
    let words: Vec<&str> = s.split('_').collect();
    words.iter()
        .map(|word| capitalize_first(word))
        .collect::<String>()
}

/// Capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}