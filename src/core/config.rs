//! Configuration types for various components

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use crate::core::errors::ConfigError;

/// Configuration for the Protocol Buffer parser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    /// Include paths for resolving imports
    pub include_paths: Vec<PathBuf>,
    /// Whether to preserve comments
    pub preserve_comments: bool,
    /// Whether to validate syntax strictly
    pub strict_validation: bool,
    /// Maximum recursion depth for imports
    pub max_import_depth: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            include_paths: vec![PathBuf::from(".")],
            preserve_comments: true,
            strict_validation: true,
            max_import_depth: 10,
        }
    }
}

/// Configuration for HTTP annotation extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorConfig {
    /// Whether to infer common query parameters
    pub infer_query_params: bool,
    /// List of common query parameter names to infer
    pub common_query_params: Vec<String>,
    /// Whether to validate HTTP method compatibility
    pub validate_http_methods: bool,
    /// Whether to allow custom HTTP methods
    pub allow_custom_methods: bool,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            infer_query_params: true,
            common_query_params: vec![
                "page".to_string(),
                "limit".to_string(),
                "offset".to_string(),
                "sort".to_string(),
                "order".to_string(),
                "filter".to_string(),
                "search".to_string(),
            ],
            validate_http_methods: true,
            allow_custom_methods: false,
        }
    }
}

/// Configuration for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// Target framework (currently only poem-openapi)
    pub target_framework: TargetFramework,
    /// Whether to generate service traits
    pub generate_service_traits: bool,
    /// Whether to use dependency injection pattern
    pub use_dependency_injection: bool,
    /// Custom type mappings
    pub type_mappings: HashMap<String, String>,
    /// Additional imports to include
    pub additional_imports: Vec<String>,
    /// Code formatting options
    pub formatting: FormattingConfig,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            target_framework: TargetFramework::PoemOpenApi,
            generate_service_traits: true,
            use_dependency_injection: true,
            type_mappings: HashMap::new(),
            additional_imports: Vec::new(),
            formatting: FormattingConfig::default(),
        }
    }
}

/// Target framework for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetFramework {
    PoemOpenApi,
    // Future: Axum, Warp, etc.
}

/// Code formatting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattingConfig {
    /// Indentation style
    pub indent_style: IndentStyle,
    /// Number of spaces for indentation (if using spaces)
    pub indent_size: usize,
    /// Maximum line length
    pub max_line_length: usize,
    /// Whether to format generated code with rustfmt
    pub use_rustfmt: bool,
}

impl Default for FormattingConfig {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::Spaces,
            indent_size: 4,
            max_line_length: 100,
            use_rustfmt: true,
        }
    }
}

/// Indentation style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndentStyle {
    Spaces,
    Tabs,
}

/// Configuration for template engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Template directory path
    pub template_dir: Option<PathBuf>,
    /// Whether to use built-in templates
    pub use_builtin_templates: bool,
    /// Custom template overrides
    pub template_overrides: HashMap<String, String>,
    /// Template helper configurations
    pub helpers: HashMap<String, HelperConfig>,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            template_dir: None,
            use_builtin_templates: true,
            template_overrides: HashMap::new(),
            helpers: HashMap::new(),
        }
    }
}

/// Configuration for template helpers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperConfig {
    /// Helper type
    pub helper_type: HelperType,
    /// Helper-specific configuration
    pub config: HashMap<String, String>,
}

/// Template helper types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HelperType {
    SnakeCase,
    CamelCase,
    PascalCase,
    TypeMapping,
    Custom(String),
}



/// Main configuration for the entire library
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ProtoHttpParserConfig {
    /// Parser configuration
    pub parser: ParserConfig,
    /// Extractor configuration
    pub extractor: ExtractorConfig,
    /// Generator configuration
    pub generator: GeneratorConfig,
    /// Template configuration
    pub template: TemplateConfig,
}


