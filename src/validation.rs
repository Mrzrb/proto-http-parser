//! Validation engine for Protocol Buffer files and HTTP annotations
//!
//! This module provides comprehensive validation functionality including:
//! - Type checking and reference validation
//! - HTTP annotation consistency checks
//! - Error recovery and suggestion generation
//! - Multi-error collection and reporting

use crate::core::{
    data::*,
    errors::*,
    Validator, ValidationReport, ValidationSuggestion,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Validation engine for Protocol Buffer files and HTTP annotations
pub struct ValidationEngine {
    /// Configuration for validation behavior
    config: ValidationConfig,
    /// Collected errors during validation
    errors: Vec<ValidationError>,
    /// Type registry for reference validation
    type_registry: TypeRegistry,
}

/// Configuration for the validation engine
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to perform strict type checking
    pub strict_type_checking: bool,
    /// Whether to validate HTTP method compatibility
    pub validate_http_methods: bool,
    /// Whether to check for unused imports
    pub check_unused_imports: bool,
    /// Whether to validate field references in HTTP annotations
    pub validate_field_references: bool,
    /// Maximum number of errors to collect before stopping
    pub max_errors: usize,
    /// Whether to generate suggestions for common errors
    pub generate_suggestions: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict_type_checking: true,
            validate_http_methods: true,
            check_unused_imports: true,
            validate_field_references: true,
            max_errors: 50,
            generate_suggestions: true,
        }
    }
}

/// Registry for tracking type definitions and references
#[derive(Debug, Default)]
pub struct TypeRegistry {
    /// Map of type names to their definitions
    types: HashMap<String, TypeDefinition>,
    /// Map of package names to their types
    packages: HashMap<String, HashSet<String>>,
    /// Map of imports to their resolved paths
    imports: HashMap<String, PathBuf>,
    /// Set of used types for unused import detection
    used_types: HashSet<String>,
}

/// Information about a type definition
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// The type name
    pub name: String,
    /// The package it belongs to
    pub package: Option<String>,
    /// The file where it's defined
    pub file_path: Option<PathBuf>,
    /// Line number where it's defined
    pub line: Option<usize>,
    /// The kind of type (message, enum, service)
    pub kind: TypeKind,
    /// Fields for messages, values for enums
    pub members: Vec<TypeMember>,
}

/// Kind of type definition
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Message,
    Enum,
    Service,
    Scalar,
}

/// Member of a type (field, enum value, etc.)
#[derive(Debug, Clone)]
pub struct TypeMember {
    /// Member name
    pub name: String,
    /// Member type (for fields)
    pub type_ref: Option<TypeReference>,
    /// Line number where defined
    pub line: Option<usize>,
}

/// Validation result with collected errors and suggestions
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Collected validation errors
    pub errors: Vec<ValidationError>,
    /// Generated suggestions for fixing errors
    pub suggestions: Vec<ErrorSuggestion>,
    /// Type registry after validation
    pub type_registry: TypeRegistry,
}

/// Suggestion for fixing a validation error
#[derive(Debug, Clone)]
pub struct ErrorSuggestion {
    /// The error this suggestion addresses
    pub error_type: String,
    /// Human-readable suggestion message
    pub message: String,
    /// Suggested fix (if applicable)
    pub suggested_fix: Option<String>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
}

impl ValidationEngine {
    /// Create a new validation engine with default configuration
    pub fn new() -> Self {
        Self::with_config(ValidationConfig::default())
    }

