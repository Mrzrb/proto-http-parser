//! Tests for the plugin system

use proto_http_parser_v2::*;
use proto_http_parser_v2::plugins::*;
use proto_http_parser_v2::plugins::examples::*;
use proto_http_parser_v2::plugins::config::*;

#[test]
fn test_plugin_manager_creation() {
    let manager = PluginManager::new();
    assert_eq!(manager.plugins().len(), 0);
}

#[test]
fn test_plugin_registration() {
    let mut manager = PluginManager::new();
    
    let validator = NamingConventionValidator::new();
    let config = PluginConfig::default();
    
    let result = manager.register_proto_validator(validator, config);
    assert!(result.is_ok());
    
    assert_eq!(manager.plugins().len(), 1);
    assert!(manager.get_plugin("naming_convention_validator").is_some());
}

#[test]
fn test_plugin_config_builder() {
    let config = PluginConfigBuilder::new()
        .enabled(true)
        .priority(10)
        .setting("test_key", "test_value")
        .bool_setting("test_bool", true)
        .number_setting("test_number", 42)
        .build();
    
    assert!(config.enabled);
    assert_eq!(config.priority, 10);
    assert_eq!(config.settings.len(), 3);
}

#[test]
fn test_naming_convention_validator() {
    let validator = NamingConventionValidator::new();
    
    // Test plugin metadata
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

#[test]
fn test_plugin_config_loader_defaults() {
    let loader = PluginConfigLoader::new();
    
    let naming_config = loader.create_default_config("naming_convention_validator");
    assert!(naming_config.settings.contains_key("service_pattern"));
    
    let rest_config = loader.create_default_config("rest_api_validator");
    assert!(rest_config.settings.contains_key("require_resource_paths"));
    
    let formatter_config = loader.create_default_config("custom_code_formatter");
    assert!(formatter_config.settings.contains_key("indent_size"));
    
    let doc_config = loader.create_default_config("documentation_generator");
    assert!(doc_config.settings.contains_key("include_examples"));
}

#[test]
fn test_coordinator_plugin_integration() {
    let mut coordinator = ProtoHttpCoordinator::new();
    
    // Test that plugin manager is accessible
    assert_eq!(coordinator.plugin_manager().plugins().len(), 0);
    
    // Register a plugin
    let validator = NamingConventionValidator::new();
    let config = PluginConfig::default();
    
    let result = coordinator.plugin_manager_mut()
        .register_proto_validator(validator, config);
    assert!(result.is_ok());
    
    // Verify plugin is registered
    assert_eq!(coordinator.plugin_manager().plugins().len(), 1);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_plugin_config_file_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"
        {
            "test_plugin": {
                "enabled": true,
                "priority": 5,
                "settings": {
                    "test_setting": "test_value"
                }
            }
        }
        "#;
        
        temp_file.write_all(config_json.as_bytes()).unwrap();
        
        let loader = PluginConfigLoader::new();
        let configs = loader.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(configs.len(), 1);
        let test_config = configs.get("test_plugin").unwrap();
        assert!(test_config.enabled);
        assert_eq!(test_config.priority, 5);
    }
    
    #[test]
    fn test_plugin_config_save_and_load() {
        let loader = PluginConfigLoader::new();
        let mut configs = std::collections::HashMap::new();
        
        configs.insert("test_plugin".to_string(), PluginConfigBuilder::new()
            .enabled(true)
            .priority(10)
            .setting("test_key", "test_value")
            .build());
        
        let temp_file = NamedTempFile::new().unwrap();
        loader.save_to_file(&configs, temp_file.path()).unwrap();
        
        let loaded_configs = loader.load_from_file(temp_file.path()).unwrap();
        assert_eq!(loaded_configs.len(), 1);
        
        let loaded_config = loaded_configs.get("test_plugin").unwrap();
        assert!(loaded_config.enabled);
        assert_eq!(loaded_config.priority, 10);
    }
}