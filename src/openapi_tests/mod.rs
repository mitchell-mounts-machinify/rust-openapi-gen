// Comprehensive tests for OpenAPI 3.0 type serialization and deserialization

#[cfg(test)]
mod tests {
    use crate::openapi::*;

    use std::collections::HashMap;

    // ============================================================================
    // Basic OpenAPI Document Tests
    // ============================================================================

    #[test]
    fn test_minimal_openapi_serialization() {
        let api = OpenAPI::new("Test API", "1.0.0");
        
        let json_result = api.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_result).unwrap();
        
        assert_eq!(parsed["openapi"], "3.0.0");
        assert_eq!(parsed["info"]["title"], "Test API");
        assert_eq!(parsed["info"]["version"], "1.0.0");
        assert!(parsed["paths"].is_object());
        assert!(parsed["paths"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_minimal_openapi_deserialization() {
        let json_str = r#"{
            "openapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {}
        }"#;
        
        let api: OpenAPI = serde_json::from_str(json_str).unwrap();
        
        assert_eq!(api.openapi, "3.0.0");
        assert_eq!(api.info.title, "Test API");
        assert_eq!(api.info.version, "1.0.0");
        assert!(api.paths.is_empty());
        assert!(api.components.is_none());
    }

    #[test]
    fn test_openapi_roundtrip() {
        let original = OpenAPI::new("Roundtrip API", "2.0.0");
        
        let json = original.to_json().unwrap();
        let deserialized: OpenAPI = serde_json::from_str(&json).unwrap();
        
        assert_eq!(original, deserialized);
    }

    // ============================================================================
    // Info Object Tests
    // ============================================================================

    #[test]
    fn test_info_with_description_serialization() {
        let info = Info {
            title: "My API".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test API description".to_string()),
            terms_of_service: None,
            contact: None,
            license: None,
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["title"], "My API");
        assert_eq!(parsed["version"], "1.0.0");
        assert_eq!(parsed["description"], "A test API description");
    }

    #[test]
    fn test_info_without_description_serialization() {
        let info = Info {
            title: "Simple API".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            terms_of_service: None,
            contact: None,
            license: None,
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["title"], "Simple API");
        assert_eq!(parsed["version"], "0.1.0");
        assert!(parsed.get("description").is_none());
    }

    #[test]
    fn test_info_deserialization() {
        let json_str = r#"{
            "title": "Deserialized API",
            "version": "3.0.0",
            "description": "From JSON"
        }"#;
        
        let info: Info = serde_json::from_str(json_str).unwrap();
        
        assert_eq!(info.title, "Deserialized API");
        assert_eq!(info.version, "3.0.0");
        assert_eq!(info.description, Some("From JSON".to_string()));
    }

    // ============================================================================
    // PathItem Tests
    // ============================================================================