impl ProtoHttpParserConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Load configuration from multiple sources with precedence:
    /// 1. Environment variables (highest priority)
    /// 2. Configuration file
    /// 3. Default values (lowest priority)
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Self::default();
        
        // Try to load from default config file locations
        let config_paths = [
            "proto-http-parser.toml",
            ".proto-http-parser.toml",
            "config/proto-http-parser.toml",
        ];
        
        for path in &config_paths {
            if std::path::Path::new(path).exists() {
                config = config.merge_from_file(path)?;
                break;
            }
        }
        
        // Override with environment variables
        config = config.merge_from_env()?;
        
        // Validate the final configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Load configuration from a specific file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::FileError {
                path: path.to_path_buf(),
                error: e.to_string(),
            })?;
        
        let config: Self = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError {
                error: e.to_string(),
            })?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();
        config = config.merge_from_env()?;
        config.validate()?;
        Ok(config)
    }
    
    /// Merge configuration from a file, keeping existing values where not specified
    pub fn merge_from_file<P: AsRef<std::path::Path>>(mut self, path: P) -> Result<Self, ConfigError> {
        let file_config = Self::from_file(path)?;
        self.merge(file_config);
        Ok(self)
    }
    
    /// Merge configuration from environment variables
    pub fn merge_from_env(mut self) -> Result<Self, ConfigError> {
        // Parser configuration
        if let Ok(value) = std::env::var("PROTO_HTTP_PARSER_PRESERVE_COMMENTS") {
            self.parser.preserve_comments = value.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    key: "PROTO_HTTP_PARSER_PRESERVE_COMMENTS".to_string(),
                    value,
                    expected: "boolean".to_string(),
                })?;
        }
        
        if let Ok(value) = std::env::var("PROTO_HTTP_PARSER_STRICT_VALIDATION") {
            self.parser.strict_validation = value.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    key: "PROTO_HTTP_PARSER_STRICT_VALIDATION".to_string(),
                    value,
                    expected: "boolean".to_string(),
                })?;
        }
        
        if let Ok(value) = std::env::var("PROTO_HTTP_PARSER_MAX_IMPORT_DEPTH") {
            self.parser.max_import_depth = value.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    key: "PROTO_HTTP_PARSER_MAX_IMPORT_DEPTH".to_string(),
                    value,
                    expected: "positive integer".to_string(),
                })?;
        }
        
        if let Ok(value) = std::env::var("PROTO_HTTP_PARSER_INCLUDE_PATHS") {
            self.parser.include_paths = value.split(':')
                .map(|s| std::path::PathBuf::from(s.trim()))
                .collect();
        }
        
        // Generator configuration
        if let Ok(value) = std::env::var("PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS") {
            self.generator.generate_service_traits = value.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    key: "PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS".to_string(),
                    value,
                    expected: "boolean".to_string(),
                })?;
        }
        
        if let Ok(value) = std::env::var("PROTO_HTTP_PARSER_USE_DEPENDENCY_INJECTION") {
            self.generator.use_dependency_injection = value.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    key: "PROTO_HTTP_PARSER_USE_DEPENDENCY_INJECTION".to_string(),
                    value,
                    expected: "boolean".to_string(),
                })?;
        }
        
        if let Ok(value) = std::env::var("PROTO_HTTP_PARSER_USE_RUSTFMT") {
            self.generator.formatting.use_rustfmt = value.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    key: "PROTO_HTTP_PARSER_USE_RUSTFMT".to_string(),
                    value,
                    expected: "boolean".to_string(),
                })?;
        }
        
        // Extractor configuration
        if let Ok(value) = std::env::var("PROTO_HTTP_PARSER_INFER_QUERY_PARAMS") {
            self.extractor.infer_query_params = value.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    key: "PROTO_HTTP_PARSER_INFER_QUERY_PARAMS".to_string(),
                    value,
                    expected: "boolean".to_string(),
                })?;
        }
        
        if let Ok(value) = std::env::var("PROTO_HTTP_PARSER_VALIDATE_HTTP_METHODS") {
            self.extractor.validate_http_methods = value.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    key: "PROTO_HTTP_PARSER_VALIDATE_HTTP_METHODS".to_string(),
                    value,
                    expected: "boolean".to_string(),
                })?;
        }
        
        Ok(self)
    }
    
    /// Merge another configuration into this one, with the other taking precedence
    pub fn merge(&mut self, other: Self) {
        // For now, we do a simple replacement merge
        // In a more sophisticated implementation, we might merge individual fields
        *self = other;
    }
    
    /// Validate the configuration for consistency and correctness
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate parser configuration
        if self.parser.max_import_depth == 0 {
            return Err(ConfigError::ValidationError {
                field: "parser.max_import_depth".to_string(),
                message: "must be greater than 0".to_string(),
            });
        }
        
        if self.parser.max_import_depth > 100 {
            return Err(ConfigError::ValidationError {
                field: "parser.max_import_depth".to_string(),
                message: "should not exceed 100 to prevent infinite recursion".to_string(),
            });
        }
        
        // Validate generator configuration
        if self.generator.formatting.indent_size == 0 {
            return Err(ConfigError::ValidationError {
                field: "generator.formatting.indent_size".to_string(),
                message: "must be greater than 0".to_string(),
            });
        }
        
        if self.generator.formatting.indent_size > 8 {
            return Err(ConfigError::ValidationError {
                field: "generator.formatting.indent_size".to_string(),
                message: "should not exceed 8 for readability".to_string(),
            });
        }
        
        if self.generator.formatting.max_line_length < 50 {
            return Err(ConfigError::ValidationError {
                field: "generator.formatting.max_line_length".to_string(),
                message: "should be at least 50 characters".to_string(),
            });
        }
        
        // Validate include paths exist
        for path in &self.parser.include_paths {
            if !path.exists() {
                return Err(ConfigError::ValidationError {
                    field: "parser.include_paths".to_string(),
                    message: format!("path does not exist: {}", path.display()),
                });
            }
        }
        
        Ok(())
    }
    
    /// Save configuration to a file
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), ConfigError> {
        let path = path.as_ref();
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializationError {
                error: e.to_string(),
            })?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConfigError::FileError {
                    path: parent.to_path_buf(),
                    error: e.to_string(),
                })?;
        }
        
        std::fs::write(path, content)
            .map_err(|e| ConfigError::FileError {
                path: path.to_path_buf(),
                error: e.to_string(),
            })?;
        
        Ok(())
    }
    
    /// Create a configuration builder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
    
    /// Builder pattern for parser configuration
    pub fn with_parser_config(mut self, config: ParserConfig) -> Self {
        self.parser = config;
        self
    }
    
    /// Builder pattern for extractor configuration
    pub fn with_extractor_config(mut self, config: ExtractorConfig) -> Self {
        self.extractor = config;
        self
    }
    
    /// Builder pattern for generator configuration
    pub fn with_generator_config(mut self, config: GeneratorConfig) -> Self {
        self.generator = config;
        self
    }
    
    /// Builder pattern for template configuration
    pub fn with_template_config(mut self, config: TemplateConfig) -> Self {
        self.template = config;
        self
    }
}

