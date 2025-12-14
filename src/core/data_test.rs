//! Tests for core data structures

#[cfg(test)]
mod tests {
    use super::super::data::*;

    #[test]
    fn test_proto_file_creation() {
        let proto_file = ProtoFile::new();
        assert_eq!(proto_file.syntax, ProtocolVersion::Proto3);
        assert!(proto_file.package.is_none());
        assert!(proto_file.imports.is_empty());
        assert!(proto_file.services.is_empty());
        assert!(proto_file.messages.is_empty());
        assert!(proto_file.enums.is_empty());
    }

    #[test]
    fn test_type_reference_creation() {
        let type_ref = TypeReference::new("User".to_string());
        assert_eq!(type_ref.name, "User");
        assert!(type_ref.package.is_none());
        assert!(!type_ref.is_stream);
        assert_eq!(type_ref.fully_qualified_name(), "User");
    }

    #[test]
    fn test_type_reference_with_package() {
        let type_ref = TypeReference::with_package("User".to_string(), "com.example".to_string());
        assert_eq!(type_ref.name, "User");
        assert_eq!(type_ref.package, Some("com.example".to_string()));
        assert_eq!(type_ref.fully_qualified_name(), "com.example.User");
    }

    #[test]
    fn test_scalar_type_detection() {
        let string_type = TypeReference::new("string".to_string());
        assert!(string_type.is_scalar());

        let int32_type = TypeReference::new("int32".to_string());
        assert!(int32_type.is_scalar());

        let custom_type = TypeReference::new("User".to_string());
        assert!(!custom_type.is_scalar());
    }

    #[test]
    fn test_well_known_type_detection() {
        let timestamp_type = TypeReference::with_package(
            "Timestamp".to_string(),
            "google.protobuf".to_string(),
        );
        assert!(timestamp_type.is_well_known_type());

        let custom_type = TypeReference::new("User".to_string());
        assert!(!custom_type.is_well_known_type());
    }

    #[test]
    fn test_http_route_creation() {
        let route = HttpRoute::new(
            "UserService".to_string(),
            "GetUser".to_string(),
            HttpMethod::Get,
            "/users/{id}".to_string(),
        );

        assert_eq!(route.service_name, "UserService");
        assert_eq!(route.method_name, "GetUser");
        assert_eq!(route.http_method, HttpMethod::Get);
        assert_eq!(route.path_template, "/users/{id}");
        assert_eq!(route.operation_id(), "UserService_GetUser");
        assert!(!route.has_path_parameters());
        assert!(!route.has_query_parameters());
        assert!(!route.has_request_body());
    }

    #[test]
    fn test_http_route_with_parameters() {
        let route = HttpRoute::new(
            "UserService".to_string(),
            "GetUser".to_string(),
            HttpMethod::Get,
            "/users/{id}".to_string(),
        )
        .with_path_parameter(PathParameter::new("id".to_string(), ParameterType::String))
        .with_query_parameter(QueryParameter::optional("limit".to_string(), ParameterType::Integer))
        .with_response_type(TypeReference::new("User".to_string()));

        assert!(route.has_path_parameters());
        assert!(route.has_query_parameters());
        assert!(!route.has_request_body());
        assert_eq!(route.path_parameters.len(), 1);
        assert_eq!(route.query_parameters.len(), 1);
        assert_eq!(route.response_type.name, "User");
    }

    #[test]
    fn test_service_creation() {
        let service = Service::new("UserService".to_string())
            .with_method(RpcMethod::new(
                "GetUser".to_string(),
                TypeReference::new("GetUserRequest".to_string()),
                TypeReference::new("User".to_string()),
            ));

        assert_eq!(service.name, "UserService");
        assert_eq!(service.methods.len(), 1);
        assert_eq!(service.methods[0].name, "GetUser");
        assert!(!service.methods[0].is_http_enabled());
        assert_eq!(service.http_methods().len(), 0);
    }

    #[test]
    fn test_service_with_http_methods() {
        let http_annotation = HttpAnnotation {
            method: HttpMethod::Get,
            path: "/users/{id}".to_string(),
            body: None,
            additional_bindings: Vec::new(),
        };

        let service = Service::new("UserService".to_string())
            .with_method(
                RpcMethod::new(
                    "GetUser".to_string(),
                    TypeReference::new("GetUserRequest".to_string()),
                    TypeReference::new("User".to_string()),
                )
                .with_http_annotation(http_annotation),
            );

        assert_eq!(service.http_methods().len(), 1);
        assert!(service.methods[0].is_http_enabled());
    }

    #[test]
    fn test_type_registry() {
        let mut registry = TypeRegistry::new();
        
        // Create a proto file with some types
        let mut proto_file = ProtoFile::new();
        proto_file.package = Some("com.example".to_string());
        proto_file.messages.push(Message {
            name: "User".to_string(),
            fields: Vec::new(),
            nested_messages: Vec::new(),
            nested_enums: Vec::new(),
            options: Vec::new(),
            comments: Vec::new(),
        });

        // Register the file
        registry.register_file("user.proto", &proto_file);

        // Test type resolution
        let type_ref = TypeReference::new("User".to_string());
        let resolved = registry.resolve_type(&type_ref, Some("com.example"));
        
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();
        assert_eq!(resolved.resolved_name, "com.example.User");
        assert_eq!(resolved.definition_location, TypeLocation::Local);
    }

    #[test]
    fn test_proto_file_type_resolution() {
        let mut proto_file = ProtoFile::new();
        proto_file.package = Some("com.example".to_string());
        proto_file.messages.push(Message {
            name: "User".to_string(),
            fields: Vec::new(),
            nested_messages: Vec::new(),
            nested_enums: Vec::new(),
            options: Vec::new(),
            comments: Vec::new(),
        });

        // Test local type resolution
        let type_ref = TypeReference::new("User".to_string());
        let resolved = proto_file.resolve_type(&type_ref);
        
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();
        assert_eq!(resolved.resolved_name, "com.example.User");
        assert_eq!(resolved.definition_location, TypeLocation::Local);

        // Test scalar type resolution
        let scalar_ref = TypeReference::new("string".to_string());
        let resolved_scalar = proto_file.resolve_type(&scalar_ref);
        
        assert!(resolved_scalar.is_some());
        let resolved_scalar = resolved_scalar.unwrap();
        assert_eq!(resolved_scalar.resolved_name, "string");
        assert_eq!(resolved_scalar.definition_location, TypeLocation::Builtin);
        assert!(resolved_scalar.is_scalar);
    }

    #[test]
    fn test_request_body_creation() {
        let entire_body = RequestBody::entire_message();
        assert!(entire_body.is_entire_message);
        assert!(entire_body.field.is_none());
        assert_eq!(entire_body.content_type, "application/json");

        let field_body = RequestBody::field("user".to_string())
            .with_content_type("application/x-protobuf".to_string());
        assert!(!field_body.is_entire_message);
        assert_eq!(field_body.field, Some("user".to_string()));
        assert_eq!(field_body.content_type, "application/x-protobuf");
    }

    #[test]
    fn test_parameter_creation() {
        let path_param = PathParameter::new("id".to_string(), ParameterType::String);
        assert_eq!(path_param.name, "id");
        assert!(path_param.required); // Path parameters are always required

        let optional_query = QueryParameter::optional("limit".to_string(), ParameterType::Integer);
        assert_eq!(optional_query.name, "limit");
        assert!(!optional_query.required);

        let required_query = QueryParameter::required("filter".to_string(), ParameterType::String);
        assert_eq!(required_query.name, "filter");
        assert!(required_query.required);
    }
}