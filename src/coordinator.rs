//! Main code generation coordinator
//!
//! This module provides the main orchestration logic that coordinates parsing,
//! validation, HTTP extraction, and code generation into a unified workflow.

use crate::core::*;
use crate::parser::NomProtoParser;
use crate::extractor::GoogleApiHttpExtractor;
use crate::generator::PoemOpenApiGenerator;
use crate::validation::ValidationEngine;
use crate::error_reporter::ErrorReporter;
use crate::plugins::PluginManager;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Main coordinator for the proto-http-parser-v2 library
/// 
/// This struct orchestrates the entire process of parsing Protocol Buffer files,
/// extracting HTTP annotations, validating the results, and generating code.
/// It provides both single-file and batch processing capabilities.
pub struct ProtoHttpCoordinator {
    config: ProtoHttpParserConfig,
    parser: NomProtoParser,
    extractor: GoogleApiHttpExtractor,
    generator: PoemOpenApiGenerator,
    validator: ValidationEngine,
    error_reporter: ErrorReporter,
    plugin_manager: PluginManager,
}

impl ProtoHttpCoordinator {
    /// Create a new coordinator with default configuration
    pub fn new() -> Self {
        let config = ProtoHttpParserConfig::default();
        Self::with_config(config)
    }
    
    /// Create a new coordinator with custom configuration
    pub fn with_config(config: ProtoHttpParserConfig) -> Self {
        let parser = NomProtoParser::with_config(config.parser.clone());
        let extractor = GoogleApiHttpExtractor::with_config(config.extractor.clone());
        let generator = PoemOpenApiGenerator::with_config(config.generator.clone());
        let validator = ValidationEngine::new();
        let error_reporter = ErrorReporter::new();
        let plugin_manager = PluginManager::new();
        
        Self {
            config,
            parser,
            extractor,
            generator,
            validator,
            error_reporter,
            plugin_manager,
        }
    }
    
    /// Process a single proto file and generate code
    pub fn process_file<P: AsRef<Path>>(&self, proto_path: P) -> Result<ProcessResult, ProtoHttpParserError> {
        let proto_path = proto_path.as_ref();
        
        // Step 1: Parse the proto file
        let proto_file = self.parser.parse_file(proto_path)
            .map_err(|e| ProtoHttpParserError::Parse(e))?;
        
        // Step 2: Validate the parsed proto file
        self.validator.validate_proto_file(&proto_file)
            .map_err(|e| ProtoHttpParserError::Validation(e))?;
        
        // Step 2.1: Run plugin validators
        let plugin_validation_errors = self.plugin_manager.validate_proto_file(&proto_file)
            .map_err(|e| ProtoHttpParserError::Plugin(e))?;
        
        if !plugin_validation_errors.is_empty() {
            return Err(ProtoHttpParserError::Validation(plugin_validation_errors[0].clone()));
        }
        
        // Step 3: Extract HTTP routes
        let routes = self.extractor.extract_routes(&proto_file)
            .map_err(|e| ProtoHttpParserError::Validation(ValidationError::HttpAnnotationError {
                message: format!("Failed to extract HTTP routes: {}", e),
            }))?;
        
        // Step 4: Validate HTTP annotations
        self.extractor.validate_annotations(&routes)
            .map_err(|e| ProtoHttpParserError::Validation(ValidationError::HttpAnnotationError {
                message: format!("HTTP annotation validation failed: {}", e),
            }))?;
        
        // Step 4.1: Run plugin HTTP validators
        let plugin_http_errors = self.plugin_manager.validate_http_routes(&routes)
            .map_err(|e| ProtoHttpParserError::Plugin(e))?;
        
        if !plugin_http_errors.is_empty() {
            return Err(ProtoHttpParserError::Validation(plugin_http_errors[0].clone()));
        }
        
        // Step 5: Generate code for each service
        let mut generated_files = HashMap::new();
        
        for service in &proto_file.services {
            // Filter routes for this service
            let service_routes: Vec<HttpRoute> = routes.iter()
                .filter(|route| route.service_name == service.name)
                .cloned()
                .collect();
            
            // Generate controller code
            let controller_code = self.generator.generate_controller(service, &service_routes)
                .map_err(|e| ProtoHttpParserError::CodeGeneration(e))?;
            
            let controller_filename = format!("{}_controller.rs", to_snake_case(&service.name));
            generated_files.insert(controller_filename, controller_code.clone());
            
            // Generate service trait if configured
            if self.config.generator.generate_service_traits {
                let service_trait_code = self.generator.generate_service_trait(service, &service_routes)
                    .map_err(|e| ProtoHttpParserError::CodeGeneration(e))?;
                
                let trait_filename = format!("{}_service.rs", to_snake_case(&service.name));
                generated_files.insert(trait_filename, service_trait_code);
            }
        }
        
        Ok(ProcessResult {
            proto_file,
            routes,
            generated_files,
        })
    }
    
