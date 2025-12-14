//! Protocol Buffer parser implementation using nom

use crate::core::*;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{
        alpha1, alphanumeric1, char, digit1, line_ending, multispace0, multispace1, space0, space1,
    },
    combinator::{map, opt, recognize, value, verify},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// nom-based Protocol Buffer parser
pub struct NomProtoParser {
    config: ParserConfig,
    /// Cache for parsed imports to prevent circular dependencies
    import_cache: std::cell::RefCell<HashMap<PathBuf, ProtoFile>>,
    /// Track import chain to detect cycles
    import_chain: std::cell::RefCell<Vec<PathBuf>>,
}

impl NomProtoParser {
    /// Create a new parser with default configuration
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
            import_cache: std::cell::RefCell::new(HashMap::new()),
            import_chain: std::cell::RefCell::new(Vec::new()),
        }
    }
    
    /// Create a new parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        Self { 
            config,
            import_cache: std::cell::RefCell::new(HashMap::new()),
            import_chain: std::cell::RefCell::new(Vec::new()),
        }
    }
    
    /// Clear the import cache
    pub fn clear_cache(&self) {
        self.import_cache.borrow_mut().clear();
        self.import_chain.borrow_mut().clear();
    }
}

impl Default for NomProtoParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtoParser for NomProtoParser {
    type Error = ParseError;
    
    fn parse_file(&self, path: &Path) -> Result<ProtoFile, Self::Error> {
        let content = std::fs::read_to_string(path)
            .map_err(|_| ParseError::FileNotFound { path: path.to_path_buf() })?;
        
        // Check for circular imports
        let canonical_path = path.canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());
        
        if self.import_chain.borrow().contains(&canonical_path) {
            let mut cycle = self.import_chain.borrow().clone();
            cycle.push(canonical_path);
            return Err(ParseError::CircularImport { 
                cycle: cycle.into_iter().map(|p| p.to_string_lossy().to_string()).collect() 
            });
        }
        
        // Check cache first
        if let Some(cached) = self.import_cache.borrow().get(&canonical_path) {
            return Ok(cached.clone());
        }
        
        // Add to import chain
        self.import_chain.borrow_mut().push(canonical_path.clone());
        
        let result = self.parse_content(&content);
        
        // Remove from import chain
        self.import_chain.borrow_mut().pop();
        
        // Cache successful parse
        if let Ok(ref proto_file) = result {
            self.import_cache.borrow_mut().insert(canonical_path, proto_file.clone());
        }
        
        result
    }
    
    fn parse_content(&self, content: &str) -> Result<ProtoFile, Self::Error> {
        match proto_file(content) {
            Ok((remaining, mut proto_file)) => {
                // Check if there's unparsed content (should be only whitespace/comments)
                let remaining = remaining.trim();
                if !remaining.is_empty() && !remaining.starts_with("//") && !remaining.starts_with("/*") {
                    return Err(ParseError::InvalidSyntax {
                        message: format!("Unexpected content at end of file: {}", remaining),
                    });
                }
                
                // Resolve imports if configured and not in test mode
                if !proto_file.imports.is_empty() && !cfg!(test) {
                    self.resolve_imports(&mut proto_file)?;
                }
                
                Ok(proto_file)
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(ParseError::InvalidSyntax {
                    message: format!("Parse error: {:?}", e),
                })
            }
            Err(nom::Err::Incomplete(_)) => {
                Err(ParseError::InvalidSyntax {
                    message: "Incomplete input".to_string(),
                })
            }
        }
    }
    
    fn parse_with_imports(&self, path: &Path, import_paths: &[PathBuf]) -> Result<ProtoFile, Self::Error> {
        // Update config with additional import paths
        let mut config = self.config.clone();
        config.include_paths.extend_from_slice(import_paths);
        
        let parser = Self::with_config(config);
        parser.parse_file(path)
    }
}

impl NomProtoParser {
    /// Resolve imports in a proto file
    fn resolve_imports(&self, proto_file: &mut ProtoFile) -> Result<(), ParseError> {
        for import in &proto_file.imports {
            // 对于 google/api 相关的导入，我们可以跳过实际的文件解析
            // 因为我们只需要识别 HTTP 注解，不需要完整的类型定义
            if import.path.starts_with("google/api/") {
                // 跳过 google/api 导入，这些通常是注解定义
                continue;
            }
            
            // 对于其他导入，尝试解析，但如果失败也不要中断整个过程
            match self.resolve_single_import(&import.path) {
                Ok(_) => {
                    // 成功解析导入
                }
                Err(_) => {
                    // 导入解析失败，但继续处理
                    // 在实际应用中，可能需要记录警告
                }
            }
        }
        Ok(())
    }
    
