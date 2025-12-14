//! Example demonstrating the plugin system
//!
//! This example shows how to use the plugin system to extend the functionality
//! of the proto-http-parser-v2 library with custom validators and formatters.

use proto_http_parser::*;
use proto_http_parser::plugins::examples::*;
use proto_http_parser::plugins::config::*;
use std::collections::HashMap;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Proto HTTP Parser v2 - Plugin System Example");
    println!("============================================");
    
    // Create a coordinator with plugin support
    let mut coordinator = ProtoHttpCoordinator::new();
    
    // Example 1: Register a naming convention validator plugin
    println!("\n1. Registering naming convention validator plugin...");
    
    let naming_validator = NamingConventionValidator::new();
    let naming_config = PluginConfigBuilder::new()
        .enabled(true)
        .priority(10)
        .setting("service_pattern", "^[A-Z][a-zA-Z0-9]*Service$")
        .setting("method_pattern", "^[A-Z][a-zA-Z0-9]*$")
        .build();
    
    coordinator.plugin_manager_mut()
        .register_proto_validator(naming_validator, naming_config)?;
    
    println!("✓ Naming convention validator registered");
    
    // Example 2: Register a REST API validator plugin
    println!("\n2. Registering REST API validator plugin...");
    
    let rest_validator = RestApiValidator::new();
    let rest_config = PluginConfigBuilder::new()
        .enabled(true)
        .priority(5)
        .bool_setting("require_resource_paths", true)
        .bool_setting("allow_nested_resources", false)
        .number_setting("max_path_depth", 3)
        .build();
    
    coordinator.plugin_manager_mut()
        .register_http_validator(rest_validator, rest_config)?;
    
    println!("✓ REST API validator registered");
    
    // Example 3: Register a custom code formatter plugin
    println!("\n3. Registering custom code formatter plugin...");
    
    let formatter = CustomCodeFormatter::new();
    let formatter_config = PluginConfigBuilder::new()
        .enabled(true)
        .priority(1)
        .number_setting("indent_size", 2)
        .bool_setting("use_tabs", false)
        .number_setting("max_line_length", 120)
        .build();
    
    coordinator.plugin_manager_mut()
        .register_code_formatter(formatter, formatter_config)?;
    
    println!("✓ Custom code formatter registered");
    
    // Example 4: Register a documentation generator plugin
    println!("\n4. Registering documentation generator plugin...");
    
    let doc_generator = DocumentationGenerator::new();
    let doc_config = PluginConfigBuilder::new()
        .enabled(true)
        .priority(0)
        .bool_setting("include_examples", true)
        .setting("output_format", "markdown")
        .build();
    
    coordinator.plugin_manager_mut()
        .register_code_generator(doc_generator, doc_config)?;
    
    println!("✓ Documentation generator registered");
    
    // Example 5: List all registered plugins
    println!("\n5. Listing all registered plugins:");
    
    let plugin_info = coordinator.plugin_manager().list_plugins();
    for info in &plugin_info {
        println!("  - {} v{}: {} (enabled: {}, priority: {})", 
            info.name, info.version, info.description, info.enabled, info.priority);
        println!("    Capabilities: {:?}", info.capabilities);
    }
    
    // Example 6: Demonstrate plugin configuration from file
    println!("\n6. Creating example plugin configuration file...");
    
    let config_loader = PluginConfigLoader::new();
    let mut example_configs = HashMap::new();
    
    // Add example configurations
    example_configs.insert("naming_convention_validator".to_string(), 
        config_loader.create_default_config("naming_convention_validator"));
    example_configs.insert("rest_api_validator".to_string(), 
        config_loader.create_default_config("rest_api_validator"));
    example_configs.insert("custom_code_formatter".to_string(), 
        config_loader.create_default_config("custom_code_formatter"));
    
    // Save to a temporary file for demonstration
    let temp_config_path = "example_plugin_config.json";
    config_loader.save_to_file(&example_configs, temp_config_path)?;
    
    println!("✓ Example configuration saved to: {}", temp_config_path);
    
    // Load the configuration back
    let loaded_configs = config_loader.load_from_file(temp_config_path)?;
    println!("✓ Configuration loaded successfully with {} plugins", loaded_configs.len());
    
    // Example 7: Demonstrate plugin usage with a mock proto file
    println!("\n7. Testing plugins with mock data...");
    
    // Create a mock proto file for testing
    let mock_proto_file = create_mock_proto_file();
    
    // Test proto validation plugins
    match coordinator.plugin_manager().validate_proto_file(&mock_proto_file) {
        Ok(errors) => {
            if errors.is_empty() {
                println!("✓ Proto validation passed");
            } else {
                println!("⚠ Proto validation found {} issues:", errors.len());
                for error in &errors {
                    println!("  - {}", error);
                }
            }
        }
        Err(e) => {
            println!("✗ Proto validation failed: {}", e);
        }
    }
    
    // Create mock HTTP routes for testing
    let mock_routes = create_mock_http_routes();
    
    // Test HTTP validation plugins
    match coordinator.plugin_manager().validate_http_routes(&mock_routes) {
        Ok(errors) => {
            if errors.is_empty() {
                println!("✓ HTTP validation passed");
            } else {
                println!("⚠ HTTP validation found {} issues:", errors.len());
                for error in &errors {
                    println!("  - {}", error);
                }
            }
        }
        Err(e) => {
            println!("✗ HTTP validation failed: {}", e);
        }
    }
    
    // Test code generation plugins
    if let Some(service) = mock_proto_file.services.first() {
        match coordinator.plugin_manager().generate_code(service, &mock_routes) {
            Ok(generated_files) => {
                println!("✓ Code generation produced {} files:", generated_files.len());
                for (filename, _) in &generated_files {
                    println!("  - {}", filename);
                }
            }
            Err(e) => {
                println!("✗ Code generation failed: {}", e);
            }
        }
    }
    
    // Test code formatting
    let sample_code = r#"
