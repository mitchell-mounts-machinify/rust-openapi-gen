# stonehm - Documentation-Driven OpenAPI Generation for Axum

stonehm automatically generates comprehensive OpenAPI 3.0 specifications for Axum web applications by analyzing handler functions and their documentation. The core principle is **"documentation is the spec"** - write clear, natural documentation and get complete OpenAPI specs automatically.

## Key Features

- Generate OpenAPI 3.0 specs from rustdoc comments
- Automatic error handling from `Result<T, E>` return types  
- Type-safe schema generation via derive macros
- Compile-time processing with zero runtime overhead
- Drop-in replacement for `axum::Router`

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
stonehm = "0.1"
stonehm-macros = "0.1"
axum = "0.7"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
```

### 30-Second Example

```rust
use axum::{Json, extract::Path};
use serde::{Serialize, Deserialize};
use stonehm::{api_router, api_handler};
use stonehm_macros::{StonehmSchema, api_error};

// Define your data types
#[derive(Serialize, StonehmSchema)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[api_error]
enum ApiError {
    /// 404: User not found
    UserNotFound { id: u32 },
    
    /// 500: Internal server error
    DatabaseError,
}

/// Get user by ID
///
/// Retrieves a user's information using their unique identifier.
/// Returns detailed user data including name and email.
#[api_handler]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, ApiError> {
    Ok(Json(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    }))
}

#[tokio::main]
async fn main() {
    let app = api_router!("User API", "1.0.0")
        .get("/users/:id", get_user)
        .with_openapi_routes()  // Adds /openapi.json and /openapi.yaml
        .into_router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    println!("OpenAPI spec: http://127.0.0.1:3000/openapi.json");
    
    axum::serve(listener, app).await.unwrap();
}
```

**That's it!** You now have a fully documented API with automatic OpenAPI generation.

**Automatic OpenAPI generation includes:**
- Path parameter documentation
- 200 response with User schema  
- 400 Bad Request with ApiError schema
- 500 Internal Server Error with ApiError schema

## Documentation Approaches

stonehm supports three documentation approaches to fit different needs:

### 1. Automatic Documentation (Recommended)

Let stonehm infer everything from your code structure:

```rust
/// Get user profile
///
/// Retrieves the current user's profile information.
#[api_handler]
async fn get_profile() -> Result<Json<User>, ApiError> {
    Ok(Json(User::default()))
}
```

**Automatically generates:**
- 200 response with User schema
- 400 Bad Request with ApiError schema  
- 500 Internal Server Error with ApiError schema

```

### 2. Structured Documentation

Add detailed parameter and response documentation:

```rust
/// Update user profile
///
/// Updates the user's profile information. Only provided fields
/// will be updated, others remain unchanged.
///
/// # Parameters
/// - id (path): The user's unique identifier
/// - version (query): API version to use
/// - authorization (header): Bearer token for authentication
///
/// # Request Body
/// Content-Type: application/json
/// User update data with optional fields for name, email, and preferences.
///
/// # Responses
/// - 200: User successfully updated
/// - 400: Invalid user data provided
/// - 401: Authentication required
/// - 404: User not found
/// - 422: Validation failed
#[api_handler]
async fn update_profile(
    Path(id): Path<u32>,
    Json(request): Json<UpdateUserRequest>
) -> Result<Json<User>, ApiError> {
    Ok(Json(User::default()))
}
```

### 3. Elaborate Documentation

For complex APIs requiring detailed error schemas:

```rust
/// Delete user account
///
/// Permanently removes a user account and all associated data.
/// This action cannot be undone.
///
/// # Parameters
/// - id (path): The unique user identifier to delete
///
/// # Responses
/// - 204: User successfully deleted
/// - 404:
///   description: User not found
///   content:
///     application/json:
///       schema: NotFoundError
/// - 403:
///   description: Insufficient permissions to delete user
///   content:
///     application/json:
///       schema: PermissionError
/// - 409:
///   description: Cannot delete user with active subscriptions
///   content:
///     application/json:
///       schema: ConflictError
#[api_handler]
async fn delete_user(Path(id): Path<u32>) -> Result<(), ApiError> {
    Ok(())
}
```

## Schema Generation

stonehm uses the `StonehmSchema` derive macro for automatic schema generation:

