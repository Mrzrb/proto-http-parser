//! Performance demonstration and optimization example

use proto_http_parser::*;
use std::time::{Duration, Instant};

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Proto HTTP Parser v2 - Performance Example");
    
    // Sample proto content for performance testing
    let proto_content = generate_test_proto(50); // 50 services
    
    println!("Generated test proto with {} lines", proto_content.lines().count());
    
    // Example 1: Baseline performance measurement
    println!("\n=== Example 1: Baseline Performance ===");
    
    let start = Instant::now();
    let coordinator = ProtoHttpCoordinator::new();
    let result = coordinator.process_content(&proto_content)?;
    let baseline_duration = start.elapsed();
    
    println!("Baseline processing time: {:?}", baseline_duration);
    println!("Generated {} files", 
        result.generated_files.len());
    
    // Example 2: Optimized configuration for speed
    println!("\n=== Example 2: Speed-Optimized Configuration ===");
    
    let speed_config = ConfigBuilder::new()
        .preserve_comments(false)      // Skip comment processing
        .use_rustfmt(false)           // Skip rustfmt for faster generation
        .strict_validation(false)      // Skip strict validation
        .max_import_depth(3)          // Limit import recursion
        .build()?;
    
    let start = Instant::now();
    let coordinator = ProtoHttpCoordinator::with_config(speed_config);
    let result = coordinator.process_content(&proto_content)?;
    let optimized_duration = start.elapsed();
    
    println!("Optimized processing time: {:?}", optimized_duration);
    println!("Speed improvement: {:.2}x", 
        baseline_duration.as_secs_f64() / optimized_duration.as_secs_f64());
    
    // Example 3: Memory-optimized configuration
    println!("\n=== Example 3: Memory Usage Analysis ===");
    
    let memory_config = ConfigBuilder::new()
        .preserve_comments(false)
        .generate_service_traits(false)  // Skip trait generation to save memory
        .build()?;
    
    let start = Instant::now();
    let coordinator = ProtoHttpCoordinator::with_config(memory_config);
    let result = coordinator.process_content(&proto_content)?;
    let memory_duration = start.elapsed();
    
    println!("Memory-optimized processing time: {:?}", memory_duration);
    println!("Generated {} files (no traits)", result.generated_files.len());
    
    // Example 4: Batch processing performance
    println!("\n=== Example 4: Batch Processing Performance ===");
    
    let small_protos = (0..10).map(|i| generate_test_proto(5)).collect::<Vec<_>>();
    
    // Sequential processing
    let start = Instant::now();
    let coordinator = ProtoHttpCoordinator::new();
    let mut sequential_results = Vec::new();
    
    for proto in &small_protos {
        let result = coordinator.process_content(proto)?;
        sequential_results.push(result);
    }
    let sequential_duration = start.elapsed();
    
    println!("Sequential processing (10 files): {:?}", sequential_duration);
    
    // Parallel processing (simulated - in real usage you'd use rayon or similar)
    let start = Instant::now();
    let mut parallel_results = Vec::new();
    
    for proto in &small_protos {
        let coordinator = ProtoHttpCoordinator::new(); // Each thread gets its own coordinator
        let result = coordinator.process_content(proto)?;
        parallel_results.push(result);
    }
    let parallel_duration = start.elapsed();
    
    println!("Parallel processing (10 files): {:?}", parallel_duration);
    println!("Note: This is simulated parallel processing. Use rayon for real parallelism.");
    
    // Example 5: Performance profiling by component
    println!("\n=== Example 5: Component Performance Profiling ===");
    
    // Parsing only
    let start = Instant::now();
    let parser = NomProtoParser::new();
    let proto_file = parser.parse_content(&proto_content)?;
    let parse_duration = start.elapsed();
    
    // HTTP extraction only
    let start = Instant::now();
    let extractor = GoogleApiHttpExtractor::new();
    let routes = extractor.extract_routes(&proto_file)?;
    let extract_duration = start.elapsed();
    
    // Code generation only
    let start = Instant::now();
    let generator = PoemOpenApiGenerator::new();
    let mut generation_duration = Duration::ZERO;
    
    for service in &proto_file.services {
        let service_routes: Vec<_> = routes.iter()
            .filter(|r| r.service_name == service.name)
            .cloned()
            .collect();
        
        let gen_start = Instant::now();
        let _controller = generator.generate_controller(service, &service_routes)?;
        let _trait_code = generator.generate_service_trait(service, &service_routes)?;
        generation_duration += gen_start.elapsed();
    }
    
    println!("Component performance breakdown:");
    println!("  Parsing: {:?} ({:.1}%)", 
        parse_duration, 
        parse_duration.as_secs_f64() / baseline_duration.as_secs_f64() * 100.0);
    println!("  HTTP extraction: {:?} ({:.1}%)", 
        extract_duration,
        extract_duration.as_secs_f64() / baseline_duration.as_secs_f64() * 100.0);
    println!("  Code generation: {:?} ({:.1}%)", 
        generation_duration,
        generation_duration.as_secs_f64() / baseline_duration.as_secs_f64() * 100.0);
    
    // Example 6: Scaling analysis
    println!("\n=== Example 6: Scaling Analysis ===");
    
    let sizes = vec![1, 5, 10, 25, 50];
    let mut scaling_results = Vec::new();
    
    for size in sizes {
        let test_proto = generate_test_proto(size);
        
        let start = Instant::now();
        let coordinator = ProtoHttpCoordinator::new();
        let result = coordinator.process_content(&test_proto)?;
        let duration = start.elapsed();
        
        scaling_results.push((size, duration, result.generated_files.len()));
        
        println!("  {} services: {:?} ({} files)", 
            size, duration, result.generated_files.len());
    }
    
    // Calculate scaling factor
    if scaling_results.len() >= 2 {
        let (size1, duration1, _) = scaling_results[0];
        let (size2, duration2, _) = scaling_results[scaling_results.len() - 1];
        
        let size_ratio = size2 as f64 / size1 as f64;
        let time_ratio = duration2.as_secs_f64() / duration1.as_secs_f64();
        
        println!("Scaling factor: {:.2}x time for {:.0}x services", time_ratio, size_ratio);
        
        if time_ratio / size_ratio < 1.5 {
            println!("✓ Good linear scaling performance");
        } else {
            println!("⚠ Performance may degrade with large proto files");
        }
    }
    
    // Example 7: Memory usage estimation
    println!("\n=== Example 7: Memory Usage Estimation ===");
    
    let proto_file_size = proto_content.len();
    let total_generated_size: usize = result.generated_files.values()
        .map(|code| code.content.len())
        .sum();
    
    println!("Input proto size: {} bytes", proto_file_size);
    println!("Generated code size: {} bytes", total_generated_size);
    println!("Expansion ratio: {:.1}x", 
        total_generated_size as f64 / proto_file_size as f64);
    
    // Example 8: Performance recommendations
    println!("\n=== Example 8: Performance Recommendations ===");
    
    println!("For optimal performance:");
    println!("1. Use build.rs for code generation (not runtime)");
    println!("2. Disable rustfmt during development for faster builds");
    println!("3. Set preserve_comments=false if comments aren't needed");
    println!("4. Use strict_validation=false for trusted proto files");
    println!("5. Limit max_import_depth for complex import hierarchies");
    println!("6. Process multiple files in parallel when possible");
    
    // Performance configuration example
    let production_config = ConfigBuilder::new()
        .preserve_comments(true)       // Keep for documentation
        .use_rustfmt(true)            // Format for production
        .strict_validation(true)       // Validate thoroughly
        .generate_service_traits(true) // Full feature set
        .use_dependency_injection(true)
        .build()?;
    
    let development_config = ConfigBuilder::new()
        .preserve_comments(false)      // Skip for speed
        .use_rustfmt(false)           // Skip for speed
        .strict_validation(false)      // Skip for speed
        .generate_service_traits(true) // Keep for development
        .use_dependency_injection(true)
        .build()?;
    
    println!("\nConfiguration recommendations:");
    println!("- Production: Full validation and formatting enabled");
    println!("- Development: Speed optimizations enabled");
    println!("- CI/CD: Balance between speed and validation");
    
    println!("\n✓ Performance analysis completed!");
    
    Ok(())
}

