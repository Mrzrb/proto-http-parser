//! Example plugins demonstrating the plugin system
//!
//! This module contains example implementations of various plugin types
//! to demonstrate how to extend the proto-http-parser-v2 library.

use super::*;
use crate::core::*;
use crate::errors::*;
use std::collections::HashSet;

/// Example custom validator plugin that enforces naming conventions
pub struct NamingConventionValidator {
    name: String,
    service_name_pattern: regex::Regex,
    method_name_pattern: regex::Regex,
    message_name_pattern: regex::Regex,
}

impl NamingConventionValidator {
    /// Create a new naming convention validator with default patterns
    pub fn new() -> Self {
        Self {
            name: "naming_convention_validator".to_string(),
            // Services should be PascalCase ending with "Service"
            service_name_pattern: regex::Regex::new(r"^[A-Z][a-zA-Z0-9]*Service$").unwrap(),
            // Methods should be PascalCase
            method_name_pattern: regex::Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap(),
            // Messages should be PascalCase
            message_name_pattern: regex::Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap(),
        }
    }
    
    /// Create a validator with custom patterns
    pub fn with_patterns(
        service_pattern: &str,
        method_pattern: &str,
        message_pattern: &str,
    ) -> Result<Self, PluginError> {
        Ok(Self {
            name: "naming_convention_validator".to_string(),
            service_name_pattern: regex::Regex::new(service_pattern)
                .map_err(|e| PluginError::ConfigurationError {
                    message: format!("Invalid service pattern: {}", e),
                })?,
            method_name_pattern: regex::Regex::new(method_pattern)
                .map_err(|e| PluginError::ConfigurationError {
                    message: format!("Invalid method pattern: {}", e),
                })?,
            message_name_pattern: regex::Regex::new(message_pattern)
                .map_err(|e| PluginError::ConfigurationError {
                    message: format!("Invalid message pattern: {}", e),
                })?,
        })
    }
}

impl Plugin for NamingConventionValidator {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Validates naming conventions for services, methods, and messages"
    }
    
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        // Update patterns from configuration if provided
        if let Some(service_pattern) = config.settings.get("service_pattern") {
            if let Some(pattern_str) = service_pattern.as_str() {
                self.service_name_pattern = regex::Regex::new(pattern_str)
                    .map_err(|e| PluginError::ConfigurationError {
                        message: format!("Invalid service pattern in config: {}", e),
                    })?;
            }
        }
        
        if let Some(method_pattern) = config.settings.get("method_pattern") {
            if let Some(pattern_str) = method_pattern.as_str() {
                self.method_name_pattern = regex::Regex::new(pattern_str)
                    .map_err(|e| PluginError::ConfigurationError {
                        message: format!("Invalid method pattern in config: {}", e),
                    })?;
            }
        }
        
        if let Some(message_pattern) = config.settings.get("message_pattern") {
            if let Some(pattern_str) = message_pattern.as_str() {
                self.message_name_pattern = regex::Regex::new(pattern_str)
                    .map_err(|e| PluginError::ConfigurationError {
                        message: format!("Invalid message pattern in config: {}", e),
                    })?;
            }
        }
        
        Ok(())
    }
    
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::ProtoValidator]
    }
}

impl ProtoValidatorPlugin for NamingConventionValidator {
    fn validate_proto_file(&self, proto_file: &ProtoFile) -> Result<Vec<ValidationError>, PluginError> {
        let mut errors = Vec::new();
        
        // Validate service names
        for service in &proto_file.services {
            if !self.service_name_pattern.is_match(&service.name) {
                errors.push(ValidationError::InvalidHttpAnnotation {
                    message: format!(
                        "Service name '{}' does not match naming convention. Expected pattern: {}",
                        service.name,
                        self.service_name_pattern.as_str()
                    ),
                    line: 0, // In a real implementation, we'd track line numbers
                });
            }
            
            // Validate method names
            for method in &service.methods {
                if !self.method_name_pattern.is_match(&method.name) {
                    errors.push(ValidationError::InvalidHttpAnnotation {
                        message: format!(
                            "Method name '{}' does not match naming convention. Expected pattern: {}",
                            method.name,
                            self.method_name_pattern.as_str()
                        ),
                        line: 0,
                    });
                }
            }
        }
        
        // Validate message names
        for message in &proto_file.messages {
            if !self.message_name_pattern.is_match(&message.name) {
                errors.push(ValidationError::InvalidHttpAnnotation {
                    message: format!(
                        "Message name '{}' does not match naming convention. Expected pattern: {}",
                        message.name,
                        self.message_name_pattern.as_str()
                    ),
                    line: 0,
                });
            }
        }
        
        Ok(errors)
    }
}

/// Example HTTP route validator that checks for common REST API patterns
pub struct RestApiValidator {
    name: String,
    require_resource_paths: bool,
    allow_nested_resources: bool,
    max_path_depth: usize,
}

