//! Plugin system for extending proto-http-parser-v2
//!
//! This module provides a flexible plugin system that allows users to extend
//! the functionality of the proto-http-parser-v2 library with custom validators,
//! code generators, template engines, and other extensions.

use crate::core::*;
use crate::errors::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Example plugin implementations
pub mod examples;

/// Plugin configuration management
pub mod config;

/// Plugin interface for extending the proto-http-parser-v2 library
pub trait Plugin: Send + Sync {
    /// Get the plugin name
    fn name(&self) -> &str;
    
    /// Get the plugin version
    fn version(&self) -> &str;
    
    /// Get the plugin description
    fn description(&self) -> &str;
    
    /// Initialize the plugin with configuration
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), PluginError>;
    
    /// Get the plugin capabilities
    fn capabilities(&self) -> Vec<PluginCapability>;
    
    /// Check if the plugin is compatible with the given library version
    fn is_compatible(&self, library_version: &str) -> bool {
        // Default implementation: assume compatibility
        let _ = library_version;
        true
    }
}

/// Plugin capabilities that define what extension points a plugin supports
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PluginCapability {
    /// Custom validation of proto files
    ProtoValidator,
    /// Custom validation of HTTP annotations
    HttpValidator,
    /// Custom code generation
    CodeGenerator,
    /// Custom template engine
    TemplateEngine,
    /// Custom code formatting
    CodeFormatter,
    /// Custom error reporting
    ErrorReporter,
    /// Custom configuration processing
    ConfigProcessor,
    /// Custom file processing
    FileProcessor,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin-specific configuration as key-value pairs
    pub settings: HashMap<String, serde_json::Value>,
    /// Whether the plugin is enabled
    pub enabled: bool,
    /// Plugin priority (higher numbers = higher priority)
    pub priority: i32,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            settings: HashMap::new(),
            enabled: true,
            priority: 0,
        }
    }
}



/// Extension point for proto file validation
pub trait ProtoValidatorPlugin: Plugin {
    /// Validate a proto file and return any validation errors
    fn validate_proto_file(&self, proto_file: &ProtoFile) -> Result<Vec<ValidationError>, PluginError>;
}

/// Extension point for HTTP annotation validation
pub trait HttpValidatorPlugin: Plugin {
    /// Validate HTTP routes and return any validation errors
    fn validate_http_routes(&self, routes: &[HttpRoute]) -> Result<Vec<ValidationError>, PluginError>;
}

/// Extension point for custom code generation
pub trait CodeGeneratorPlugin: Plugin {
    /// Generate custom code for a service
    fn generate_code(&self, service: &Service, routes: &[HttpRoute]) -> Result<GeneratedCode, PluginError>;
    
    /// Get the file extension for generated files
    fn file_extension(&self) -> &str;
    
    /// Get the output filename pattern
    fn filename_pattern(&self) -> &str {
        "{service_name}.{extension}"
    }
}

/// Extension point for custom template engines
pub trait TemplateEnginePlugin: Plugin {
    /// Render a template with the given context
    fn render_template(&self, template_name: &str, context: &TemplateContext) -> Result<String, PluginError>;
    
    /// Register a custom template
    fn register_template(&mut self, name: &str, content: &str) -> Result<(), PluginError>;
    
    /// Get available templates
    fn available_templates(&self) -> Vec<String>;
}

/// Extension point for custom code formatting
pub trait CodeFormatterPlugin: Plugin {
    /// Format generated code
    fn format_code(&self, code: &str, language: &str) -> Result<String, PluginError>;
    
    /// Get supported languages
    fn supported_languages(&self) -> Vec<String>;
}

/// Extension point for custom error reporting
pub trait ErrorReporterPlugin: Plugin {
    /// Format an error for display
    fn format_error(&self, error: &ProtoHttpParserError) -> Result<String, PluginError>;
    