    /// Process proto content from a string and generate code
    pub fn process_content(&self, content: &str) -> Result<ProcessResult, ProtoHttpParserError> {
        // Step 1: Parse the proto content
        let proto_file = self.parser.parse_content(content)
            .map_err(|e| ProtoHttpParserError::Parse(e))?;
        
        // Step 2: Validate the parsed proto file
        self.validator.validate_proto_file(&proto_file)
            .map_err(|e| ProtoHttpParserError::Validation(e))?;
        
        // Step 2.1: Run plugin validators
        let plugin_validation_errors = self.plugin_manager.validate_proto_file(&proto_file)
            .map_err(|e| ProtoHttpParserError::Plugin(e))?;
        
        if !plugin_validation_errors.is_empty() {
            return Err(ProtoHttpParserError::Validation(plugin_validation_errors[0].clone()));
        }
        
        // Step 3: Extract HTTP routes
        let routes = self.extractor.extract_routes(&proto_file)
            .map_err(|e| ProtoHttpParserError::Validation(ValidationError::HttpAnnotationError {
                message: format!("Failed to extract HTTP routes: {}", e),
            }))?;
        
        // Step 4: Validate HTTP annotations
        self.extractor.validate_annotations(&routes)
            .map_err(|e| ProtoHttpParserError::Validation(ValidationError::HttpAnnotationError {
                message: format!("HTTP annotation validation failed: {}", e),
            }))?;
        
        // Step 4.1: Run plugin HTTP validators
        let plugin_http_errors = self.plugin_manager.validate_http_routes(&routes)
            .map_err(|e| ProtoHttpParserError::Plugin(e))?;
        
        if !plugin_http_errors.is_empty() {
            return Err(ProtoHttpParserError::Validation(plugin_http_errors[0].clone()));
        }
        
        // Step 5: Generate code for each service
        let mut generated_files = HashMap::new();
        
        for service in &proto_file.services {
            // Filter routes for this service
            let service_routes: Vec<HttpRoute> = routes.iter()
                .filter(|route| route.service_name == service.name)
                .cloned()
                .collect();
            
            // Generate controller code
            let controller_code = self.generator.generate_controller(service, &service_routes)
                .map_err(|e| ProtoHttpParserError::CodeGeneration(e))?;
            
            let controller_filename = format!("{}_controller.rs", to_snake_case(&service.name));
            generated_files.insert(controller_filename, controller_code.clone());
            
            // Generate service trait if configured
            if self.config.generator.generate_service_traits {
                let service_trait_code = self.generator.generate_service_trait(service, &service_routes)
                    .map_err(|e| ProtoHttpParserError::CodeGeneration(e))?;
                
                let trait_filename = format!("{}_service.rs", to_snake_case(&service.name));
                generated_files.insert(trait_filename, service_trait_code);
            }
        }
        
        Ok(ProcessResult {
            proto_file,
            routes,
            generated_files,
        })
    }
    
