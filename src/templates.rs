//! Template engine implementation using Handlebars

use crate::core::*;
use handlebars::{Handlebars, Helper, Context, RenderContext, Output, HelperResult, RenderError};
use serde_json::{Value as JsonValue, Map};
use std::collections::HashMap;

/// Handlebars-based template engine
pub struct HandlebarsTemplateEngine {
    handlebars: Handlebars<'static>,
    config: TemplateConfig,
}

impl HandlebarsTemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        
        // Configure handlebars
        handlebars.set_strict_mode(false);
        
        // Register built-in helpers
        Self::register_builtin_helpers(&mut handlebars);
        
        // Register built-in templates
        let mut engine = Self {
            handlebars,
            config: TemplateConfig::default(),
        };
        
        engine.register_builtin_templates().expect("Failed to register built-in templates");
        
        engine
    }
    
    /// Create a new template engine with custom configuration
    pub fn with_config(config: TemplateConfig) -> Self {
        let mut engine = Self::new();
        
        // Apply template overrides from config
        for (name, content) in &config.template_overrides {
            engine.register_template(name, content).expect("Failed to register template override");
        }
        
        engine.config = config;
        engine
    }
    
    /// Register built-in template helpers
    fn register_builtin_helpers(handlebars: &mut Handlebars<'static>) {
        // Snake case helper
        handlebars.register_helper("snake_case", Box::new(SnakeCaseHelper));
        
        // Camel case helper
        handlebars.register_helper("camel_case", Box::new(CamelCaseHelper));
        
        // Pascal case helper
        handlebars.register_helper("pascal_case", Box::new(PascalCaseHelper));
        
        // Type mapping helper
        handlebars.register_helper("map_type", Box::new(TypeMappingHelper));
        
        // Parameter type mapping helper
        handlebars.register_helper("map_param_type", Box::new(ParameterTypeMappingHelper));
        
        // Pluralize helper
        handlebars.register_helper("pluralize", Box::new(PluralizeHelper));
        
        // HTTP method helper
        handlebars.register_helper("http_method_lower", Box::new(HttpMethodLowerHelper));
        
        // Path parameter extraction helper
        handlebars.register_helper("extract_path_params", Box::new(PathParamHelper));
        
        // Import generation helper
        handlebars.register_helper("generate_imports", Box::new(ImportHelper));
    }
    
    /// Register built-in templates
    fn register_builtin_templates(&mut self) -> Result<(), TemplateError> {
        // Controller template
        self.register_template("controller", CONTROLLER_TEMPLATE)?;
        
        // Service trait template
        self.register_template("service_trait", SERVICE_TRAIT_TEMPLATE)?;
        
        // Import template
        self.register_template("imports", IMPORTS_TEMPLATE)?;
        
        // Method template
        self.register_template("method", METHOD_TEMPLATE)?;
        
        // Type definition template
        self.register_template("type_def", TYPE_DEF_TEMPLATE)?;
        
        Ok(())
    }
}

impl Default for HandlebarsTemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine for HandlebarsTemplateEngine {
    type Error = TemplateError;
    
    fn render(&self, template_name: &str, context: &TemplateContext) -> Result<String, Self::Error> {
        // Convert TemplateContext to JSON for handlebars
        let json_context = self.context_to_json(context)?;
        
        self.handlebars
            .render(template_name, &json_context)
            .map_err(|e| TemplateError::RenderingFailed {
                message: format!("Template '{}': {}", template_name, e),
            })
    }
    
    fn register_template(&mut self, name: &str, content: &str) -> Result<(), Self::Error> {
        self.handlebars
            .register_template_string(name, content)
            .map_err(|e| TemplateError::CompilationFailed {
                message: format!("Template '{}': {}", name, e),
            })
    }
    
    fn register_helper(&mut self, name: &str, helper: Box<dyn TemplateHelper>) -> Result<(), Self::Error> {
        let wrapper = TemplateHelperWrapper { helper };
        self.handlebars.register_helper(name, Box::new(wrapper));
        Ok(())
    }
}

