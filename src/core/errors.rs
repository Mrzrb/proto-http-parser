//! Error types for the Proto HTTP Parser library

use thiserror::Error;
use std::path::PathBuf;

/// Main error type for the Proto HTTP Parser library
#[derive(Debug, Error)]
pub enum ProtoHttpParserError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Code generation error: {0}")]
    CodeGeneration(#[from] CodeGenerationError),
    
    #[error("Template error: {0}")]
    Template(#[from] TemplateError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),
}

/// Errors that occur during Protocol Buffer parsing
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Syntax error at line {line}, column {column}: {message}")]
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },
    
    #[error("Unexpected token '{token}' at line {line}, expected {expected}")]
    UnexpectedToken {
        token: String,
        line: usize,
        expected: String,
    },
    
    #[error("Import not found: {import_path}")]
    ImportNotFound {
        import_path: String,
    },
    
    #[error("Circular import detected: {cycle:?}")]
    CircularImport {
        cycle: Vec<String>,
    },
    
    #[error("Invalid Protocol Buffer syntax: {message}")]
    InvalidSyntax {
        message: String,
    },
    
    #[error("Unsupported feature: {feature}")]
    UnsupportedFeature {
        feature: String,
    },
    
    #[error("File not found: {path}")]
    FileNotFound {
        path: PathBuf,
    },
    
    #[error("Invalid encoding in file: {path}")]
    InvalidEncoding {
        path: PathBuf,
    },
}

/// Errors that occur during validation
#[derive(Debug, Clone, Error)]
pub enum ValidationError {
    #[error("Undefined type: {type_name} at line {line}")]
    UndefinedType {
        type_name: String,
        line: usize,
    },
    
    #[error("Duplicate definition: {name} at line {line}")]
    DuplicateDefinition {
        name: String,
        line: usize,
    },
    
    #[error("Invalid HTTP annotation: {message} at line {line}")]
    InvalidHttpAnnotation {
        message: String,
        line: usize,
    },
    
    #[error("Conflicting HTTP routes: {route1} and {route2}")]
    ConflictingRoutes {
        route1: String,
        route2: String,
    },
    
    #[error("Invalid path parameter: {param} in path {path}")]
    InvalidPathParameter {
        param: String,
        path: String,
    },
    
    #[error("Missing required field: {field} in {context}")]
    MissingRequiredField {
        field: String,
        context: String,
    },
    
    #[error("Invalid field reference: {field} in {message}")]
    InvalidFieldReference {
        field: String,
        message: String,
    },
    
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        expected: String,
        found: String,
    },
    
    #[error("HTTP annotation error: {message}")]
    HttpAnnotationError {
        message: String,
    },
}

/// Errors that occur during code generation
#[derive(Debug, Error)]
pub enum CodeGenerationError {
    #[error("Template not found: {template_name}")]
    TemplateNotFound {
        template_name: String,
    },
    
    #[error("Invalid template syntax: {message}")]
    InvalidTemplateSyntax {
        message: String,
    },
    
    #[error("Code formatting failed: {message}")]
    FormattingFailed {
        message: String,
    },
    
    #[error("Unsupported type mapping: {from_type} -> {to_type}")]
    UnsupportedTypeMapping {
        from_type: String,
        to_type: String,
    },
    
    #[error("Missing dependency: {dependency}")]
    MissingDependency {
        dependency: String,
    },
    
    #[error("Invalid identifier: {identifier}")]
    InvalidIdentifier {
        identifier: String,
    },
    
    #[error("Generation context error: {message}")]
    ContextError {
        message: String,
    },
}