```rust
use serde::{Serialize, Deserialize};
use stonehm_macros::StonehmSchema;

#[derive(Serialize, Deserialize, StonehmSchema)]
struct CreateUserRequest {
    name: String,
    email: String,
    age: Option<u32>,
    preferences: UserPreferences,
}

#[derive(Serialize, StonehmSchema)]
struct UserResponse {
    id: u32,
    name: String,
    email: String,
    created_at: String,
    is_active: bool,
}

#[api_error]
enum ApiError {
    /// 400: Invalid input provided
    InvalidInput { field: String, message: String },
    
    /// 404: User not found
    UserNotFound { id: u32 },
    
    /// 409: Email already exists
    EmailAlreadyExists { email: String },
    
    /// 500: Internal server error
    DatabaseError,
    
    /// 422: Validation failed
    ValidationFailed,
}
```

**Supported types**: All primitive types, `Option<T>`, `Vec<T>`, nested structs, and enums.

## Router Setup

### Basic Setup

```rust
use stonehm::api_router;

#[tokio::main]
async fn main() {
    let app = api_router!("My API", "1.0.0")
        .get("/users/:id", get_user)
        .post("/users", create_user)
        .put("/users/:id", update_user)
        .delete("/users/:id", delete_user)
        .with_openapi_routes()  // Adds /openapi.json and /openapi.yaml
        .into_router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Custom OpenAPI Endpoints

```rust
// Default endpoints
.with_openapi_routes()  // Creates /openapi.json and /openapi.yaml

// Custom prefix
.with_openapi_routes_prefix("/api/docs")  // Creates /api/docs.json and /api/docs.yaml

// Custom paths
.with_openapi_routes_prefix("/v1/spec")   // Creates /v1/spec.json and /v1/spec.yaml
```

## Documentation Format Reference

### Summary and Description

```text
/// Brief one-line summary
///
/// Detailed description that can span multiple paragraphs.
/// This becomes the OpenAPI description field.
```

### Parameters Section

```text
/// # Parameters  
/// - id (path): The unique user identifier
/// - page (query): Page number for pagination
/// - limit (query): Maximum results per page  
/// - authorization (header): Bearer token for authentication
```

### Request Body Section

```text
/// # Request Body
/// Content-Type: application/json
/// Detailed description of the expected request body structure
/// and any validation requirements.
```

### Response Documentation

**Simple format** (covers most use cases):
```text
/// # Responses
/// - 200: User successfully created
/// - 400: Invalid user data provided
/// - 409: Email address already exists
```

**Elaborate format** (for detailed error documentation):
```text
/// # Responses
/// - 201: User successfully created
/// - 400:
///   description: Validation failed
///   content:
///     application/json:
///       schema: ValidationError
/// - 409:
///   description: Email already exists
///   content:
///     application/json:
///       schema: ConflictError
```

## Best Practices

### 1. Use Result Types for Error Handling

Return `Result<Json<T>, E>` to get automatic error responses:

```rust
/// Recommended - Automatic error handling
#[api_handler]
async fn get_user() -> Result<Json<User>, ApiError> {
    Ok(Json(User { id: 1, name: "John".to_string(), email: "john@example.com".to_string() }))
}

/// Manual - Requires explicit response documentation
#[api_handler]  
async fn get_user_manual() -> Json<User> {
    Json(User { id: 1, name: "John".to_string(), email: "john@example.com".to_string() })
}
```

**Generated OpenAPI for automatic error handling:**
```yaml
responses:
  '200':
    description: Success
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/User'
  '400':
    description: Bad Request
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/ApiError'
  '500':
    description: Internal Server Error
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/ApiError'
```

**Manual documentation requires explicit responses:**
```yaml
responses:
  '200':
    description: Success
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/User'
```

### 2. Use api_error Macro for Error Types

```rust
use stonehm_macros::api_error;

#[api_error]
enum ApiError {
    /// 404: User not found
    UserNotFound { id: u32 },
    
    /// 400: Validation failed
    ValidationError { field: String, message: String },
    
    /// 500: Internal server error
    DatabaseError,
}
```

The `api_error` macro automatically generates `IntoResponse`, `Serialize`, and `StonehmSchema` implementations, eliminating all boilerplate.

### 3. Keep Documentation Natural

Focus on business logic, not OpenAPI details:

```text
/// Good - describes what the endpoint does
/// Creates a new user account with email verification

