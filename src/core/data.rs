//! Core data structures for Protocol Buffer parsing and HTTP route representation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Protocol Buffer version
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProtocolVersion {
    Proto2,
    Proto3,
}

/// Represents a complete Protocol Buffer file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtoFile {
    /// Protocol Buffer syntax version
    pub syntax: ProtocolVersion,
    /// Package name
    pub package: std::option::Option<String>,
    /// Import statements
    pub imports: Vec<Import>,
    /// File-level options
    pub options: Vec<ProtoOption>,
    /// Service definitions
    pub services: Vec<Service>,
    /// Message type definitions
    pub messages: Vec<Message>,
    /// Enum definitions
    pub enums: Vec<Enum>,
}

/// Import statement in a proto file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Import {
    /// Import type (public, weak, or normal)
    pub import_type: ImportType,
    /// Path to the imported file
    pub path: String,
}

/// Type of import
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportType {
    Normal,
    Public,
    Weak,
}

/// Protocol Buffer option
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtoOption {
    /// Option name
    pub name: String,
    /// Option value
    pub value: OptionValue,
}

/// Option value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptionValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Identifier(String),
    MessageLiteral(HashMap<String, OptionValue>),
}

/// Service definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Service {
    /// Service name
    pub name: String,
    /// RPC methods in the service
    pub methods: Vec<RpcMethod>,
    /// Service-level options
    pub options: Vec<ProtoOption>,
    /// Documentation comments
    pub comments: Vec<Comment>,
}

/// RPC method definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcMethod {
    /// Method name
    pub name: String,
    /// Input message type
    pub input_type: TypeReference,
    /// Output message type
    pub output_type: TypeReference,
    /// Method options
    pub options: Vec<ProtoOption>,
    /// Documentation comments
    pub comments: Vec<Comment>,
    /// HTTP annotation if present
    pub http_annotation: std::option::Option<HttpAnnotation>,
}

/// Type reference (can be simple or fully qualified)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeReference {
    /// Type name
    pub name: String,
    /// Package path if fully qualified
    pub package: std::option::Option<String>,
    /// Whether this is a streaming type
    pub is_stream: bool,
}

/// HTTP annotation from google.api.http
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpAnnotation {
    /// HTTP method and path
    pub method: HttpMethod,
    /// Path template
    pub path: String,
    /// Request body field specification
    pub body: std::option::Option<String>,
    /// Additional HTTP bindings
    pub additional_bindings: Vec<HttpBinding>,
}

/// HTTP method types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Custom(String),
}

/// Additional HTTP binding for multiple routes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpBinding {
    /// HTTP method
    pub method: HttpMethod,
    /// Path template
    pub path: String,
    /// Request body field
    pub body: std::option::Option<String>,
}

/// Structured HTTP route information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpRoute {
    /// Service name
    pub service_name: String,
    /// Method name
    pub method_name: String,
    /// HTTP method
    pub http_method: HttpMethod,
    /// Path template with parameters
    pub path_template: String,
    /// Extracted path parameters
    pub path_parameters: Vec<PathParameter>,
    /// Inferred query parameters
    pub query_parameters: Vec<QueryParameter>,
    /// Request body configuration
    pub request_body: std::option::Option<RequestBody>,
    /// Input type (request message type)
    pub input_type: TypeReference,
    /// Response type
    pub response_type: TypeReference,
}

/// Path parameter definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Whether the parameter is required
    pub required: bool,
}

/// Query parameter definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Whether the parameter is required
    pub required: bool,
}

/// Parameter type information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Custom(String),
}

/// Request body configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestBody {
    /// Field name in the request message
    pub field: std::option::Option<String>,
    /// Content type
    pub content_type: String,
    /// Whether the entire message is the body
    pub is_entire_message: bool,
}

/// Message type definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Message name
    pub name: String,
    /// Message fields
    pub fields: Vec<Field>,
    /// Nested messages
    pub nested_messages: Vec<Message>,
    /// Nested enums
    pub nested_enums: Vec<Enum>,
    /// Message options
    pub options: Vec<ProtoOption>,
    /// Documentation comments
    pub comments: Vec<Comment>,
}