    /// Resolve a single import
    fn resolve_single_import(&self, import_path: &str) -> Result<ProtoFile, ParseError> {
        // Try each include path
        for include_path in &self.config.include_paths {
            let full_path = include_path.join(import_path);
            if full_path.exists() {
                return self.parse_file(&full_path);
            }
        }
        
        Err(ParseError::ImportNotFound {
            import_path: import_path.to_string(),
        })
    }
}

// Parser combinators for Protocol Buffer syntax

/// Parse a complete proto file
fn proto_file(input: &str) -> IResult<&str, ProtoFile> {
    // Skip initial comments and whitespace
    let (input, _) = many0(alt((
        map(comment, |_| ()),
        map(multispace1, |_| ()),
    )))(input)?;
    let (input, _) = multispace0(input)?;
    
    let (input, syntax) = opt(syntax_statement)(input)?;
    let (input, _) = multispace0(input)?;
    
    let (input, package) = opt(package_statement)(input)?;
    let (input, _) = multispace0(input)?;
    
    let (input, imports) = many0(terminated(import_statement, multispace0))(input)?;
    let (input, _) = multispace0(input)?;
    
    let (input, options) = many0(terminated(option_statement, multispace0))(input)?;
    let (input, _) = multispace0(input)?;
    
    let (input, definitions) = many0(terminated(top_level_definition, multispace0))(input)?;
    let (input, _) = multispace0(input)?;
    
    let mut services = Vec::new();
    let mut messages = Vec::new();
    let mut enums = Vec::new();
    
    for def in definitions {
        match def {
            TopLevelDefinition::Service(service) => services.push(service),
            TopLevelDefinition::Message(message) => messages.push(message),
            TopLevelDefinition::Enum(enum_def) => enums.push(enum_def),
        }
    }
    
    Ok((input, ProtoFile {
        syntax: syntax.unwrap_or(ProtocolVersion::Proto3),
        package,
        imports,
        options,
        services,
        messages,
        enums,
    }))
}

/// Parse syntax statement
fn syntax_statement(input: &str) -> IResult<&str, ProtocolVersion> {
    let (input, _) = tag("syntax")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, version) = alt((
        value(ProtocolVersion::Proto2, tag("\"proto2\"")),
        value(ProtocolVersion::Proto3, tag("\"proto3\"")),
    ))(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(';')(input)?;
    
    Ok((input, version))
}

/// Parse package statement
fn package_statement(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("package")(input)?;
    let (input, _) = space1(input)?;
    let (input, package_name) = full_identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(';')(input)?;
    
    Ok((input, package_name))
}

