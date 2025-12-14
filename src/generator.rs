//! Code generation implementation for poem-openapi

use crate::core::*;
use crate::templates::HandlebarsTemplateEngine;

/// Poem OpenAPI code generator
#[allow(dead_code)]
pub struct PoemOpenApiGenerator {
    config: GeneratorConfig,
    template_engine: HandlebarsTemplateEngine,
}

impl PoemOpenApiGenerator {
    /// Create a new generator with default configuration
    pub fn new() -> Self {
        Self {
            config: GeneratorConfig::default(),
            template_engine: HandlebarsTemplateEngine::new(),
        }
    }
    
    /// Create a new generator with custom configuration
    pub fn with_config(config: GeneratorConfig) -> Self {
        Self {
            config,
            template_engine: HandlebarsTemplateEngine::new(),
        }
    }
}

impl Default for PoemOpenApiGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for PoemOpenApiGenerator {
    type Error = CodeGenerationError;
    
    fn generate_controller(&self, service: &Service, routes: &[HttpRoute]) -> Result<GeneratedCode, Self::Error> {
        // Filter routes for this service
        let service_routes: Vec<&HttpRoute> = routes.iter()
            .filter(|route| route.service_name == service.name)
            .collect();
        
        // Create template context
        let context = TemplateContext {
            service: service.clone(),
            routes: service_routes.into_iter().cloned().collect(),
            custom_data: std::collections::HashMap::new(),
        };
        
        // Render the controller template
        let content = self.template_engine
            .render("controller", &context)
            .map_err(|e| CodeGenerationError::ContextError {
                message: format!("Failed to render controller template: {}", e),
            })?;
        
        // Generate required imports
        let mut imports = vec![
            "poem_openapi::{OpenApi, payload::Json, param::Path, param::Query}".to_string(),
            "std::sync::Arc".to_string(),
        ];
        
        // Add imports for custom types used in the service
        for route in &context.routes {
            // Add import for response type if it's not a scalar
            if !route.response_type.is_scalar() && !route.response_type.is_well_known_type() {
                let type_name = &route.response_type.name;
                if !type_name.starts_with("google.protobuf.") {
                    imports.push(format!("crate::{}", type_name));
                }
            }
            
            // Add imports for path and query parameter types
            for param in &route.path_parameters {
                if let ParameterType::Custom(type_name) = &param.param_type {
                    imports.push(format!("crate::{}", type_name));
                }
            }
            
            for param in &route.query_parameters {
                if let ParameterType::Custom(type_name) = &param.param_type {
                    imports.push(format!("crate::{}", type_name));
                }
            }
            
            // Add import for request body type if present
            if let Some(request_body) = &route.request_body {
                if request_body.is_entire_message {
                    // Find the input type from the service method
                    if let Some(method) = service.methods.iter().find(|m| m.name == route.method_name) {
                        if !method.input_type.is_scalar() && !method.input_type.is_well_known_type() {
                            let type_name = &method.input_type.name;
                            if !type_name.starts_with("google.protobuf.") {
                                imports.push(format!("crate::{}", type_name));
                            }
                        }
                    }
                } else if let Some(field_type) = &request_body.field {
                    // Custom field type import
                    imports.push(format!("crate::{}", field_type));
                }
            }
        }
        
        // Remove duplicates and sort
        imports.sort();
        imports.dedup();
        
        // Generate required dependencies
        let _dependencies = vec![
            "poem-openapi".to_string(),
            "poem".to_string(),
        ];
        
        Ok(GeneratedCode::new(content)
            .with_import("poem_openapi::{OpenApi, payload::Json, param::Path, param::Query}".to_string())
            .with_import("std::sync::Arc".to_string())
            .with_dependency("poem-openapi".to_string())
            .with_dependency("poem".to_string()))
    }
    
    fn generate_service_trait(&self, service: &Service, routes: &[HttpRoute]) -> Result<GeneratedCode, Self::Error> {
        // Filter routes for this service
        let service_routes: Vec<&HttpRoute> = routes.iter()
            .filter(|route| route.service_name == service.name)
            .collect();
        
        // Create template context
        let context = TemplateContext {
            service: service.clone(),
            routes: service_routes.into_iter().cloned().collect(),
            custom_data: std::collections::HashMap::new(),
        };
        
        // Render the service trait template
        let content = self.template_engine
            .render("service_trait", &context)
            .map_err(|e| CodeGenerationError::ContextError {
                message: format!("Failed to render service trait template: {}", e),
            })?;
        
        // Generate required imports
        let mut imports = vec![
            "async_trait::async_trait".to_string(),
        ];
        
        // Add imports for custom types used in the service
        for route in &context.routes {
            // Add import for input type if it's not a scalar
            if !route.response_type.is_scalar() && !route.response_type.is_well_known_type() {
                let type_name = &route.response_type.name;
                if !type_name.starts_with("google.protobuf.") {
                    imports.push(format!("crate::{}", type_name));
                }
            }
            
            // Add imports for path and query parameter types
            for param in &route.path_parameters {
                if let ParameterType::Custom(type_name) = &param.param_type {
                    imports.push(format!("crate::{}", type_name));
                }
            }
            
            for param in &route.query_parameters {
                if let ParameterType::Custom(type_name) = &param.param_type {
                    imports.push(format!("crate::{}", type_name));
                }
            }
        }
        
        // Remove duplicates and sort
        imports.sort();
        imports.dedup();
        
        // Generate required dependencies
        let _dependencies = vec![
            "async-trait".to_string(),
        ];
        
        Ok(GeneratedCode::new(content)
            .with_import("async_trait::async_trait".to_string())
            .with_dependency("async-trait".to_string()))
    }
}