impl HandlebarsTemplateEngine {
    /// Convert TemplateContext to JSON for handlebars rendering
    fn context_to_json(&self, context: &TemplateContext) -> Result<JsonValue, TemplateError> {
        let mut map = Map::new();
        
        // Add service data
        let service_json = serde_json::to_value(&context.service)
            .map_err(|e| TemplateError::InvalidData {
                message: format!("Failed to serialize service: {}", e),
            })?;
        map.insert("service".to_string(), service_json);
        
        // Add routes data
        let routes_json = serde_json::to_value(&context.routes)
            .map_err(|e| TemplateError::InvalidData {
                message: format!("Failed to serialize routes: {}", e),
            })?;
        map.insert("routes".to_string(), routes_json);
        
        // Add custom data
        for (key, value) in &context.custom_data {
            let json_value = template_value_to_json(value)?;
            map.insert(key.clone(), json_value);
        }
        
        Ok(JsonValue::Object(map))
    }
}

/// Convert TemplateValue to JSON Value
fn template_value_to_json(value: &TemplateValue) -> Result<JsonValue, TemplateError> {
    match value {
        TemplateValue::String(s) => Ok(JsonValue::String(s.clone())),
        TemplateValue::Number(n) => Ok(serde_json::Number::from_f64(*n)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null)),
        TemplateValue::Boolean(b) => Ok(JsonValue::Bool(*b)),
        TemplateValue::Array(arr) => {
            let json_arr: Result<Vec<JsonValue>, _> = arr.iter()
                .map(template_value_to_json)
                .collect();
            Ok(JsonValue::Array(json_arr?))
        }
        TemplateValue::Object(obj) => {
            let mut map = Map::new();
            for (k, v) in obj {
                map.insert(k.clone(), template_value_to_json(v)?);
            }
            Ok(JsonValue::Object(map))
        }
        TemplateValue::Null => Ok(JsonValue::Null),
    }
}

/// Wrapper to adapt TemplateHelper to Handlebars helper
struct TemplateHelperWrapper {
    helper: Box<dyn TemplateHelper>,
}

impl handlebars::HelperDef for TemplateHelperWrapper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        // Convert handlebars parameters to TemplateValue
        let args: Vec<TemplateValue> = h.params().iter()
            .map(|param| json_to_template_value(param.value()))
            .collect();
        
        match self.helper.call(&args) {
            Ok(result) => {
                let output = match result {
                    TemplateValue::String(s) => s,
                    TemplateValue::Number(n) => n.to_string(),
                    TemplateValue::Boolean(b) => b.to_string(),
                    _ => "".to_string(),
                };
                out.write(&output)?;
                Ok(())
            }
            Err(e) => Err(RenderError::new(format!("Helper error: {}", e))),
        }
    }
}

/// Convert JSON Value to TemplateValue
fn json_to_template_value(value: &JsonValue) -> TemplateValue {
    match value {
        JsonValue::String(s) => TemplateValue::String(s.clone()),
        JsonValue::Number(n) => TemplateValue::Number(n.as_f64().unwrap_or(0.0)),
        JsonValue::Bool(b) => TemplateValue::Boolean(*b),
        JsonValue::Array(arr) => {
            let template_arr = arr.iter().map(json_to_template_value).collect();
            TemplateValue::Array(template_arr)
        }
        JsonValue::Object(obj) => {
            let mut template_obj = HashMap::new();
            for (k, v) in obj {
                template_obj.insert(k.clone(), json_to_template_value(v));
            }
            TemplateValue::Object(template_obj)
        }
        JsonValue::Null => TemplateValue::Null,
    }
}

// Built-in helper implementations

/// Snake case conversion helper
struct SnakeCaseHelper;

impl handlebars::HelperDef for SnakeCaseHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("snake_case helper requires a string parameter"))?;
        
        let snake_case = to_snake_case(param);
        out.write(&snake_case)?;
        Ok(())
    }
}

