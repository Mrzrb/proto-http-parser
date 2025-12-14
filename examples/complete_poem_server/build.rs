//! Build script for generating API controllers from proto files

use proto_http_parser_v2::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/");

    // Configure the HTTP controller generator
    let config = ConfigBuilder::new()
        .preserve_comments(true)
        .generate_service_traits(true)
        .use_dependency_injection(true)
        .infer_query_params(false)
        .use_rustfmt(true)
        .indent_size(4)
        .max_line_length(100)
        .add_include_path("proto") // Add proto directory to include path
        .add_type_mapping("google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>")
        .add_type_mapping("google.protobuf.Duration", "chrono::Duration")
        .add_import("use chrono::{DateTime, Utc};")
        .add_import("use serde::{Deserialize, Serialize};")
        .add_import("use uuid::Uuid;")
        .build()?;

    // Use build integration for automatic generation
    BuildIntegration::new()
        .with_config(config)
        .add_proto_file("proto/api.proto")
        .output_dir("src/generated")
        .verbose(true)
        .generate()?;

    println!("✓ Generated API controllers from proto files");

    Ok(())
}