/// Message field definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: FieldType,
    /// Field number
    pub number: u32,
    /// Field label (optional, required, repeated)
    pub label: FieldLabel,
    /// Field options
    pub options: Vec<ProtoOption>,
    /// Documentation comments
    pub comments: Vec<Comment>,
}

/// Field type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    /// Scalar types
    Double,
    Float,
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    Sint64,
    Fixed32,
    Fixed64,
    Sfixed32,
    Sfixed64,
    Bool,
    String,
    Bytes,
    /// Message or enum type
    MessageOrEnum(TypeReference),
}

/// Field label
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldLabel {
    Optional,
    Required,
    Repeated,
}

/// Enum definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Enum {
    /// Enum name
    pub name: String,
    /// Enum values
    pub values: Vec<EnumValue>,
    /// Enum options
    pub options: Vec<ProtoOption>,
    /// Documentation comments
    pub comments: Vec<Comment>,
}

/// Enum value definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumValue {
    /// Value name
    pub name: String,
    /// Value number
    pub number: i32,
    /// Value options
    pub options: Vec<ProtoOption>,
    /// Documentation comments
    pub comments: Vec<Comment>,
}

/// Documentation comment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    /// Comment text
    pub text: String,
    /// Comment type (leading, trailing, detached)
    pub comment_type: CommentType,
}

/// Comment type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommentType {
    Leading,
    Trailing,
    Detached,
}

/// Type definition information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDefinition {
    /// Simple name of the type
    pub name: String,
    /// Fully qualified name including package
    pub fully_qualified_name: String,
    /// Kind of type definition
    pub definition_type: TypeDefinitionKind,
    /// Package where this type is defined
    pub package: std::option::Option<String>,
}

/// Kind of type definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeDefinitionKind {
    Message,
    Enum,
    Service,
}

/// Resolved type information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedType {
    /// Original type reference
    pub original_reference: TypeReference,
    /// Resolved fully qualified name
    pub resolved_name: String,
    /// Where the type is defined
    pub definition_location: TypeLocation,
    /// Whether this is a scalar type
    pub is_scalar: bool,
    /// Whether this is a well-known type
    pub is_well_known: bool,
}

/// Location where a type is defined
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeLocation {
    /// Defined in the current file
    Local,
    /// Defined in an imported file
    External,
    /// Built-in scalar type
    Builtin,
    /// Well-known Protocol Buffer type
    WellKnown,
}

/// Dependency information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    /// Path to the imported file
    pub import_path: String,
    /// Type of import
    pub import_type: ImportType,
    /// Whether this is a public import
    pub is_public: bool,
    /// Whether this is a weak import
    pub is_weak: bool,
}

/// Unresolved type reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnresolvedTypeReference {
    /// The type reference that couldn't be resolved
    pub type_reference: TypeReference,
    /// Context where this type reference was found
    pub context: String,
}

/// Type registry for managing type definitions across multiple files
#[derive(Debug, Clone, Default)]
pub struct TypeRegistry {
    /// All known type definitions
    pub types: std::collections::HashMap<String, TypeDefinition>,
    /// File dependencies
    pub dependencies: std::collections::HashMap<String, Vec<Dependency>>,
}

impl TypeRegistry {
    /// Create a new empty type registry
    pub fn new() -> Self {
        Self {
            types: std::collections::HashMap::new(),
            dependencies: std::collections::HashMap::new(),
        }
    }
    
    /// Register types from a proto file
    pub fn register_file(&mut self, file_path: &str, proto_file: &ProtoFile) {
        // Register all types from this file
        for type_def in proto_file.get_all_types() {
            self.types.insert(type_def.fully_qualified_name.clone(), type_def);
        }
        
        // Register dependencies
        self.dependencies.insert(file_path.to_string(), proto_file.get_dependencies());
    }
    