/// Camel case conversion helper
struct CamelCaseHelper;

impl handlebars::HelperDef for CamelCaseHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("camel_case helper requires a string parameter"))?;
        
        let camel_case = to_camel_case(param);
        out.write(&camel_case)?;
        Ok(())
    }
}

/// Pascal case conversion helper
struct PascalCaseHelper;

impl handlebars::HelperDef for PascalCaseHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("pascal_case helper requires a string parameter"))?;
        
        let pascal_case = to_pascal_case(param);
        out.write(&pascal_case)?;
        Ok(())
    }
}

/// Type mapping helper
struct TypeMappingHelper;

impl handlebars::HelperDef for TypeMappingHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let type_name = h.param(0).and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("map_type helper requires a type name parameter"))?;
        
        let mapped_type = map_proto_type_to_rust(type_name);
        out.write(&mapped_type)?;
        Ok(())
    }
}

/// Parameter type mapping helper
struct ParameterTypeMappingHelper;

impl handlebars::HelperDef for ParameterTypeMappingHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        // Try to get the parameter type as a string first
        if let Some(param_type_str) = h.param(0).and_then(|v| v.value().as_str()) {
            let mapped_type = match param_type_str {
                "String" => "String".to_string(),
                "Integer" => "i32".to_string(),
                "Float" => "f64".to_string(),
                "Boolean" => "bool".to_string(),
                custom => custom.to_string(), // Custom types remain as-is
            };
            out.write(&mapped_type)?;
            return Ok(());
        }
        
        // Try to get it as an object (ParameterType enum)
        if let Some(param_type_obj) = h.param(0) {
            let value = param_type_obj.value();
            
            // Handle ParameterType enum variants
            if let Some(obj) = value.as_object() {
                if obj.contains_key("String") {
                    out.write("String")?;
                } else if obj.contains_key("Integer") {
                    out.write("i32")?;
                } else if obj.contains_key("Float") {
                    out.write("f64")?;
                } else if obj.contains_key("Boolean") {
                    out.write("bool")?;
                } else if let Some(custom) = obj.get("Custom").and_then(|v| v.as_str()) {
                    out.write(custom)?;
                } else {
                    out.write("String")?; // Default fallback
                }
                return Ok(());
            }
        }
        
        // Fallback to String if we can't determine the type
        out.write("String")?;
        Ok(())
    }
}

/// Pluralize helper
struct PluralizeHelper;

impl handlebars::HelperDef for PluralizeHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let word = h.param(0).and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("pluralize helper requires a string parameter"))?;
        
        let plural = pluralize(word);
        out.write(&plural)?;
        Ok(())
    }
}

/// HTTP method lowercase helper
struct HttpMethodLowerHelper;

impl handlebars::HelperDef for HttpMethodLowerHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let method = h.param(0).and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("http_method_lower helper requires a string parameter"))?;
        
        out.write(&method.to_lowercase())?;
        Ok(())
    }
}

/// Path parameter extraction helper
struct PathParamHelper;

impl handlebars::HelperDef for PathParamHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let path = h.param(0).and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("extract_path_params helper requires a path parameter"))?;
        
        let params = extract_path_parameters(path);
        let params_str = params.join(", ");
        out.write(&params_str)?;
        Ok(())
    }
}

/// Import generation helper
struct ImportHelper;

impl handlebars::HelperDef for ImportHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let imports = h.param(0).and_then(|v| v.value().as_array())
            .ok_or_else(|| RenderError::new("generate_imports helper requires an array parameter"))?;
        
        let mut import_lines = Vec::new();
        for import in imports {
            if let Some(import_str) = import.as_str() {
                import_lines.push(format!("use {};", import_str));
            }
        }
        
        out.write(&import_lines.join("\n"))?;
        Ok(())
    }
}

// Utility functions for string conversion

/// Convert string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;
    
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_was_upper {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_was_upper = true;
        } else {
            result.push(c);
            prev_was_upper = false;
        }
    }
    
    result
}

