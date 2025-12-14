//! Integration tests for the validation engine

use proto_http_parser::*;
use proto_http_parser::core::{
    data::{ProtoFile, Service, RpcMethod, TypeReference, Message, Field, FieldType, FieldLabel, HttpRoute, HttpMethod},
    errors::ValidationError,
    Validator,
};

#[test]
fn test_validation_engine_basic_functionality() {
    let mut engine = ValidationEngine::new();
    
    // Create a simple proto file with some issues
    let mut proto_file = ProtoFile::new();
    proto_file.package = Some("test.package".to_string());
    
    // Add a service with undefined input/output types
    let service = Service::new("TestService".to_string())
        .with_method(RpcMethod::new(
            "TestMethod".to_string(),
            TypeReference::new("UndefinedInput".to_string()),
            TypeReference::new("UndefinedOutput".to_string()),
        ));
    
    proto_file.services.push(service);
    
    // Validate the proto file
    let result = engine.validate_proto_file_internal(&proto_file);
    
    // Should have validation errors for undefined types
    assert!(!result.is_valid);
    assert!(!result.errors.is_empty());
    
    // Check that we have undefined type errors
    let undefined_errors: Vec<_> = result.errors.iter()
        .filter(|e| matches!(e, ValidationError::UndefinedType { .. }))
        .collect();
    
    assert_eq!(undefined_errors.len(), 2); // Input and output types
}

#[test]
fn test_validation_engine_with_valid_proto() {
    let mut engine = ValidationEngine::new();
    
    // Create a valid proto file
    let mut proto_file = ProtoFile::new();
    proto_file.package = Some("test.package".to_string());
    
    // Add a message type
    let message = Message {
        name: "TestMessage".to_string(),
        fields: vec![
            Field {
                name: "id".to_string(),
                field_type: FieldType::String,
                number: 1,
                label: FieldLabel::Optional,
                options: vec![],
                comments: vec![],
            }
        ],
        nested_messages: vec![],
        nested_enums: vec![],
        options: vec![],
        comments: vec![],
    };
    
    proto_file.messages.push(message);
    
    // Add a service with valid types
    let service = Service::new("TestService".to_string())
        .with_method(RpcMethod::new(
            "TestMethod".to_string(),
            TypeReference::new("TestMessage".to_string()),
            TypeReference::new("TestMessage".to_string()),
        ));
    
    proto_file.services.push(service);
    
    // Validate the proto file
    let result = engine.validate_proto_file_internal(&proto_file);
    
    // Should be valid
    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn test_validation_engine_duplicate_definitions() {
    let mut engine = ValidationEngine::new();
    
    // Create a proto file with duplicate service names
    let mut proto_file = ProtoFile::new();
    proto_file.package = Some("test.package".to_string());
    
    // Add two services with the same name
    proto_file.services.push(Service::new("DuplicateService".to_string()));
    proto_file.services.push(Service::new("DuplicateService".to_string()));
    
    // Validate the proto file
    let result = engine.validate_proto_file_internal(&proto_file);
    
    // Should have validation errors for duplicate definitions
    assert!(!result.is_valid);
    assert!(!result.errors.is_empty());
    
    // Check that we have duplicate definition errors
    let duplicate_errors: Vec<_> = result.errors.iter()
        .filter(|e| matches!(e, ValidationError::DuplicateDefinition { .. }))
        .collect();
    
    assert_eq!(duplicate_errors.len(), 1);
}

#[test]
fn test_http_route_validation() {
    let mut engine = ValidationEngine::new();
    
    // Create HTTP routes with conflicts
    let routes = vec![
        HttpRoute::new(
            "TestService".to_string(),
            "Method1".to_string(),
            HttpMethod::Get,
            "/users/{id}".to_string(),
        ),
        HttpRoute::new(
            "TestService".to_string(),
            "Method2".to_string(),
            HttpMethod::Get,
            "/users/{id}".to_string(), // Same path and method - conflict
        ),
    ];
    
    // Validate the routes
    let result = engine.validate_http_routes_internal(&routes);
    
    // Should have validation errors for conflicting routes
    assert!(!result.is_valid);
    assert!(!result.errors.is_empty());
    
    // Check that we have conflicting route errors
    let conflict_errors: Vec<_> = result.errors.iter()
        .filter(|e| matches!(e, ValidationError::ConflictingRoutes { .. }))
        .collect();
    
    assert_eq!(conflict_errors.len(), 1);
}

#[test]
fn test_path_template_validation() {
    let engine = ValidationEngine::new();
    
    // Test valid path templates
    assert!(engine.validate_path_template("/users").is_ok());
    assert!(engine.validate_path_template("/users/{id}").is_ok());
    assert!(engine.validate_path_template("/users/{user_id}/posts/{post_id}").is_ok());
    
    // Test invalid path templates
    assert!(engine.validate_path_template("").is_err());
    assert!(engine.validate_path_template("users").is_err()); // Missing leading slash
    assert!(engine.validate_path_template("/users/{id").is_err()); // Unmatched brace
    assert!(engine.validate_path_template("/users/id}").is_err()); // Unmatched brace
    assert!(engine.validate_path_template("/users/{{id}}").is_err()); // Nested braces
}

#[test]
fn test_error_suggestion_generation() {
    let mut engine = ValidationEngine::new();
    
    // Create a proto file with an undefined type that's similar to a defined type
    let mut proto_file = ProtoFile::new();
    proto_file.package = Some("test.package".to_string());
    
    // Add a message type
    let message = Message {
        name: "UserMessage".to_string(),
        fields: vec![],
        nested_messages: vec![],
        nested_enums: vec![],
        options: vec![],
        comments: vec![],
    };
    
    proto_file.messages.push(message);
    
    // Add a service with a similar but incorrect type name
    let service = Service::new("TestService".to_string())
        .with_method(RpcMethod::new(
            "TestMethod".to_string(),
            TypeReference::new("UserMesage".to_string()), // Typo: missing 's'
            TypeReference::new("UserMessage".to_string()),
        ));
    
    proto_file.services.push(service);
    
    // Validate the proto file
    let result = engine.validate_proto_file_internal(&proto_file);
    
    // Should have validation errors and suggestions
    assert!(!result.is_valid);
    assert!(!result.errors.is_empty());
    assert!(!result.suggestions.is_empty());
    
    // Check that we have a suggestion for the typo
    let suggestion = &result.suggestions[0];
    assert!(suggestion.message.contains("UserMessage"));
    assert!(suggestion.confidence > 0.5);
}

#[test]
fn test_validator_trait_implementation() {
    let engine = ValidationEngine::new();
    
    // Create a simple proto file
    let mut proto_file = ProtoFile::new();
    proto_file.package = Some("test.package".to_string());
    
    // Test the Validator trait methods
    let result = engine.validate_proto_file(&proto_file);
    assert!(result.is_ok());
    
    let report = result.unwrap();
    assert!(report.is_valid);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
    assert!(report.suggestions.is_empty());
}