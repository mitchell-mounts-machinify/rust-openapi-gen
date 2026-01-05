# Schema Reference Support in OpenAPI Types

## Overview

The `openapi.rs` module now fully supports OpenAPI 3.0 schema references using the `ReferenceOr<T>` type. This allows schemas to be either:
1. **Inline definitions** - Schema defined directly where it's used
2. **References** - `$ref` pointers to schemas defined in `components/schemas`

## The ReferenceOr<T> Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ReferenceOr<T> {
    /// A reference to a component (e.g., "#/components/schemas/User")
    Reference {
        #[serde(rename = "$ref")]
        reference: String,
    },
    /// An inline definition
    Item(T),
}
```

### Key Features

- **Untagged enum** - Serde automatically picks the right variant during serialization/deserialization
- **Type-safe** - Cannot accidentally use references in invalid contexts
- **Generic** - Can be reused for other OpenAPI types that support references
- **Helper methods** - Convenient constructors and accessors

## Usage Examples

### Creating References

```rust
use crate::openapi::ReferenceOr;

// Create a reference to a schema
let user_ref = ReferenceOr::<Schema>::new_ref("#/components/schemas/User");

// Create an inline schema
let inline_schema = ReferenceOr::new_item(Schema {
    schema_type: Some("string".to_string()),
    title: None,
    description: None,
    properties: None,
    required: None,
    reference: None,
});
```

### Using in Parameters

```rust
let parameter = Parameter {
    name: "userId".to_string(),
    location: "path".to_string(),
    description: Some("The user ID".to_string()),
    required: true,
    schema: ReferenceOr::new_ref("#/components/schemas/UserId"),
};
```

### Using in Responses

```rust
let mut content = HashMap::new();
content.insert("application/json".to_string(), MediaType {
    schema: Some(ReferenceOr::new_ref("#/components/schemas/User")),
});

let response = Response {
    description: "User retrieved successfully".to_string(),
    content: Some(content),
};
```

### Using in Schema Properties

```rust
let mut properties = HashMap::new();

// Inline property
properties.insert("id".to_string(), ReferenceOr::new_item(Schema {
    schema_type: Some("integer".to_string()),
    // ... other fields
}));

// Referenced property
properties.insert("address".to_string(), 
    ReferenceOr::new_ref("#/components/schemas/Address"));

let user_schema = Schema {
    schema_type: Some("object".to_string()),
    properties: Some(properties),
    // ... other fields
};
```

## Helper Methods

### Constructors

```rust
// Create a reference
ReferenceOr::new_ref("#/components/schemas/User")

// Create an inline item
ReferenceOr::new_item(schema)
```

### Checking Type

```rust
// Check if it's a reference
if schema.is_ref() {
    println!("This is a reference");
}
```

### Accessing Values

```rust
// Get the reference string (returns None if it's an item)
if let Some(ref_str) = schema.as_ref_str() {
    println!("References: {}", ref_str);
}

// Get the inline item (returns None if it's a reference)
if let Some(item) = schema.as_item() {
    println!("Type: {:?}", item.schema_type);
}
```

## Serialization Examples

### Reference Serialization

```rust
let schema_ref = ReferenceOr::<Schema>::new_ref("#/components/schemas/User");
let json = serde_json::to_string(&schema_ref).unwrap();
```

**Output:**
```json
{
  "$ref": "#/components/schemas/User"
}
```

### Inline Schema Serialization

```rust
let inline = ReferenceOr::new_item(Schema {
    schema_type: Some("string".to_string()),
    description: Some("User name".to_string()),
    // ...
});
let json = serde_json::to_string(&inline).unwrap();
```

**Output:**
```json
{
  "type": "string",
  "description": "User name"
}
```

## Where References Are Supported

| Location | Type | Support |
|----------|------|---------|
| Parameter schema | `Parameter.schema` | ✅ `ReferenceOr<Schema>` |
| MediaType schema | `MediaType.schema` | ✅ `Option<ReferenceOr<Schema>>` |
| Schema properties | `Schema.properties` | ✅ `HashMap<String, ReferenceOr<Schema>>` |
| Components schemas | `Components.schemas` | ✅ `HashMap<String, ReferenceOr<Schema>>` |

## Complete Example

```rust
use crate::openapi::*;