/// Convert string to camelCase
fn to_camel_case(s: &str) -> String {
    let words: Vec<&str> = s.split('_').collect();
    let mut result = String::new();
    
    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            result.push_str(&word.to_lowercase());
        } else {
            result.push_str(&capitalize_first(word));
        }
    }
    
    result
}

/// Convert string to PascalCase
fn to_pascal_case(s: &str) -> String {
    // If the string contains underscores, split on them
    if s.contains('_') {
        let words: Vec<&str> = s.split('_').collect();
        words.iter()
            .map(|word| capitalize_first(word))
            .collect::<String>()
    } else {
        // If no underscores, assume it's already camelCase or PascalCase
        // Just ensure the first letter is capitalized while preserving the rest
        capitalize_first_preserve_case(s)
    }
}

/// Capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

/// Capitalize first letter while preserving the rest of the string case
fn capitalize_first_preserve_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Map Protocol Buffer types to Rust types
fn map_proto_type_to_rust(proto_type: &str) -> String {
    match proto_type {
        "string" => "String".to_string(),
        "int32" | "sint32" | "sfixed32" => "i32".to_string(),
        "int64" | "sint64" | "sfixed64" => "i64".to_string(),
        "uint32" | "fixed32" => "u32".to_string(),
        "uint64" | "fixed64" => "u64".to_string(),
        "double" => "f64".to_string(),
        "float" => "f32".to_string(),
        "bool" => "bool".to_string(),
        "bytes" => "Vec<u8>".to_string(),
        "google.protobuf.Timestamp" => "chrono::DateTime<chrono::Utc>".to_string(),
        "google.protobuf.Duration" => "std::time::Duration".to_string(),
        "google.protobuf.Empty" => "()".to_string(),
        _ => proto_type.to_string(), // Custom types remain as-is
    }
}

/// Simple pluralization
fn pluralize(word: &str) -> String {
    if word.ends_with('s') || word.ends_with("sh") || word.ends_with("ch") {
        format!("{}es", word)
    } else if word.ends_with('y') {
        format!("{}ies", &word[..word.len()-1])
    } else {
        format!("{}s", word)
    }
}

/// Extract path parameters from a path template
fn extract_path_parameters(path: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut in_param = false;
    let mut current_param = String::new();
    
    for c in path.chars() {
        match c {
            '{' => {
                in_param = true;
                current_param.clear();
            }
            '}' => {
                if in_param {
                    // Normalize parameter name by replacing dots with underscores
                    let normalized_param = current_param.replace('.', "_");
                    params.push(normalized_param);
                    in_param = false;
                }
            }
            _ => {
                if in_param {
                    current_param.push(c);
                }
            }
        }
    }
    
    params
}

// Built-in templates