/// Configuration builder for fluent configuration creation
#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    config: ProtoHttpParserConfig,
}

impl ConfigBuilder {
    /// Create a new configuration builder with default values
    pub fn new() -> Self {
        Self {
            config: ProtoHttpParserConfig::default(),
        }
    }
    
    /// Set parser configuration
    pub fn parser(mut self, parser: ParserConfig) -> Self {
        self.config.parser = parser;
        self
    }
    
    /// Set extractor configuration
    pub fn extractor(mut self, extractor: ExtractorConfig) -> Self {
        self.config.extractor = extractor;
        self
    }
    
    /// Set generator configuration
    pub fn generator(mut self, generator: GeneratorConfig) -> Self {
        self.config.generator = generator;
        self
    }
    
    /// Set template configuration
    pub fn template(mut self, template: TemplateConfig) -> Self {
        self.config.template = template;
        self
    }
    
    /// Configure parser to preserve comments
    pub fn preserve_comments(mut self, preserve: bool) -> Self {
        self.config.parser.preserve_comments = preserve;
        self
    }
    
    /// Configure parser strict validation
    pub fn strict_validation(mut self, strict: bool) -> Self {
        self.config.parser.strict_validation = strict;
        self
    }
    
    /// Set maximum import depth
    pub fn max_import_depth(mut self, depth: usize) -> Self {
        self.config.parser.max_import_depth = depth;
        self
    }
    
