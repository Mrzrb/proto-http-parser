//! Unit tests for controller generation
//! 
//! Tests basic controller structure generation, dependency injection pattern,
//! and various HTTP method handler generation.
//! 
//! Requirements: 3.1, 3.2, 3.3, 3.4, 3.5

use proto_http_parser::*;

#[cfg(test)]
mod controller_unit_tests {
    use super::*;

    /// Test basic controller structure generation
    #[test]
    fn test_basic_controller_structure() {
        let generator = PoemOpenApiGenerator::new();
        
        // Create a simple service
        let service = Service::new("UserService".to_string())
            .with_method(RpcMethod::new(
                "GetUser".to_string(),
                TypeReference::new("GetUserRequest".to_string()),
                TypeReference::new("User".to_string())
            ));
        
        // Create a simple HTTP route
        let routes = vec![
            HttpRoute::new(
                "UserService".to_string(),
                "GetUser".to_string(),
                HttpMethod::Get,
                "/users/{id}".to_string(),
            )
            .with_path_parameter(PathParameter::new("id".to_string(), ParameterType::String))
            .with_response_type(TypeReference::new("User".to_string()))
        ];
        
        let result = generator.generate_controller(&service, &routes);
        assert!(result.is_ok(), "Controller generation should succeed");
        
        let controller_code = result.unwrap();
        
        // Verify basic structure
        assert!(!controller_code.content.trim().is_empty(), "Generated controller should not be empty");
        
        // Should contain controller struct definition
        assert!(controller_code.content.contains("pub struct UserServiceController"), 
                "Should contain controller struct definition");
        
        // Should contain generic parameter for service trait
        assert!(controller_code.content.contains("<T: UserServiceService>"), 
                "Should contain generic parameter for service trait");
        
        // Should contain constructor
        assert!(controller_code.content.contains("pub fn new(service: T)"), 
                "Should contain constructor method");
        
        // Should contain Arc wrapper for dependency injection
        assert!(controller_code.content.contains("Arc::new(service)"), 
                "Should wrap service in Arc for dependency injection");
        
        // Should contain OpenApi implementation
        assert!(controller_code.content.contains("#[poem_openapi::OpenApi]"), 
                "Should contain OpenApi attribute");
        
        // Should contain method implementation
        assert!(controller_code.content.contains("async fn get_user"), 
                "Should contain get_user method implementation");
        
        // Should contain path parameter
        assert!(controller_code.content.contains("id: Path<String>"), 
                "Should contain path parameter");
        
        // Should contain service call
        assert!(controller_code.content.contains("self.service.get_user"), 
                "Should contain service method call");
    }

    /// Test dependency injection pattern implementation
    #[test]
    fn test_dependency_injection_pattern() {
        let generator = PoemOpenApiGenerator::new();
        
        let service = Service::new("ProductService".to_string())
            .with_method(RpcMethod::new(
                "CreateProduct".to_string(),
                TypeReference::new("CreateProductRequest".to_string()),
                TypeReference::new("Product".to_string())
            ));
        
        let routes = vec![
            HttpRoute::new(
                "ProductService".to_string(),
                "CreateProduct".to_string(),
                HttpMethod::Post,
                "/products".to_string(),
            )
            .with_request_body(RequestBody::entire_message())
            .with_response_type(TypeReference::new("Product".to_string()))
        ];
        
        let result = generator.generate_controller(&service, &routes);
        assert!(result.is_ok(), "Controller generation should succeed");
        
        let controller_code = result.unwrap();
        
        // Verify dependency injection pattern
        assert!(controller_code.content.contains("service: Arc<T>"), 
                "Should store service as Arc<T>");
        
        // Should accept service trait implementation in constructor
        assert!(controller_code.content.contains("pub fn new(service: T) -> Self"), 
                "Constructor should accept service trait implementation");
        
        // Should have proper trait bounds
        assert!(controller_code.content.contains("T: ProductServiceService + Send + Sync + 'static"), 
                "Should have proper trait bounds for async context");
        
        // Should delegate to injected service
        assert!(controller_code.content.contains("self.service.create_product"), 
                "Should delegate to injected service implementation");
    }