impl RestApiValidator {
    /// Create a new REST API validator with default settings
    pub fn new() -> Self {
        Self {
            name: "rest_api_validator".to_string(),
            require_resource_paths: true,
            allow_nested_resources: true,
            max_path_depth: 4,
        }
    }
}

impl Plugin for RestApiValidator {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Validates HTTP routes against REST API best practices"
    }
    
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(require_resource) = config.settings.get("require_resource_paths") {
            self.require_resource_paths = require_resource.as_bool().unwrap_or(true);
        }
        
        if let Some(allow_nested) = config.settings.get("allow_nested_resources") {
            self.allow_nested_resources = allow_nested.as_bool().unwrap_or(true);
        }
        
        if let Some(max_depth) = config.settings.get("max_path_depth") {
            self.max_path_depth = max_depth.as_u64().unwrap_or(4) as usize;
        }
        
        Ok(())
    }
    
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::HttpValidator]
    }
}

impl HttpValidatorPlugin for RestApiValidator {
    fn validate_http_routes(&self, routes: &[HttpRoute]) -> Result<Vec<ValidationError>, PluginError> {
        let mut errors = Vec::new();
        let mut seen_paths = HashSet::new();
        
        for route in routes {
            // Check for duplicate paths with same HTTP method
            let path_key = format!("{} {}", route.http_method, route.path_template);
            if seen_paths.contains(&path_key) {
                errors.push(ValidationError::ConflictingRoutes {
                    route1: path_key.clone(),
                    route2: path_key.clone(),
                });
            }
            seen_paths.insert(path_key);
            
            // Validate path depth
            let path_segments: Vec<&str> = route.path_template
                .split('/')
                .filter(|s| !s.is_empty())
                .collect();
            
            if path_segments.len() > self.max_path_depth {
                errors.push(ValidationError::InvalidHttpAnnotation {
                    message: format!(
                        "Path '{}' exceeds maximum depth of {} segments",
                        route.path_template,
                        self.max_path_depth
                    ),
                    line: 0,
                });
            }
            
            // Check for resource-based paths if required
            if self.require_resource_paths {
                let has_resource = path_segments.iter().any(|segment| {
                    !segment.starts_with('{') && !segment.ends_with('}')
                });
                
                if !has_resource {
                    errors.push(ValidationError::InvalidHttpAnnotation {
                        message: format!(
                            "Path '{}' should contain at least one resource segment",
                            route.path_template
                        ),
                        line: 0,
                    });
                }
            }
            
            // Validate nested resources if not allowed
            if !self.allow_nested_resources {
                let resource_count = path_segments.iter()
                    .filter(|segment| !segment.starts_with('{') && !segment.ends_with('}'))
                    .count();
                
                if resource_count > 1 {
                    errors.push(ValidationError::InvalidHttpAnnotation {
                        message: format!(
                            "Path '{}' contains nested resources, which are not allowed",
                            route.path_template
                        ),
                        line: 0,
                    });
                }
            }
        }
        
        Ok(errors)
    }
}

/// Example code formatter plugin that applies custom formatting rules
pub struct CustomCodeFormatter {
    name: String,
    indent_size: usize,
    use_tabs: bool,
    max_line_length: usize,
}

impl CustomCodeFormatter {
    /// Create a new code formatter with default settings
    pub fn new() -> Self {
        Self {
            name: "custom_code_formatter".to_string(),
            indent_size: 4,
            use_tabs: false,
            max_line_length: 100,
        }
    }
}

impl Plugin for CustomCodeFormatter {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Custom code formatter with configurable indentation and line length"
    }
    
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(indent_size) = config.settings.get("indent_size") {
            self.indent_size = indent_size.as_u64().unwrap_or(4) as usize;
        }
        
        if let Some(use_tabs) = config.settings.get("use_tabs") {
            self.use_tabs = use_tabs.as_bool().unwrap_or(false);
        }
        
        if let Some(max_line_length) = config.settings.get("max_line_length") {
            self.max_line_length = max_line_length.as_u64().unwrap_or(100) as usize;
        }
        
        Ok(())
    }
    
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::CodeFormatter]
    }
}

impl CodeFormatterPlugin for CustomCodeFormatter {
    fn format_code(&self, code: &str, language: &str) -> Result<String, PluginError> {
        if language != "rust" {
            return Err(PluginError::ExecutionError {
                plugin: self.name().to_string(),
                message: format!("Unsupported language: {}", language),
            });
        }
        
        let mut formatted_lines = Vec::new();
        let mut current_indent = 0;
        
        for line in code.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                formatted_lines.push(String::new());
                continue;
            }
            
