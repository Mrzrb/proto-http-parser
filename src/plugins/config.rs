//! Plugin configuration management
//!
//! This module provides utilities for loading and managing plugin configurations
//! from various sources including files, environment variables, and code.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Plugin configuration loader
#[derive(Debug, Clone)]
pub struct PluginConfigLoader {
    /// Default configuration directory
    config_dir: Option<std::path::PathBuf>,
    /// Environment variable prefix for plugin configs
    env_prefix: String,
}

impl PluginConfigLoader {
    /// Create a new plugin configuration loader
    pub fn new() -> Self {
        Self {
            config_dir: None,
            env_prefix: "PROTO_HTTP_PARSER_PLUGIN".to_string(),
        }
    }
    
    /// Set the configuration directory
    pub fn with_config_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.config_dir = Some(dir.as_ref().to_path_buf());
        self
    }
    
    /// Set the environment variable prefix
    pub fn with_env_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.env_prefix = prefix.into();
        self
    }
    
    /// Load plugin configurations from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<HashMap<String, PluginConfig>, PluginError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| PluginError::LoadingError {
                message: format!("Failed to read plugin config file: {}", e),
            })?;
        
        let configs: HashMap<String, PluginConfig> = serde_json::from_str(&content)
            .map_err(|e| PluginError::ConfigurationError {
                message: format!("Failed to parse plugin config: {}", e),
            })?;
        
        Ok(configs)
    }
    
    /// Load plugin configurations from environment variables
    pub fn load_from_env(&self) -> Result<HashMap<String, PluginConfig>, PluginError> {
        let mut configs = HashMap::new();
        
        for (key, value) in std::env::vars() {
            if key.starts_with(&self.env_prefix) {
                // Parse environment variable name: PROTO_HTTP_PARSER_PLUGIN_<PLUGIN_NAME>_<SETTING>
                let parts: Vec<&str> = key.split('_').collect();
                if parts.len() >= 5 {
                    let plugin_name = parts[4].to_lowercase();
                    let setting_name = parts[5..].join("_").to_lowercase();
                    
                    let config = configs.entry(plugin_name).or_insert_with(PluginConfig::default);
                    
                    // Try to parse as JSON value, fallback to string
                    let json_value = serde_json::from_str(&value)
                        .unwrap_or_else(|_| serde_json::Value::String(value));
                    
                    config.settings.insert(setting_name, json_value);
                }
            }
        }
        
        Ok(configs)
    }
    
    /// Load plugin configurations from multiple sources with precedence
    /// Order: file -> environment -> defaults
    pub fn load_merged<P: AsRef<Path>>(&self, file_path: Option<P>) -> Result<HashMap<String, PluginConfig>, PluginError> {
        let mut merged_configs = HashMap::new();
        
        // Start with file configuration if provided
        if let Some(path) = file_path {
            let file_configs = self.load_from_file(path)?;
            merged_configs.extend(file_configs);
        }
        
        // Override with environment variables
        let env_configs = self.load_from_env()?;
        for (plugin_name, env_config) in env_configs {
            let config = merged_configs.entry(plugin_name).or_insert_with(PluginConfig::default);
            
            // Merge settings
            config.settings.extend(env_config.settings);
            
            // Override boolean flags
            if env_config.enabled != PluginConfig::default().enabled {
                config.enabled = env_config.enabled;
            }
            if env_config.priority != PluginConfig::default().priority {
                config.priority = env_config.priority;
            }
        }
        
        Ok(merged_configs)
    }
    
    /// Create a default configuration for a plugin
    pub fn create_default_config(&self, plugin_name: &str) -> PluginConfig {
        let mut config = PluginConfig::default();
        
        // Add some common default settings based on plugin name
        match plugin_name {
            "naming_convention_validator" => {
                config.settings.insert("service_pattern".to_string(), 
                    serde_json::Value::String("^[A-Z][a-zA-Z0-9]*Service$".to_string()));
                config.settings.insert("method_pattern".to_string(), 
                    serde_json::Value::String("^[A-Z][a-zA-Z0-9]*$".to_string()));
                config.settings.insert("message_pattern".to_string(), 
                    serde_json::Value::String("^[A-Z][a-zA-Z0-9]*$".to_string()));
            }
            "rest_api_validator" => {
                config.settings.insert("require_resource_paths".to_string(), 
                    serde_json::Value::Bool(true));
                config.settings.insert("allow_nested_resources".to_string(), 
                    serde_json::Value::Bool(true));
                config.settings.insert("max_path_depth".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(4)));
            }
            "custom_code_formatter" => {
                config.settings.insert("indent_size".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(4)));
                config.settings.insert("use_tabs".to_string(), 
                    serde_json::Value::Bool(false));
                config.settings.insert("max_line_length".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(100)));
            }
            "documentation_generator" => {
                config.settings.insert("include_examples".to_string(), 
                    serde_json::Value::Bool(true));
                config.settings.insert("output_format".to_string(), 
                    serde_json::Value::String("markdown".to_string()));
            }
            _ => {
                // No specific defaults for unknown plugins
            }
        }
        
        config
    }
    
    /// Save plugin configurations to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, configs: &HashMap<String, PluginConfig>, path: P) -> Result<(), PluginError> {
        let json = serde_json::to_string_pretty(configs)
            .map_err(|e| PluginError::ConfigurationError {
                message: format!("Failed to serialize plugin configs: {}", e),
            })?;
        
        std::fs::write(path.as_ref(), json)
            .map_err(|e| PluginError::LoadingError {
                message: format!("Failed to write plugin config file: {}", e),
            })?;
        
        Ok(())
    }
}

