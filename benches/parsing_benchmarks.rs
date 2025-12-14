//! Performance benchmarks for proto-http-parser-v2

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use proto_http_parser_v2::*;
use std::time::Duration;

// Sample proto content for benchmarking
const SIMPLE_PROTO: &str = r#"
syntax = "proto3";

package simple.v1;

service SimpleService {
    rpc GetItem(GetItemRequest) returns (Item);
}

message GetItemRequest {
    string item_id = 1;
}

message Item {
    string id = 1;
    string name = 2;
}
"#;

const COMPLEX_PROTO: &str = r#"
syntax = "proto3";

package complex.v1;

service UserService {
    rpc GetUser(GetUserRequest) returns (User);
    rpc CreateUser(CreateUserRequest) returns (User);
    rpc UpdateUser(UpdateUserRequest) returns (User);
    rpc DeleteUser(DeleteUserRequest) returns (DeleteUserResponse);
    rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
}

service ProductService {
    rpc GetProduct(GetProductRequest) returns (Product);
    rpc CreateProduct(CreateProductRequest) returns (Product);
    rpc ListProducts(ListProductsRequest) returns (ListProductsResponse);
    rpc SearchProducts(SearchProductsRequest) returns (SearchProductsResponse);
}

message GetUserRequest {
    string user_id = 1;
}

message CreateUserRequest {
    string name = 1;
    string email = 2;
    UserType type = 3;
}

message UpdateUserRequest {
    User user = 1;
}

message DeleteUserRequest {
    string user_id = 1;
}

message DeleteUserResponse {
    bool success = 1;
}

message ListUsersRequest {
    int32 page_size = 1;
    string page_token = 2;
    UserType type = 3;
}

message ListUsersResponse {
    repeated User users = 1;
    string next_page_token = 2;
    int32 total_count = 3;
}

message User {
    string id = 1;
    string name = 2;
    string email = 3;
    UserType type = 4;
    UserStatus status = 5;
    int64 created_at = 6;
    int64 updated_at = 7;
    repeated string tags = 8;
    map<string, string> metadata = 9;
}

enum UserType {
    USER_TYPE_UNSPECIFIED = 0;
    USER_TYPE_REGULAR = 1;
    USER_TYPE_PREMIUM = 2;
    USER_TYPE_ADMIN = 3;
}

enum UserStatus {
    USER_STATUS_UNSPECIFIED = 0;
    USER_STATUS_ACTIVE = 1;
    USER_STATUS_INACTIVE = 2;
    USER_STATUS_SUSPENDED = 3;
}

message GetProductRequest {
    string product_id = 1;
}

message CreateProductRequest {
    string name = 1;
    string description = 2;
    double price = 3;
    string category_id = 4;
}

message ListProductsRequest {
    int32 page_size = 1;
    string page_token = 2;
    string category_id = 3;
    double min_price = 4;
    double max_price = 5;
}

message ListProductsResponse {
    repeated Product products = 1;
    string next_page_token = 2;
    int32 total_count = 3;
}

message SearchProductsRequest {
    string query = 1;
    int32 page_size = 2;
    string page_token = 3;
    repeated string categories = 4;
}

message SearchProductsResponse {
    repeated Product products = 1;
    string next_page_token = 2;
    int32 total_count = 3;
    repeated string suggestions = 4;
}

message Product {
    string id = 1;
    string name = 2;
    string description = 3;
    double price = 4;
    string category_id = 5;
    ProductStatus status = 6;
    int64 created_at = 7;
    int64 updated_at = 8;
    repeated string images = 9;
    map<string, string> attributes = 10;
}