    /// Resolve a type reference using the registry
    pub fn resolve_type(&self, type_ref: &TypeReference, current_package: std::option::Option<&str>) -> std::option::Option<ResolvedType> {
        // If it's already fully qualified, look it up directly
        if type_ref.package.is_some() {
            let qualified_name = type_ref.fully_qualified_name();
            if let std::option::Option::Some(type_def) = self.types.get(&qualified_name) {
                return std::option::Option::Some(ResolvedType {
                    original_reference: type_ref.clone(),
                    resolved_name: qualified_name,
                    definition_location: TypeLocation::External,
                    is_scalar: false,
                    is_well_known: type_def.fully_qualified_name.starts_with("google.protobuf."),
                });
            }
        }
        
        // Check if it's a scalar type
        if type_ref.is_scalar() {
            return std::option::Option::Some(ResolvedType {
                original_reference: type_ref.clone(),
                resolved_name: type_ref.name.clone(),
                definition_location: TypeLocation::Builtin,
                is_scalar: true,
                is_well_known: false,
            });
        }
        
        // Try to resolve in current package first
        if let std::option::Option::Some(package) = current_package {
            let qualified_name = format!("{}.{}", package, type_ref.name);
            if let std::option::Option::Some(type_def) = self.types.get(&qualified_name) {
                return std::option::Option::Some(ResolvedType {
                    original_reference: type_ref.clone(),
                    resolved_name: qualified_name,
                    definition_location: TypeLocation::Local,
                    is_scalar: false,
                    is_well_known: type_def.fully_qualified_name.starts_with("google.protobuf."),
                });
            }
        }
        
        // Try to find the type by simple name
        for (qualified_name, type_def) in &self.types {
            if type_def.name == type_ref.name {
                return std::option::Option::Some(ResolvedType {
                    original_reference: type_ref.clone(),
                    resolved_name: qualified_name.clone(),
                    definition_location: if type_def.package == current_package.map(|s| s.to_string()) {
                        TypeLocation::Local
                    } else {
                        TypeLocation::External
                    },
                    is_scalar: false,
                    is_well_known: qualified_name.starts_with("google.protobuf."),
                });
            }
        }
        
        // Check well-known types
        let well_known_name = if type_ref.name.starts_with("google.protobuf.") {
            type_ref.name.clone()
        } else {
            format!("google.protobuf.{}", type_ref.name)
        };
        
        let well_known_types = [
            "google.protobuf.Timestamp",
            "google.protobuf.Duration", 
            "google.protobuf.Empty",
            "google.protobuf.Any",
            "google.protobuf.Struct",
            "google.protobuf.Value",
            "google.protobuf.ListValue",
            "google.protobuf.NullValue",
        ];
        
        if well_known_types.contains(&well_known_name.as_str()) {
            return std::option::Option::Some(ResolvedType {
                original_reference: type_ref.clone(),
                resolved_name: well_known_name,
                definition_location: TypeLocation::WellKnown,
                is_scalar: false,
                is_well_known: true,
            });
        }
        
        std::option::Option::None
    }
    
    /// Get all unresolved types across all registered files
    pub fn get_unresolved_types(&self) -> Vec<String> {
        // This would require keeping track of all type references
        // For now, return empty - this could be enhanced later
        Vec::new()
    }
    
    /// Check for circular dependencies
    pub fn check_circular_dependencies(&self) -> std::option::Option<Vec<String>> {
        // Implement cycle detection algorithm
        // For now, return None - this could be enhanced later
        std::option::Option::None
    }
}

impl ProtoFile {
    /// Create a new empty ProtoFile
    pub fn new() -> Self {
        Self {
            syntax: ProtocolVersion::Proto3,
            package: std::option::Option::None,
            imports: Vec::new(),
            options: Vec::new(),
            services: Vec::new(),
            messages: Vec::new(),
            enums: Vec::new(),
        }
    }
    