            // Adjust indentation based on braces
            if trimmed.ends_with('{') {
                formatted_lines.push(self.format_line(trimmed, current_indent));
                current_indent += 1;
            } else if trimmed.starts_with('}') {
                current_indent = current_indent.saturating_sub(1);
                formatted_lines.push(self.format_line(trimmed, current_indent));
            } else {
                formatted_lines.push(self.format_line(trimmed, current_indent));
            }
        }
        
        Ok(formatted_lines.join("\n"))
    }
    
    fn supported_languages(&self) -> Vec<String> {
        vec!["rust".to_string()]
    }
}

impl CustomCodeFormatter {
    fn format_line(&self, line: &str, indent_level: usize) -> String {
        let indent = if self.use_tabs {
            "\t".repeat(indent_level)
        } else {
            " ".repeat(indent_level * self.indent_size)
        };
        
        let formatted = format!("{}{}", indent, line);
        
        // Wrap long lines (simplified implementation)
        if formatted.len() > self.max_line_length {
            // In a real implementation, we'd do smarter line wrapping
            formatted
        } else {
            formatted
        }
    }
}

/// Example documentation generator plugin
pub struct DocumentationGenerator {
    name: String,
    include_examples: bool,
    output_format: String,
}

impl DocumentationGenerator {
    /// Create a new documentation generator
    pub fn new() -> Self {
        Self {
            name: "documentation_generator".to_string(),
            include_examples: true,
            output_format: "markdown".to_string(),
        }
    }
}

impl Plugin for DocumentationGenerator {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Generates API documentation from proto files"
    }
    
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(include_examples) = config.settings.get("include_examples") {
            self.include_examples = include_examples.as_bool().unwrap_or(true);
        }
        
        if let Some(format) = config.settings.get("output_format") {
            if let Some(format_str) = format.as_str() {
                self.output_format = format_str.to_string();
            }
        }
        
        Ok(())
    }
    
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::CodeGenerator]
    }
}

impl CodeGeneratorPlugin for DocumentationGenerator {
    fn generate_code(&self, service: &Service, routes: &[HttpRoute]) -> Result<GeneratedCode, PluginError> {
        let mut content = String::new();
        
        match self.output_format.as_str() {
            "markdown" => {
                content.push_str(&format!("# {} API\n\n", service.name));
                
                for route in routes {
                    if route.service_name == service.name {
                        content.push_str(&format!(
                            "## {} {}\n\n",
                            route.http_method,
                            route.path_template
                        ));
                        
                        content.push_str(&format!("Method: `{}`\n\n", route.method_name));
                        
                        if !route.path_parameters.is_empty() {
                            content.push_str("### Path Parameters\n\n");
                            for param in &route.path_parameters {
                                content.push_str(&format!("- `{}`: {:?}\n", param.name, param.param_type));
                            }
                            content.push('\n');
                        }
                        
                        if !route.query_parameters.is_empty() {
                            content.push_str("### Query Parameters\n\n");
                            for param in &route.query_parameters {
                                content.push_str(&format!("- `{}`: {:?}\n", param.name, param.param_type));
                            }
                            content.push('\n');
                        }
                        
                        if self.include_examples {
                            content.push_str("### Example\n\n");
                            content.push_str(&format!(
                                "```\n{} {}\n```\n\n",
                                route.http_method,
                                route.path_template
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(PluginError::ExecutionError {
                    plugin: self.name().to_string(),
                    message: format!("Unsupported output format: {}", self.output_format),
                });
            }
        }
        
        Ok(GeneratedCode {
            content,
            imports: vec![],
            dependencies: vec![],
        })
    }
    
    fn file_extension(&self) -> &str {
        match self.output_format.as_str() {
            "markdown" => "md",
            _ => "txt",
        }
    }
    
    fn filename_pattern(&self) -> &str {
        "{service_name}_api.{extension}"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_naming_convention_validator() {
        let validator = NamingConventionValidator::new();
        assert_eq!(validator.name(), "naming_convention_validator");
        assert_eq!(validator.version(), "1.0.0");
        assert!(validator.capabilities().contains(&PluginCapability::ProtoValidator));
    }
    
    #[test]
    fn test_rest_api_validator() {
        let validator = RestApiValidator::new();
        assert_eq!(validator.name(), "rest_api_validator");
        assert!(validator.capabilities().contains(&PluginCapability::HttpValidator));
    }
    
    #[test]
    fn test_custom_code_formatter() {
        let formatter = CustomCodeFormatter::new();
        assert_eq!(formatter.name(), "custom_code_formatter");
        assert!(formatter.capabilities().contains(&PluginCapability::CodeFormatter));
        assert!(formatter.supported_languages().contains(&"rust".to_string()));
    }
    
    #[test]
    fn test_documentation_generator() {
        let generator = DocumentationGenerator::new();
        assert_eq!(generator.name(), "documentation_generator");
        assert_eq!(generator.file_extension(), "md");
        assert!(generator.capabilities().contains(&PluginCapability::CodeGenerator));
    }
}