    /// Process multiple proto files in batch
    pub fn process_files<P: AsRef<Path>>(&self, proto_paths: &[P]) -> Result<BatchProcessResult, ProtoHttpParserError> {
        let mut results = HashMap::new();
        let mut errors = Vec::new();
        
        for proto_path in proto_paths {
            let path = proto_path.as_ref();
            match self.process_file(path) {
                Ok(result) => {
                    results.insert(path.to_path_buf(), result);
                }
                Err(error) => {
                    errors.push((path.to_path_buf(), error));
                }
            }
        }
        
        Ok(BatchProcessResult {
            results,
            errors,
        })
    }
    
    /// Process all proto files in a directory
    pub fn process_directory<P: AsRef<Path>>(&self, dir_path: P) -> Result<BatchProcessResult, ProtoHttpParserError> {
        let dir_path = dir_path.as_ref();
        
        // Find all .proto files in the directory
        let proto_files = find_proto_files(dir_path)
            .map_err(|e| ProtoHttpParserError::Io(e))?;
        
        self.process_files(&proto_files)
    }
    
    /// Write generated code to files in the specified output directory
    pub fn write_generated_code<P: AsRef<Path>>(&self, result: &ProcessResult, output_dir: P) -> Result<(), ProtoHttpParserError> {
        let output_dir = output_dir.as_ref();
        
        // Create output directory if it doesn't exist
        std::fs::create_dir_all(output_dir)
            .map_err(|e| ProtoHttpParserError::Io(e))?;
        
        // Write each generated file
        for (filename, generated_code) in &result.generated_files {
            let file_path = output_dir.join(filename);
            
            // Format the code if rustfmt is enabled
            let content = if self.config.generator.formatting.use_rustfmt {
                format_rust_code(&generated_code.content)
                    .unwrap_or_else(|_| generated_code.content.clone())
            } else {
                generated_code.content.clone()
            };
            
            std::fs::write(&file_path, content)
                .map_err(|e| ProtoHttpParserError::Io(e))?;
        }
        
        Ok(())
    }
    
    /// Write batch results to files
    pub fn write_batch_results<P: AsRef<Path>>(&self, batch_result: &BatchProcessResult, output_dir: P) -> Result<(), ProtoHttpParserError> {
        let output_dir = output_dir.as_ref();
        
        for (proto_path, result) in &batch_result.results {
            // Create a subdirectory for each proto file
            let proto_name = proto_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            
            let service_output_dir = output_dir.join(proto_name);
            self.write_generated_code(result, service_output_dir)?;
        }
        
        Ok(())
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &ProtoHttpParserConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn update_config(&mut self, config: ProtoHttpParserConfig) {
        self.config = config.clone();
        self.parser = NomProtoParser::with_config(config.parser.clone());
        self.extractor = GoogleApiHttpExtractor::with_config(config.extractor.clone());
        self.generator = PoemOpenApiGenerator::with_config(config.generator.clone());
    }
    
    /// Get access to the plugin manager
    pub fn plugin_manager(&self) -> &PluginManager {
        &self.plugin_manager
    }
    
    /// Get mutable access to the plugin manager
    pub fn plugin_manager_mut(&mut self) -> &mut PluginManager {
        &mut self.plugin_manager
    }
    
    /// Load plugins from a configuration file
    pub fn load_plugins_from_config<P: AsRef<Path>>(&mut self, config_path: P) -> Result<(), ProtoHttpParserError> {
        self.plugin_manager.load_from_config(config_path)
            .map_err(|e| ProtoHttpParserError::Plugin(e))
    }
}

impl Default for ProtoHttpCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of processing a single proto file
#[derive(Debug, Clone)]
pub struct ProcessResult {
    /// The parsed proto file
    pub proto_file: ProtoFile,
    /// Extracted HTTP routes
    pub routes: Vec<HttpRoute>,
    /// Generated code files (filename -> generated code)
    pub generated_files: HashMap<String, GeneratedCode>,
}

/// Result of batch processing multiple proto files
#[derive(Debug)]
pub struct BatchProcessResult {
    /// Successful results (proto path -> process result)
    pub results: HashMap<PathBuf, ProcessResult>,
    /// Errors encountered (proto path -> error)
    pub errors: Vec<(PathBuf, ProtoHttpParserError)>,
}

impl BatchProcessResult {
    /// Check if all files were processed successfully
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
    