fn generate_test_proto(num_services: usize) -> String {
    let mut proto = String::new();
    
    proto.push_str(r#"
syntax = "proto3";

package test.v1;

import "google/api/annotations.proto";
import "google/protobuf/timestamp.proto";

"#);
    
    for i in 0..num_services {
        proto.push_str(&format!(r#"
service TestService{} {{
    rpc GetItem(GetItemRequest{}) returns (Item{}) {{
        option (google.api.http) = {{
            get: "/v1/service{}/items/{{item_id}}"
        }};
    }}
    
    rpc CreateItem(CreateItemRequest{}) returns (Item{}) {{
        option (google.api.http) = {{
            post: "/v1/service{}/items"
            body: "*"
        }};
    }}
    
    rpc UpdateItem(UpdateItemRequest{}) returns (Item{}) {{
        option (google.api.http) = {{
            put: "/v1/service{}/items/{{item.id}}"
            body: "item"
        }};
    }}
    
    rpc DeleteItem(DeleteItemRequest{}) returns (DeleteItemResponse{}) {{
        option (google.api.http) = {{
            delete: "/v1/service{}/items/{{item_id}}"
        }};
    }}
    
    rpc ListItems(ListItemsRequest{}) returns (ListItemsResponse{}) {{
        option (google.api.http) = {{
            get: "/v1/service{}/items"
        }};
    }}
}}

message GetItemRequest{} {{
    string item_id = 1;
}}

message CreateItemRequest{} {{
    string name = 1;
    string description = 2;
    ItemType{} type = 3;
}}

message UpdateItemRequest{} {{
    Item{} item = 1;
}}

message DeleteItemRequest{} {{
    string item_id = 1;
}}

message DeleteItemResponse{} {{
    bool success = 1;
}}

message ListItemsRequest{} {{
    int32 page_size = 1;
    string page_token = 2;
    ItemType{} type = 3;
}}

message ListItemsResponse{} {{
    repeated Item{} items = 1;
    string next_page_token = 2;
    int32 total_count = 3;
}}

message Item{} {{
    string id = 1;
    string name = 2;
    string description = 3;
    ItemType{} type = 4;
    ItemStatus{} status = 5;
    google.protobuf.Timestamp created_at = 6;
    google.protobuf.Timestamp updated_at = 7;
}}

enum ItemType{} {{
    ITEM_TYPE_UNSPECIFIED = 0;
    ITEM_TYPE_BASIC = 1;
    ITEM_TYPE_PREMIUM = 2;
}}

enum ItemStatus{} {{
    ITEM_STATUS_UNSPECIFIED = 0;
    ITEM_STATUS_ACTIVE = 1;
    ITEM_STATUS_INACTIVE = 2;
}}

"#, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i));
    }
    
    proto
}