enum ProductStatus {
    PRODUCT_STATUS_UNSPECIFIED = 0;
    PRODUCT_STATUS_AVAILABLE = 1;
    PRODUCT_STATUS_OUT_OF_STOCK = 2;
    PRODUCT_STATUS_DISCONTINUED = 3;
}
"#;

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    
    // Benchmark simple proto parsing
    group.bench_function("simple_proto", |b| {
        let parser = NomProtoParser::new();
        b.iter(|| {
            let result = parser.parse_content(black_box(SIMPLE_PROTO));
            black_box(result.unwrap())
        })
    });
    
    // Benchmark complex proto parsing
    group.bench_function("complex_proto", |b| {
        let parser = NomProtoParser::new();
        b.iter(|| {
            let result = parser.parse_content(black_box(COMPLEX_PROTO));
            black_box(result.unwrap())
        })
    });
    
    // Benchmark parsing with different configurations
    let configs = vec![
        ("default", ConfigBuilder::new().build().unwrap()),
        ("no_comments", ConfigBuilder::new().preserve_comments(false).build().unwrap()),
        ("strict", ConfigBuilder::new().strict_validation(true).build().unwrap()),
    ];
    
    for (config_name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("complex_proto_with_config", config_name),
            &config,
            |b, config| {
                let parser = NomProtoParser::with_config(config.parser.clone());
                b.iter(|| {
                    let result = parser.parse_content(black_box(COMPLEX_PROTO));
                    black_box(result.unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn bench_http_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("http_extraction");
    
    // Pre-parse the proto files
    let parser = NomProtoParser::new();
    let simple_proto_file = parser.parse_content(SIMPLE_PROTO).unwrap();
    let complex_proto_file = parser.parse_content(COMPLEX_PROTO).unwrap();
    
    // Benchmark HTTP route extraction
    group.bench_function("simple_proto", |b| {
        let extractor = GoogleApiHttpExtractor::new();
        b.iter(|| {
            let result = extractor.extract_routes(black_box(&simple_proto_file));
            black_box(result.unwrap())
        })
    });
    
    group.bench_function("complex_proto", |b| {
        let extractor = GoogleApiHttpExtractor::new();
        b.iter(|| {
            let result = extractor.extract_routes(black_box(&complex_proto_file));
            black_box(result.unwrap())
        })
    });
    
    // Benchmark with different extractor configurations
    let configs = vec![
        ("default", ExtractorConfig::default()),
        ("no_query_inference", ExtractorConfig {
            infer_query_params: false,
            ..Default::default()
        }),
        ("strict_validation", ExtractorConfig {
            validate_http_methods: true,
            ..Default::default()
        }),
    ];
    
    for (config_name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("complex_proto_with_config", config_name),
            &config,
            |b, config| {
                let extractor = GoogleApiHttpExtractor::with_config(config.clone());
                b.iter(|| {
                    let result = extractor.extract_routes(black_box(&complex_proto_file));
                    black_box(result.unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn bench_code_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("code_generation");
    
    // Pre-parse and extract routes
    let parser = NomProtoParser::new();
    let extractor = GoogleApiHttpExtractor::new();
    
    let simple_proto_file = parser.parse_content(SIMPLE_PROTO).unwrap();
    let simple_routes = extractor.extract_routes(&simple_proto_file).unwrap();
    
    let complex_proto_file = parser.parse_content(COMPLEX_PROTO).unwrap();
    let complex_routes = extractor.extract_routes(&complex_proto_file).unwrap();
    
    // Benchmark controller generation
    group.bench_function("simple_controller", |b| {
        let generator = PoemOpenApiGenerator::new();
        let service = &simple_proto_file.services[0];
        let service_routes: Vec<_> = simple_routes.iter()
            .filter(|r| r.service_name == service.name)
            .cloned()
            .collect();
        
        b.iter(|| {
            let result = generator.generate_controller(black_box(service), black_box(&service_routes));
            black_box(result.unwrap())
        })
    });
    
    group.bench_function("complex_controller", |b| {
        let generator = PoemOpenApiGenerator::new();
        let service = &complex_proto_file.services[0]; // UserService
        let service_routes: Vec<_> = complex_routes.iter()
            .filter(|r| r.service_name == service.name)
            .cloned()
            .collect();
        
        b.iter(|| {
            let result = generator.generate_controller(black_box(service), black_box(&service_routes));
            black_box(result.unwrap())
        })
    });
    
    // Benchmark service trait generation
    group.bench_function("simple_trait", |b| {
        let generator = PoemOpenApiGenerator::new();
        let service = &simple_proto_file.services[0];
        let service_routes: Vec<_> = simple_routes.iter()
            .filter(|r| r.service_name == service.name)
            .cloned()
            .collect();
        
        b.iter(|| {
            let result = generator.generate_service_trait(black_box(service), black_box(&service_routes));
            black_box(result.unwrap())
        })
    });
    
    group.bench_function("complex_trait", |b| {
        let generator = PoemOpenApiGenerator::new();
        let service = &complex_proto_file.services[0]; // UserService
        let service_routes: Vec<_> = complex_routes.iter()
            .filter(|r| r.service_name == service.name)
            .cloned()
            .collect();
        
        b.iter(|| {
            let result = generator.generate_service_trait(black_box(service), black_box(&service_routes));
            black_box(result.unwrap())
        })
    });
    
    // Benchmark with different generator configurations
    let configs = vec![
        ("default", GeneratorConfig::default()),
        ("no_rustfmt", GeneratorConfig {
            formatting: FormattingConfig {
                use_rustfmt: false,
                ..Default::default()
            },
            ..Default::default()
        }),
        ("no_traits", GeneratorConfig {
            generate_service_traits: false,
            ..Default::default()
        }),
    ];
    
    for (config_name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("complex_controller_with_config", config_name),
            &config,
            |b, config| {
                let generator = PoemOpenApiGenerator::with_config(config.clone());
                let service = &complex_proto_file.services[0];
                let service_routes: Vec<_> = complex_routes.iter()
                    .filter(|r| r.service_name == service.name)
                    .cloned()
                    .collect();
                
                b.iter(|| {
                    let result = generator.generate_controller(black_box(service), black_box(&service_routes));
                    black_box(result.unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    
    // Benchmark complete processing pipeline
    group.bench_function("simple_proto_complete", |b| {
        let coordinator = ProtoHttpCoordinator::new();
        b.iter(|| {
            let result = coordinator.process_content(black_box(SIMPLE_PROTO));
            black_box(result.unwrap())
        })
    });
    
    group.bench_function("complex_proto_complete", |b| {
        let coordinator = ProtoHttpCoordinator::new();
        b.iter(|| {
            let result = coordinator.process_content(black_box(COMPLEX_PROTO));
            black_box(result.unwrap())
        })
    });
    
    // Benchmark with different configurations
    let configs = vec![
        ("default", ConfigBuilder::new().build().unwrap()),
        ("optimized", ConfigBuilder::new()
            .preserve_comments(false)
            .use_rustfmt(false)
            .strict_validation(false)
            .build().unwrap()),
        ("full_features", ConfigBuilder::new()
            .preserve_comments(true)
            .generate_service_traits(true)
            .use_dependency_injection(true)
            .infer_query_params(true)
            .use_rustfmt(true)
            .build().unwrap()),
    ];
    
    for (config_name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("complex_proto_with_config", config_name),
            &config,
            |b, config| {
                let coordinator = ProtoHttpCoordinator::with_config(config.clone());
                b.iter(|| {
                    let result = coordinator.process_content(black_box(COMPLEX_PROTO));
                    black_box(result.unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // Benchmark memory usage with large proto files
    let large_proto = generate_large_proto(100); // 100 services
    
    group.bench_function("large_proto_parsing", |b| {
        let parser = NomProtoParser::new();
        b.iter(|| {
            let result = parser.parse_content(black_box(&large_proto));
            black_box(result.unwrap())
        })
    });
    
    group.bench_function("large_proto_complete", |b| {
        let coordinator = ProtoHttpCoordinator::new();
        b.iter(|| {
            let result = coordinator.process_content(black_box(&large_proto));
            black_box(result.unwrap())
        })
    });
    
    group.finish();
}

fn generate_large_proto(num_services: usize) -> String {
    let mut proto = String::new();
    proto.push_str(r#"
syntax = "proto3";

package large.v1;

import "google/api/annotations.proto";
import "google/protobuf/timestamp.proto";

"#);
    
    for i in 0..num_services {
        proto.push_str(&format!(r#"
service Service{} {{
    rpc GetItem{}(GetItem{}Request) returns (Item{}) {{
        option (google.api.http) = {{
            get: "/v1/service{}/items/{{item_id}}"
        }};
    }}
    
    rpc CreateItem{}(CreateItem{}Request) returns (Item{}) {{
        option (google.api.http) = {{
            post: "/v1/service{}/items"
            body: "*"
        }};
    }}
    
    rpc ListItems{}(ListItems{}Request) returns (ListItems{}Response) {{
        option (google.api.http) = {{
            get: "/v1/service{}/items"
        }};
    }}
}}

message GetItem{}Request {{
    string item_id = 1;
}}

message CreateItem{}Request {{
    string name = 1;
    string description = 2;
}}

message ListItems{}Request {{
    int32 page_size = 1;
    string page_token = 2;
}}

message ListItems{}Response {{
    repeated Item{} items = 1;
    string next_page_token = 2;
}}

message Item{} {{
    string id = 1;
    string name = 2;
    string description = 3;
    google.protobuf.Timestamp created_at = 4;
}}

"#, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i));
    }
    
    proto
}

// Configure benchmark groups
criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets = 
        bench_parsing,
        bench_http_extraction,
        bench_code_generation,
        bench_end_to_end,
        bench_memory_usage
);

criterion_main!(benches);