impl Default for PluginConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin configuration builder for programmatic configuration
#[derive(Debug, Clone)]
pub struct PluginConfigBuilder {
    config: PluginConfig,
}

impl PluginConfigBuilder {
    /// Create a new plugin configuration builder
    pub fn new() -> Self {
        Self {
            config: PluginConfig::default(),
        }
    }
    
    /// Set whether the plugin is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }
    
    /// Set the plugin priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.config.priority = priority;
        self
    }
    
    /// Add a string setting
    pub fn setting<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.config.settings.insert(key.into(), serde_json::Value::String(value.into()));
        self
    }
    
    /// Add a boolean setting
    pub fn bool_setting<K: Into<String>>(mut self, key: K, value: bool) -> Self {
        self.config.settings.insert(key.into(), serde_json::Value::Bool(value));
        self
    }
    
    /// Add a number setting
    pub fn number_setting<K: Into<String>>(mut self, key: K, value: i64) -> Self {
        self.config.settings.insert(key.into(), serde_json::Value::Number(serde_json::Number::from(value)));
        self
    }
    
    /// Add a JSON value setting
    pub fn json_setting<K: Into<String>>(mut self, key: K, value: serde_json::Value) -> Self {
        self.config.settings.insert(key.into(), value);
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> PluginConfig {
        self.config
    }
}

impl Default for PluginConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Example plugin configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigFile {
    /// Plugin configurations
    pub plugins: HashMap<String, PluginConfig>,
    /// Global plugin settings
    pub global: GlobalPluginSettings,
}

/// Global plugin settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPluginSettings {
    /// Whether plugins are enabled globally
    pub enabled: bool,
    /// Default plugin priority
    pub default_priority: i32,
    /// Plugin loading timeout in seconds
    pub loading_timeout: u64,
    /// Maximum number of plugins to load
    pub max_plugins: usize,
}

impl Default for GlobalPluginSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            default_priority: 0,
            loading_timeout: 30,
            max_plugins: 50,
        }
    }
}

impl Default for PluginConfigFile {
    fn default() -> Self {
        Self {
            plugins: HashMap::new(),
            global: GlobalPluginSettings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
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
        assert_eq!(config.settings.get("test_key"), Some(&serde_json::Value::String("test_value".to_string())));
        assert_eq!(config.settings.get("test_bool"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(config.settings.get("test_number"), Some(&serde_json::Value::Number(serde_json::Number::from(42))));
    }
    
    #[test]
    fn test_plugin_config_loader_file() {
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
        assert_eq!(test_config.settings.get("test_setting"), Some(&serde_json::Value::String("test_value".to_string())));
    }
    
    #[test]
    fn test_default_config_creation() {
        let loader = PluginConfigLoader::new();
        
        let naming_config = loader.create_default_config("naming_convention_validator");
        assert!(naming_config.settings.contains_key("service_pattern"));
        
        let rest_config = loader.create_default_config("rest_api_validator");
        assert!(rest_config.settings.contains_key("require_resource_paths"));
        
        let formatter_config = loader.create_default_config("custom_code_formatter");
        assert!(formatter_config.settings.contains_key("indent_size"));
    }
}