    /// Generate error suggestions
    fn suggest_fix(&self, error: &ProtoHttpParserError) -> Result<Option<String>, PluginError>;
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    /// Registered plugins
    plugins: HashMap<String, Arc<dyn Plugin>>,
    /// Plugin configurations
    configs: HashMap<String, PluginConfig>,
    /// Extension point mappings
    validators: Vec<Arc<dyn ProtoValidatorPlugin>>,
    http_validators: Vec<Arc<dyn HttpValidatorPlugin>>,
    code_generators: Vec<Arc<dyn CodeGeneratorPlugin>>,
    template_engines: Vec<Arc<dyn TemplateEnginePlugin>>,
    code_formatters: Vec<Arc<dyn CodeFormatterPlugin>>,
    error_reporters: Vec<Arc<dyn ErrorReporterPlugin>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            configs: HashMap::new(),
            validators: Vec::new(),
            http_validators: Vec::new(),
            code_generators: Vec::new(),
            template_engines: Vec::new(),
            code_formatters: Vec::new(),
            error_reporters: Vec::new(),
        }
    }
    
    /// Register a proto validator plugin
    pub fn register_proto_validator<P>(&mut self, mut plugin: P, config: PluginConfig) -> Result<(), PluginError>
    where
        P: ProtoValidatorPlugin + 'static,
    {
        let name = plugin.name().to_string();
        
        // Check compatibility
        if !plugin.is_compatible(crate::VERSION) {
            return Err(PluginError::Incompatible {
                name: name.clone(),
                required: "compatible version".to_string(),
                found: crate::VERSION.to_string(),
            });
        }
        
        // Initialize the plugin
        plugin.initialize(&config)?;
        
        let plugin_arc = Arc::new(plugin);
        
        // Store in validators list
        self.validators.push(plugin_arc.clone());
        
        // Store plugin and config
        self.plugins.insert(name.clone(), plugin_arc);
        self.configs.insert(name, config);
        
        Ok(())
    }
    
    /// Register an HTTP validator plugin
    pub fn register_http_validator<P>(&mut self, mut plugin: P, config: PluginConfig) -> Result<(), PluginError>
    where
        P: HttpValidatorPlugin + 'static,
    {
        let name = plugin.name().to_string();
        
        if !plugin.is_compatible(crate::VERSION) {
            return Err(PluginError::Incompatible {
                name: name.clone(),
                required: "compatible version".to_string(),
                found: crate::VERSION.to_string(),
            });
        }
        
        plugin.initialize(&config)?;
        let plugin_arc = Arc::new(plugin);
        
        self.http_validators.push(plugin_arc.clone());
        self.plugins.insert(name.clone(), plugin_arc);
        self.configs.insert(name, config);
        
        Ok(())
    }
    
    /// Register a code generator plugin
    pub fn register_code_generator<P>(&mut self, mut plugin: P, config: PluginConfig) -> Result<(), PluginError>
    where
        P: CodeGeneratorPlugin + 'static,
    {
        let name = plugin.name().to_string();
        
        if !plugin.is_compatible(crate::VERSION) {
            return Err(PluginError::Incompatible {
                name: name.clone(),
                required: "compatible version".to_string(),
                found: crate::VERSION.to_string(),
            });
        }
        
        plugin.initialize(&config)?;
        let plugin_arc = Arc::new(plugin);
        
        self.code_generators.push(plugin_arc.clone());
        self.plugins.insert(name.clone(), plugin_arc);
        self.configs.insert(name, config);
        
        Ok(())
    }
    
    /// Register a code formatter plugin
    pub fn register_code_formatter<P>(&mut self, mut plugin: P, config: PluginConfig) -> Result<(), PluginError>
    where
        P: CodeFormatterPlugin + 'static,
    {
        let name = plugin.name().to_string();
        
        if !plugin.is_compatible(crate::VERSION) {
            return Err(PluginError::Incompatible {
                name: name.clone(),
                required: "compatible version".to_string(),
                found: crate::VERSION.to_string(),
            });
        }
        
        plugin.initialize(&config)?;
        let plugin_arc = Arc::new(plugin);
        
        self.code_formatters.push(plugin_arc.clone());
        self.plugins.insert(name.clone(), plugin_arc);
        self.configs.insert(name, config);
        
        Ok(())
    }
    
    /// Register an error reporter plugin
    pub fn register_error_reporter<P>(&mut self, mut plugin: P, config: PluginConfig) -> Result<(), PluginError>
    where
        P: ErrorReporterPlugin + 'static,
    {
        let name = plugin.name().to_string();
        
        if !plugin.is_compatible(crate::VERSION) {
            return Err(PluginError::Incompatible {
                name: name.clone(),
                required: "compatible version".to_string(),
                found: crate::VERSION.to_string(),
            });
        }
        
        plugin.initialize(&config)?;
        let plugin_arc = Arc::new(plugin);
        
        self.error_reporters.push(plugin_arc.clone());
        self.plugins.insert(name.clone(), plugin_arc);
        self.configs.insert(name, config);
        
        Ok(())
    }
    
    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(name).cloned()
    }
    
    /// Get all registered plugins
    pub fn plugins(&self) -> &HashMap<String, Arc<dyn Plugin>> {
        &self.plugins
    }
    
    /// Run proto validation plugins
    pub fn validate_proto_file(&self, proto_file: &ProtoFile) -> Result<Vec<ValidationError>, PluginError> {
        let mut all_errors = Vec::new();
        
        for validator in &self.validators {
            match validator.validate_proto_file(proto_file) {
                Ok(mut errors) => all_errors.append(&mut errors),
                Err(e) => return Err(e),
            }
        }
        
        Ok(all_errors)
    }
    
    /// Run HTTP validation plugins
    pub fn validate_http_routes(&self, routes: &[HttpRoute]) -> Result<Vec<ValidationError>, PluginError> {
        let mut all_errors = Vec::new();
        
        for validator in &self.http_validators {
            match validator.validate_http_routes(routes) {
                Ok(mut errors) => all_errors.append(&mut errors),
                Err(e) => return Err(e),
            }
        }
        
        Ok(all_errors)
    }
    
    /// Run code generation plugins
    pub fn generate_code(&self, service: &Service, routes: &[HttpRoute]) -> Result<Vec<(String, GeneratedCode)>, PluginError> {
        let mut generated_files = Vec::new();
        
        for generator in &self.code_generators {
            match generator.generate_code(service, routes) {
                Ok(code) => {
                    let filename = generator.filename_pattern()
                        .replace("{service_name}", &to_snake_case(&service.name))
                        .replace("{extension}", generator.file_extension());
                    generated_files.push((filename, code));
                }
                Err(e) => return Err(e),
            }
        }
        
        Ok(generated_files)
    }
    
    /// Format code using formatter plugins
    pub fn format_code(&self, code: &str, language: &str) -> Result<String, PluginError> {
        for formatter in &self.code_formatters {
            if formatter.supported_languages().contains(&language.to_string()) {
                return formatter.format_code(code, language);
            }
        }
        
        // No formatter found, return original code
        Ok(code.to_string())
    }
    
    /// Format error using error reporter plugins
    pub fn format_error(&self, error: &ProtoHttpParserError) -> Result<String, PluginError> {
        for reporter in &self.error_reporters {
            match reporter.format_error(error) {
                Ok(formatted) => return Ok(formatted),
                Err(_) => continue, // Try next reporter
            }
        }
        
        // No custom formatter, use default
        Ok(error.to_string())
    }
    
    /// Get error suggestions from plugins
    pub fn suggest_fix(&self, error: &ProtoHttpParserError) -> Result<Option<String>, PluginError> {
        for reporter in &self.error_reporters {
            match reporter.suggest_fix(error) {
                Ok(Some(suggestion)) => return Ok(Some(suggestion)),
                Ok(None) => continue,
                Err(_) => continue, // Try next reporter
            }
        }
        
        Ok(None)
    }
    
    /// Load plugins from a configuration file
    pub fn load_from_config<P: AsRef<Path>>(&mut self, config_path: P) -> Result<(), PluginError> {
        let config_content = std::fs::read_to_string(config_path.as_ref())
            .map_err(|e| PluginError::LoadingError {
                message: format!("Failed to read plugin config: {}", e),
            })?;
        
        let plugin_configs: HashMap<String, PluginConfig> = serde_json::from_str(&config_content)
            .map_err(|e| PluginError::ConfigurationError {
                message: format!("Failed to parse plugin config: {}", e),
            })?;
        
        // Store configurations for later use when plugins are registered
        for (name, config) in plugin_configs {
            self.configs.insert(name, config);
        }
        
        Ok(())
    }
    
    /// Get plugin configuration
    pub fn get_config(&self, plugin_name: &str) -> Option<&PluginConfig> {
        self.configs.get(plugin_name)
    }
    
    /// Update plugin configuration
    pub fn update_config(&mut self, plugin_name: &str, config: PluginConfig) -> Result<(), PluginError> {
        if let Some(_plugin) = self.plugins.get_mut(plugin_name) {
            // Note: This is a simplified approach. In a real implementation,
            // we might need to reinitialize the plugin with the new config.
            self.configs.insert(plugin_name.to_string(), config);
            Ok(())
        } else {
            Err(PluginError::NotFound {
                name: plugin_name.to_string(),
            })
        }
    }
    
    /// List all registered plugins with their status
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.iter().map(|(name, plugin)| {
            let config = self.configs.get(name);
            PluginInfo {
                name: name.clone(),
                version: plugin.version().to_string(),
                description: plugin.description().to_string(),
                capabilities: plugin.capabilities(),
                enabled: config.map(|c| c.enabled).unwrap_or(true),
                priority: config.map(|c| c.priority).unwrap_or(0),
            }
        }).collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a registered plugin
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin capabilities
    pub capabilities: Vec<PluginCapability>,
    /// Whether the plugin is enabled
    pub enabled: bool,
    /// Plugin priority
    pub priority: i32,
}



/// Utility function to convert string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock plugin for testing
    struct MockValidatorPlugin {
        name: String,
    }
    
    impl Plugin for MockValidatorPlugin {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn version(&self) -> &str {
            "1.0.0"
        }
        
        fn description(&self) -> &str {
            "Mock validator plugin for testing"
        }
        
        fn initialize(&mut self, _config: &PluginConfig) -> Result<(), PluginError> {
            Ok(())
        }
        
        fn capabilities(&self) -> Vec<PluginCapability> {
            vec![PluginCapability::ProtoValidator]
        }
    }
    
    impl ProtoValidatorPlugin for MockValidatorPlugin {
        fn validate_proto_file(&self, _proto_file: &ProtoFile) -> Result<Vec<ValidationError>, PluginError> {
            Ok(vec![])
        }
    }
    
    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.plugins().len(), 0);
    }
    
    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert!(config.enabled);
        assert_eq!(config.priority, 0);
        assert!(config.settings.is_empty());
    }
    
    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("UserService"), "user_service");
        assert_eq!(to_snake_case("HTTPClient"), "h_t_t_p_client");
        assert_eq!(to_snake_case("simple"), "simple");
    }
}