    /// Add include path for proto imports
    pub fn add_include_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.parser.include_paths.push(path.into());
        self
    }
    
    /// Set include paths for proto imports
    pub fn include_paths<P: Into<PathBuf>>(mut self, paths: Vec<P>) -> Self {
        self.config.parser.include_paths = paths.into_iter().map(|p| p.into()).collect();
        self
    }
    
    /// Enable or disable service trait generation
    pub fn generate_service_traits(mut self, generate: bool) -> Self {
        self.config.generator.generate_service_traits = generate;
        self
    }
    
    /// Enable or disable dependency injection pattern
    pub fn use_dependency_injection(mut self, use_di: bool) -> Self {
        self.config.generator.use_dependency_injection = use_di;
        self
    }
    
    /// Enable or disable query parameter inference
    pub fn infer_query_params(mut self, infer: bool) -> Self {
        self.config.extractor.infer_query_params = infer;
        self
    }
    
    /// Enable or disable rustfmt formatting
    pub fn use_rustfmt(mut self, use_fmt: bool) -> Self {
        self.config.generator.formatting.use_rustfmt = use_fmt;
        self
    }
    
    /// Set indentation style
    pub fn indent_style(mut self, style: IndentStyle) -> Self {
        self.config.generator.formatting.indent_style = style;
        self
    }
    
    /// Set indentation size
    pub fn indent_size(mut self, size: usize) -> Self {
        self.config.generator.formatting.indent_size = size;
        self
    }
    
    /// Set maximum line length
    pub fn max_line_length(mut self, length: usize) -> Self {
        self.config.generator.formatting.max_line_length = length;
        self
    }
    
    /// Add a custom type mapping
    pub fn add_type_mapping<S: Into<String>>(mut self, from: S, to: S) -> Self {
        self.config.generator.type_mappings.insert(from.into(), to.into());
        self
    }
    
    /// Add an additional import
    pub fn add_import<S: Into<String>>(mut self, import: S) -> Self {
        self.config.generator.additional_imports.push(import.into());
        self
    }
    
    /// Set template directory
    pub fn template_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.config.template.template_dir = Some(dir.into());
        self
    }
    
    /// Enable or disable built-in templates
    pub fn use_builtin_templates(mut self, use_builtin: bool) -> Self {
        self.config.template.use_builtin_templates = use_builtin;
        self
    }
    
    /// Load configuration from file and merge with current settings
    pub fn load_from_file<P: AsRef<std::path::Path>>(mut self, path: P) -> Result<Self, ConfigError> {
        let file_config = ProtoHttpParserConfig::from_file(path)?;
        self.config.merge(file_config);
        Ok(self)
    }
    
    /// Load configuration from environment variables and merge with current settings
    pub fn load_from_env(mut self) -> Result<Self, ConfigError> {
        self.config = self.config.merge_from_env()?;
        Ok(self)
    }
    
    /// Build the final configuration
    pub fn build(self) -> Result<ProtoHttpParserConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
    
    /// Build the final configuration without validation
    pub fn build_unchecked(self) -> ProtoHttpParserConfig {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .preserve_comments(false)
            .strict_validation(true)
            .max_import_depth(5)
            .generate_service_traits(true)
            .use_dependency_injection(false)
            .infer_query_params(true)
            .use_rustfmt(false)
            .indent_size(2)
            .max_line_length(120)
            .build()
            .unwrap();
        
        assert!(!config.parser.preserve_comments);
        assert!(config.parser.strict_validation);
        assert_eq!(config.parser.max_import_depth, 5);
        assert!(config.generator.generate_service_traits);
        assert!(!config.generator.use_dependency_injection);
        assert!(config.extractor.infer_query_params);
        assert!(!config.generator.formatting.use_rustfmt);
        assert_eq!(config.generator.formatting.indent_size, 2);
        assert_eq!(config.generator.formatting.max_line_length, 120);
    }
    
    #[test]
    fn test_config_validation() {
        let config = ConfigBuilder::new()
            .max_import_depth(0)
            .build();
        
        assert!(config.is_err());
        
        let config = ConfigBuilder::new()
            .indent_size(0)
            .build();
        
        assert!(config.is_err());
    }
    
    #[test]
    fn test_config_file_operations() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let original_config = ConfigBuilder::new()
            .preserve_comments(false)
            .max_import_depth(15)
            .build()
            .unwrap();
        
        // Save configuration
        original_config.to_file(&config_path).unwrap();
        
        // Load configuration
        let loaded_config = ProtoHttpParserConfig::from_file(&config_path).unwrap();
        
        assert_eq!(loaded_config.parser.preserve_comments, false);
        assert_eq!(loaded_config.parser.max_import_depth, 15);
    }
    
    #[test]
    fn test_config_merge() {
        let mut base_config = ProtoHttpParserConfig::default();
        let override_config = ConfigBuilder::new()
            .preserve_comments(false)
            .max_import_depth(20)
            .build()
            .unwrap();
        
        base_config.merge(override_config);
        
        assert!(!base_config.parser.preserve_comments);
        assert_eq!(base_config.parser.max_import_depth, 20);
    }
    
    #[test]
    fn test_environment_variable_parsing() {
        // Set environment variables
        std::env::set_var("PROTO_HTTP_PARSER_PRESERVE_COMMENTS", "false");
        std::env::set_var("PROTO_HTTP_PARSER_MAX_IMPORT_DEPTH", "25");
        std::env::set_var("PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS", "false");
        
        let config = ProtoHttpParserConfig::from_env().unwrap();
        
        assert!(!config.parser.preserve_comments);
        assert_eq!(config.parser.max_import_depth, 25);
        assert!(!config.generator.generate_service_traits);
        
        // Clean up
        std::env::remove_var("PROTO_HTTP_PARSER_PRESERVE_COMMENTS");
        std::env::remove_var("PROTO_HTTP_PARSER_MAX_IMPORT_DEPTH");
        std::env::remove_var("PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS");
    }
}