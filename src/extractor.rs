//! HTTP annotation extractor implementation

use crate::core::*;
use regex::Regex;
use std::collections::HashSet;

/// Google API HTTP annotation extractor
pub struct GoogleApiHttpExtractor {
    config: ExtractorConfig,
    path_param_regex: Regex,
}

impl GoogleApiHttpExtractor {
    /// Create a new extractor with default configuration
    pub fn new() -> Self {
        Self {
            config: ExtractorConfig::default(),
            path_param_regex: Regex::new(r"\{([^}]+)\}").expect("Invalid regex"),
        }
    }
    
    /// Create a new extractor with custom configuration
    pub fn with_config(config: ExtractorConfig) -> Self {
        Self { 
            config,
            path_param_regex: Regex::new(r"\{([^}]+)\}").expect("Invalid regex"),
        }
    }
    
    /// Extract path parameters from a path template
    fn extract_path_parameters(&self, path_template: &str, _input_message: &TypeReference) -> Result<Vec<PathParameter>, ValidationError> {
        let mut parameters = Vec::new();
        
        for cap in self.path_param_regex.captures_iter(path_template) {
            let raw_param_name = cap.get(1).unwrap().as_str();
            
            // Validate parameter name
            if raw_param_name.is_empty() {
                return Err(ValidationError::InvalidPathParameter {
                    param: raw_param_name.to_string(),
                    path: path_template.to_string(),
                });
            }
            
            // Convert nested field references to valid Rust identifiers
            // e.g., "book.id" -> "book_id", "user.profile.name" -> "user_profile_name"
            let param_name = self.normalize_parameter_name(raw_param_name);
            
            // Infer parameter type based on common patterns
            let param_type = self.infer_parameter_type(&param_name);
            
            parameters.push(PathParameter::new(param_name, param_type));
        }
        
        Ok(parameters)
    }
    
    /// Normalize parameter names to valid Rust identifiers
    /// Converts nested field references like "book.id" to "book_id"
    fn normalize_parameter_name(&self, param_name: &str) -> String {
        // Replace dots with underscores to create valid Rust identifiers
        param_name.replace('.', "_")
    }
    
    /// Infer parameter type from parameter name
    pub fn infer_parameter_type(&self, param_name: &str) -> ParameterType {
        match param_name.to_lowercase().as_str() {
            name if name.ends_with("_id") || name == "id" => ParameterType::String,
            name if name.contains("count") || name.contains("size") || name.contains("limit") => ParameterType::Integer,
            name if name.contains("rate") || name.contains("ratio") => ParameterType::Float,
            name if name.contains("enabled") || name.contains("active") => ParameterType::Boolean,
            _ => ParameterType::String, // Default to string
        }
    }
    
    /// Extract query parameters based on configuration
    fn extract_query_parameters(&self, _method: &RpcMethod) -> Vec<QueryParameter> {
        if !self.config.infer_query_params {
            return Vec::new();
        }
        
        let mut parameters = Vec::new();
        
        // Add common query parameters based on configuration
        for param_name in &self.config.common_query_params {
            let param_type = self.infer_parameter_type(param_name);
            parameters.push(QueryParameter::optional(param_name.clone(), param_type));
        }
        
        parameters
    }
    