    /// Get all type definitions in this file
    pub fn get_all_types(&self) -> Vec<TypeDefinition> {
        let mut types = Vec::new();
        
        // Add messages
        for message in &self.messages {
            types.push(TypeDefinition {
                name: message.name.clone(),
                fully_qualified_name: self.qualify_type_name(&message.name),
                definition_type: TypeDefinitionKind::Message,
                package: self.package.clone(),
            });
            
            // Add nested types
            types.extend(self.get_nested_types_from_message(message));
        }
        
        // Add enums
        for enum_def in &self.enums {
            types.push(TypeDefinition {
                name: enum_def.name.clone(),
                fully_qualified_name: self.qualify_type_name(&enum_def.name),
                definition_type: TypeDefinitionKind::Enum,
                package: self.package.clone(),
            });
        }
        
        types
    }
    
    /// Get nested types from a message recursively
    fn get_nested_types_from_message(&self, message: &Message) -> Vec<TypeDefinition> {
        let mut types = Vec::new();
        
        for nested_message in &message.nested_messages {
            let qualified_name = format!("{}.{}", 
                self.qualify_type_name(&message.name), 
                nested_message.name
            );
            
            types.push(TypeDefinition {
                name: nested_message.name.clone(),
                fully_qualified_name: qualified_name,
                definition_type: TypeDefinitionKind::Message,
                package: self.package.clone(),
            });
            
            types.extend(self.get_nested_types_from_message(nested_message));
        }
        
        for nested_enum in &message.nested_enums {
            let qualified_name = format!("{}.{}", 
                self.qualify_type_name(&message.name), 
                nested_enum.name
            );
            
            types.push(TypeDefinition {
                name: nested_enum.name.clone(),
                fully_qualified_name: qualified_name,
                definition_type: TypeDefinitionKind::Enum,
                package: self.package.clone(),
            });
        }
        
        types
    }
    
    /// Qualify a type name with the package
    fn qualify_type_name(&self, type_name: &str) -> String {
        match &self.package {
            std::option::Option::Some(package) => format!("{}.{}", package, type_name),
            std::option::Option::None => type_name.to_string(),
        }
    }
    
    /// Resolve a type reference within this file's context
    pub fn resolve_type(&self, type_ref: &TypeReference) -> std::option::Option<ResolvedType> {
        // If it's already fully qualified, use as-is
        if type_ref.package.is_some() {
            return std::option::Option::Some(ResolvedType {
                original_reference: type_ref.clone(),
                resolved_name: type_ref.fully_qualified_name(),
                definition_location: TypeLocation::External,
                is_scalar: type_ref.is_scalar(),
                is_well_known: type_ref.is_well_known_type(),
            });
        }
        
        // Check if it's a scalar type
        if type_ref.is_scalar() {
            return std::option::Option::Some(ResolvedType {
                original_reference: type_ref.clone(),
                resolved_name: type_ref.name.clone(),
                definition_location: TypeLocation::Builtin,
                is_scalar: true,
                is_well_known: false,
            });
        }
        
        // Check if it's defined in this file
        let all_types = self.get_all_types();
        for type_def in &all_types {
            if type_def.name == type_ref.name || type_def.fully_qualified_name == type_ref.name {
                return std::option::Option::Some(ResolvedType {
                    original_reference: type_ref.clone(),
                    resolved_name: type_def.fully_qualified_name.clone(),
                    definition_location: TypeLocation::Local,
                    is_scalar: false,
                    is_well_known: false,
                });
            }
        }
        
        // Check if it's a well-known type
        let qualified_name = if type_ref.name.starts_with("google.protobuf.") {
            type_ref.name.clone()
        } else {
            format!("google.protobuf.{}", type_ref.name)
        };
        
        let well_known_ref = TypeReference {
            name: type_ref.name.clone(),
            package: std::option::Option::Some("google.protobuf".to_string()),
            is_stream: type_ref.is_stream,
        };
        
        if well_known_ref.is_well_known_type() {
            return std::option::Option::Some(ResolvedType {
                original_reference: type_ref.clone(),
                resolved_name: qualified_name,
                definition_location: TypeLocation::WellKnown,
                is_scalar: false,
                is_well_known: true,
            });
        }
        
        // Type not found
        std::option::Option::None
    }
    
