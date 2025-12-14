//! Sample build.rs file demonstrating proto-http-parser-v2 integration
//! 
//! This file shows various ways to integrate proto-http-parser-v2 into your build process.
//! Copy the relevant sections to your own build.rs file.

use proto_http_parser_v2::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Choose one of the following integration patterns:
    
    // Pattern 1: Simple integration with default configuration
    simple_integration()?;
    
    // Pattern 2: Integration with configuration file
    // config_file_integration()?;
    
    // Pattern 3: Integration with environment variables
    // env_integration()?;
    
    // Pattern 4: Integration with custom configuration
    // custom_config_integration()?;
    
    // Pattern 5: Auto-configuration integration
    // auto_config_integration()?;
    
    Ok(())
}

/// Pattern 1: Simple integration with default configuration
fn simple_integration() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/");
    
    BuildIntegration::new()
        .add_proto_file("proto/user.proto")
        .add_proto_file("proto/product.proto")
        .output_dir("src/generated")
        .generate()?;
    
    Ok(())
}

/// Pattern 2: Integration with configuration file
#[allow(dead_code)]
fn config_file_integration() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/");
    println!("cargo:rerun-if-changed=proto-http-parser.toml");
    
    BuildIntegration::new()
        .with_config_file("proto-http-parser.toml")
        .add_proto_directory("proto")?
        .output_dir("src/generated")
        .verbose(true)
        .generate()?;
    
    Ok(())
}

/// Pattern 3: Integration with environment variables
#[allow(dead_code)]
fn env_integration() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/");
    
    // Environment variables can be set in .cargo/config.toml:
    // [env]
    // PROTO_HTTP_PARSER_GENERATE_SERVICE_TRAITS = "true"
    // PROTO_HTTP_PARSER_USE_DEPENDENCY_INJECTION = "true"
    // PROTO_HTTP_PARSER_INFER_QUERY_PARAMS = "true"
    
    BuildIntegration::new()
        .with_env_config()
        .add_proto_glob("proto/*.proto")?
        .output_dir_from_env("generated")  // Uses OUT_DIR/generated
        .verbose(std::env::var("VERBOSE").is_ok())
        .generate()?;
    
    Ok(())
}

/// Pattern 4: Integration with custom configuration
#[allow(dead_code)]
fn custom_config_integration() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/");
    
    BuildIntegration::new()
        .configure(|builder| {
            builder
                .preserve_comments(cfg!(debug_assertions))
                .strict_validation(true)
                .max_import_depth(10)
                .add_include_path("proto")
                .add_include_path("third_party/googleapis")
                .generate_service_traits(true)
                .use_dependency_injection(true)
                .infer_query_params(true)
                .use_rustfmt(!cfg!(feature = "fast-build"))
                .indent_size(4)
                .max_line_length(100)
                // Add common type mappings
                .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
                .add_type_mapping("google.protobuf.Duration", "std::time::Duration")
                .add_type_mapping("google.protobuf.Empty", "()")
                // Add common imports
                .add_import("use chrono::{DateTime, Utc};")
                .add_import("use serde::{Deserialize, Serialize};")
        })
        .add_proto_directory("proto")?
        .output_dir("src/generated")
        .verbose(std::env::var("CARGO_FEATURE_VERBOSE").is_ok())
        .generate()?;
    
    Ok(())
}

/// Pattern 5: Auto-configuration integration
#[allow(dead_code)]
fn auto_config_integration() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/");
    println!("cargo:rerun-if-changed=proto-http-parser.toml");
    println!("cargo:rerun-if-changed=.proto-http-parser.toml");
    
    // This will automatically load configuration from:
    // 1. Environment variables
    // 2. proto-http-parser.toml (if exists)
    // 3. .proto-http-parser.toml (if exists)
    // 4. config/proto-http-parser.toml (if exists)
    
    BuildIntegration::with_auto_config()?
        .add_proto_directory("proto")?
        .output_dir("src/generated")
        .verbose(true)
        .generate()?;
    
    Ok(())
}

/// Advanced pattern: Conditional compilation based on features
#[allow(dead_code)]
fn feature_based_integration() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/");
    
    let mut integration = BuildIntegration::new();
    
    // Configure based on Cargo features
    integration = integration.configure(|builder| {
        let mut builder = builder
            .preserve_comments(cfg!(debug_assertions))
            .strict_validation(true);
        
        // Enable service traits only if the "service-traits" feature is enabled
        if cfg!(feature = "service-traits") {
            builder = builder.generate_service_traits(true);
        }
        
        // Use dependency injection only if the "dependency-injection" feature is enabled
        if cfg!(feature = "dependency-injection") {
            builder = builder.use_dependency_injection(true);
        }
        
        // Skip rustfmt for faster builds in development
        if cfg!(feature = "fast-build") {
            builder = builder.use_rustfmt(false);
        }
        
        builder
    });
    
    // Add proto files based on features
    integration = integration.add_proto_file("proto/core.proto");
    
    if cfg!(feature = "user-service") {
        integration = integration.add_proto_file("proto/user.proto");
    }
    
    if cfg!(feature = "product-service") {
        integration = integration.add_proto_file("proto/product.proto");
    }
    
    integration
        .output_dir("src/generated")
        .generate()?;
    
    Ok(())
}

/// Pattern for workspace builds
#[allow(dead_code)]
fn workspace_integration() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../proto/");
    
    // In a workspace, you might want to share proto files
    BuildIntegration::new()
        .with_config_file("../proto-http-parser.toml")  // Shared config
        .add_proto_directory("../proto")?               // Shared proto files
        .output_dir("src/generated")
        .verbose(true)
        .generate()?;
    
    Ok(())
}

/// Pattern for multiple output directories
#[allow(dead_code)]
fn multi_output_integration() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/");
    
    // Generate client code
    BuildIntegration::new()
        .configure(|builder| {
            builder
                .generate_service_traits(true)
                .use_dependency_injection(false)  // Clients don't need DI
        })
        .add_proto_file("proto/user.proto")
        .output_dir("src/generated/client")
        .generate()?;
    
    // Generate server code
    BuildIntegration::new()
        .configure(|builder| {
            builder
                .generate_service_traits(true)
                .use_dependency_injection(true)   // Servers use DI
        })
        .add_proto_file("proto/user.proto")
        .output_dir("src/generated/server")
        .generate()?;
    
    Ok(())
}