    /// Parse HTTP method from option value
    fn parse_http_method(&self, method_str: &str) -> Result<HttpMethod, ValidationError> {
        match method_str.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "PATCH" => Ok(HttpMethod::Patch),
            "DELETE" => Ok(HttpMethod::Delete),
            custom if self.config.allow_custom_methods => Ok(HttpMethod::Custom(custom.to_string())),
            custom => Err(ValidationError::InvalidHttpAnnotation {
                message: format!("Unsupported HTTP method: {}", custom),
                line: 0, // TODO: Add proper line tracking
            }),
        }
    }
    
    /// Extract HTTP annotation from method options or direct annotation
    fn extract_http_annotation(&self, method: &RpcMethod) -> Result<Option<HttpAnnotation>, ValidationError> {
        // First check if there's a direct HTTP annotation (for testing)
        if let Some(ref annotation) = method.http_annotation {
            return Ok(Some(annotation.clone()));
        }
        
        // Look for google.api.http option in method options
        for option in &method.options {
            if option.name == "google.api.http" {
                return self.parse_http_option(&option.value);
            }
        }
        
        Ok(None)
    }
    
    /// Parse HTTP option value into HttpAnnotation
    fn parse_http_option(&self, option_value: &OptionValue) -> Result<Option<HttpAnnotation>, ValidationError> {
        match option_value {
            OptionValue::MessageLiteral(fields) => {
                let mut http_method = None;
                let mut path = None;
                let mut body = None;
                let mut additional_bindings = Vec::new();
                
                // Parse HTTP method and path
                for (key, value) in fields {
                    match key.as_str() {
                        "get" | "post" | "put" | "patch" | "delete" => {
                            if let OptionValue::String(path_str) = value {
                                http_method = Some(self.parse_http_method(key)?);
                                path = Some(path_str.clone());
                            }
                        }
                        "body" => {
                            if let OptionValue::String(body_str) = value {
                                body = Some(body_str.clone());
                            }
                        }
                        "additional_bindings" => {
                            // Parse additional bindings if present
                            additional_bindings = self.parse_additional_bindings(value)?;
                        }
                        _ => {
                            // Ignore unknown fields
                        }
                    }
                }
                
                if let (Some(method), Some(path_str)) = (http_method, path) {
                    Ok(Some(HttpAnnotation {
                        method,
                        path: path_str,
                        body,
                        additional_bindings,
                    }))
                } else {
                    Err(ValidationError::InvalidHttpAnnotation {
                        message: "Missing HTTP method or path".to_string(),
                        line: 0,
                    })
                }
            }
            _ => Err(ValidationError::InvalidHttpAnnotation {
                message: "Invalid HTTP annotation format".to_string(),
                line: 0,
            }),
        }
    }
    
    /// Parse additional HTTP bindings
    fn parse_additional_bindings(&self, _value: &OptionValue) -> Result<Vec<HttpBinding>, ValidationError> {
        // TODO: Implement additional bindings parsing
        Ok(Vec::new())
    }
    
    /// Determine request body configuration
    fn determine_request_body(&self, _method: &RpcMethod, http_annotation: &HttpAnnotation) -> Option<RequestBody> {
        match http_annotation.method {
            HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch => {
                match &http_annotation.body {
                    Some(body_field) if body_field == "*" => {
                        Some(RequestBody::entire_message())
                    }
                    Some(body_field) => {
                        Some(RequestBody::field(body_field.clone()))
                    }
                    None => {
                        // Default to entire message for POST/PUT/PATCH
                        Some(RequestBody::entire_message())
                    }
                }
            }
            _ => None, // GET/DELETE typically don't have request bodies
        }
    }
    
    /// Validate path template syntax
    pub fn validate_path_template(&self, path_template: &str) -> Result<(), ValidationError> {
        // Check for basic path template validity
        if path_template.is_empty() {
            return Err(ValidationError::InvalidHttpAnnotation {
                message: "Empty path template".to_string(),
                line: 0,
            });
        }
        
        if !path_template.starts_with('/') {
            return Err(ValidationError::InvalidHttpAnnotation {
                message: "Path template must start with '/'".to_string(),
                line: 0,
            });
        }
        
        // Validate parameter syntax - check for nested braces and proper matching
        let mut brace_count = 0;
        let mut in_param = false;
        
        for ch in path_template.chars() {
            match ch {
                '{' => {
                    if in_param {
                        // Nested braces are not allowed
                        return Err(ValidationError::InvalidHttpAnnotation {
                            message: "Nested braces are not allowed in path template".to_string(),
                            line: 0,
                        });
                    }
                    brace_count += 1;
                    in_param = true;
                }
                '}' => {
                    brace_count -= 1;
                    in_param = false;
                    if brace_count < 0 {
                        return Err(ValidationError::InvalidHttpAnnotation {
                            message: "Unmatched closing brace in path template".to_string(),
                            line: 0,
                        });
                    }
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
    
    /// Check for conflicting routes
    fn check_route_conflicts(&self, routes: &[HttpRoute]) -> Result<(), ValidationError> {
        let mut route_signatures = HashSet::new();
        
        for route in routes {
            let signature = format!("{} {}", route.http_method.as_str(), route.path_template);
            
            if route_signatures.contains(&signature) {
                return Err(ValidationError::ConflictingRoutes {
                    route1: signature.clone(),
                    route2: signature,
                });
            }
            
            route_signatures.insert(signature);
        }
        
        Ok(())
    }
}

impl Default for GoogleApiHttpExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpAnnotationExtractor for GoogleApiHttpExtractor {
    type Error = ValidationError;
    
    fn extract_routes(&self, proto_file: &ProtoFile) -> Result<Vec<HttpRoute>, Self::Error> {
        let mut routes = Vec::new();
        
        for service in &proto_file.services {
            for method in &service.methods {
                if let Some(http_annotation) = self.extract_http_annotation(method)? {
                    // Validate path template
                    self.validate_path_template(&http_annotation.path)?;
                    
                    // Extract path parameters
                    let path_parameters = self.extract_path_parameters(&http_annotation.path, &method.input_type)?;
                    
                    // Extract query parameters
                    let query_parameters = self.extract_query_parameters(method);
                    
                    // Determine request body
                    let request_body = self.determine_request_body(method, &http_annotation);
                    
                    // Create HTTP route
                    let route = HttpRoute {
                        service_name: service.name.clone(),
                        method_name: method.name.clone(),
                        http_method: http_annotation.method.clone(),
                        path_template: http_annotation.path.clone(),
                        path_parameters,
                        query_parameters,
                        request_body,
                        response_type: method.output_type.clone(),
                    };
                    
                    routes.push(route);
                    
                    // Process additional bindings
                    for binding in &http_annotation.additional_bindings {
                        self.validate_path_template(&binding.path)?;
                        
                        let path_parameters = self.extract_path_parameters(&binding.path, &method.input_type)?;
                        let query_parameters = self.extract_query_parameters(method);
                        let request_body = if binding.body.is_some() {
                            match &binding.body {
                                Some(body_field) if body_field == "*" => Some(RequestBody::entire_message()),
                                Some(body_field) => Some(RequestBody::field(body_field.clone())),
                                None => None,
                            }
                        } else {
                            None
                        };
                        
                        let additional_route = HttpRoute {
                            service_name: service.name.clone(),
                            method_name: method.name.clone(),
                            http_method: binding.method.clone(),
                            path_template: binding.path.clone(),
                            path_parameters,
                            query_parameters,
                            request_body,
                            response_type: method.output_type.clone(),
                        };
                        
                        routes.push(additional_route);
                    }
                }
            }
        }
        
        Ok(routes)
    }
    
    fn validate_annotations(&self, routes: &[HttpRoute]) -> Result<(), Self::Error> {
        // Check for conflicting routes
        self.check_route_conflicts(routes)?;
        
        // Validate each route
        for route in routes {
            // Validate HTTP method compatibility
            if self.config.validate_http_methods {
                match route.http_method {
                    HttpMethod::Get | HttpMethod::Delete => {
                        if route.has_request_body() {
                            return Err(ValidationError::InvalidHttpAnnotation {
                                message: format!("{} methods should not have request bodies", route.http_method.as_str()),
                                line: 0,
                            });
                        }
                    }
                    HttpMethod::Custom(_) if !self.config.allow_custom_methods => {
                        return Err(ValidationError::InvalidHttpAnnotation {
                            message: "Custom HTTP methods are not allowed".to_string(),
                            line: 0,
                        });
                    }
                    _ => {}
                }
            }
            
            // Validate path template
            self.validate_path_template(&route.path_template)?;
        }
        
        Ok(())
    }
}