    /// Test various HTTP method handler generation
    #[test]
    fn test_various_http_methods() {
        let generator = PoemOpenApiGenerator::new();
        
        let service = Service::new("ApiService".to_string())
            .with_method(RpcMethod::new(
                "GetResource".to_string(),
                TypeReference::new("GetResourceRequest".to_string()),
                TypeReference::new("Resource".to_string())
            ))
            .with_method(RpcMethod::new(
                "CreateResource".to_string(),
                TypeReference::new("CreateResourceRequest".to_string()),
                TypeReference::new("Resource".to_string())
            ))
            .with_method(RpcMethod::new(
                "UpdateResource".to_string(),
                TypeReference::new("UpdateResourceRequest".to_string()),
                TypeReference::new("Resource".to_string())
            ))
            .with_method(RpcMethod::new(
                "DeleteResource".to_string(),
                TypeReference::new("DeleteResourceRequest".to_string()),
                TypeReference::new("Empty".to_string())
            ));
        
        let routes = vec![
            HttpRoute::new(
                "ApiService".to_string(),
                "GetResource".to_string(),
                HttpMethod::Get,
                "/resources/{id}".to_string(),
            )
            .with_path_parameter(PathParameter::new("id".to_string(), ParameterType::String))
            .with_response_type(TypeReference::new("Resource".to_string())),
            
            HttpRoute::new(
                "ApiService".to_string(),
                "CreateResource".to_string(),
                HttpMethod::Post,
                "/resources".to_string(),
            )
            .with_request_body(RequestBody::entire_message())
            .with_response_type(TypeReference::new("Resource".to_string())),
            
            HttpRoute::new(
                "ApiService".to_string(),
                "UpdateResource".to_string(),
                HttpMethod::Put,
                "/resources/{id}".to_string(),
            )
            .with_path_parameter(PathParameter::new("id".to_string(), ParameterType::String))
            .with_request_body(RequestBody::entire_message())
            .with_response_type(TypeReference::new("Resource".to_string())),
            
            HttpRoute::new(
                "ApiService".to_string(),
                "DeleteResource".to_string(),
                HttpMethod::Delete,
                "/resources/{id}".to_string(),
            )
            .with_path_parameter(PathParameter::new("id".to_string(), ParameterType::String))
            .with_response_type(TypeReference::new("Empty".to_string())),
        ];
        
        let result = generator.generate_controller(&service, &routes);
        assert!(result.is_ok(), "Controller generation should succeed");
        
        let controller_code = result.unwrap();
        
        // Verify GET method
        assert!(controller_code.content.contains(r#"method = "get""#), 
                "Should contain GET method annotation");
        assert!(controller_code.content.contains("async fn get_resource"), 
                "Should contain get_resource method");
        
        // Verify POST method
        assert!(controller_code.content.contains(r#"method = "post""#), 
                "Should contain POST method annotation");
        assert!(controller_code.content.contains("async fn create_resource"), 
                "Should contain create_resource method");
        
        // Verify PUT method
        assert!(controller_code.content.contains(r#"method = "put""#), 
                "Should contain PUT method annotation");
        assert!(controller_code.content.contains("async fn update_resource"), 
                "Should contain update_resource method");
        
        // Verify DELETE method
        assert!(controller_code.content.contains(r#"method = "delete""#), 
                "Should contain DELETE method annotation");
        assert!(controller_code.content.contains("async fn delete_resource"), 
                "Should contain delete_resource method");
    }

    /// Test controller generation with query parameters
    #[test]
    fn test_controller_with_query_parameters() {
        let generator = PoemOpenApiGenerator::new();
        
        let service = Service::new("SearchService".to_string())
            .with_method(RpcMethod::new(
                "SearchUsers".to_string(),
                TypeReference::new("SearchUsersRequest".to_string()),
                TypeReference::new("SearchUsersResponse".to_string())
            ));
        
        let routes = vec![
            HttpRoute::new(
                "SearchService".to_string(),
                "SearchUsers".to_string(),
                HttpMethod::Get,
                "/users/search".to_string(),
            )
            .with_query_parameter(QueryParameter::new("query".to_string(), ParameterType::String, true))
            .with_query_parameter(QueryParameter::new("limit".to_string(), ParameterType::Integer, false))
            .with_query_parameter(QueryParameter::new("offset".to_string(), ParameterType::Integer, false))
            .with_response_type(TypeReference::new("SearchUsersResponse".to_string()))
        ];
        
        let result = generator.generate_controller(&service, &routes);
        assert!(result.is_ok(), "Controller generation should succeed");
        
        let controller_code = result.unwrap();
        
        // Should contain required query parameter
        assert!(controller_code.content.contains("query: Query<String>"), 
                "Should contain required query parameter");
        
        // Should contain optional query parameters with default annotation
        assert!(controller_code.content.contains("#[oai(default)]") && 
                controller_code.content.contains("limit: Query<Option<i32>>"), 
                "Should contain optional query parameter with default annotation");
        
        assert!(controller_code.content.contains("offset: Query<Option<i32>>"), 
                "Should contain optional query parameter");
        
        // Should pass parameters to service method
        assert!(controller_code.content.contains("query.0") && 
                controller_code.content.contains("limit.0") && 
                controller_code.content.contains("offset.0"), 
                "Should pass query parameters to service method");
    }

    /// Test controller generation with request body
    #[test]
    fn test_controller_with_request_body() {
        let generator = PoemOpenApiGenerator::new();
        
        let service = Service::new("UserService".to_string())
            .with_method(RpcMethod::new(
                "UpdateUser".to_string(),
                TypeReference::new("UpdateUserRequest".to_string()),
                TypeReference::new("User".to_string())
            ));
        
        let routes = vec![
            HttpRoute::new(
                "UserService".to_string(),
                "UpdateUser".to_string(),
                HttpMethod::Put,
                "/users/{id}".to_string(),
            )
            .with_path_parameter(PathParameter::new("id".to_string(), ParameterType::String))
            .with_request_body(RequestBody::entire_message())
            .with_response_type(TypeReference::new("User".to_string()))
        ];
        
        let result = generator.generate_controller(&service, &routes);
        assert!(result.is_ok(), "Controller generation should succeed");
        
        let controller_code = result.unwrap();
        
        // Should contain JSON body parameter
        assert!(controller_code.content.contains("body: Json<"), 
                "Should contain JSON body parameter");
        
        // Should pass body to service method
        assert!(controller_code.content.contains("body.0"), 
                "Should pass body to service method");
        
        // Should return JSON response
        assert!(controller_code.content.contains("-> poem_openapi::payload::Json<"), 
                "Should return JSON response");
        
        assert!(controller_code.content.contains("Json(result)"), 
                "Should wrap result in JSON");
    }

    /// Test controller generation with empty service (no routes)
    #[test]
    fn test_controller_with_empty_service() {
        let generator = PoemOpenApiGenerator::new();
        
        let service = Service::new("EmptyService".to_string());
        let routes = vec![];
        
        let result = generator.generate_controller(&service, &routes);
        assert!(result.is_ok(), "Empty controller generation should succeed");
        
        let controller_code = result.unwrap();
        
        // Should still generate valid controller structure
        assert!(controller_code.content.contains("pub struct EmptyServiceController"), 
                "Should contain controller struct even for empty service");
        
        assert!(controller_code.content.contains("impl<T: EmptyServiceService>"), 
                "Should contain impl block even for empty service");
        
        // Should not contain any method implementations
        assert!(!controller_code.content.contains("async fn"), 
                "Should not contain any method implementations for empty service");
    }

    /// Test that generated controller includes proper imports and dependencies
    #[test]
    fn test_controller_imports_and_dependencies() {
        let generator = PoemOpenApiGenerator::new();
        
        let service = Service::new("TestService".to_string())
            .with_method(RpcMethod::new(
                "TestMethod".to_string(),
                TypeReference::new("TestRequest".to_string()),
                TypeReference::new("TestResponse".to_string())
            ));
        
        let routes = vec![
            HttpRoute::new(
                "TestService".to_string(),
                "TestMethod".to_string(),
                HttpMethod::Get,
                "/test".to_string(),
            )
            .with_response_type(TypeReference::new("TestResponse".to_string()))
        ];
        
        let result = generator.generate_controller(&service, &routes);
        assert!(result.is_ok(), "Controller generation should succeed");
        
        let controller_code = result.unwrap();
        
        // Should include required imports
        assert!(controller_code.imports.contains(&"poem_openapi::{OpenApi, payload::Json, param::Path, param::Query}".to_string()), 
                "Should include poem_openapi imports");
        
        assert!(controller_code.imports.contains(&"std::sync::Arc".to_string()), 
                "Should include Arc import");
        
        // Should include required dependencies
        assert!(controller_code.dependencies.contains(&"poem-openapi".to_string()), 
                "Should include poem-openapi dependency");
        
        assert!(controller_code.dependencies.contains(&"poem".to_string()), 
                "Should include poem dependency");
    }
}