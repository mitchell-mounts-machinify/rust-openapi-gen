# OpenAPI 3.0 Type Tests

This test suite ensures that the OpenAPI 3.0 type definitions in `src/openapi.rs` correctly serialize and deserialize according to the OpenAPI 3.0 specification.

## Purpose

As we transition from manual JSON string building (in `lib.rs`) to proper serde-based serialization (in `openapi.rs`), these tests serve as:

1. **Safety net**: Ensure we don't break compatibility during refactoring
2. **Specification compliance**: Verify our types match OpenAPI 3.0 spec
3. **Documentation**: Show examples of how each type should be used
4. **Regression prevention**: Catch bugs before they reach production

## Test Organization

Tests are organized by OpenAPI component type:

### Basic OpenAPI Document Tests
- `test_minimal_openapi_serialization` - Minimal valid OpenAPI document
- `test_minimal_openapi_deserialization` - Parse minimal document
- `test_openapi_roundtrip` - Serialize → Deserialize → Compare

### Info Object Tests
- `test_info_with_description_serialization` - Info with optional fields
- `test_info_without_description_serialization` - Info without optional fields
- `test_info_deserialization` - Parse Info from JSON

### PathItem Tests
- `test_empty_path_item_serialization` - PathItem with no operations
- `test_path_item_with_get_operation` - PathItem with GET operation
- `test_path_item_method_names_lowercase` - Verify HTTP methods are lowercase

### Operation Tests
- `test_minimal_operation_serialization` - Operation with only required fields
- `test_operation_with_summary_and_description` - Operation with optional fields
- `test_operation_camel_case_fields` - Verify camelCase for `requestBody`

### Parameter Tests
- `test_path_parameter_serialization` - Path parameter with schema
- `test_query_parameter_deserialization` - Parse query parameter

### RequestBody Tests
- `test_request_body_serialization` - RequestBody with content types
- `test_request_body_deserialization` - Parse RequestBody from JSON

### Response Tests
- `test_simple_response_serialization` - Response without content
- `test_response_with_content_serialization` - Response with media type

### Schema Tests
- `test_simple_string_schema_serialization` - Basic string schema
- `test_object_schema_with_properties` - Object with properties and required fields
- `test_schema_default` - Verify default Schema values

### Components Tests
- `test_components_serialization` - Components with schemas

### Complete Document Tests
- `test_complete_openapi_document_serialization` - Full document with all parts
- `test_complete_openapi_document_deserialization` - Parse complete document
- `test_openapi_document_complete_roundtrip` - Full roundtrip test

### ReferenceOr Tests (Schema References)
- `test_schema_reference_serialization` - Serialize a schema reference ($ref)
- `test_schema_reference_deserialization` - Deserialize a schema reference
- `test_inline_schema_in_reference_or` - Inline schema wrapped in ReferenceOr
- `test_parameter_with_schema_reference` - Parameter using schema reference
- `test_media_type_with_schema_reference` - MediaType with referenced schema
- `test_response_with_referenced_schema` - Response content with schema reference
- `test_components_with_schema_references` - Components containing references
- `test_schema_with_referenced_properties` - Schema properties as references
- `test_reference_or_roundtrip_reference` - Roundtrip test for references
- `test_reference_or_roundtrip_item` - Roundtrip test for inline items
- `test_complete_api_with_references` - Full API document using references

## Running Tests

```bash
# Run all OpenAPI tests
cargo test openapi_tests

# Run a specific test
cargo test openapi_tests::tests::test_minimal_openapi_serialization

# Run with output
cargo test openapi_tests -- --nocapture
```

## Adding New Tests

When adding new fields or types to `openapi.rs`, follow this pattern:

1. **Create a serialization test** - Build the struct, serialize to JSON, verify output
2. **Create a deserialization test** - Start with JSON, parse to struct, verify fields
3. **Create a roundtrip test** - Serialize → Deserialize → Compare equality

### Example Template