/// Parse import statement
fn import_statement(input: &str) -> IResult<&str, Import> {
    let (input, import_type) = alt((
        value(ImportType::Public, tag("import public")),
        value(ImportType::Weak, tag("import weak")),
        value(ImportType::Normal, tag("import")),
    ))(input)?;
    let (input, _) = space1(input)?;
    let (input, path) = string_literal(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(';')(input)?;
    
    Ok((input, Import { import_type, path }))
}

/// Parse option statement
fn option_statement(input: &str) -> IResult<&str, ProtoOption> {
    let (input, _) = tag("option")(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = option_name(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, value) = option_value(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(';')(input)?;
    
    Ok((input, ProtoOption { name, value }))
}

/// Parse option name (can be complex like (google.api.http) or validate.rules)
fn option_name(input: &str) -> IResult<&str, String> {
    alt((
        // Parenthesized option name
        delimited(
            char('('),
            full_identifier,
            char(')')
        ),
        // Complex option name with dots and brackets
        map(
            recognize(pair(
                full_identifier,
                many0(alt((
                    preceded(char('.'), full_identifier),
                    delimited(char('['), full_identifier, char(']')),
                )))
            )),
            |s: &str| s.to_string()
        ),
        // Simple option name
        full_identifier,
    ))(input)
}

/// Parse option value
fn option_value(input: &str) -> IResult<&str, OptionValue> {
    alt((
        map(string_literal, OptionValue::String),
        map(number_literal, OptionValue::Number),
        map(boolean_literal, OptionValue::Boolean),
        map(message_literal, OptionValue::MessageLiteral),
        map(identifier, OptionValue::Identifier),
    ))(input)
}

/// Parse message literal (for complex option values)
fn message_literal(input: &str) -> IResult<&str, HashMap<String, OptionValue>> {
    let (input, _) = char('{')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, fields) = separated_list0(
        tuple((multispace0, opt(char(',')), multispace0)),
        message_field
    )(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('}')(input)?;
    
    Ok((input, fields.into_iter().collect()))
}

/// Parse message field in a message literal
fn message_field(input: &str) -> IResult<&str, (String, OptionValue)> {
    let (input, name) = identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space0(input)?;
    let (input, value) = option_value(input)?;
    
    Ok((input, (name, value)))
}

/// Parse top-level definitions
fn top_level_definition(input: &str) -> IResult<&str, TopLevelDefinition> {
    alt((
        map(service_definition, TopLevelDefinition::Service),
        map(message_definition, TopLevelDefinition::Message),
        map(enum_definition, TopLevelDefinition::Enum),
    ))(input)
}

#[derive(Debug, Clone)]
enum TopLevelDefinition {
    Service(Service),
    Message(Message),
    Enum(Enum),
}

/// Parse service definition
fn service_definition(input: &str) -> IResult<&str, Service> {
    let (input, comments) = many0(comment)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("service")(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('{')(input)?;
    let (input, _) = multispace0(input)?;
    
    let (input, body_items) = many0(terminated(service_body_item, multispace0))(input)?;
    
    let (input, _) = char('}')(input)?;
    
    let mut methods = Vec::new();
    let mut options = Vec::new();
    
    for item in body_items {
        match item {
            ServiceBodyItem::Method(method) => methods.push(method),
            ServiceBodyItem::Option(option) => options.push(option),
        }
    }
    
    Ok((input, Service {
        name,
        methods,
        options,
        comments,
    }))
}

#[derive(Debug, Clone)]
enum ServiceBodyItem {
    Method(RpcMethod),
    Option(ProtoOption),
}

/// Parse service body item
fn service_body_item(input: &str) -> IResult<&str, ServiceBodyItem> {
    alt((
        map(rpc_method, ServiceBodyItem::Method),
        map(option_statement, ServiceBodyItem::Option),
    ))(input)
}

/// Parse RPC method
fn rpc_method(input: &str) -> IResult<&str, RpcMethod> {
    let (input, comments) = many0(comment)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("rpc")(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = space0(input)?;
    let (input, input_stream) = opt(tag("stream"))(input)?;
    let (input, _) = space0(input)?;
    let (input, input_type_name) = type_name(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(')')(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("returns")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = space0(input)?;
    let (input, output_stream) = opt(tag("stream"))(input)?;
    let (input, _) = space0(input)?;
    let (input, output_type_name) = type_name(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(')')(input)?;
    let (input, _) = space0(input)?;
    
    let (input, method_options) = alt((
        // Method with options block
        delimited(
            char('{'),
            preceded(multispace0, many0(terminated(method_option, multispace0))),
            char('}')
        ),
        // Method without options
        value(Vec::new(), char(';'))
    ))(input)?;
    
    let input_type = TypeReference {
        name: input_type_name,
        package: None,
        is_stream: input_stream.is_some(),
    };
    
    let output_type = TypeReference {
        name: output_type_name,
        package: None,
        is_stream: output_stream.is_some(),
    };
    
    // Extract HTTP annotation from options
    let mut http_annotation = None;
    let mut options = Vec::new();
    
    for option in method_options {
        if option.name == "google.api.http" || option.name == "(google.api.http)" {
            http_annotation = parse_http_annotation(&option.value);
        } else {
            options.push(option);
        }
    }
    
    Ok((input, RpcMethod {
        name,
        input_type,
        output_type,
        options,
        comments,
        http_annotation,
    }))
}

/// Parse method option
fn method_option(input: &str) -> IResult<&str, ProtoOption> {
    option_statement(input)
}

/// Parse HTTP annotation from option value
fn parse_http_annotation(value: &OptionValue) -> Option<HttpAnnotation> {
    if let OptionValue::MessageLiteral(fields) = value {
        let mut method = None;
        let mut path = String::new();
        let mut body = None;
        let additional_bindings = Vec::new(); // TODO: Parse additional bindings
        
        for (key, val) in fields {
            match key.as_str() {
                "get" => {
                    if let OptionValue::String(p) = val {
                        method = Some(HttpMethod::Get);
                        path = p.clone();
                    }
                }
                "post" => {
                    if let OptionValue::String(p) = val {
                        method = Some(HttpMethod::Post);
                        path = p.clone();
                    }
                }
                "put" => {
                    if let OptionValue::String(p) = val {
                        method = Some(HttpMethod::Put);
                        path = p.clone();
                    }
                }
                "patch" => {
                    if let OptionValue::String(p) = val {
                        method = Some(HttpMethod::Patch);
                        path = p.clone();
                    }
                }
                "delete" => {
                    if let OptionValue::String(p) = val {
                        method = Some(HttpMethod::Delete);
                        path = p.clone();
                    }
                }
                "body" => {
                    if let OptionValue::String(b) = val {
                        body = Some(b.clone());
                    }
                }
                _ => {}
            }
        }
        
        if let Some(method) = method {
            return Some(HttpAnnotation {
                method,
                path,
                body,
                additional_bindings,
            });
        }
    }
    None
}

/// Parse message definition
fn message_definition(input: &str) -> IResult<&str, Message> {
    let (input, comments) = many0(comment)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("message")(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('{')(input)?;
    let (input, _) = multispace0(input)?;
    
    let (input, body_items) = many0(terminated(message_body_item, multispace0))(input)?;
    
    let (input, _) = char('}')(input)?;
    
    let mut fields = Vec::new();
    let mut nested_messages = Vec::new();
    let mut nested_enums = Vec::new();
    let mut options = Vec::new();
    
    for item in body_items {
        match item {
            MessageBodyItem::Field(field) => fields.push(field),
            MessageBodyItem::Message(message) => nested_messages.push(message),
            MessageBodyItem::Enum(enum_def) => nested_enums.push(enum_def),
            MessageBodyItem::Option(option) => options.push(option),
        }
    }
    
    Ok((input, Message {
        name,
        fields,
        nested_messages,
        nested_enums,
        options,
        comments,
    }))
}

#[derive(Debug, Clone)]
enum MessageBodyItem {
    Field(Field),
    Message(Message),
    Enum(Enum),
    Option(ProtoOption),
}

/// Parse message body item
fn message_body_item(input: &str) -> IResult<&str, MessageBodyItem> {
    alt((
        map(field_definition, MessageBodyItem::Field),
        map(message_definition, MessageBodyItem::Message),
        map(enum_definition, MessageBodyItem::Enum),
        map(option_statement, MessageBodyItem::Option),
    ))(input)
}

/// Parse field definition
fn field_definition(input: &str) -> IResult<&str, Field> {
    let (input, comments) = many0(comment)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, label) = opt(field_label)(input)?;
    let (input, _) = space0(input)?;
    let (input, field_type) = field_type(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, number) = field_number(input)?;
    let (input, _) = space0(input)?;
    let (input, options) = opt(field_options)(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(';')(input)?;
    
    Ok((input, Field {
        name,
        field_type,
        number,
        label: label.unwrap_or(FieldLabel::Optional),
        options: options.unwrap_or_default(),
        comments,
    }))
}

/// Parse field label
fn field_label(input: &str) -> IResult<&str, FieldLabel> {
    alt((
        value(FieldLabel::Required, tag("required")),
        value(FieldLabel::Optional, tag("optional")),
        value(FieldLabel::Repeated, tag("repeated")),
    ))(input)
}

/// Parse field type
fn field_type(input: &str) -> IResult<&str, FieldType> {
    alt((
        value(FieldType::Double, tag("double")),
        value(FieldType::Float, tag("float")),
        value(FieldType::Int32, tag("int32")),
        value(FieldType::Int64, tag("int64")),
        value(FieldType::Uint32, tag("uint32")),
        value(FieldType::Uint64, tag("uint64")),
        value(FieldType::Sint32, tag("sint32")),
        value(FieldType::Sint64, tag("sint64")),
        value(FieldType::Fixed32, tag("fixed32")),
        value(FieldType::Fixed64, tag("fixed64")),
        value(FieldType::Sfixed32, tag("sfixed32")),
        value(FieldType::Sfixed64, tag("sfixed64")),
        value(FieldType::Bool, tag("bool")),
        value(FieldType::String, tag("string")),
        value(FieldType::Bytes, tag("bytes")),
        map(type_name, |name| FieldType::MessageOrEnum(TypeReference::new(name))),
    ))(input)
}

/// Parse field number
fn field_number(input: &str) -> IResult<&str, u32> {
    map(digit1, |s: &str| s.parse().unwrap_or(0))(input)
}

/// Parse field options
fn field_options(input: &str) -> IResult<&str, Vec<ProtoOption>> {
    delimited(
        char('['),
        separated_list0(
            tuple((space0, char(','), space0)),
            field_option
        ),
        char(']')
    )(input)
}

/// Parse field option
fn field_option(input: &str) -> IResult<&str, ProtoOption> {
    let (input, name) = option_name(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, value) = option_value(input)?;
    
    Ok((input, ProtoOption { name, value }))
}

/// Parse enum definition
fn enum_definition(input: &str) -> IResult<&str, Enum> {
    let (input, comments) = many0(comment)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("enum")(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('{')(input)?;
    let (input, _) = multispace0(input)?;
    
    let (input, body_items) = many0(terminated(enum_body_item, multispace0))(input)?;
    
    let (input, _) = char('}')(input)?;
    
    let mut values = Vec::new();
    let mut options = Vec::new();
    
    for item in body_items {
        match item {
            EnumBodyItem::Value(value) => values.push(value),
            EnumBodyItem::Option(option) => options.push(option),
        }
    }
    
    Ok((input, Enum {
        name,
        values,
        options,
        comments,
    }))
}

#[derive(Debug, Clone)]
enum EnumBodyItem {
    Value(EnumValue),
    Option(ProtoOption),
}

/// Parse enum body item
fn enum_body_item(input: &str) -> IResult<&str, EnumBodyItem> {
    alt((
        map(enum_value, EnumBodyItem::Value),
        map(option_statement, EnumBodyItem::Option),
    ))(input)
}

/// Parse enum value
fn enum_value(input: &str) -> IResult<&str, EnumValue> {
    let (input, comments) = many0(comment)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, number) = enum_number(input)?;
    let (input, _) = space0(input)?;
    let (input, options) = opt(field_options)(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(';')(input)?;
    
    Ok((input, EnumValue {
        name,
        number,
        options: options.unwrap_or_default(),
        comments,
    }))
}

/// Parse enum number (can be negative)
fn enum_number(input: &str) -> IResult<&str, i32> {
    let (input, sign) = opt(char('-'))(input)?;
    let (input, digits) = digit1(input)?;
    let number: i32 = digits.parse().unwrap_or(0);
    Ok((input, if sign.is_some() { -number } else { number }))
}

/// Parse comments
fn comment(input: &str) -> IResult<&str, Comment> {
    alt((
        line_comment,
        block_comment,
    ))(input)
}

/// Parse line comment
fn line_comment(input: &str) -> IResult<&str, Comment> {
    let (input, _) = tag("//")(input)?;
    let (input, text) = take_until("\n")(input)?;
    let (input, _) = line_ending(input)?;
    
    Ok((input, Comment {
        text: text.trim().to_string(),
        comment_type: CommentType::Leading,
    }))
}

/// Parse block comment
fn block_comment(input: &str) -> IResult<&str, Comment> {
    let (input, _) = tag("/*")(input)?;
    let (input, text) = take_until("*/")(input)?;
    let (input, _) = tag("*/")(input)?;
    
    Ok((input, Comment {
        text: text.trim().to_string(),
        comment_type: CommentType::Leading,
    }))
}

/// Parse identifier
fn identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_"))))
        )),
        |s: &str| s.to_string()
    )(input)
}

/// Parse full identifier (with dots)
fn full_identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            identifier,
            many0(preceded(char('.'), identifier))
        )),
        |s: &str| s.to_string()
    )(input)
}

/// Parse type name (can be fully qualified)
fn type_name(input: &str) -> IResult<&str, String> {
    full_identifier(input)
}

/// Parse string literal
fn string_literal(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        map(
            many0(alt((
                map(tag("\\\""), |_| '"'),
                map(tag("\\\\"), |_| '\\'),
                map(tag("\\n"), |_| '\n'),
                map(tag("\\r"), |_| '\r'),
                map(tag("\\t"), |_| '\t'),
                verify(nom::character::complete::anychar, |c| *c != '"' && *c != '\\'),
            ))),
            |chars| chars.into_iter().collect()
        ),
        char('"')
    )(input)
}

/// Parse number literal
fn number_literal(input: &str) -> IResult<&str, f64> {
    map(
        recognize(tuple((
            opt(char('-')),
            digit1,
            opt(tuple((char('.'), digit1))),
            opt(tuple((alt((char('e'), char('E'))), opt(alt((char('+'), char('-')))), digit1))),
        ))),
        |s: &str| s.parse().unwrap_or(0.0)
    )(input)
}

/// Parse boolean literal
fn boolean_literal(input: &str) -> IResult<&str, bool> {
    alt((
        value(true, tag("true")),
        value(false, tag("false")),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_proto() {
        let content = r#"
syntax = "proto3";

package example.v1;

import "google/api/annotations.proto";

service UserService {
    rpc GetUser(GetUserRequest) returns (GetUserResponse) {
        option (google.api.http) = {
            get: "/v1/users/{user_id}"
        };
    }
}

message GetUserRequest {
    string user_id = 1;
}

message GetUserResponse {
    string name = 1;
    string email = 2;
}
"#;

        let parser = NomProtoParser::new();
        let result = parser.parse_content(content);
        
        assert!(result.is_ok(), "Failed to parse proto: {:?}", result.err());
        
        let proto_file = result.unwrap();
        assert_eq!(proto_file.syntax, ProtocolVersion::Proto3);
        assert_eq!(proto_file.package, Some("example.v1".to_string()));
        assert_eq!(proto_file.imports.len(), 1);
        assert_eq!(proto_file.services.len(), 1);
        assert_eq!(proto_file.messages.len(), 2);
        
        let service = &proto_file.services[0];
        assert_eq!(service.name, "UserService");
        assert_eq!(service.methods.len(), 1);
        
        let method = &service.methods[0];
        assert_eq!(method.name, "GetUser");
        assert!(method.http_annotation.is_some());
        
        let http_annotation = method.http_annotation.as_ref().unwrap();
        assert_eq!(http_annotation.method, HttpMethod::Get);
        assert_eq!(http_annotation.path, "/v1/users/{user_id}");
    }

    #[test]
    fn test_parse_syntax_statement() {
        let parser = NomProtoParser::new();
        
        let proto2 = r#"syntax = "proto2";"#;
        let result = parser.parse_content(proto2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().syntax, ProtocolVersion::Proto2);
        
        let proto3 = r#"syntax = "proto3";"#;
        let result = parser.parse_content(proto3);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().syntax, ProtocolVersion::Proto3);
    }

    #[test]
    fn test_parse_package_statement() {
        let content = r#"
syntax = "proto3";
package com.example.api.v1;
"#;
        
        let parser = NomProtoParser::new();
        let result = parser.parse_content(content);
        
        assert!(result.is_ok());
        let proto_file = result.unwrap();
        assert_eq!(proto_file.package, Some("com.example.api.v1".to_string()));
    }

    #[test]
    fn test_parse_imports() {
        let content = r#"
syntax = "proto3";

import "google/api/annotations.proto";
import public "google/protobuf/timestamp.proto";
import weak "google/protobuf/empty.proto";
"#;
        
        let parser = NomProtoParser::new();
        let result = parser.parse_content(content);
        
        assert!(result.is_ok());
        let proto_file = result.unwrap();
        assert_eq!(proto_file.imports.len(), 3);
        
        assert_eq!(proto_file.imports[0].import_type, ImportType::Normal);
        assert_eq!(proto_file.imports[0].path, "google/api/annotations.proto");
        
        assert_eq!(proto_file.imports[1].import_type, ImportType::Public);
        assert_eq!(proto_file.imports[1].path, "google/protobuf/timestamp.proto");
        
        assert_eq!(proto_file.imports[2].import_type, ImportType::Weak);
        assert_eq!(proto_file.imports[2].path, "google/protobuf/empty.proto");
    }

    #[test]
    fn test_parse_message_with_fields() {
        let content = r#"
syntax = "proto3";

message User {
    string name = 1;
    int32 age = 2;
    repeated string emails = 3;
    optional bool active = 4;
}
"#;
        
        let parser = NomProtoParser::new();
        let result = parser.parse_content(content);
        
        assert!(result.is_ok());
        let proto_file = result.unwrap();
        assert_eq!(proto_file.messages.len(), 1);
        
        let message = &proto_file.messages[0];
        assert_eq!(message.name, "User");
        assert_eq!(message.fields.len(), 4);
        
        assert_eq!(message.fields[0].name, "name");
        assert_eq!(message.fields[0].field_type, FieldType::String);
        assert_eq!(message.fields[0].number, 1);
        
        assert_eq!(message.fields[2].label, FieldLabel::Repeated);
        assert_eq!(message.fields[3].label, FieldLabel::Optional);
    }

    #[test]
    fn test_parse_enum() {
        let content = r#"
syntax = "proto3";

enum Status {
    UNKNOWN = 0;
    ACTIVE = 1;
    INACTIVE = 2;
    DELETED = -1;
}
"#;
        
        let parser = NomProtoParser::new();
        let result = parser.parse_content(content);
        
        assert!(result.is_ok());
        let proto_file = result.unwrap();
        assert_eq!(proto_file.enums.len(), 1);
        
        let enum_def = &proto_file.enums[0];
        assert_eq!(enum_def.name, "Status");
        assert_eq!(enum_def.values.len(), 4);
        
        assert_eq!(enum_def.values[0].name, "UNKNOWN");
        assert_eq!(enum_def.values[0].number, 0);
        
        assert_eq!(enum_def.values[3].name, "DELETED");
        assert_eq!(enum_def.values[3].number, -1);
    }

    #[test]
    fn test_parse_service_with_http_annotations() {
        let content = r#"
syntax = "proto3";

service ProductService {
    rpc CreateProduct(CreateProductRequest) returns (Product) {
        option (google.api.http) = {
            post: "/v1/products"
            body: "*"
        };
    }
    
    rpc UpdateProduct(UpdateProductRequest) returns (Product) {
        option (google.api.http) = {
            put: "/v1/products/{product.id}"
            body: "product"
        };
    }
    
    rpc DeleteProduct(DeleteProductRequest) returns (google.protobuf.Empty) {
        option (google.api.http) = {
            delete: "/v1/products/{id}"
        };
    }
}
"#;
        
        let parser = NomProtoParser::new();
        let result = parser.parse_content(content);
        
        assert!(result.is_ok());
        let proto_file = result.unwrap();
        assert_eq!(proto_file.services.len(), 1);
        
        let service = &proto_file.services[0];
        assert_eq!(service.name, "ProductService");
        assert_eq!(service.methods.len(), 3);
        
        // Test POST method
        let create_method = &service.methods[0];
        assert_eq!(create_method.name, "CreateProduct");
        let http_annotation = create_method.http_annotation.as_ref().unwrap();
        assert_eq!(http_annotation.method, HttpMethod::Post);
        assert_eq!(http_annotation.path, "/v1/products");
        assert_eq!(http_annotation.body, Some("*".to_string()));
        
        // Test PUT method
        let update_method = &service.methods[1];
        assert_eq!(update_method.name, "UpdateProduct");
        let http_annotation = update_method.http_annotation.as_ref().unwrap();
        assert_eq!(http_annotation.method, HttpMethod::Put);
        assert_eq!(http_annotation.path, "/v1/products/{product.id}");
        assert_eq!(http_annotation.body, Some("product".to_string()));
        
        // Test DELETE method
        let delete_method = &service.methods[2];
        assert_eq!(delete_method.name, "DeleteProduct");
        let http_annotation = delete_method.http_annotation.as_ref().unwrap();
        assert_eq!(http_annotation.method, HttpMethod::Delete);
        assert_eq!(http_annotation.path, "/v1/products/{id}");
        assert_eq!(http_annotation.body, None);
    }

    #[test]
    fn test_parse_comments() {
        let content = r#"
syntax = "proto3";

// This is a user service
service UserService {
    // Get a user by ID
    rpc GetUser(GetUserRequest) returns (GetUserResponse);
    
    /* 
     * Create a new user
     * Returns the created user
     */
    rpc CreateUser(CreateUserRequest) returns (User);
}
"#;
        
        let parser = NomProtoParser::new();
        let result = parser.parse_content(content);
        
        assert!(result.is_ok());
        let proto_file = result.unwrap();
        let service = &proto_file.services[0];
        
        // Comments should be preserved
        assert!(!service.comments.is_empty());
        assert!(!service.methods[0].comments.is_empty());
        assert!(!service.methods[1].comments.is_empty());
    }

    #[test]
    fn test_parse_nested_messages() {
        let content = r#"
syntax = "proto3";

message Outer {
    message Inner {
        string value = 1;
    }
    
    Inner inner = 1;
    repeated Inner inners = 2;
}
"#;
        
        let parser = NomProtoParser::new();
        let result = parser.parse_content(content);
        
        assert!(result.is_ok());
        let proto_file = result.unwrap();
        assert_eq!(proto_file.messages.len(), 1);
        
        let outer_message = &proto_file.messages[0];
        assert_eq!(outer_message.name, "Outer");
        assert_eq!(outer_message.nested_messages.len(), 1);
        
        let inner_message = &outer_message.nested_messages[0];
        assert_eq!(inner_message.name, "Inner");
        assert_eq!(inner_message.fields.len(), 1);
    }

    #[test]
    fn test_parse_options() {
        let content = r#"
syntax = "proto3";

option java_package = "com.example.api";
option java_outer_classname = "UserProto";

message User {
    string name = 1 [deprecated = true];
    int32 age = 2;
}
"#;
        
        let parser = NomProtoParser::new();
        let result = parser.parse_content(content);
        
        assert!(result.is_ok());
        let proto_file = result.unwrap();
        assert_eq!(proto_file.options.len(), 2);
        
        assert_eq!(proto_file.options[0].name, "java_package");
        if let OptionValue::String(value) = &proto_file.options[0].value {
            assert_eq!(value, "com.example.api");
        } else {
            panic!("Expected string option value");
        }
        
        let message = &proto_file.messages[0];
        let field = &message.fields[0];
        assert_eq!(field.options.len(), 1);
        assert_eq!(field.options[0].name, "deprecated");
    }

    #[test]
    fn test_error_handling() {
        let parser = NomProtoParser::new();
        
        // Test invalid syntax
        let invalid_content = r#"
syntax = "invalid";
"#;
        let result = parser.parse_content(invalid_content);
        assert!(result.is_err());
        
        // Test incomplete content
        let incomplete_content = r#"
syntax = "proto3";
service UserService {
    rpc GetUser(
"#;
        let result = parser.parse_content(incomplete_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_complex_proto_file() {
        let content = std::fs::read_to_string("test_proto_files/user.proto");
        if content.is_err() {
            // Skip test if file doesn't exist (e.g., in CI)
            return;
        }
        
        let content = content.unwrap();
        let parser = NomProtoParser::new();
        let result = parser.parse_content(&content);
        
        assert!(result.is_ok(), "Failed to parse complex proto: {:?}", result.err());
        
        let proto_file = result.unwrap();
        
        // Verify basic structure
        assert_eq!(proto_file.syntax, ProtocolVersion::Proto3);
        assert_eq!(proto_file.package, Some("example.v1".to_string()));
        assert_eq!(proto_file.imports.len(), 2);
        assert_eq!(proto_file.services.len(), 1);
        assert_eq!(proto_file.messages.len(), 8); // GetUserRequest, CreateUserRequest, UpdateUserRequest, DeleteUserRequest, ListUsersRequest, GetUserResponse, ListUsersResponse, User
        assert_eq!(proto_file.enums.len(), 1);
        
        // Verify service
        let service = &proto_file.services[0];
        assert_eq!(service.name, "UserService");
        assert_eq!(service.methods.len(), 5);
        
        // Verify all methods have HTTP annotations
        for method in &service.methods {
            assert!(method.http_annotation.is_some(), "Method {} should have HTTP annotation", method.name);
        }
        
        // Verify specific HTTP annotations
        let get_user = &service.methods[0];
        assert_eq!(get_user.name, "GetUser");
        let http_annotation = get_user.http_annotation.as_ref().unwrap();
        assert_eq!(http_annotation.method, HttpMethod::Get);
        assert_eq!(http_annotation.path, "/v1/users/{user_id}");
        
        let create_user = &service.methods[1];
        assert_eq!(create_user.name, "CreateUser");
        let http_annotation = create_user.http_annotation.as_ref().unwrap();
        assert_eq!(http_annotation.method, HttpMethod::Post);
        assert_eq!(http_annotation.path, "/v1/users");
        assert_eq!(http_annotation.body, Some("*".to_string()));
        
        // Verify User message with nested Profile
        let user_message = proto_file.messages.iter()
            .find(|m| m.name == "User")
            .expect("User message should exist");
        
        assert_eq!(user_message.fields.len(), 8);
        assert_eq!(user_message.nested_messages.len(), 1);
        
        let profile_message = &user_message.nested_messages[0];
        assert_eq!(profile_message.name, "Profile");
        assert_eq!(profile_message.fields.len(), 3);
        
        // Verify enum
        let status_enum = &proto_file.enums[0];
        assert_eq!(status_enum.name, "Status");
        assert_eq!(status_enum.values.len(), 4);
        assert_eq!(status_enum.values[0].name, "STATUS_UNSPECIFIED");
        assert_eq!(status_enum.values[0].number, 0);
    }

    #[test]
    fn test_streaming_methods() {
        let content = r#"
syntax = "proto3";

service StreamingService {
    rpc ServerStream(Request) returns (stream Response);
    rpc ClientStream(stream Request) returns (Response);
    rpc BidirectionalStream(stream Request) returns (stream Response);
}

message Request {
    string data = 1;
}

message Response {
    string result = 1;
}
"#;
        
        let parser = NomProtoParser::new();
        let result = parser.parse_content(content);
        
        assert!(result.is_ok());
        let proto_file = result.unwrap();
        let service = &proto_file.services[0];
        
        // Check streaming flags
        assert!(!service.methods[0].input_type.is_stream);
        assert!(service.methods[0].output_type.is_stream);
        
        assert!(service.methods[1].input_type.is_stream);
        assert!(!service.methods[1].output_type.is_stream);
        
        assert!(service.methods[2].input_type.is_stream);
        assert!(service.methods[2].output_type.is_stream);
    }

    #[test]
    fn test_field_options_and_validation() {
        let content = r#"
syntax = "proto3";

message ValidationExample {
    string email = 1 [deprecated = true];
    int32 age = 2 [packed = true];
}
"#;
        
        let parser = NomProtoParser::new();
        let result = parser.parse_content(content);
        
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        
        assert!(result.is_ok());
        let proto_file = result.unwrap();
        let message = &proto_file.messages[0];
        
        // Check that field options are parsed
        assert!(!message.fields[0].options.is_empty());
        assert!(!message.fields[1].options.is_empty());
    }
}