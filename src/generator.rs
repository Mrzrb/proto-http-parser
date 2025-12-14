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
        
        // Collect all message types used in this service
        let service_routes_slice: Vec<HttpRoute> = service_routes.iter().map(|&r| r.clone()).collect();
        let message_types = self.collect_message_types(service, &service_routes_slice);
        
        // Debug output
        println!("DEBUG: Controller - Collected message types for {}: {:?}", service.name, message_types);
        
        // Check if we need Path or Query imports
        let has_path_params = service_routes.iter().any(|route| !route.path_parameters.is_empty());
        let has_query_params = service_routes.iter().any(|route| !route.query_parameters.is_empty());
        
        // Create template context
        let mut custom_data = std::collections::HashMap::new();
        if !message_types.is_empty() {
            // Create a single string with the import list including braces
            let import_list = format!("{{{}}}", message_types.join(", "));
            custom_data.insert("message_types".to_string(), TemplateValue::String(import_list));
        }
        custom_data.insert("has_path_params".to_string(), TemplateValue::Boolean(has_path_params));
        custom_data.insert("has_query_params".to_string(), TemplateValue::Boolean(has_query_params));
        
        // Add input_type for each route by matching with service methods
        let mut enriched_routes = Vec::new();
        for route in &service_routes {
            let mut route_clone = (*route).clone();
            
            // Find the corresponding service method to get input type
            if let Some(method) = service.methods.iter().find(|m| m.name == route.method_name) {
                // Add input_type to the route context (we'll need to modify the route structure or use custom_data)
                // For now, we'll add it to custom_data with a route-specific key
                let input_type_key = format!("{}_input_type", route.method_name);
                custom_data.insert(input_type_key, TemplateValue::String(method.input_type.name.clone()));
            }
            
            enriched_routes.push(route_clone);
        }
        
        let context = TemplateContext {
            service: service.clone(),
            routes: enriched_routes,
            custom_data,
        };
        
        // Render the controller template
        let content = self.template_engine
            .render("controller", &context)
            .map_err(|e| CodeGenerationError::ContextError {
                message: format!("Failed to render controller template: {}", e),
            })?;
        
        // Generate required imports
        let mut imports = vec![
            "std::sync::Arc".to_string(),
        ];
        
        // Add conditional imports
        let mut openapi_imports = vec!["OpenApi".to_string(), "payload::Json".to_string()];
        if has_path_params {
            openapi_imports.push("param::Path".to_string());
        }
        if has_query_params {
            openapi_imports.push("param::Query".to_string());
        }
        imports.push(format!("poem_openapi::{{{}}}", openapi_imports.join(", ")));
        
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
            .with_import("std::sync::Arc".to_string())
            .with_dependency("poem-openapi".to_string())
            .with_dependency("poem".to_string()))
    }
    
    fn generate_service_trait(&self, service: &Service, routes: &[HttpRoute]) -> Result<GeneratedCode, Self::Error> {
        // Filter routes for this service
        let service_routes: Vec<&HttpRoute> = routes.iter()
            .filter(|route| route.service_name == service.name)
            .collect();
        
        // Collect all message types used in this service
        let service_routes_slice: Vec<HttpRoute> = service_routes.iter().map(|&r| r.clone()).collect();
        let message_types = self.collect_message_types(service, &service_routes_slice);
        
        // Create template context
        let mut custom_data = std::collections::HashMap::new();
        if !message_types.is_empty() {
            // Create a single string with the import list including braces
            let import_list = format!("{{{}}}", message_types.join(", "));
            custom_data.insert("message_types".to_string(), TemplateValue::String(import_list));
        }
        
        let context = TemplateContext {
            service: service.clone(),
            routes: service_routes.into_iter().cloned().collect(),
            custom_data,
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

impl PoemOpenApiGenerator {
    /// Collect all message types used in a service
    fn collect_message_types(&self, service: &Service, routes: &[HttpRoute]) -> Vec<String> {
        let mut message_types = std::collections::HashSet::new();
        
        // Collect types from routes (more accurate than service methods)
        for route in routes {
            // Add input type (for request body)
            if let Some(request_body) = &route.request_body {
                if request_body.is_entire_message {
                    if !route.input_type.is_scalar() && !route.input_type.is_well_known_type() {
                        let type_name = &route.input_type.name;
                        if !type_name.starts_with("google.protobuf.") {
                            message_types.insert(type_name.clone());
                        }
                    }
                }
            }
            
            // Add response type
            if !route.response_type.is_scalar() && !route.response_type.is_well_known_type() {
                let type_name = &route.response_type.name;
                if !type_name.starts_with("google.protobuf.") {
                    message_types.insert(type_name.clone());
                }
            }
            
            // Add parameter types (for custom types used in path/query parameters)
            for param in &route.path_parameters {
                if let ParameterType::Custom(type_name) = &param.param_type {
                    message_types.insert(type_name.clone());
                }
            }
            
            for param in &route.query_parameters {
                if let ParameterType::Custom(type_name) = &param.param_type {
                    message_types.insert(type_name.clone());
                }
            }
        }
        
        // Convert to sorted vector
        let mut result: Vec<String> = message_types.into_iter().collect();
        result.sort();
        result
    }
}