pub struct UserController {
pub service: Arc<dyn UserService>,
}
impl UserController {
pub fn new(service: Arc<dyn UserService>) -> Self {
Self { service }
}
}
"#;
    
    match coordinator.plugin_manager().format_code(sample_code, "rust") {
        Ok(formatted) => {
            println!("✓ Code formatting successful");
            println!("Formatted code:\n{}", formatted);
        }
        Err(e) => {
            println!("✗ Code formatting failed: {}", e);
        }
    }
    
    // Clean up
    std::fs::remove_file(temp_config_path).ok();
    
    println!("\n✓ Plugin system example completed successfully!");
    
    Ok(())
}

/// Create a mock proto file for testing
fn create_mock_proto_file() -> ProtoFile {
    use proto_http_parser::core::*;
    
    ProtoFile {
        syntax: ProtocolVersion::Proto3,
        package: Some("example.v1".to_string()),
        imports: vec![],
        options: vec![],
        services: vec![
            Service {
                name: "UserService".to_string(), // Good naming convention
                methods: vec![
                    RpcMethod {
                        name: "GetUser".to_string(), // Good naming convention
                        input_type: TypeReference {
                            name: "GetUserRequest".to_string(),
                            package: None,
                            is_stream: false,
                        },
                        output_type: TypeReference {
                            name: "User".to_string(),
                            package: None,
                            is_stream: false,
                        },
                        options: vec![],
                        comments: vec![],
                        http_annotation: Some(HttpAnnotation {
                            method: HttpMethod::Get,
                            path: "/users/{id}".to_string(),
                            body: None,
                            additional_bindings: vec![],
                        }),
                    },
                    RpcMethod {
                        name: "badMethodName".to_string(), // Bad naming convention
                        input_type: TypeReference {
                            name: "CreateUserRequest".to_string(),
                            package: None,
                            is_stream: false,
                        },
                        output_type: TypeReference {
                            name: "User".to_string(),
                            package: None,
                            is_stream: false,
                        },
                        options: vec![],
                        comments: vec![],
                        http_annotation: Some(HttpAnnotation {
                            method: HttpMethod::Post,
                            path: "/users".to_string(),
                            body: Some("*".to_string()),
                            additional_bindings: vec![],
                        }),
                    },
                ],
                options: vec![],
                comments: vec![],
            },
        ],
        messages: vec![
            Message {
                name: "User".to_string(), // Good naming convention
                fields: vec![],
                nested_messages: vec![],
                nested_enums: vec![],
                options: vec![],
                comments: vec![],
            },
            Message {
                name: "badMessageName".to_string(), // Bad naming convention
                fields: vec![],
                nested_messages: vec![],
                nested_enums: vec![],
                options: vec![],
                comments: vec![],
            },
        ],
        enums: vec![],
    }
}

/// Create mock HTTP routes for testing
fn create_mock_http_routes() -> Vec<HttpRoute> {
    use proto_http_parser::core::*;
    
    vec![
        HttpRoute {
            service_name: "UserService".to_string(),
            method_name: "GetUser".to_string(),
            http_method: HttpMethod::Get,
            path_template: "/users/{id}".to_string(), // Good REST pattern
            path_parameters: vec![
                PathParameter {
                    name: "id".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                },
            ],
            query_parameters: vec![],
            request_body: None,
            input_type: TypeReference {
                name: "GetUserRequest".to_string(),
                package: None,
                is_stream: false,
            },
            response_type: TypeReference {
                name: "User".to_string(),
                package: None,
                is_stream: false,
            },
        },
        HttpRoute {
            service_name: "UserService".to_string(),
            method_name: "badMethodName".to_string(),
            http_method: HttpMethod::Get,
            path_template: "/very/deeply/nested/resource/path/that/exceeds/limits".to_string(), // Bad: too deep
            path_parameters: vec![],
            query_parameters: vec![],
            request_body: None,
            input_type: TypeReference {
                name: "Empty".to_string(),
                package: None,
                is_stream: false,
            },
            response_type: TypeReference {
                name: "User".to_string(),
                package: None,
                is_stream: false,
            },
        },
    ]
}