```rust
#[test]
fn test_new_field_serialization() {
    // 1. Create the struct
    let my_struct = MyStruct {
        new_field: Some("value".to_string()),
        // ... other fields
    };
    
    // 2. Serialize to JSON
    let json = serde_json::to_string(&my_struct).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    
    // 3. Verify the output
    assert_eq!(parsed["newField"], "value");
    assert!(parsed.get("optionalField").is_none()); // Optional fields omitted
}

#[test]
fn test_new_field_deserialization() {
    let json_str = r#"{
        "newField": "value"
    }"#;
    
    let my_struct: MyStruct = serde_json::from_str(json_str).unwrap();
    
    assert_eq!(my_struct.new_field, Some("value".to_string()));
}

#[test]
fn test_new_field_roundtrip() {
    let original = MyStruct {
        new_field: Some("test".to_string()),
    };
    
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: MyStruct = serde_json::from_str(&json).unwrap();
    
    assert_eq!(original, deserialized);
}
```

## OpenAPI 3.0 Specification Compliance

These tests verify compliance with key aspects of the OpenAPI 3.0 specification:

- **Field naming**: Uses camelCase (e.g., `requestBody`, not `request_body`)
- **HTTP methods**: Lowercase in PathItem (e.g., `get`, `post`, not `GET`, `POST`)
- **Optional fields**: Omitted from JSON when `None` (via `skip_serializing_if`)
- **Required fields**: Always present in serialized output
- **Schema type**: Renamed to `type` in JSON (via `#[serde(rename = "type")]`)
- **Parameter location**: Renamed to `in` in JSON (via `#[serde(rename = "in")]`)
- **Schema references**: Supports `$ref` via `ReferenceOr<T>` type for referencing components

## Missing Types (To Be Added)

The following OpenAPI 3.0 types are not yet implemented:

- [ ] `Tag` - API tag definitions
- [ ] `ExternalDocs` - External documentation
- [ ] `Contact` - Contact information in Info
- [ ] `License` - License information in Info
- [ ] `Server` - API server definitions
- [ ] `SecurityScheme` - Security scheme definitions
- [ ] `SecurityRequirement` - Security requirements
- [ ] `Callback` - Callback definitions
- [ ] `Example` - Example values
- [ ] `Link` - Link definitions
- [ ] `Header` - Header definitions
- [ ] `Encoding` - Encoding definitions
- [ ] `Discriminator` - Schema discriminator

When adding these types, create corresponding test cases following the patterns above.

## Test Coverage Goals

- ✅ Basic type serialization/deserialization
- ✅ Optional field handling
- ✅ Field naming (camelCase, renamed fields)
- ✅ Complete document roundtrip
- ✅ Schema references ($ref) via ReferenceOr<T>
- ⏳ Array types
- ⏳ Enum types
- ⏳ AllOf/OneOf/AnyOf
- ⏳ Edge cases and error handling

## Schema Reference Support

The `ReferenceOr<T>` type enables proper OpenAPI schema references:

### Usage Examples

```rust
use crate::openapi::ReferenceOr;

// Create a reference to a schema in components
let user_ref = ReferenceOr::<Schema>::new_ref("#/components/schemas/User");

// Create an inline schema
let inline = ReferenceOr::new_item(Schema {
    schema_type: Some("string".to_string()),
    // ... other fields
});

// Use in responses
let media_type = MediaType {
    schema: Some(ReferenceOr::new_ref("#/components/schemas/Error")),
};

// Check what type it is
if user_ref.is_ref() {
    println!("Reference: {}", user_ref.as_ref_str().unwrap());
}
```

### Where References are Supported

- ✅ **Parameter schemas** - `Parameter.schema: ReferenceOr<Schema>`
- ✅ **MediaType schemas** - `MediaType.schema: Option<ReferenceOr<Schema>>`
- ✅ **Schema properties** - `Schema.properties: Option<HashMap<String, ReferenceOr<Schema>>>`
- ✅ **Components schemas** - `Components.schemas: HashMap<String, ReferenceOr<Schema>>`

This matches the OpenAPI 3.0 specification where almost any schema can be either inline or a reference.

## Next Steps

1. **Extend openapi.rs** - Add missing OpenAPI 3.0 types (Contact, License, ExternalDocs, etc.)
2. **Add tests** - Create tests for each new type following the patterns here
3. **Integrate with lib.rs** - Gradually replace string-based JSON building with serde serialization
4. **Validate against spec** - Use OpenAPI validator tools to verify output
5. **Performance testing** - Ensure serde approach is performant

## Resources

- [OpenAPI 3.0 Specification](https://spec.openapis.org/oas/v3.0.0)
- [Serde Documentation](https://serde.rs/)
- [OpenAPI Examples](https://github.com/OAI/OpenAPI-Specification/tree/main/examples)