const CONTROLLER_TEMPLATE: &str = r#"
use poem_openapi::{OpenApi, payload::Json, param::Path, param::Query};
use std::sync::Arc;
// Import types from proto module using relative path from generated directory
{{#if message_types}}
use super::{{message_types}};
{{/if}}
use super::{{snake_case service.name}}_service::{{pascal_case service.name}}Service;

/// {{service.name}} controller generated from Protocol Buffer service
#[derive(Clone)]
pub struct {{pascal_case service.name}}Controller<T: {{pascal_case service.name}}Service> {
    service: Arc<T>,
}

impl<T: {{pascal_case service.name}}Service> {{pascal_case service.name}}Controller<T> {
    /// Create a new controller with the given service implementation
    pub fn new(service: T) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}

#[poem_openapi::OpenApi]
impl<T: {{pascal_case service.name}}Service + Send + Sync + 'static> {{pascal_case service.name}}Controller<T> {
{{#each routes}}
    /// {{method_name}} endpoint
    #[oai(path = "{{path_template}}", method = "{{http_method_lower http_method}}")]
    async fn {{snake_case method_name}}(
        &self,
        {{#each path_parameters}}
        {{snake_case name}}: Path<{{map_param_type param_type}}>,
        {{/each}}
        {{#each query_parameters}}
        {{#unless required}}#[oai(default)]{{/unless}} {{snake_case name}}: Query<{{#unless required}}Option<{{/unless}}{{map_param_type param_type}}{{#unless required}}>{{/unless}}>,
        {{/each}}
        {{#if request_body}}
        {{#if request_body.is_entire_message}}
        body: Json<{{map_type input_type.name}}>,
        {{else}}
        body: Json<String>,
        {{/if}}
        {{/if}}
    ) -> poem_openapi::payload::Json<{{map_type response_type.name}}> {
        let result = self.service.{{snake_case method_name}}(
            {{#each path_parameters}}
            {{snake_case name}}.0,
            {{/each}}
            {{#each query_parameters}}
            {{snake_case name}}.0,
            {{/each}}
            {{#if request_body}}
            body.0,
            {{/if}}
        ).await.unwrap();
        
        Json(result)
    }

{{/each}}
}
"#;

const SERVICE_TRAIT_TEMPLATE: &str = r#"
use async_trait::async_trait;
// Import types from proto module using relative path from generated directory
{{#if message_types}}
use super::{{message_types}};
{{/if}}

/// Service trait for {{service.name}}
/// 
/// Implement this trait to provide business logic for the {{service.name}} service.
/// The generated controller will delegate to your implementation.
#[async_trait]
pub trait {{pascal_case service.name}}Service {
{{#each routes}}
    /// {{method_name}} operation
    async fn {{snake_case method_name}}(
        &self,
        {{#each path_parameters}}
        {{snake_case name}}: {{map_param_type param_type}},
        {{/each}}
        {{#each query_parameters}}
        {{snake_case name}}: {{#unless required}}Option<{{/unless}}{{map_param_type param_type}}{{#unless required}}>{{/unless}},
        {{/each}}
        {{#if request_body}}
        {{#if request_body.is_entire_message}}
        request: {{map_type input_type.name}},
        {{else}}
        {{snake_case request_body.field}}: String,
        {{/if}}
        {{/if}}
    ) -> Result<{{map_type response_type.name}}, Box<dyn std::error::Error>>;

{{/each}}
}
"#;

const IMPORTS_TEMPLATE: &str = r#"
// Generated imports
{{#if imports}}
{{generate_imports imports}}
{{/if}}

// Additional imports
use poem_openapi::{OpenApi, payload::Json, param::Path, param::Query};
use async_trait::async_trait;
use std::sync::Arc;
"#;

const METHOD_TEMPLATE: &str = r#"
/// {{method_name}} endpoint
#[oai(path = "{{path_template}}", method = "{{http_method_lower http_method}}")]
async fn {{snake_case method_name}}(
    &self,
    {{#each path_parameters}}
    {{snake_case name}}: Path<{{map_type param_type}}>,
    {{/each}}
    {{#each query_parameters}}
    {{#unless required}}#[oai(default)]{{/unless}} {{snake_case name}}: Query<{{#unless required}}Option<{{/unless}}{{map_type param_type}}{{#unless required}}>{{/unless}}>,
    {{/each}}
    {{#if request_body}}
    {{#if request_body.is_entire_message}}
    body: Json<{{map_type ../input_type.name}}>,
    {{else}}
    body: Json<{{map_type request_body.field}}>,
    {{/if}}
    {{/if}}
) -> poem_openapi::payload::Json<{{map_type response_type.name}}> {
    let result = self.service.{{snake_case method_name}}(
        {{#each path_parameters}}
        {{snake_case name}}.0,
        {{/each}}
        {{#each query_parameters}}
        {{snake_case name}}.0,
        {{/each}}
        {{#if request_body}}
        body.0,
        {{/if}}
    ).await;
    
    Json(result)
}
"#;

const TYPE_DEF_TEMPLATE: &str = r#"
/// {{name}} type definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct {{pascal_case name}} {
    {{#each fields}}
    /// {{name}} field
    pub {{snake_case name}}: {{map_type field_type}},
    {{/each}}
}
"#;