    /// Get the number of successfully processed files
    pub fn success_count(&self) -> usize {
        self.results.len()
    }
    
    /// Get the number of files that failed to process
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
    
    /// Get all generated files from successful results
    pub fn all_generated_files(&self) -> HashMap<String, GeneratedCode> {
        let mut all_files = HashMap::new();
        
        for result in self.results.values() {
            for (filename, code) in &result.generated_files {
                all_files.insert(filename.clone(), code.clone());
            }
        }
        
        all_files
    }
}

/// Build.rs integration API
pub mod build_integration {
    use super::*;
    use crate::core::config::ConfigBuilder;
    use crate::core::errors::ConfigError;
    
    /// Builder for build.rs integration with enhanced configuration support
    pub struct BuildIntegration {
        coordinator: ProtoHttpCoordinator,
        proto_files: Vec<PathBuf>,
        output_dir: PathBuf,
        config_sources: Vec<ConfigSource>,
        verbose: bool,
    }
    
    /// Configuration source for build integration
    pub enum ConfigSource {
        File(PathBuf),
        Environment,
        Builder(Box<dyn Fn(ConfigBuilder) -> ConfigBuilder + Send + Sync>),
    }
    
    impl std::fmt::Debug for ConfigSource {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ConfigSource::File(path) => f.debug_tuple("File").field(path).finish(),
                ConfigSource::Environment => f.debug_tuple("Environment").finish(),
                ConfigSource::Builder(_) => f.debug_tuple("Builder").field(&"<function>").finish(),
            }
        }
    }
    
    impl BuildIntegration {
        /// Create a new build integration with default configuration
        pub fn new() -> Self {
            Self {
                coordinator: ProtoHttpCoordinator::new(),
                proto_files: Vec::new(),
                output_dir: PathBuf::from("src/generated"),
                config_sources: Vec::new(),
                verbose: false,
            }
        }
        
        /// Create build integration with automatic configuration loading
        pub fn with_auto_config() -> Result<Self, Box<dyn std::error::Error>> {
            let config = ProtoHttpParserConfig::load()?;
            Ok(Self {
                coordinator: ProtoHttpCoordinator::with_config(config),
                proto_files: Vec::new(),
                output_dir: PathBuf::from("src/generated"),
                config_sources: Vec::new(),
                verbose: false,
            })
        }
        
        /// Set custom configuration
        pub fn with_config(mut self, config: ProtoHttpParserConfig) -> Self {
            self.coordinator = ProtoHttpCoordinator::with_config(config);
            self
        }
        
        /// Load configuration from a file
        pub fn with_config_file<P: AsRef<Path>>(mut self, path: P) -> Self {
            self.config_sources.push(ConfigSource::File(path.as_ref().to_path_buf()));
            self
        }
        
        /// Load configuration from environment variables
        pub fn with_env_config(mut self) -> Self {
            self.config_sources.push(ConfigSource::Environment);
            self
        }
        
        /// Configure using a builder function
        pub fn configure<F>(mut self, builder_fn: F) -> Self 
        where 
            F: Fn(ConfigBuilder) -> ConfigBuilder + Send + Sync + 'static,
        {
            self.config_sources.push(ConfigSource::Builder(Box::new(builder_fn)));
            self
        }
        
        /// Enable verbose output during build
        pub fn verbose(mut self, verbose: bool) -> Self {
            self.verbose = verbose;
            self
        }
        
        /// Add a proto file to process
        pub fn add_proto_file<P: AsRef<Path>>(mut self, path: P) -> Self {
            self.proto_files.push(path.as_ref().to_path_buf());
            self
        }
        
        /// Add multiple proto files
        pub fn add_proto_files<P: AsRef<Path>>(mut self, paths: &[P]) -> Self {
            for path in paths {
                self.proto_files.push(path.as_ref().to_path_buf());
            }
            self
        }
        
        /// Add all proto files from a directory
        pub fn add_proto_directory<P: AsRef<Path>>(mut self, dir: P) -> Result<Self, ProtoHttpParserError> {
            let proto_files = find_proto_files(dir.as_ref())
                .map_err(|e| ProtoHttpParserError::Io(e))?;
            
            for file in proto_files {
                self.proto_files.push(file);
            }
            
            Ok(self)
        }
        
        /// Add proto files matching a glob pattern
        pub fn add_proto_glob<S: AsRef<str>>(mut self, pattern: S) -> Result<Self, Box<dyn std::error::Error>> {
            // Simple glob implementation - in a real implementation you'd use the glob crate
            let pattern = pattern.as_ref();
            if pattern.ends_with("*.proto") {
                let dir = pattern.trim_end_matches("*.proto");
                let dir_path = if dir.is_empty() { "." } else { dir };
                return self.add_proto_directory(dir_path).map_err(|e| e.into());
            }
            
            // For now, treat as a single file
            self.proto_files.push(PathBuf::from(pattern));
            Ok(self)
        }
        
        /// Set the output directory for generated code
        pub fn output_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
            self.output_dir = dir.as_ref().to_path_buf();
            self
        }
        
        /// Set output directory relative to OUT_DIR environment variable
        pub fn output_dir_from_env(mut self, subdir: &str) -> Self {
            if let Ok(out_dir) = std::env::var("OUT_DIR") {
                self.output_dir = PathBuf::from(out_dir).join(subdir);
            }
            self
        }
        
        /// Generate code for all configured proto files
        pub fn generate(mut self) -> Result<BuildResult, Box<dyn std::error::Error>> {
            // Apply configuration sources in order
            let mut config = self.coordinator.config().clone();
            
            for source in &self.config_sources {
                match source {
                    ConfigSource::File(path) => {
                        if self.verbose {
                            println!("cargo:warning=Loading config from {}", path.display());
                        }
                        config = config.merge_from_file(path)?;
                    }
                    ConfigSource::Environment => {
                        if self.verbose {
                            println!("cargo:warning=Loading config from environment variables");
                        }
                        config = config.merge_from_env()?;
                    }
                    ConfigSource::Builder(builder_fn) => {
                        if self.verbose {
                            println!("cargo:warning=Applying configuration builder");
                        }
                        let builder = ConfigBuilder::new();
                        config = builder_fn(builder).build_unchecked();
                    }
                }
            }
            
            // Update coordinator with final configuration
            self.coordinator.update_config(config);
            
            if self.verbose {
                println!("cargo:warning=Processing {} proto files", self.proto_files.len());
                for file in &self.proto_files {
                    println!("cargo:warning=  - {}", file.display());
                }
            }
            
            // Process all proto files
            let batch_result = self.coordinator.process_files(&self.proto_files)?;
            
            // Check for errors
            if !batch_result.is_success() {
                let error_messages: Vec<String> = batch_result.errors.iter()
                    .map(|(path, error)| format!("{}: {}", path.display(), error))
                    .collect();
                
                return Err(format!("Failed to process {} proto files:\n{}", 
                    batch_result.error_count(),
                    error_messages.join("\n")
                ).into());
            }
            
            // Write generated code to a flat directory structure for build integration
            let generated_files = self.write_flat_batch_results(&batch_result)?;
            
            // Print cargo rerun-if-changed directives
            for proto_file in &self.proto_files {
                println!("cargo:rerun-if-changed={}", proto_file.display());
            }
            
            // Also watch config files
            for source in &self.config_sources {
                if let ConfigSource::File(path) = source {
                    println!("cargo:rerun-if-changed={}", path.display());
                }
            }
            
            if self.verbose {
                println!("cargo:warning=Generated {} files in {}", 
                    generated_files.len(), 
                    self.output_dir.display()
                );
            }
            
            Ok(BuildResult {
                generated_files,
                output_dir: self.output_dir,
                processed_files: self.proto_files,
            })
        }
        
        /// Write batch results to a flat directory structure (all files in the same directory)
        fn write_flat_batch_results(&self, batch_result: &BatchProcessResult) -> Result<Vec<PathBuf>, ProtoHttpParserError> {
            // Create output directory if it doesn't exist
            std::fs::create_dir_all(&self.output_dir)
                .map_err(|e| ProtoHttpParserError::Io(e))?;
            
            let mut generated_files = Vec::new();
            
            // Write all generated files to the same directory
            for result in batch_result.results.values() {
                for (filename, generated_code) in &result.generated_files {
                    let file_path = self.output_dir.join(filename);
                    
                    // Format the code if rustfmt is enabled
                    let content = if self.coordinator.config().generator.formatting.use_rustfmt {
                        format_rust_code(&generated_code.content)
                            .unwrap_or_else(|_| generated_code.content.clone())
                    } else {
                        generated_code.content.clone()
                    };
                    
                    std::fs::write(&file_path, content)
                        .map_err(|e| ProtoHttpParserError::Io(e))?;
                    
                    generated_files.push(file_path);
                }
            }
            
            Ok(generated_files)
        }
    }
    
    impl Default for BuildIntegration {
        fn default() -> Self {
            Self::new()
        }
    }
    
    /// Result of build integration
    #[derive(Debug)]
    pub struct BuildResult {
        /// List of generated files
        pub generated_files: Vec<PathBuf>,
        /// Output directory where files were written
        pub output_dir: PathBuf,
        /// List of proto files that were processed
        pub processed_files: Vec<PathBuf>,
    }
    
    impl BuildResult {
        /// Get the number of generated files
        pub fn file_count(&self) -> usize {
            self.generated_files.len()
        }
        
        /// Check if any files were generated
        pub fn has_files(&self) -> bool {
            !self.generated_files.is_empty()
        }
        
        /// Get generated files with a specific extension
        pub fn files_with_extension(&self, ext: &str) -> Vec<&PathBuf> {
            self.generated_files.iter()
                .filter(|path| path.extension().and_then(|s| s.to_str()) == Some(ext))
                .collect()
        }
    }
}