    /// Get all dependencies (imported files)
    pub fn get_dependencies(&self) -> Vec<Dependency> {
        self.imports.iter().map(|import| {
            Dependency {
                import_path: import.path.clone(),
                import_type: import.import_type.clone(),
                is_public: matches!(import.import_type, ImportType::Public),
                is_weak: matches!(import.import_type, ImportType::Weak),
            }
        }).collect()
    }
    
    /// Find all unresolved type references
    pub fn find_unresolved_types(&self) -> Vec<UnresolvedTypeReference> {
        let mut unresolved = Vec::new();
        
        // Check service method types
        for service in &self.services {
            for method in &service.methods {
                if self.resolve_type(&method.input_type).is_none() {
                    unresolved.push(UnresolvedTypeReference {
                        type_reference: method.input_type.clone(),
                        context: format!("Service {}, method {} input", service.name, method.name),
                    });
                }
                
                if self.resolve_type(&method.output_type).is_none() {
                    unresolved.push(UnresolvedTypeReference {
                        type_reference: method.output_type.clone(),
                        context: format!("Service {}, method {} output", service.name, method.name),
                    });
                }
            }
        }
        
        // Check message field types
        for message in &self.messages {
            unresolved.extend(self.find_unresolved_in_message(message, &message.name));
        }
        
        unresolved
    }
    
    /// Find unresolved types in a message recursively
    fn find_unresolved_in_message(&self, message: &Message, context_path: &str) -> Vec<UnresolvedTypeReference> {
        let mut unresolved = Vec::new();
        
        for field in &message.fields {
            if let FieldType::MessageOrEnum(type_ref) = &field.field_type {
                if self.resolve_type(type_ref).is_none() {
                    unresolved.push(UnresolvedTypeReference {
                        type_reference: type_ref.clone(),
                        context: format!("{}.{}", context_path, field.name),
                    });
                }
            }
        }
        
        // Check nested messages
        for nested_message in &message.nested_messages {
            let nested_context = format!("{}.{}", context_path, nested_message.name);
            unresolved.extend(self.find_unresolved_in_message(nested_message, &nested_context));
        }
        
        unresolved
    }
}

impl Default for ProtoFile {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpMethod {
    /// Convert HTTP method to string
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Custom(method) => method,
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl HttpRoute {
    /// Create a new HTTP route
    pub fn new(
        service_name: String,
        method_name: String,
        http_method: HttpMethod,
        path_template: String,
    ) -> Self {
        Self {
            service_name,
            method_name,
            http_method,
            path_template,
            path_parameters: Vec::new(),
            query_parameters: Vec::new(),
            request_body: std::option::Option::None,
            input_type: TypeReference::new("Empty".to_string()),
            response_type: TypeReference::new("Empty".to_string()),
        }
    }
    
    /// Add a path parameter
    pub fn with_path_parameter(mut self, param: PathParameter) -> Self {
        self.path_parameters.push(param);
        self
    }
    
    /// Add a query parameter
    pub fn with_query_parameter(mut self, param: QueryParameter) -> Self {
        self.query_parameters.push(param);
        self
    }
    
    /// Set the request body
    pub fn with_request_body(mut self, body: RequestBody) -> Self {
        self.request_body = std::option::Option::Some(body);
        self
    }
    
    /// Set the response type
    pub fn with_response_type(mut self, response_type: TypeReference) -> Self {
        self.response_type = response_type;
        self
    }
    
    /// Get the OpenAPI operation ID
    pub fn operation_id(&self) -> String {
        format!("{}_{}", self.service_name, self.method_name)
    }
    
    /// Check if this route has path parameters
    pub fn has_path_parameters(&self) -> bool {
        !self.path_parameters.is_empty()
    }
    
    /// Check if this route has query parameters
    pub fn has_query_parameters(&self) -> bool {
        !self.query_parameters.is_empty()
    }
    
    /// Check if this route has a request body
    pub fn has_request_body(&self) -> bool {
        self.request_body.is_some()
    }
}

impl PathParameter {
    /// Create a new path parameter
    pub fn new(name: String, param_type: ParameterType) -> Self {
        Self {
            name,
            param_type,
            required: true, // Path parameters are always required
        }
    }
}

impl QueryParameter {
    /// Create a new query parameter
    pub fn new(name: String, param_type: ParameterType, required: bool) -> Self {
        Self {
            name,
            param_type,
            required,
        }
    }
    