/// Errors that occur in the template engine
#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Template compilation failed: {message}")]
    CompilationFailed {
        message: String,
    },
    
    #[error("Template rendering failed: {message}")]
    RenderingFailed {
        message: String,
    },
    
    #[error("Helper function error: {helper_name}: {message}")]
    HelperError {
        helper_name: String,
        message: String,
    },
    
    #[error("Invalid template data: {message}")]
    InvalidData {
        message: String,
    },
    
    #[error("Template not registered: {template_name}")]
    TemplateNotRegistered {
        template_name: String,
    },
}

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid configuration: {message}")]
    Invalid {
        message: String,
    },
    
    #[error("Configuration file not found: {path}")]
    FileNotFound {
        path: PathBuf,
    },
    
    #[error("Configuration parsing failed: {message}")]
    ParseFailed {
        message: String,
    },
    
    #[error("Missing required configuration: {key}")]
    MissingRequired {
        key: String,
    },
    
    #[error("File error for {path}: {error}")]
    FileError {
        path: PathBuf,
        error: String,
    },
    
    #[error("Parse error: {error}")]
    ParseError {
        error: String,
    },
    
    #[error("Invalid value for {key}: '{value}', expected {expected}")]
    InvalidValue {
        key: String,
        value: String,
        expected: String,
    },
    
    #[error("Validation error for {field}: {message}")]
    ValidationError {
        field: String,
        message: String,
    },
    
    #[error("Serialization error: {error}")]
    SerializationError {
        error: String,
    },
}

/// Plugin-specific errors
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin initialization failed: {message}")]
    InitializationFailed { message: String },
    
    #[error("Plugin not found: {name}")]
    NotFound { name: String },
    
    #[error("Plugin incompatible: {name} requires library version {required}, found {found}")]
    Incompatible { name: String, required: String, found: String },
    
    #[error("Plugin configuration error: {message}")]
    ConfigurationError { message: String },
    
    #[error("Plugin execution error in {plugin}: {message}")]
    ExecutionError { plugin: String, message: String },
    
    #[error("Plugin dependency error: {message}")]
    DependencyError { message: String },
    
    #[error("Plugin loading error: {message}")]
    LoadingError { message: String },
}

/// Context information for errors
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// File path where the error occurred
    pub file_path: Option<PathBuf>,
    /// Line number
    pub line: Option<usize>,
    /// Column number
    pub column: Option<usize>,
    /// Additional context information
    pub context: Option<String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new() -> Self {
        Self {
            file_path: None,
            line: None,
            column: None,
            context: None,
        }
    }
    
    /// Set the file path
    pub fn with_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.file_path = Some(path.into());
        self
    }
    
    /// Set the line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }
    
    /// Set the column number
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }
    
    /// Set additional context
    pub fn with_context<S: Into<String>>(mut self, context: S) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Result type alias for parse operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Result type alias for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Result type alias for code generation operations
pub type CodeGenerationResult<T> = Result<T, CodeGenerationError>;

/// Result type alias for template operations
pub type TemplateResult<T> = Result<T, TemplateError>;

/// Trait for adding context to errors
pub trait WithContext<T> {
    /// Add context to an error
    fn with_context(self, context: ErrorContext) -> Result<T, ProtoHttpParserError>;
}

impl<T, E> WithContext<T> for Result<T, E>
where
    E: Into<ProtoHttpParserError>,
{
    fn with_context(self, _context: ErrorContext) -> Result<T, ProtoHttpParserError> {
        self.map_err(|e| e.into())
    }
}

/// Error collector for gathering multiple errors during validation
#[derive(Debug, Default)]
pub struct ErrorCollector {
    /// Collected errors
    errors: Vec<ProtoHttpParserError>,
    /// Maximum number of errors to collect
    max_errors: usize,
    /// Whether to continue after errors
    continue_on_error: bool,
}