// Utility functions

/// Convert a string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    
    result
}

/// Find all .proto files in a directory recursively
fn find_proto_files<P: AsRef<Path>>(dir: P) -> std::io::Result<Vec<PathBuf>> {
    let mut proto_files = Vec::new();
    
    fn visit_dir(dir: &Path, proto_files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                visit_dir(&path, proto_files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("proto") {
                proto_files.push(path);
            }
        }
        Ok(())
    }
    
    visit_dir(dir.as_ref(), &mut proto_files)?;
    Ok(proto_files)
}

/// Format Rust code using rustfmt
fn format_rust_code(code: &str) -> Result<String, std::io::Error> {
    use std::process::{Command, Stdio};
    use std::io::Write;
    
    let mut child = Command::new("rustfmt")
        .arg("--emit=stdout")
        .arg("--quiet")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(code.as_bytes())?;
    }
    
    let output = child.wait_with_output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("rustfmt failed: {}", String::from_utf8_lossy(&output.stderr))
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("UserService"), "user_service");
        assert_eq!(to_snake_case("HTTPClient"), "h_t_t_p_client");
        assert_eq!(to_snake_case("XMLParser"), "x_m_l_parser");
        assert_eq!(to_snake_case("simple"), "simple");
        assert_eq!(to_snake_case(""), "");
    }
    
    #[test]
    fn test_coordinator_creation() {
        let coordinator = ProtoHttpCoordinator::new();
        assert!(coordinator.config().parser.preserve_comments);
        assert!(coordinator.config().generator.generate_service_traits);
    }
    
    #[test]
    fn test_build_integration_builder() {
        let _integration = build_integration::BuildIntegration::new()
            .add_proto_file("test.proto")
            .output_dir("generated");
        
        // Just test that the builder pattern works without errors
        // The actual functionality is tested in integration tests
    }
}