/// Avoid - implementation details
/// Returns HTTP 201 with application/json content-type
```

### 4. Choose the Right Documentation Level

```text
/// Simple for basic APIs
/// # Responses
/// - 200: Success
/// - 400: Bad request

/// Elaborate for complex error handling
/// # Responses  
/// - 400:
///   description: Validation failed
///   content:
///     application/json:
///       schema: ValidationError
```

## Automatic vs Manual Response Documentation

| Return Type | Automatic Behavior | When to Use Manual |
|-------------|--------------------|--------------------|
| `Json<T>` | 200 response with T schema | Simple endpoints |
| `Result<Json<T>, E>` | 200 with T schema<br/>400, 500 with E schema | Most endpoints (recommended) |
| `()` or `StatusCode` | 200 empty response | DELETE operations |
| Custom types | Depends on implementation | Advanced use cases |

## Common Troubleshooting

**Q: My error responses aren't appearing**  
A: Ensure your function returns `Result<Json<T>, E>` and `E` implements `IntoResponse`.

**Q: Schemas aren't in the OpenAPI spec**  
A: Add `#[derive(StonehmSchema)]` to your types and use them in function signatures.

**Q: Path parameters not documented**  
A: Add them to the `# Parameters` section with `(path)` type specification.

**Q: Custom response schemas not working**  
A: Use the elaborate response format with explicit schema references.

## API Reference

### Macros

| Macro | Purpose | Example |
|-------|---------|---------|
| `api_router!(title, version)` | Create documented router | `api_router!("My API", "1.0.0")` |
| `#[api_handler]` | Mark handler for documentation | `#[api_handler] async fn get_user() {}` |
| `#[derive(StonehmSchema)]` | Generate JSON schema | `#[derive(Serialize, StonehmSchema)] struct User {}` |

### Router Methods

```rust
let app = api_router!("API", "1.0.0")
    .get("/users", list_users)           // GET route
    .post("/users", create_user)         // POST route  
    .put("/users/:id", update_user)      // PUT route
    .delete("/users/:id", delete_user)   // DELETE route
    .patch("/users/:id", patch_user)     // PATCH route
    .with_openapi_routes()               // Add OpenAPI endpoints
    .into_router();                      // Convert to axum::Router
```

### OpenAPI Endpoints

| Method | Creates | Description |
|--------|---------|-------------|
| `.with_openapi_routes()` | `/openapi.json`<br/>`/openapi.yaml` | Default OpenAPI endpoints |
| `.with_openapi_routes_prefix("/api")` | `/api.json`<br/>`/api.yaml` | Custom prefix |

### Response Type Mapping

| Rust Type | OpenAPI Response | Automatic Errors |
|-----------|------------------|------------------|
| `Json<T>` | 200 with T schema | None |
| `Result<Json<T>, E>` | 200 with T schema | 400, 500 with E schema |
| `()` | 204 No Content | None |
| `StatusCode` | Custom status | None |

## Examples

### Full REST API Example

```rust
use axum::{Json, extract::{Path, Query}};
use serde::{Serialize, Deserialize};
use stonehm::{api_router, api_handler};
use stonehm_macros::StonehmSchema;

#[derive(Serialize, Deserialize, StonehmSchema)]
struct User {
    id: u32,
    name: String,
    email: String,
    created_at: String,
}

#[derive(Deserialize, StonehmSchema)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct UserQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

#[api_error]
enum ApiError {
    /// 404: User not found
    UserNotFound { id: u32 },
    
    /// 400: Validation failed
    ValidationError { field: String, message: String },
    
    /// 500: Internal server error
    DatabaseError,
}

/// List users with pagination
///
/// Retrieves a paginated list of users from the database.
///
/// # Parameters
/// - page (query): Page number (default: 1)
/// - limit (query): Users per page (default: 10, max: 100)
#[api_handler]
async fn list_users(Query(query): Query<UserQuery>) -> Result<Json<Vec<User>>, ApiError> {
    Ok(Json(vec![]))
}

/// Get user by ID
///
/// Retrieves detailed user information by ID.
#[api_handler]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, ApiError> {
    Ok(Json(User {
        id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
    }))
}

/// Create new user
///
/// Creates a new user account with the provided information.
///
/// # Request Body
/// Content-Type: application/json
/// User creation data with required name and email fields.
#[api_handler]
async fn create_user(Json(req): Json<CreateUserRequest>) -> Result<Json<User>, ApiError> {
    Ok(Json(User {
        id: 42,
        name: req.name,
        email: req.email,
        created_at: "2024-01-01T00:00:00Z".to_string(),
    }))
}

#[tokio::main]
async fn main() {
    let app = api_router!("User Management API", "1.0.0")
        .get("/users", list_users)
        .get("/users/:id", get_user)
        .post("/users", create_user)
        .with_openapi_routes()
        .into_router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    println!("OpenAPI spec: http://127.0.0.1:3000/openapi.json");
    axum::serve(listener, app).await.unwrap();
}
```