// Create an API with schema references
let mut api = OpenAPI::new("Reference Example", "1.0.0");

// Define a User schema in components
let mut schemas = HashMap::new();
schemas.insert("User".to_string(), ReferenceOr::new_item(Schema {
    schema_type: Some("object".to_string()),
    title: Some("User".to_string()),
    description: Some("A user in the system".to_string()),
    properties: Some({
        let mut props = HashMap::new();
        props.insert("id".to_string(), ReferenceOr::new_item(Schema {
            schema_type: Some("integer".to_string()),
            // ...
        }));
        props.insert("name".to_string(), ReferenceOr::new_item(Schema {
            schema_type: Some("string".to_string()),
            // ...
        }));
        props
    }),
    required: Some(vec!["id".to_string(), "name".to_string()]),
    reference: None,
}));

api.components = Some(Components { schemas });

// Use the schema reference in a response
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
    summary: Some("Get user".to_string()),
    description: None,
    parameters: vec![],
    request_body: None,
    responses,
};

let path_item = PathItem {
    get: Some(operation),
    post: None,
    put: None,
    delete: None,
    patch: None,
};

api.paths.insert("/users/{id}".to_string(), path_item);

// Serialize to JSON
let json = api.to_json().unwrap();
```

**Output:**
```json
{
  "openapi": "3.0.0",
  "info": {
    "title": "Reference Example",
    "version": "1.0.0"
  },
  "paths": {
    "/users/{id}": {
      "get": {
        "summary": "Get user",
        "responses": {
          "200": {
            "description": "Success",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/User"
                }
              }
            }
          }
        }
      }
    }
  },
  "components": {
    "schemas": {
      "User": {
        "type": "object",
        "title": "User",
        "description": "A user in the system",
        "properties": {
          "id": { "type": "integer" },
          "name": { "type": "string" }
        },
        "required": ["id", "name"]
      }
    }
  }
}
```

## Benefits

1. **Type Safety** - Cannot accidentally mix references and inline schemas
2. **Spec Compliance** - Matches OpenAPI 3.0 specification exactly
3. **Reusability** - Define schemas once in components, reference everywhere
4. **Smaller JSON** - References reduce duplication in the generated spec
5. **Better Tooling** - OpenAPI tools can resolve and validate references

## Testing

All reference functionality is thoroughly tested in `src/openapi_tests/mod.rs`:

- ✅ Reference serialization/deserialization
- ✅ Inline schema serialization/deserialization
- ✅ References in parameters, responses, media types
- ✅ Schema properties as references
- ✅ Complete API documents with references
- ✅ Roundtrip tests (serialize → deserialize → compare)

Run tests with:
```bash
cargo test openapi_tests
```

## Future Enhancements

Potential areas for expansion:

- [ ] Support `ReferenceOr` for other types (RequestBody, Response, Parameter)
- [ ] Add reference validation (ensure referenced schemas exist)
- [ ] Add helper methods to automatically register schemas in components
- [ ] Support external references (URLs to other OpenAPI documents)

## Reference Format

OpenAPI 3.0 references follow this format:

```
#/components/schemas/SchemaName
```

Where:
- `#` - Refers to the current document
- `/components/schemas/` - Path to the schemas section
- `SchemaName` - The name of the schema in components

## See Also

- [OpenAPI 3.0 Reference Objects](https://spec.openapis.org/oas/v3.0.0#reference-object)
- [OpenAPI 3.0 Schema Object](https://spec.openapis.org/oas/v3.0.0#schema-object)
- [Test Suite Documentation](src/openapi_tests/README.md)
- [Migration Plan](MIGRATION_PLAN.md)