impl ErrorCollector {
    /// Create a new error collector
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            max_errors: 100,
            continue_on_error: true,
        }
    }

    /// Create an error collector with custom limits
    pub fn with_limits(max_errors: usize, continue_on_error: bool) -> Self {
        Self {
            errors: Vec::new(),
            max_errors,
            continue_on_error,
        }
    }

    /// Add an error to the collection
    pub fn add_error<E: Into<ProtoHttpParserError>>(&mut self, error: E) {
        if self.errors.len() < self.max_errors {
            self.errors.push(error.into());
        }
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the number of collected errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if we should continue processing after errors
    pub fn should_continue(&self) -> bool {
        self.continue_on_error && self.errors.len() < self.max_errors
    }

    /// Get all collected errors
    pub fn errors(&self) -> &[ProtoHttpParserError] {
        &self.errors
    }

    /// Take all errors, leaving the collector empty
    pub fn take_errors(&mut self) -> Vec<ProtoHttpParserError> {
        std::mem::take(&mut self.errors)
    }

    /// Convert to a result, returning the first error if any exist
    pub fn into_result<T>(self, value: T) -> Result<T, ProtoHttpParserError> {
        if let Some(error) = self.errors.into_iter().next() {
            Err(error)
        } else {
            Ok(value)
        }
    }

    /// Convert to a result with all errors
    pub fn into_result_with_all_errors<T>(self, value: T) -> Result<T, MultipleErrors> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(MultipleErrors {
                errors: self.errors,
            })
        }
    }
}

/// Error type for multiple validation errors
#[derive(Debug, Error)]
#[error("Multiple errors occurred: {}", format_errors(&self.errors))]
pub struct MultipleErrors {
    /// The collected errors
    pub errors: Vec<ProtoHttpParserError>,
}

impl MultipleErrors {
    /// Create a new multiple errors instance
    pub fn new(errors: Vec<ProtoHttpParserError>) -> Self {
        Self { errors }
    }

    /// Get the errors
    pub fn errors(&self) -> &[ProtoHttpParserError] {
        &self.errors
    }

    /// Get the number of errors
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Check if there are no errors
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Format multiple errors for display
fn format_errors(errors: &[ProtoHttpParserError]) -> String {
    if errors.len() == 1 {
        errors[0].to_string()
    } else {
        format!("{} errors (showing first: {})", errors.len(), errors[0])
    }
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Skip the current element and continue
    Skip,
    /// Use a default value and continue
    UseDefault,
    /// Stop processing immediately
    Stop,
    /// Try an alternative parsing approach
    Retry,
}

/// Error recovery context
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    /// The recovery strategy to use
    pub strategy: RecoveryStrategy,
    /// Additional context information
    pub context: String,
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

/// Trait for error recovery
pub trait ErrorRecovery {
    /// Attempt to recover from an error
    fn recover(&self, error: &ProtoHttpParserError) -> Option<RecoveryContext>;
}

/// Default error recovery implementation
#[derive(Debug, Default)]
pub struct DefaultErrorRecovery;

impl ErrorRecovery for DefaultErrorRecovery {
    fn recover(&self, error: &ProtoHttpParserError) -> Option<RecoveryContext> {
        match error {
            ProtoHttpParserError::Parse(ParseError::Syntax { .. }) => {
                Some(RecoveryContext {
                    strategy: RecoveryStrategy::Skip,
                    context: "Skipping malformed syntax element".to_string(),
                    suggested_fix: Some("Check syntax against Protocol Buffer specification".to_string()),
                })
            }
            ProtoHttpParserError::Parse(ParseError::UnexpectedToken { expected, .. }) => {
                Some(RecoveryContext {
                    strategy: RecoveryStrategy::Retry,
                    context: format!("Expected token: {}", expected),
                    suggested_fix: Some(format!("Replace with expected token: {}", expected)),
                })
            }
            ProtoHttpParserError::Validation(ValidationError::UndefinedType { .. }) => {
                Some(RecoveryContext {
                    strategy: RecoveryStrategy::Skip,
                    context: "Skipping undefined type reference".to_string(),
                    suggested_fix: Some("Define the missing type or check import statements".to_string()),
                })
            }
            _ => None,
        }
    }
}