    #[test]
    fn test_empty_path_item_serialization() {
        let path_item = PathItem {
            get: None,
            post: None,
            put: None,
            delete: None,
            patch: None,
            head: None,
            options: None,
        };
        
        let json = serde_json::to_string(&path_item).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert!(parsed.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_path_item_with_get_operation() {
        let operation = Operation {
            summary: Some("Get items".to_string()),
            description: None,
            handler_function: None,
            tags: vec![],
            parameters: vec![],
            request_body: None,
            responses: HashMap::new(),
            security: None,
        };
        
        let path_item = PathItem {
            get: Some(operation),
            post: None,
            put: None,
            delete: None,
            patch: None,
            head: None,
            options: None,
        };
        
        let json = serde_json::to_string(&path_item).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert!(parsed["get"].is_object());
        assert_eq!(parsed["get"]["summary"], "Get items");
        assert!(parsed.get("post").is_none());
    }

    #[test]
    fn test_path_item_method_names_lowercase() {
        let operation = Operation {
            summary: Some("Test".to_string()),
            description: None,
            handler_function: None,
            tags: vec![],
            parameters: vec![],
            request_body: None,
            responses: HashMap::new(),
            security: None,
        };
        
        let path_item = PathItem {
            get: Some(operation.clone()),
            post: Some(operation.clone()),
            put: Some(operation.clone()),
            delete: Some(operation.clone()),
            patch: Some(operation),
            head: None,
            options: None,
        };
        
        let json = serde_json::to_string(&path_item).unwrap();
        
        // Verify all methods are lowercase in JSON
        assert!(json.contains(r#""get""#));
        assert!(json.contains(r#""post""#));
        assert!(json.contains(r#""put""#));
        assert!(json.contains(r#""delete""#));
        assert!(json.contains(r#""patch""#));
    }

    // ============================================================================
    // Operation Tests
    // ============================================================================

    #[test]
    fn test_minimal_operation_serialization() {
        let operation = Operation {
            summary: None,
            description: None,
            handler_function: None,
            tags: vec![],
            parameters: vec![],
            request_body: None,
            responses: HashMap::new(),
            security: None,
        };
        
        let json = serde_json::to_string(&operation).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert!(parsed.get("summary").is_none());
        assert!(parsed.get("description").is_none());
        assert!(parsed.get("parameters").is_none());
        assert!(parsed.get("requestBody").is_none());
        assert!(parsed["responses"].is_object());
    }

    #[test]
    fn test_operation_with_summary_and_description() {
        let operation = Operation {
            summary: Some("Get user by ID".to_string()),
            description: Some("Retrieves a user's information".to_string()),
            handler_function: None,
            tags: vec![],
            parameters: vec![],
            request_body: None,
            responses: HashMap::new(),
            security: None,
        };
        
        let json = serde_json::to_string(&operation).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["summary"], "Get user by ID");
        assert_eq!(parsed["description"], "Retrieves a user's information");
    }

    #[test]
    fn test_operation_camel_case_fields() {
        let mut responses = HashMap::new();
        responses.insert("200".to_string(), Response {
            description: "Success".to_string(),
            content: None,
        });
        
        let operation = Operation {
            summary: None,
            description: None,
            handler_function: None,
            tags: vec![],
            parameters: vec![],
            request_body: Some(RequestBody {
                description: None,
                content: HashMap::new(),
                required: true,
            }),
            responses,
            security: None,
        };
        
        let json = serde_json::to_string(&operation).unwrap();
        
        // Verify camelCase field name
        assert!(json.contains(r#""requestBody""#));
        assert!(!json.contains(r#""request_body""#));
    }

    // ============================================================================
    // Parameter Tests
    // ============================================================================

    #[test]
    fn test_path_parameter_serialization() {
        let schema = Schema {
            schema_type: Some("string".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
            reference: None,
        };
        
        let parameter = Parameter {
            name: "userId".to_string(),
            location: "path".to_string(),
            description: Some("The user ID".to_string()),
            required: true,
            schema: ReferenceOr::new_item(schema),
        };
        
        let json = serde_json::to_string(&parameter).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["name"], "userId");
        assert_eq!(parsed["in"], "path");
        assert_eq!(parsed["description"], "The user ID");
        assert_eq!(parsed["required"], true);
        assert_eq!(parsed["schema"]["type"], "string");
    }

    #[test]
    fn test_query_parameter_deserialization() {
        let json_str = r#"{
            "name": "limit",
            "in": "query",
            "description": "Max items to return",
            "required": false,
            "schema": {
                "type": "integer"
            }
        }"#;
        
        let parameter: Parameter = serde_json::from_str(json_str).unwrap();
        
        assert_eq!(parameter.name, "limit");
        assert_eq!(parameter.location, "query");
        assert_eq!(parameter.description, Some("Max items to return".to_string()));
        assert_eq!(parameter.required, false);
        assert!(parameter.schema.as_item().is_some());
        assert_eq!(parameter.schema.as_item().unwrap().schema_type, Some("integer".to_string()));
    }

    // ============================================================================
    // RequestBody Tests
    // ============================================================================

    #[test]
    fn test_request_body_serialization() {
        let schema = Schema {
            schema_type: Some("object".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
            reference: None,
        };
        
        let mut content = HashMap::new();
        content.insert("application/json".to_string(), MediaType {
            schema: Some(ReferenceOr::new_item(schema)),
        });
        
        let request_body = RequestBody {
            description: Some("User data".to_string()),
            content,
            required: true,
        };
        
        let json = serde_json::to_string(&request_body).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["description"], "User data");
        assert_eq!(parsed["required"], true);
        assert!(parsed["content"]["application/json"].is_object());
    }

    #[test]
    fn test_request_body_deserialization() {
        let json_str = r#"{
            "description": "Create user request",
            "required": true,
            "content": {
                "application/json": {
                    "schema": {
                        "type": "object"
                    }
                }
            }
        }"#;
        
        let request_body: RequestBody = serde_json::from_str(json_str).unwrap();
        
        assert_eq!(request_body.description, Some("Create user request".to_string()));
        assert_eq!(request_body.required, true);
        assert!(request_body.content.contains_key("application/json"));
    }

    // ============================================================================
    // Response Tests
    // ============================================================================

    #[test]
    fn test_simple_response_serialization() {
        let response = Response {
            description: "Successful operation".to_string(),
            content: None,
        };
        
        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["description"], "Successful operation");
        assert!(parsed.get("content").is_none());
    }

    #[test]
    fn test_response_with_content_serialization() {
        let schema = Schema {
            schema_type: Some("object".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
            reference: None,
        };
        
        let mut content = HashMap::new();
        content.insert("application/json".to_string(), MediaType {
            schema: Some(ReferenceOr::new_item(schema)),
        });
        
        let response = Response {
            description: "User retrieved".to_string(),
            content: Some(content),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["description"], "User retrieved");
        assert!(parsed["content"]["application/json"]["schema"]["type"] == "object");
    }

    // ============================================================================
    // Schema Tests
    // ============================================================================

    #[test]
    fn test_simple_string_schema_serialization() {
        let schema = Schema {
            schema_type: Some("string".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
            reference: None,
        };
        
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["type"], "string");
        assert!(parsed.get("title").is_none());
    }

    #[test]
    fn test_object_schema_with_properties() {
        let mut properties = HashMap::new();
        properties.insert("id".to_string(), ReferenceOr::new_item(Schema {
            schema_type: Some("integer".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
            reference: None,
        }));
        properties.insert("name".to_string(), ReferenceOr::new_item(Schema {
            schema_type: Some("string".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
            reference: None,
        }));
        
        let schema = Schema {
            schema_type: Some("object".to_string()),
            title: Some("User".to_string()),
            description: Some("A user object".to_string()),
            properties: Some(properties),
            required: Some(vec!["id".to_string(), "name".to_string()]),
            reference: None,
        };
        
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["type"], "object");
        assert_eq!(parsed["title"], "User");
        assert_eq!(parsed["description"], "A user object");
        assert!(parsed["properties"]["id"]["type"] == "integer");
        assert!(parsed["properties"]["name"]["type"] == "string");
        assert_eq!(parsed["required"][0], "id");
        assert_eq!(parsed["required"][1], "name");
    }

    #[test]
    fn test_schema_default() {
        let schema = Schema::default();
        
        assert_eq!(schema.schema_type, Some("object".to_string()));
        assert_eq!(schema.title, None);
        assert_eq!(schema.description, None);
        assert_eq!(schema.properties, None);
        assert_eq!(schema.required, None);
    }

    // ============================================================================
    // Components Tests
    // ============================================================================

    #[test]
    fn test_components_serialization() {
        let mut schemas = HashMap::new();
        schemas.insert("User".to_string(), ReferenceOr::new_item(Schema {
            schema_type: Some("object".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
            reference: None,
        }));
        
        let components = Components { 
            schemas,
            security_schemes: None,
        };
        
        let json = serde_json::to_string(&components).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert!(parsed["schemas"]["User"]["type"] == "object");
    }

    // ============================================================================
    // Complete OpenAPI Document Tests
    // ============================================================================

    #[test]
    fn test_complete_openapi_document_serialization() {
        let mut api = OpenAPI::new("Complete API", "1.0.0");
        api.info.description = Some("A complete example".to_string());
        
        // Add a path with GET operation
        let mut responses = HashMap::new();
        responses.insert("200".to_string(), Response {
            description: "Success".to_string(),
            content: None,
        });
        
        let get_operation = Operation {
            summary: Some("List users".to_string()),
            description: Some("Returns a list of users".to_string()),
            handler_function: None,
            tags: vec![],
            parameters: vec![],
            request_body: None,
            responses,
            security: None,
        };
        
        let path_item = PathItem {
            get: Some(get_operation),
            post: None,
            put: None,
            delete: None,
            patch: None,
            head: None,
            options: None,
        };
        
        api.paths.insert("/users".to_string(), path_item);
        
        // Add components
        let mut schemas = HashMap::new();
        schemas.insert("User".to_string(), ReferenceOr::new_item(Schema {
            schema_type: Some("object".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
            reference: None,
        }));
        
        api.components = Some(Components { 
            schemas,
            security_schemes: None,
        });
        
        let json = api.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["openapi"], "3.0.0");
        assert_eq!(parsed["info"]["title"], "Complete API");
        assert_eq!(parsed["info"]["description"], "A complete example");
        assert!(parsed["paths"]["/users"]["get"].is_object());
        assert_eq!(parsed["paths"]["/users"]["get"]["summary"], "List users");
        assert!(parsed["components"]["schemas"]["User"].is_object());
    }

    #[test]
    fn test_complete_openapi_document_deserialization() {
        let json_str = r#"{
            "openapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "2.0.0",
                "description": "API description"
            },
            "paths": {
                "/items": {
                    "get": {
                        "summary": "Get items",
                        "responses": {
                            "200": {
                                "description": "Success"
                            }
                        }
                    }
                }
            },
            "components": {
                "schemas": {
                    "Item": {
                        "type": "object"
                    }
                }
            }
        }"#;
        
        let api: OpenAPI = serde_json::from_str(json_str).unwrap();
        
        assert_eq!(api.openapi, "3.0.0");
        assert_eq!(api.info.title, "Test API");
        assert_eq!(api.info.version, "2.0.0");
        assert_eq!(api.info.description, Some("API description".to_string()));
        assert!(api.paths.contains_key("/items"));
        assert!(api.components.is_some());
    }

    #[test]
    fn test_openapi_document_complete_roundtrip() {
        // Create a complex OpenAPI document
        let mut api = OpenAPI::new("Roundtrip Test", "3.0.0");
        api.info.description = Some("Testing roundtrip serialization".to_string());
        
        let mut responses = HashMap::new();
        responses.insert("200".to_string(), Response {
            description: "Successful response".to_string(),
            content: None,
        });
        responses.insert("404".to_string(), Response {
            description: "Not found".to_string(),
            content: None,
        });
        
        let operation = Operation {
            summary: Some("Test operation".to_string()),
            description: Some("A test operation".to_string()),
            handler_function: None,
            tags: vec![],
            parameters: vec![],
            request_body: None,
            responses: responses.clone(),
            security: None,
        };
        
        let path_item = PathItem {
            get: Some(operation),
            post: None,
            put: None,
            delete: None,
            patch: None,
            head: None,
            options: None,
        };
        
        api.paths.insert("/test".to_string(), path_item);
        
        // Serialize and deserialize
        let json = api.to_json().unwrap();
        let deserialized: OpenAPI = serde_json::from_str(&json).unwrap();
        
        // Verify equality
        assert_eq!(api, deserialized);
    }

    // ============================================================================
    // ReferenceOr Tests
    // ============================================================================

    #[test]
    fn test_schema_reference_serialization() {
        use crate::openapi::ReferenceOr;
        
        let schema_ref = ReferenceOr::<Schema>::new_ref("#/components/schemas/User");
        
        let json = serde_json::to_string(&schema_ref).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["$ref"], "#/components/schemas/User");
    }

    #[test]
    fn test_schema_reference_deserialization() {

        
        let json_str = r##"{"$ref": "#/components/schemas/Product"}"##;
        
        let schema_ref: ReferenceOr<Schema> = serde_json::from_str(json_str).unwrap();
        
        assert!(schema_ref.is_ref());
        assert_eq!(schema_ref.as_ref_str(), Some("#/components/schemas/Product"));
    }

    #[test]
    fn test_inline_schema_in_reference_or() {
        use crate::openapi::ReferenceOr;
        
        let inline_schema = Schema {
            schema_type: Some("string".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
            reference: None,
        };
        
        let schema = ReferenceOr::new_item(inline_schema);
        
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["type"], "string");
        assert!(parsed.get("$ref").is_none());
    }

    #[test]
    fn test_parameter_with_schema_reference() {
        use crate::openapi::ReferenceOr;
        
        let parameter = Parameter {
            name: "userId".to_string(),
            location: "path".to_string(),
            description: Some("User identifier".to_string()),
            required: true,
            schema: ReferenceOr::new_ref("#/components/schemas/UserId"),
        };
        
        let json = serde_json::to_string(&parameter).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["name"], "userId");
        assert_eq!(parsed["schema"]["$ref"], "#/components/schemas/UserId");
    }

    #[test]
    fn test_media_type_with_schema_reference() {
        use crate::openapi::ReferenceOr;
        
        let media_type = MediaType {
            schema: Some(ReferenceOr::new_ref("#/components/schemas/UserResponse")),
        };
        
        let json = serde_json::to_string(&media_type).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["schema"]["$ref"], "#/components/schemas/UserResponse");
    }

    #[test]
    fn test_response_with_referenced_schema() {
        use crate::openapi::ReferenceOr;
        
        let mut content = HashMap::new();
        content.insert("application/json".to_string(), MediaType {
            schema: Some(ReferenceOr::new_ref("#/components/schemas/Error")),
        });
        
        let response = Response {
            description: "Error response".to_string(),
            content: Some(content),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["description"], "Error response");
        assert_eq!(parsed["content"]["application/json"]["schema"]["$ref"], "#/components/schemas/Error");
    }

    #[test]
    fn test_components_with_schema_references() {
        use crate::openapi::ReferenceOr;
        
        let mut schemas = HashMap::new();
        
        // Add an inline schema
        schemas.insert("User".to_string(), ReferenceOr::new_item(Schema {
            schema_type: Some("object".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
            reference: None,
        }));
        
        // Add a reference (though this is unusual in components)
        schemas.insert("AliasToUser".to_string(), 
            ReferenceOr::new_ref("#/components/schemas/User"));
        
        let components = Components { 
            schemas,
            security_schemes: None,
        };
        
        let json = serde_json::to_string(&components).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["schemas"]["User"]["type"], "object");
        assert_eq!(parsed["schemas"]["AliasToUser"]["$ref"], "#/components/schemas/User");
    }

    #[test]
    fn test_schema_with_referenced_properties() {
        use crate::openapi::ReferenceOr;
        
        let mut properties = HashMap::new();
        properties.insert("id".to_string(), ReferenceOr::new_item(Schema {
            schema_type: Some("integer".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
            reference: None,
        }));
        properties.insert("address".to_string(), 
            ReferenceOr::new_ref("#/components/schemas/Address"));
        
        let schema = Schema {
            schema_type: Some("object".to_string()),
            title: None,
            description: None,
            properties: Some(properties),
            required: Some(vec!["id".to_string()]),
            reference: None,
        };
        
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["type"], "object");
        assert_eq!(parsed["properties"]["id"]["type"], "integer");
        assert_eq!(parsed["properties"]["address"]["$ref"], "#/components/schemas/Address");
    }

    #[test]
    fn test_reference_or_roundtrip_reference() {
        use crate::openapi::ReferenceOr;
        
        let original = ReferenceOr::<Schema>::new_ref("#/components/schemas/TestSchema");
        
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ReferenceOr<Schema> = serde_json::from_str(&json).unwrap();
        
        assert_eq!(original, deserialized);
        assert!(deserialized.is_ref());
    }

    #[test]
    fn test_reference_or_roundtrip_item() {
        use crate::openapi::ReferenceOr;
        
        let original = ReferenceOr::new_item(Schema {
            schema_type: Some("boolean".to_string()),
            title: None,
            description: Some("A boolean flag".to_string()),
            properties: None,
            required: None,
            reference: None,
        });
        
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ReferenceOr<Schema> = serde_json::from_str(&json).unwrap();
        
        assert_eq!(original, deserialized);
        assert!(!deserialized.is_ref());
    }

    #[test]
    fn test_complete_api_with_references() {
        use crate::openapi::ReferenceOr;
        
        let mut api = OpenAPI::new("Reference Test API", "1.0.0");
        
        // Add a path with referenced schema
        let mut content = HashMap::new();
        content.insert("application/json".to_string(), MediaType {
            schema: Some(ReferenceOr::new_ref("#/components/schemas/User")),
        });
        
        let mut responses = HashMap::new();
        responses.insert("200".to_string(), Response {
            description: "Success".to_string(),
            content: Some(content),
        });
        
        let operation = Operation {
            summary: None,
            description: None,
            handler_function: None,
            tags: vec![],
            parameters: vec![],
            request_body: None,
            responses,
            security: None,
        };
        
        let path_item = PathItem {
            get: Some(operation),
            post: None,
            put: None,
            delete: None,
            patch: None,
            head: None,
            options: None,
        };
        
        api.paths.insert("/users/{id}".to_string(), path_item);
        
        // Add components with schema definition
        let mut schemas = HashMap::new();
        schemas.insert("User".to_string(), ReferenceOr::new_item(Schema {
            schema_type: Some("object".to_string()),
            title: Some("User".to_string()),
            description: None,
            properties: None,
            required: None,
            reference: None,
        }));
        
        api.components = Some(Components { 
            schemas,
            security_schemes: None,
        });
        
        // Test serialization
        let json = api.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(
            parsed["paths"]["/users/{id}"]["get"]["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/User"
        );
        assert_eq!(parsed["components"]["schemas"]["User"]["type"], "object");
        
        // Test roundtrip
        let deserialized: OpenAPI = serde_json::from_str(&json).unwrap();
        assert_eq!(api, deserialized);
    }
}