## Development

### Running Examples

```bash
# Clone the repository
git clone https://github.com/melito/stonehm.git
cd stonehm

# Run the example server
cargo run -p hello_world

# Test OpenAPI generation
cargo run -p hello_world -- --test-schema

# Use default endpoints (/openapi.json, /openapi.yaml)
cargo run -p hello_world -- --default
```

### Testing Schema Generation

```bash
# Generate and view the OpenAPI spec
cargo run -p hello_world -- --test-schema | jq '.'

# Check specific endpoints
cargo run -p hello_world -- --test-schema | jq '.paths."/users".post'

# View all schemas
cargo run -p hello_world -- --test-schema | jq '.components.schemas'
```

## Architecture Deep Dive

### Compile-Time Registration System

stonehm uses the [`inventory`](https://crates.io/crates/inventory) crate to implement a compile-time registration system that collects API documentation and schema information from across your entire codebase.

#### How It Works

**1. Registration Structs**

stonehm defines two core registration types:

```rust
#[derive(Debug, Clone)]
pub struct HandlerDocumentation {
    pub function_name: &'static str,
    pub summary: &'static str,
    pub description: &'static str,
    pub parameters: &'static str,    // JSON string: ["id (path): User ID"]
    pub responses: &'static str,     // JSON string: ["200: Success", "404: Not found"]
    pub request_body: &'static str,  // JSON string: ["Type: CreateUserRequest"]
    pub tags: &'static str,          // JSON string: ["users", "admin"]
}

#[derive(Debug, Clone)]
pub struct SchemaRegistration {
    pub type_name: &'static str,
    pub schema_json: &'static str,   // OpenAPI JSON schema as string
}
```

**2. Collection Declaration**

```rust
inventory::collect!(HandlerDocumentation);
inventory::collect!(SchemaRegistration);
```

This tells the `inventory` crate to collect all submitted instances of these types from across your compiled program.

**3. Automatic Registration via Proc Macros**

When you use `#[api_handler]`, the macro analyzes your function and generates:

```rust
// For a function like:
#[api_handler]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, ApiError> { ... }

// The macro generates:
inventory::submit! {
    stonehm::HandlerDocumentation {
        function_name: "get_user",
        summary: "Get user by ID",
        description: "Retrieves user information using their unique identifier",
        parameters: "[\"id (path): User ID\"]",
        responses: "[\"200: Success\", \"400: Bad Request\", \"500: Internal Server Error\"]",
        request_body: "[]",
        tags: "[]",
    }
}
```

Similarly, `#[derive(StonehmSchema)]` generates schema registrations:

```rust
// For a struct like:
#[derive(Serialize, StonehmSchema)]
struct User {
    id: u32,
    name: String,
    email: String,
}

// The macro generates:
inventory::submit! {
    stonehm::SchemaRegistration {
        type_name: "User",
        schema_json: r#"{"type":"object","properties":{"id":{"type":"integer"},"name":{"type":"string"},"email":{"type":"string"}},"required":["id","name","email"]}"#,
    }
}
```

**4. Runtime Collection and OpenAPI Generation**

When your application starts and generates the OpenAPI specification:

```rust
// Collect all handler documentation
let handler_docs: HashMap<&str, &HandlerDocumentation> = inventory::iter::<HandlerDocumentation>()
    .map(|doc| (doc.function_name, doc))
    .collect();

// Collect all schema registrations  
let mut schemas = HashMap::new();
for schema_reg in inventory::iter::<SchemaRegistration>() {
    if self.used_schemas.contains(schema_reg.type_name) {
        schemas.insert(schema_reg.type_name.to_string(), schema_reg.schema_json.to_string());
    }
}
```

#### Key Constraints

**Compile-Time Constants Required**: The `inventory::submit!` macro requires all fields to be compile-time constants (`&'static str`). This means:

✅ **Works**: Static strings, string literals, `const` values
```rust
inventory::submit! {
    SchemaRegistration {
        type_name: "User",  // String literal
        schema_json: USER_SCHEMA,  // const value
    }
}
```

❌ **Doesn't Work**: Function calls, dynamic strings, heap-allocated strings
```rust
inventory::submit! {
    SchemaRegistration {
        type_name: "User",
        schema_json: generate_schema(),  // Function call - ERROR!
    }
}
```

#### Zero Runtime Overhead

This system provides **zero runtime overhead** because:
- All registration happens at compile time
- The `inventory::iter()` calls simply iterate over a pre-built static registry
- No heap allocations or dynamic lookups during OpenAPI generation
- Unused schemas are automatically detected and excluded

#### Schema Usage Tracking

stonehm automatically tracks which schemas are actually used in your API:

```rust
// Only include schemas that are referenced in handler signatures
let used_schemas: HashSet<String> = inventory::iter::<SchemaRegistration>()
    .map(|reg| reg.type_name.to_string())
    .collect();

// Warn about unused schemas in development
let registered_schemas: HashSet<String> = inventory::iter::<SchemaRegistration>()
    .map(|reg| reg.type_name.to_string())
    .collect();

for unused in registered_schemas.difference(&used_schemas) {
    log::warn!("Schema '{}' is registered but not used in any handlers", unused);
}
```

### Extending stonehm

#### Adding Custom Schema Types

To add support for new schema types, you need to understand the compile-time constraint:

```rust
// ❌ This won't work because of dynamic generation
fn register_dynamic_schema() {
    let schema = generate_complex_schema(); // Dynamic
    inventory::submit! {
        SchemaRegistration {
            type_name: "DynamicType",
            schema_json: &schema,  // ERROR: not &'static str
        }
    }
}

// ✅ This works with static constants
const CUSTOM_SCHEMA: &str = r#"{"type":"object","properties":{"custom":{"type":"string"}}}"#;

inventory::submit! {
    SchemaRegistration {
        type_name: "CustomType", 
        schema_json: CUSTOM_SCHEMA,
    }
}
```

#### Working with Dynamic Schemas

For dynamic schema generation (like complex enum support), you have three options:

1. **Generate at Compile Time**: Use proc macros to generate static strings
2. **Embed in Router**: Handle dynamic schemas in the `ApiRouter` itself
3. **Hybrid Approach**: Use static placeholders and runtime replacement

Example hybrid approach:
```rust
// Register placeholder at compile time
inventory::submit! {
    SchemaRegistration {
        type_name: "HttpAuthConfig",
        schema_json: "__DYNAMIC_HTTP_AUTH_CONFIG__",  // Placeholder
    }
}

// Replace at runtime during OpenAPI generation
fn generate_openapi_spec(&self) -> String {
    let mut spec = self.collect_all_schemas();
    spec = spec.replace("__DYNAMIC_HTTP_AUTH_CONFIG__", &self.generate_http_auth_schema());
    spec
}
```

## Contributing

We welcome contributions! Please feel free to submit issues and pull requests.

### Development Setup

```bash
# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Test all examples
cargo test --workspace
```

### Understanding the Codebase

Key files to understand:
- `src/lib.rs` - Core `ApiRouter` and inventory system  
- `stonehm-macros/src/lib.rs` - `#[api_handler]` and `#[derive(StonehmSchema)]` macros
- `examples/hello_world/` - Complete working example

The inventory system is the heart of stonehm - understanding how `inventory::submit!` and `inventory::iter()` work is crucial for contributing.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**[Documentation](https://docs.rs/stonehm) | [Crates.io](https://crates.io/crates/stonehm) | [Repository](https://github.com/melito/stonehm)**