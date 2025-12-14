//! Core trait interfaces for Proto HTTP Parser v2
//! 
//! This module defines the fundamental abstractions that enable modular,
//! extensible parsing and code generation for Protocol Buffer HTTP annotations.

use std::path::{Path, PathBuf};
use std::error::Error;
use serde::{Deserialize, Serialize};

/// Core data structures representing parsed Protocol Buffer content
pub mod data;

/// Configuration types for various components
pub mod config;

/// Error types for the library
pub mod errors;

/// Tests for data structures
#[cfg(test)]
mod data_test;

// Re-export core types
pub use data::*;
pub use config::*;
pub use errors::*;

/// Trait for parsing Protocol Buffer files and extracting service definitions
/// 
/// This trait abstracts the parsing logic, allowing for different parsing
/// implementations (nom-based, custom, etc.) while maintaining a consistent interface.
pub trait ProtoParser {
    type Error: Error + Send + Sync + 'static;
    
    /// Parse a Protocol Buffer file from a file path
    fn parse_file(&self, path: &Path) -> Result<ProtoFile, Self::Error>;
    
    /// Parse Protocol Buffer content from a string
    fn parse_content(&self, content: &str) -> Result<ProtoFile, Self::Error>;
    
    /// Parse a Protocol Buffer file with import resolution
    fn parse_with_imports(&self, path: &Path, import_paths: &[PathBuf]) -> Result<ProtoFile, Self::Error>;
}

/// Trait for extracting and validating HTTP annotations from Protocol Buffer services
/// 
/// This trait handles the extraction of google.api.http annotations and converts
/// them into structured HTTP route definitions.
pub trait HttpAnnotationExtractor {
    type Error: Error + Send + Sync + 'static;
    
    /// Extract HTTP routes from a parsed Protocol Buffer file
    fn extract_routes(&self, proto_file: &ProtoFile) -> Result<Vec<HttpRoute>, Self::Error>;
    
    /// Validate HTTP annotations for consistency and correctness
    fn validate_annotations(&self, routes: &[HttpRoute]) -> Result<(), Self::Error>;
}

/// Trait for comprehensive validation of Protocol Buffer files and HTTP annotations
/// 
/// This trait provides detailed validation capabilities including type checking,
/// reference validation, and error recovery mechanisms.
pub trait Validator {
    type Error: Error + Send + Sync + 'static;
    
    /// Validate a Protocol Buffer file for syntax and semantic correctness
    fn validate_proto_file(&self, proto_file: &ProtoFile) -> Result<ValidationReport, Self::Error>;
    
    /// Validate HTTP routes for consistency and correctness
    fn validate_http_routes(&self, routes: &[HttpRoute]) -> Result<ValidationReport, Self::Error>;
    
    /// Validate type references and dependencies
    fn validate_type_references(&self, proto_file: &ProtoFile) -> Result<(), Self::Error>;
}

/// Validation report containing results and suggestions
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors found
    pub errors: Vec<ValidationError>,
    /// Warnings (non-fatal issues)
    pub warnings: Vec<ValidationWarning>,
    /// Suggestions for fixing issues
    pub suggestions: Vec<ValidationSuggestion>,
}

/// Validation warning (non-fatal issue)
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,
    /// File location if applicable
    pub location: Option<SourceLocation>,
    /// Warning category
    pub category: WarningCategory,
}

/// Categories of validation warnings
#[derive(Debug, Clone, PartialEq)]
pub enum WarningCategory {
    /// Unused imports or definitions
    Unused,
    /// Deprecated features
    Deprecated,
    /// Style or convention issues
    Style,
    /// Performance concerns
    Performance,
}

/// Validation suggestion for fixing issues
#[derive(Debug, Clone)]
pub struct ValidationSuggestion {
    /// The issue this suggestion addresses
    pub issue_type: String,
    /// Human-readable suggestion message
    pub message: String,
    /// Suggested code fix if applicable
    pub suggested_fix: Option<String>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Location where the fix should be applied
    pub location: Option<SourceLocation>,
}

/// Source code location for errors and suggestions
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// File path
    pub file: Option<PathBuf>,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Length of the relevant text
    pub length: Option<usize>,
}

/// Trait for generating code from parsed Protocol Buffer services and HTTP routes
/// 
/// This trait abstracts the code generation process, allowing for different
/// target frameworks and customization options.
pub trait CodeGenerator {
    type Error: Error + Send + Sync + 'static;
    
    /// Generate controller code for a service
    fn generate_controller(&self, service: &Service, routes: &[HttpRoute]) -> Result<GeneratedCode, Self::Error>;
    
    /// Generate service trait interface for dependency injection
    fn generate_service_trait(&self, service: &Service, routes: &[HttpRoute]) -> Result<GeneratedCode, Self::Error>;
}

/// Trait for template-based code generation
/// 
/// This trait provides a flexible template system for customizing generated code
/// structure and patterns.
pub trait TemplateEngine {
    type Error: Error + Send + Sync + 'static;
    
    /// Render a template with the given context
    fn render(&self, template_name: &str, context: &TemplateContext) -> Result<String, Self::Error>;
    
    /// Register a new template
    fn register_template(&mut self, name: &str, content: &str) -> Result<(), Self::Error>;
    
    /// Register a template helper function
    fn register_helper(&mut self, name: &str, helper: Box<dyn TemplateHelper>) -> Result<(), Self::Error>;
}

/// Trait for template helper functions
/// 
/// Template helpers provide custom logic that can be used within templates
/// for formatting, conversion, and other operations.
pub trait TemplateHelper: Send + Sync {
    /// Execute the helper function with the given arguments
    fn call(&self, args: &[TemplateValue]) -> Result<TemplateValue, Box<dyn Error>>;
}

/// Template context data for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    /// Service information
    pub service: Service,
    /// HTTP routes for the service
    pub routes: Vec<HttpRoute>,
    /// Additional custom data
    pub custom_data: std::collections::HashMap<String, TemplateValue>,
}

/// Template value type for dynamic data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<TemplateValue>),
    Object(std::collections::HashMap<String, TemplateValue>),
    Null,
}

/// Generated code result
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    /// The generated code content
    pub content: String,
    /// Required imports for the generated code
    pub imports: Vec<String>,
    /// Dependencies that need to be added to Cargo.toml
    pub dependencies: Vec<String>,
}

impl GeneratedCode {
    /// Create a new GeneratedCode instance
    pub fn new(content: String) -> Self {
        Self {
            content,
            imports: Vec::new(),
            dependencies: Vec::new(),
        }
    }
    
    /// Add an import to the generated code
    pub fn with_import(mut self, import: String) -> Self {
        self.imports.push(import);
        self
    }
    
    /// Add a dependency to the generated code
    pub fn with_dependency(mut self, dependency: String) -> Self {
        self.dependencies.push(dependency);
        self
    }
}