    /// Create a new validation engine with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self {
            config,
            errors: Vec::new(),
            type_registry: TypeRegistry::default(),
        }
    }

    /// Validate a Protocol Buffer file
    pub fn validate_proto_file_internal(&mut self, proto_file: &ProtoFile) -> ValidationResult {
        self.errors.clear();
        self.type_registry = TypeRegistry::default();

        // Build type registry first
        self.build_type_registry(proto_file);

        // Validate syntax and structure
        self.validate_syntax(proto_file);

        // Validate type references
        if self.config.strict_type_checking {
            self.validate_type_references_internal(proto_file);
        }

        // Validate services and methods
        for service in &proto_file.services {
            self.validate_service(service);
        }

        // Check for unused imports
        if self.config.check_unused_imports {
            self.check_unused_imports(proto_file);
        }

        // Generate suggestions
        let suggestions = if self.config.generate_suggestions {
            self.generate_suggestions()
        } else {
            Vec::new()
        };

        ValidationResult {
            is_valid: self.errors.is_empty(),
            errors: self.errors.clone(),
            suggestions,
            type_registry: std::mem::take(&mut self.type_registry),
        }
    }

    /// Validate HTTP routes for consistency
    pub fn validate_http_routes_internal(&mut self, routes: &[HttpRoute]) -> ValidationResult {
        self.errors.clear();

        // Check for route conflicts
        self.check_route_conflicts(routes);

        // Validate individual routes
        for route in routes {
            self.validate_http_route(route);
        }

        // Generate suggestions
        let suggestions = if self.config.generate_suggestions {
            self.generate_suggestions()
        } else {
            Vec::new()
        };

        ValidationResult {
            is_valid: self.errors.is_empty(),
            errors: self.errors.clone(),
            suggestions,
            type_registry: std::mem::take(&mut self.type_registry),
        }
    }

    /// Build type registry from proto file
    fn build_type_registry(&mut self, proto_file: &ProtoFile) {
        let package = proto_file.package.clone();

        // Register messages
        for message in &proto_file.messages {
            self.register_message(message, &package);
        }

        // Register enums
        for enum_def in &proto_file.enums {
            self.register_enum(enum_def, &package);
        }

        // Register services
        for service in &proto_file.services {
            self.register_service(service, &package);
        }

        // Register imports
        for import in &proto_file.imports {
            self.type_registry.imports.insert(
                import.path.clone(),
                PathBuf::from(&import.path),
            );
        }
    }

    /// Register a message type
    fn register_message(&mut self, message: &Message, package: &Option<String>) {
        let full_name = self.build_full_type_name(&message.name, package);
        
        let members: Vec<TypeMember> = message.fields.iter().map(|field| {
            TypeMember {
                name: field.name.clone(),
                type_ref: match &field.field_type {
                    FieldType::MessageOrEnum(type_ref) => Some(type_ref.clone()),
                    _ => None,
                },
                line: None, // TODO: Add line tracking to data structures
            }
        }).collect();

        let type_def = TypeDefinition {
            name: full_name.clone(),
            package: package.clone(),
            file_path: None, // TODO: Add file path tracking
            line: None, // TODO: Add line tracking to data structures
            kind: TypeKind::Message,
            members,
        };

        self.type_registry.types.insert(full_name.clone(), type_def);
        
        if let Some(pkg) = package {
            self.type_registry.packages
                .entry(pkg.clone())
                .or_default()
                .insert(full_name);
        }
    }

    /// Register an enum type
    fn register_enum(&mut self, enum_def: &Enum, package: &Option<String>) {
        let full_name = self.build_full_type_name(&enum_def.name, package);
        
        let members: Vec<TypeMember> = enum_def.values.iter().map(|value| {
            TypeMember {
                name: value.name.clone(),
                type_ref: None,
                line: None, // TODO: Add line tracking to data structures
            }
        }).collect();

        let type_def = TypeDefinition {
            name: full_name.clone(),
            package: package.clone(),
            file_path: None,
            line: None, // TODO: Add line tracking to data structures
            kind: TypeKind::Enum,
            members,
        };

        self.type_registry.types.insert(full_name.clone(), type_def);
        
        if let Some(pkg) = package {
            self.type_registry.packages
                .entry(pkg.clone())
                .or_default()
                .insert(full_name);
        }
    }

    /// Register a service type
    fn register_service(&mut self, service: &Service, package: &Option<String>) {
        let full_name = self.build_full_type_name(&service.name, package);
        
        let members: Vec<TypeMember> = service.methods.iter().map(|method| {
            TypeMember {
                name: method.name.clone(),
                type_ref: Some(method.input_type.clone()),
                line: None, // TODO: Add line tracking to data structures
            }
        }).collect();

        let type_def = TypeDefinition {
            name: full_name.clone(),
            package: package.clone(),
            file_path: None,
            line: None, // TODO: Add line tracking to data structures
            kind: TypeKind::Service,
            members,
        };

        self.type_registry.types.insert(full_name.clone(), type_def);
        
        if let Some(pkg) = package {
            self.type_registry.packages
                .entry(pkg.clone())
                .or_default()
                .insert(full_name);
        }
    }

    /// Build full type name with package prefix
    fn build_full_type_name(&self, name: &str, package: &Option<String>) -> String {
        match package {
            Some(pkg) => format!("{}.{}", pkg, name),
            None => name.to_string(),
        }
    }

    /// Validate syntax and basic structure
    fn validate_syntax(&mut self, proto_file: &ProtoFile) {
        // Check for duplicate service names
        let mut service_names = HashSet::new();
        for service in &proto_file.services {
            if !service_names.insert(&service.name) {
                self.add_error(ValidationError::DuplicateDefinition {
                    name: service.name.clone(),
                    line: 0, // TODO: Add line tracking to data structures
                });
            }
        }

        // Check for duplicate message names
        let mut message_names = HashSet::new();
        for message in &proto_file.messages {
            if !message_names.insert(&message.name) {
                self.add_error(ValidationError::DuplicateDefinition {
                    name: message.name.clone(),
                    line: 0, // TODO: Add line tracking to data structures
                });
            }
        }

        // Check for duplicate enum names
        let mut enum_names = HashSet::new();
        for enum_def in &proto_file.enums {
            if !enum_names.insert(&enum_def.name) {
                self.add_error(ValidationError::DuplicateDefinition {
                    name: enum_def.name.clone(),
                    line: 0, // TODO: Add line tracking to data structures
                });
            }
        }
    }

    /// Validate type references (internal implementation)
    fn validate_type_references_internal(&mut self, proto_file: &ProtoFile) {
        // Validate message field types
        for message in &proto_file.messages {
            for field in &message.fields {
                if let FieldType::MessageOrEnum(type_ref) = &field.field_type {
                    self.validate_type_reference(type_ref, None);
                }
            }
        }

        // Validate service method types
        for service in &proto_file.services {
            for method in &service.methods {
                self.validate_type_reference(&method.input_type, None);
                self.validate_type_reference(&method.output_type, None);
            }
        }
    }

    /// Validate a single type reference
    fn validate_type_reference(&mut self, type_ref: &TypeReference, _line: Option<usize>) {
        // Mark as used
        self.type_registry.used_types.insert(type_ref.name.clone());
        
        // Check if type exists
        if !self.is_type_defined(&type_ref.name) && !self.is_builtin_type(&type_ref.name) {
            self.add_error(ValidationError::UndefinedType {
                type_name: type_ref.name.clone(),
                line: 0, // TODO: Add line tracking to data structures
            });
        }
    }

    /// Check if a type is defined in the registry
    fn is_type_defined(&self, type_name: &str) -> bool {
        self.type_registry.types.contains_key(type_name) ||
        self.type_registry.types.keys().any(|key| key.ends_with(&format!(".{}", type_name)))
    }

    /// Check if a type is a built-in Protocol Buffer type
    fn is_builtin_type(&self, type_name: &str) -> bool {
        matches!(type_name, 
            "double" | "float" | "int32" | "int64" | "uint32" | "uint64" |
            "sint32" | "sint64" | "fixed32" | "fixed64" | "sfixed32" | "sfixed64" |
            "bool" | "string" | "bytes" | "google.protobuf.Timestamp" |
            "google.protobuf.Duration" | "google.protobuf.Any" |
            "google.protobuf.Empty" | "google.protobuf.Struct" |
            "google.protobuf.Value" | "google.protobuf.ListValue"
        )
    }

    /// Validate a service definition
    fn validate_service(&mut self, service: &Service) {
        // Check for duplicate method names
        let mut method_names = HashSet::new();
        for method in &service.methods {
            if !method_names.insert(&method.name) {
                self.add_error(ValidationError::DuplicateDefinition {
                    name: format!("{}.{}", service.name, method.name),
                    line: 0, // TODO: Add line tracking to data structures
                });
            }

            // Validate HTTP annotations if present
            if let Some(http_annotation) = &method.http_annotation {
                self.validate_http_annotation(http_annotation, method);
            }
        }
    }

    /// Validate HTTP annotation
    fn validate_http_annotation(&mut self, annotation: &HttpAnnotation, method: &RpcMethod) {
        // Validate path template
        if let Err(e) = self.validate_path_template(&annotation.path) {
            self.add_error(e);
        }

        // Validate body field reference
        if let Some(body) = &annotation.body {
            if body != "*" {
                self.validate_field_reference(body, &method.input_type, None);
            }
        }

        // Validate HTTP method compatibility
        if self.config.validate_http_methods {
            self.validate_http_method_compatibility(&annotation.method, &annotation.body, None);
        }
    }

    /// Validate path template syntax
    pub fn validate_path_template(&self, path: &str) -> Result<(), ValidationError> {
        if path.is_empty() {
            return Err(ValidationError::InvalidHttpAnnotation {
                message: "Path template cannot be empty".to_string(),
                line: 0,
            });
        }

        if !path.starts_with('/') {
            return Err(ValidationError::InvalidHttpAnnotation {
                message: "Path template must start with '/'".to_string(),
                line: 0,
            });
        }

        // Validate parameter syntax
        let mut brace_count = 0;
        let chars = path.chars().peekable();
        
        for ch in chars {
            match ch {
                '{' => {
                    brace_count += 1;
                    if brace_count > 1 {
                        return Err(ValidationError::InvalidHttpAnnotation {
                            message: "Nested braces are not allowed in path templates".to_string(),
                            line: 0,
                        });
                    }
                }
                '}' => {
                    if brace_count == 0 {
                        return Err(ValidationError::InvalidHttpAnnotation {
                            message: "Unmatched closing brace in path template".to_string(),
                            line: 0,
                        });
                    }
                    brace_count -= 1;
                }
                _ => {}
            }
        }

        if brace_count != 0 {
            return Err(ValidationError::InvalidHttpAnnotation {
                message: "Unmatched opening brace in path template".to_string(),
                line: 0,
            });
        }

        Ok(())
    }

    /// Validate field reference in HTTP annotation
    fn validate_field_reference(&mut self, field_path: &str, message_type: &TypeReference, _line: Option<usize>) {
        if !self.config.validate_field_references {
            return;
        }

        // For now, just check if the field path is not empty
        // TODO: Implement full field path validation
        if field_path.is_empty() {
            self.add_error(ValidationError::InvalidFieldReference {
                field: field_path.to_string(),
                message: format!("{:?}", message_type),
            });
        }
    }

    /// Validate HTTP method compatibility
    fn validate_http_method_compatibility(&mut self, method: &HttpMethod, body: &Option<String>, line: Option<usize>) {
        match method {
            HttpMethod::Get | HttpMethod::Delete => {
                if body.is_some() && body.as_ref().unwrap() != "*" {
                    self.add_error(ValidationError::InvalidHttpAnnotation {
                        message: format!("{:?} methods should not have request body", method),
                        line: line.unwrap_or(0),
                    });
                }
            }
            _ => {} // Other methods can have bodies
        }
    }

    /// Check for conflicting HTTP routes
    fn check_route_conflicts(&mut self, routes: &[HttpRoute]) {
        let mut route_signatures = HashMap::new();
        
        for route in routes {
            let signature = format!("{} {}", route.http_method, route.path_template);
            
            if let Some(_existing) = route_signatures.get(&signature) {
                self.add_error(ValidationError::ConflictingRoutes {
                    route1: signature.clone(),
                    route2: format!("{}.{}", route.service_name, route.method_name),
                });
            } else {
                route_signatures.insert(signature, format!("{}.{}", route.service_name, route.method_name));
            }
        }
    }

    /// Validate individual HTTP route
    fn validate_http_route(&mut self, route: &HttpRoute) {
        // Validate path template
        if let Err(e) = self.validate_path_template(&route.path_template) {
            self.add_error(e);
        }

        // Validate path parameters
        for param in &route.path_parameters {
            if param.name.is_empty() {
                self.add_error(ValidationError::InvalidPathParameter {
                    param: param.name.clone(),
                    path: route.path_template.clone(),
                });
            }
        }
    }

    /// Check for unused imports
    fn check_unused_imports(&mut self, proto_file: &ProtoFile) {
        for import in &proto_file.imports {
            // Simple heuristic: check if any used type starts with the import path
            let import_prefix = import.path.replace(".proto", "").replace("/", ".");
            let is_used = self.type_registry.used_types.iter()
                .any(|used_type| used_type.starts_with(&import_prefix));
            
            if !is_used {
                // This is more of a warning than an error, but we'll track it
                // TODO: Add warning system
            }
        }
    }

    /// Generate suggestions for fixing errors
    fn generate_suggestions(&self) -> Vec<ErrorSuggestion> {
        let mut suggestions = Vec::new();

        for error in &self.errors {
            match error {
                ValidationError::UndefinedType { type_name, .. } => {
                    // Suggest similar type names
                    if let Some(suggestion) = self.suggest_similar_type(type_name) {
                        suggestions.push(ErrorSuggestion {
                            error_type: "UndefinedType".to_string(),
                            message: format!("Did you mean '{}'?", suggestion),
                            suggested_fix: Some(suggestion),
                            confidence: 0.8,
                        });
                    }
                }
                ValidationError::InvalidHttpAnnotation { message, .. } => {
                    if message.contains("must start with '/'") {
                        suggestions.push(ErrorSuggestion {
                            error_type: "InvalidHttpAnnotation".to_string(),
                            message: "Add a leading '/' to the path template".to_string(),
                            suggested_fix: None,
                            confidence: 0.9,
                        });
                    }
                }
                _ => {}
            }
        }

        suggestions
    }

    /// Suggest similar type name for undefined types
    fn suggest_similar_type(&self, type_name: &str) -> Option<String> {
        let mut best_match = None;
        let mut best_distance = usize::MAX;



        for defined_type in self.type_registry.types.keys() {
            // Check distance against full name
            let distance = levenshtein_distance(type_name, defined_type);
            if distance < best_distance && distance <= 3 {
                best_distance = distance;
                best_match = Some(defined_type.clone());
            }
            
            // Also check distance against simple name (after last dot)
            if let Some(simple_name) = defined_type.split('.').next_back() {
                let simple_distance = levenshtein_distance(type_name, simple_name);
                if simple_distance < best_distance && simple_distance <= 3 {
                    best_distance = simple_distance;
                    best_match = Some(simple_name.to_string());
                }
            }
        }

        best_match
    }

    /// Add an error to the collection
    fn add_error(&mut self, error: ValidationError) {
        if self.errors.len() < self.config.max_errors {
            self.errors.push(error);
        }
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for ValidationEngine {
    type Error = ValidationError;

    fn validate_proto_file(&self, proto_file: &ProtoFile) -> Result<ValidationReport, Self::Error> {
        let mut engine = self.clone();
        let result = engine.validate_proto_file_internal(proto_file);
        
        Ok(ValidationReport {
            is_valid: result.is_valid,
            errors: result.errors,
            warnings: Vec::new(), // TODO: Implement warnings
            suggestions: result.suggestions.into_iter().map(|s| ValidationSuggestion {
                issue_type: s.error_type,
                message: s.message,
                suggested_fix: s.suggested_fix,
                confidence: s.confidence,
                location: None, // TODO: Add location tracking
            }).collect(),
        })
    }

    fn validate_http_routes(&self, routes: &[HttpRoute]) -> Result<ValidationReport, Self::Error> {
        let mut engine = self.clone();
        let result = engine.validate_http_routes_internal(routes);
        
        Ok(ValidationReport {
            is_valid: result.is_valid,
            errors: result.errors,
            warnings: Vec::new(), // TODO: Implement warnings
            suggestions: result.suggestions.into_iter().map(|s| ValidationSuggestion {
                issue_type: s.error_type,
                message: s.message,
                suggested_fix: s.suggested_fix,
                confidence: s.confidence,
                location: None, // TODO: Add location tracking
            }).collect(),
        })
    }

    fn validate_type_references(&self, proto_file: &ProtoFile) -> Result<(), Self::Error> {
        let mut engine = self.clone();
        engine.build_type_registry(proto_file);
        engine.validate_type_references_internal(proto_file);
        
        if engine.errors.is_empty() {
            Ok(())
        } else {
            Err(engine.errors.into_iter().next().unwrap())
        }
    }
}

// Make ValidationEngine cloneable for the trait implementation
impl Clone for ValidationEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            errors: Vec::new(), // Start with empty errors for new instance
            type_registry: TypeRegistry::default(), // Start with empty registry
        }
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,      // deletion
                    matrix[i][j - 1] + 1       // insertion
                ),
                matrix[i - 1][j - 1] + cost    // substitution
            );
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", "ab"), 1);
        assert_eq!(levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(levenshtein_distance("abc", "axc"), 1);
    }

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert!(config.strict_type_checking);
        assert!(config.validate_http_methods);
        assert_eq!(config.max_errors, 50);
    }

    #[test]
    fn test_path_template_validation() {
        let engine = ValidationEngine::new();
        
        // Valid paths
        assert!(engine.validate_path_template("/users").is_ok());
        assert!(engine.validate_path_template("/users/{id}").is_ok());
        assert!(engine.validate_path_template("/users/{user_id}/posts/{post_id}").is_ok());
        
        // Invalid paths
        assert!(engine.validate_path_template("").is_err());
        assert!(engine.validate_path_template("users").is_err());
        assert!(engine.validate_path_template("/users/{id").is_err());
        assert!(engine.validate_path_template("/users/id}").is_err());
        assert!(engine.validate_path_template("/users/{{id}}").is_err());
    }
}