    /// Create an optional query parameter
    pub fn optional(name: String, param_type: ParameterType) -> Self {
        Self::new(name, param_type, false)
    }
    
    /// Create a required query parameter
    pub fn required(name: String, param_type: ParameterType) -> Self {
        Self::new(name, param_type, true)
    }
}

impl RequestBody {
    /// Create a request body for the entire message
    pub fn entire_message() -> Self {
        Self {
            field: std::option::Option::None,
            content_type: "application/json".to_string(),
            is_entire_message: true,
        }
    }
    
    /// Create a request body for a specific field
    pub fn field(field_name: String) -> Self {
        Self {
            field: std::option::Option::Some(field_name),
            content_type: "application/json".to_string(),
            is_entire_message: false,
        }
    }
    
    /// Set the content type
    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = content_type;
        self
    }
}

impl Service {
    /// Create a new service
    pub fn new(name: String) -> Self {
        Self {
            name,
            methods: Vec::new(),
            options: Vec::new(),
            comments: Vec::new(),
        }
    }
    
    /// Add a method to the service
    pub fn with_method(mut self, method: RpcMethod) -> Self {
        self.methods.push(method);
        self
    }
    
    /// Get all HTTP-enabled methods
    pub fn http_methods(&self) -> Vec<&RpcMethod> {
        self.methods.iter()
            .filter(|method| method.http_annotation.is_some())
            .collect()
    }
}

impl RpcMethod {
    /// Create a new RPC method
    pub fn new(
        name: String,
        input_type: TypeReference,
        output_type: TypeReference,
    ) -> Self {
        Self {
            name,
            input_type,
            output_type,
            options: Vec::new(),
            comments: Vec::new(),
            http_annotation: std::option::Option::None,
        }
    }
    
    /// Add HTTP annotation to the method
    pub fn with_http_annotation(mut self, annotation: HttpAnnotation) -> Self {
        self.http_annotation = std::option::Option::Some(annotation);
        self
    }
    
    /// Check if this method has HTTP annotation
    pub fn is_http_enabled(&self) -> bool {
        self.http_annotation.is_some()
    }
}

impl TypeReference {
    /// Create a new type reference
    pub fn new(name: String) -> Self {
        Self {
            name,
            package: std::option::Option::None,
            is_stream: false,
        }
    }
    
    /// Create a streaming type reference
    pub fn streaming(name: String) -> Self {
        Self {
            name,
            package: std::option::Option::None,
            is_stream: true,
        }
    }
    
    /// Create a type reference with package
    pub fn with_package(name: String, package: String) -> Self {
        Self {
            name,
            package: std::option::Option::Some(package),
            is_stream: false,
        }
    }
    
    /// Get the fully qualified name
    pub fn fully_qualified_name(&self) -> String {
        match &self.package {
            std::option::Option::Some(package) => format!("{}.{}", package, self.name),
            std::option::Option::None => self.name.clone(),
        }
    }
    
    /// Check if this is a scalar type
    pub fn is_scalar(&self) -> bool {
        matches!(self.name.as_str(), 
            "double" | "float" | "int32" | "int64" | "uint32" | "uint64" |
            "sint32" | "sint64" | "fixed32" | "fixed64" | "sfixed32" | "sfixed64" |
            "bool" | "string" | "bytes"
        )
    }
    
    /// Check if this is a well-known type
    pub fn is_well_known_type(&self) -> bool {
        matches!(self.fully_qualified_name().as_str(),
            "google.protobuf.Timestamp" | "google.protobuf.Duration" |
            "google.protobuf.Empty" | "google.protobuf.Any" |
            "google.protobuf.Struct" | "google.protobuf.Value" |
            "google.protobuf.ListValue" | "google.protobuf.NullValue"
        )
    }
}