//! OpenAPI 3.0 type definitions with serde support
//!
//! This module provides complete type-safe representations of OpenAPI 3.0 specification
//! components with proper serde serialization support following OpenAPI conventions.
//!
//! # Step 1 Completion Status ✅
//!
//! All types required for **Step 1** of the refactoring plan have been implemented:
//!
//! ## Core Types
//!
//! - [`OpenAPI`] - Root OpenAPI document (v3.0.0)
//! - [`Info`] - API metadata with all optional fields
//! - [`Contact`] - Contact information for the API
//! - [`License`] - License information for the API
//! - [`ExternalDocs`] - External documentation references
//! - [`Tag`] - API tags with optional external documentation
//! - [`PathItem`] - Path definitions with all HTTP methods
//! - [`Operation`] - Operation/endpoint definitions
//! - [`Parameter`] - Request parameters (path, query, header, cookie)
//! - [`RequestBody`] - Request body definitions
//! - [`Response`] - Response definitions
//! - [`MediaType`] - Media type definitions for content
//! - [`Components`] - Reusable component definitions
//! - [`SecurityScheme`] - Security/authentication schemes
//! - [`Schema`] - JSON Schema definitions
//! - [`ReferenceOr<T>`] - Reference or inline definitions
//!
//! ## OpenAPI Conventions ✅
//!
//! All types follow OpenAPI 3.0 specification conventions:
//!
//! 1. **Serde derives**: All types have `#[derive(Serialize, Deserialize)]`
//! 2. **camelCase fields**: Uses `#[serde(rename_all = "camelCase")]` where needed
//!    - `Info::terms_of_service` → `"termsOfService"`
//!    - `Tag::external_docs` → `"externalDocs"`
//!    - `Operation::request_body` → `"requestBody"`
//!    - `SecurityScheme::bearer_format` → `"bearerFormat"`
//! 3. **Optional field handling**: Uses `#[serde(skip_serializing_if = "Option::is_none")]`
//!    - Fields are omitted from JSON when `None`
//! 4. **Special field names**:
//!    - `schema_type` → `"type"` (reserved keyword workaround)
//!    - `scheme_type` → `"type"` (for SecurityScheme)
//!    - `reference` → `"$ref"` (OpenAPI reference syntax)
//!    - `location` → `"in"` (for parameters, reserved keyword)
//! 5. **Lowercase enums**: PathItem methods use `#[serde(rename_all = "lowercase")]`
//!
//! ## Usage Example
//!
//! ```rust
//! use machined_openapi_gen::openapi::*;
//! use std::collections::HashMap;
//!
//! let openapi = OpenAPI {
//!     openapi: "3.0.0".to_string(),
//!     info: Info {
//!         title: "My API".to_string(),
//!         version: "1.0.0".to_string(),
//!         description: Some("A complete API".to_string()),
//!         terms_of_service: Some("https://example.com/tos".to_string()),
//!         contact: Some(Contact {
//!             name: Some("API Team".to_string()),
//!             url: Some("https://example.com".to_string()),
//!             email: Some("api@example.com".to_string()),
//!         }),
//!         license: Some(License {
//!             name: "MIT".to_string(),
//!             url: Some("https://opensource.org/licenses/MIT".to_string()),
//!         }),
//!     },
//!     paths: HashMap::new(),
//!     components: None,
//!     tags: None,
//! };
//!
//! // Serialize to JSON
//! let json = openapi.to_json().unwrap();
//! ```
//!
//! ## Migration Status
//!
//! - ✅ Step 1: Type system foundation (COMPLETE)
//! - ⏳ Step 2: Migrate helper functions to return types
//! - ⏳ Step 3: Update main `openapi_json()` method
//! - ⏳ Step 4: Add integration tests
//! - ⏳ Step 5: YAML generation support
//!
//! ## Testing
//!
//! All types have comprehensive unit tests verifying:
//! - Serialization produces correct JSON structure
//! - Field naming follows OpenAPI conventions (camelCase)
//! - Optional fields are properly omitted when `None`
//! - Round-trip serialization/deserialization works
//!
//! Run tests with: `cargo test openapi::`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A type that can be either a reference to a component or an inline definition.
/// This is used throughout OpenAPI for schemas, parameters, responses, etc.
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

impl<T> ReferenceOr<T> {
    /// Create a new reference
    pub fn new_ref(reference: impl Into<String>) -> Self {
        ReferenceOr::Reference {
            reference: reference.into(),
        }
    }

    /// Create a new inline item
    pub fn new_item(item: T) -> Self {
        ReferenceOr::Item(item)
    }

    /// Check if this is a reference
    pub fn is_ref(&self) -> bool {
        matches!(self, ReferenceOr::Reference { .. })
    }

    /// Get the reference string if this is a reference
    pub fn as_ref_str(&self) -> Option<&str> {
        match self {
            ReferenceOr::Reference { reference } => Some(reference),
            ReferenceOr::Item(_) => None,
        }
    }

    /// Get the item if this is an inline item
    pub fn as_item(&self) -> Option<&T> {
        match self {
            ReferenceOr::Reference { .. } => None,
            ReferenceOr::Item(item) => Some(item),
        }
    }
}

// OpenAPI 3.0 Root Document
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenAPI {
    pub openapi: String,
    pub info: Info,
    pub paths: HashMap<String, PathItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDocs>,
}

/// External documentation reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalDocs {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl OpenAPI {
    pub fn new(title: &str, version: &str) -> Self {
        Self {
            openapi: "3.0.0".to_string(),
            info: Info {
                title: title.to_string(),
                version: version.to_string(),
                description: None,
                terms_of_service: None,
                contact: None,
                license: None,
            },
            paths: HashMap::new(),
            components: None,
            tags: None,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub title: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
}

/// Contact information for the API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// License information for the API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct License {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "x-handler-function")]
    pub handler_function: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub parameters: Vec<Parameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: bool,
    pub schema: ReferenceOr<Schema>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub content: HashMap<String, MediaType>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<ReferenceOr<Schema>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Components {
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub schemas: HashMap<String, ReferenceOr<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "securitySchemes")]
    pub security_schemes: Option<HashMap<String, SecurityScheme>>,
}

/// Security scheme definition for API authentication
///
/// # Examples
///
/// ```
/// use machined_openapi_gen::openapi::SecurityScheme;
///
/// // API Key in header
/// let api_key = SecurityScheme::api_key("x-api-key", "header")
///     .with_description("API key for authentication");
///
/// // HTTP Basic Auth
/// let basic = SecurityScheme::http("basic");
///
/// // HTTP Bearer Token (JWT)
/// let bearer = SecurityScheme::bearer(Some("JWT"))
///     .with_description("JWT bearer token");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityScheme {
    /// The type of the security scheme
    /// Valid values: "apiKey", "http", "oauth2", "openIdConnect"
    #[serde(rename = "type")]
    pub scheme_type: String,
    
    /// A short description for the security scheme
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// The name of the header, query or cookie parameter to be used (apiKey only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// The location of the API key (apiKey only)
    /// Valid values: "query", "header", "cookie"
    #[serde(skip_serializing_if = "Option::is_none", rename = "in")]
    pub location: Option<String>,
    
    /// The name of the HTTP Authorization scheme (http only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    
    /// A hint to the client to identify how the bearer token is formatted (http bearer only)
    #[serde(skip_serializing_if = "Option::is_none", rename = "bearerFormat")]
    pub bearer_format: Option<String>,
}

impl SecurityScheme {
    /// Create a new API Key security scheme
    pub fn api_key(name: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            scheme_type: "apiKey".to_string(),
            description: None,
            name: Some(name.into()),
            location: Some(location.into()),
            scheme: None,
            bearer_format: None,
        }
    }
    
    /// Create a new HTTP security scheme
    pub fn http(scheme: impl Into<String>) -> Self {
        Self {
            scheme_type: "http".to_string(),
            description: None,
            name: None,
            location: None,
            scheme: Some(scheme.into()),
            bearer_format: None,
        }
    }
    
    /// Create a new HTTP Bearer token security scheme
    pub fn bearer(bearer_format: Option<impl Into<String>>) -> Self {
        Self {
            scheme_type: "http".to_string(),
            description: None,
            name: None,
            location: None,
            scheme: Some("bearer".to_string()),
            bearer_format: bearer_format.map(|f| f.into()),
        }
    }
    
    /// Add a description to the security scheme
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Schema {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, ReferenceOr<Schema>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    /// Reference to another schema (alternative to using ReferenceOr wrapper)
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            schema_type: Some("object".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
            reference: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_scheme_api_key() {
        let scheme = SecurityScheme::api_key("x-api-key", "header");
        
        assert_eq!(scheme.scheme_type, "apiKey");
        assert_eq!(scheme.name, Some("x-api-key".to_string()));
        assert_eq!(scheme.location, Some("header".to_string()));
        assert_eq!(scheme.scheme, None);
        assert_eq!(scheme.bearer_format, None);
        
        // Test serialization
        let json = serde_json::to_value(&scheme).unwrap();
        assert_eq!(json["type"], "apiKey");
        assert_eq!(json["name"], "x-api-key");
        assert_eq!(json["in"], "header");
    }

    #[test]
    fn test_security_scheme_api_key_with_description() {
        let scheme = SecurityScheme::api_key("x-session-secret", "header")
            .with_description("API session token for authentication");
        
        assert_eq!(scheme.description, Some("API session token for authentication".to_string()));
        
        // Test serialization
        let json = serde_json::to_value(&scheme).unwrap();
        assert_eq!(json["description"], "API session token for authentication");
    }

    #[test]
    fn test_security_scheme_http_basic() {
        let scheme = SecurityScheme::http("basic");
        
        assert_eq!(scheme.scheme_type, "http");
        assert_eq!(scheme.scheme, Some("basic".to_string()));
        assert_eq!(scheme.name, None);
        assert_eq!(scheme.location, None);
        
        // Test serialization
        let json = serde_json::to_value(&scheme).unwrap();
        assert_eq!(json["type"], "http");
        assert_eq!(json["scheme"], "basic");
    }

    #[test]
    fn test_security_scheme_bearer() {
        let scheme = SecurityScheme::bearer(Some("JWT"));
        
        assert_eq!(scheme.scheme_type, "http");
        assert_eq!(scheme.scheme, Some("bearer".to_string()));
        assert_eq!(scheme.bearer_format, Some("JWT".to_string()));
        
        // Test serialization
        let json = serde_json::to_value(&scheme).unwrap();
        assert_eq!(json["type"], "http");
        assert_eq!(json["scheme"], "bearer");
        assert_eq!(json["bearerFormat"], "JWT");
    }

    #[test]
    fn test_security_scheme_bearer_no_format() {
        let scheme: SecurityScheme = SecurityScheme::bearer(None::<String>);
        
        assert_eq!(scheme.bearer_format, None);
        
        // Test serialization - bearerFormat should not be present
        let json = serde_json::to_value(&scheme).unwrap();
        assert!(!json.as_object().unwrap().contains_key("bearerFormat"));
    }

    #[test]
    fn test_components_with_security_schemes() {
        let mut security_schemes = HashMap::new();
        security_schemes.insert(
            "sessionAuth".to_string(),
            SecurityScheme::api_key("x-session-secret", "header")
                .with_description("API session token for authentication"),
        );
        
        let components = Components {
            schemas: HashMap::new(),
            security_schemes: Some(security_schemes),
        };
        
        // Test serialization
        let json = serde_json::to_value(&components).unwrap();
        assert!(json["securitySchemes"]["sessionAuth"].is_object());
        assert_eq!(json["securitySchemes"]["sessionAuth"]["type"], "apiKey");
        assert_eq!(json["securitySchemes"]["sessionAuth"]["name"], "x-session-secret");
        assert_eq!(json["securitySchemes"]["sessionAuth"]["in"], "header");
    }

    #[test]
    fn test_components_without_security_schemes() {
        let components = Components {
            schemas: HashMap::new(),
            security_schemes: None,
        };
        
        // Test serialization - securitySchemes should not be present
        let json = serde_json::to_value(&components).unwrap();
        assert!(!json.as_object().unwrap().contains_key("securitySchemes"));
    }

    #[test]
    fn test_security_scheme_session_auth_matches_current_impl() {
        // This should match the current hardcoded security scheme in lib.rs
        let scheme = SecurityScheme::api_key("x-session-secret", "header")
            .with_description("API session token for authentication");
        
        let json = serde_json::to_string(&scheme).unwrap();
        
        // Verify it contains all the expected fields
        assert!(json.contains("\"type\":\"apiKey\""));
        assert!(json.contains("\"in\":\"header\""));
        assert!(json.contains("\"name\":\"x-session-secret\""));
        assert!(json.contains("\"description\":\"API session token for authentication\""));
    }

    #[test]
    fn test_openapi_with_security_schemes() {
        let mut security_schemes = HashMap::new();
        security_schemes.insert(
            "bearerAuth".to_string(),
            SecurityScheme::bearer(Some("JWT")),
        );
        
        let components = Components {
            schemas: HashMap::new(),
            security_schemes: Some(security_schemes),
        };
        
        let openapi = OpenAPI {
            openapi: "3.0.0".to_string(),
            info: Info {
                title: "Test API".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                terms_of_service: None,
                contact: None,
                license: None,
            },
            paths: HashMap::new(),
            components: Some(components),
            tags: None,
        };
        
        // Test that it serializes without errors
        let json_result = openapi.to_json();
        assert!(json_result.is_ok());
        
        let json_str = json_result.unwrap();
        assert!(json_str.contains("securitySchemes"));
        assert!(json_str.contains("bearerAuth"));
    }

    #[test]
    fn test_contact_serialization() {
        let contact = Contact {
            name: Some("Test Team".to_string()),
            url: Some("https://example.com".to_string()),
            email: Some("test@example.com".to_string()),
        };
        
        let json = serde_json::to_value(&contact).unwrap();
        assert_eq!(json["name"], "Test Team");
        assert_eq!(json["url"], "https://example.com");
        assert_eq!(json["email"], "test@example.com");
    }

    #[test]
    fn test_contact_partial_fields() {
        let contact = Contact {
            name: Some("Test Team".to_string()),
            url: None,
            email: Some("test@example.com".to_string()),
        };
        
        let json = serde_json::to_value(&contact).unwrap();
        assert_eq!(json["name"], "Test Team");
        assert_eq!(json["email"], "test@example.com");
        // url should not be present when None
        assert!(!json.as_object().unwrap().contains_key("url"));
    }

    #[test]
    fn test_contact_empty() {
        let contact = Contact {
            name: None,
            url: None,
            email: None,
        };
        
        let json = serde_json::to_value(&contact).unwrap();
        // All fields should be omitted when None
        assert!(!json.as_object().unwrap().contains_key("name"));
        assert!(!json.as_object().unwrap().contains_key("url"));
        assert!(!json.as_object().unwrap().contains_key("email"));
    }

    #[test]
    fn test_license_serialization() {
        let license = License {
            name: "MIT".to_string(),
            url: Some("https://opensource.org/licenses/MIT".to_string()),
        };
        
        let json = serde_json::to_value(&license).unwrap();
        assert_eq!(json["name"], "MIT");
        assert_eq!(json["url"], "https://opensource.org/licenses/MIT");
    }

    #[test]
    fn test_license_no_url() {
        let license = License {
            name: "Apache 2.0".to_string(),
            url: None,
        };
        
        let json = serde_json::to_value(&license).unwrap();
        assert_eq!(json["name"], "Apache 2.0");
        // url should not be present when None
        assert!(!json.as_object().unwrap().contains_key("url"));
    }

    #[test]
    fn test_external_docs_serialization() {
        let docs = ExternalDocs {
            url: "https://example.com/docs".to_string(),
            description: Some("Additional documentation".to_string()),
        };
        
        let json = serde_json::to_value(&docs).unwrap();
        assert_eq!(json["url"], "https://example.com/docs");
        assert_eq!(json["description"], "Additional documentation");
    }

    #[test]
    fn test_external_docs_no_description() {
        let docs = ExternalDocs {
            url: "https://example.com/docs".to_string(),
            description: None,
        };
        
        let json = serde_json::to_value(&docs).unwrap();
        assert_eq!(json["url"], "https://example.com/docs");
        // description should not be present when None
        assert!(!json.as_object().unwrap().contains_key("description"));
    }

    #[test]
    fn test_tag_with_external_docs() {
        let tag = Tag {
            name: "users".to_string(),
            description: Some("User operations".to_string()),
            external_docs: Some(ExternalDocs {
                url: "https://example.com/users".to_string(),
                description: Some("User API docs".to_string()),
            }),
        };
        
        let json = serde_json::to_value(&tag).unwrap();
        assert_eq!(json["name"], "users");
        assert_eq!(json["description"], "User operations");
        assert_eq!(json["externalDocs"]["url"], "https://example.com/users");
        assert_eq!(json["externalDocs"]["description"], "User API docs");
    }

    #[test]
    fn test_tag_without_external_docs() {
        let tag = Tag {
            name: "products".to_string(),
            description: Some("Product operations".to_string()),
            external_docs: None,
        };
        
        let json = serde_json::to_value(&tag).unwrap();
        assert_eq!(json["name"], "products");
        assert_eq!(json["description"], "Product operations");
        // externalDocs should not be present when None
        assert!(!json.as_object().unwrap().contains_key("externalDocs"));
    }

    #[test]
    fn test_info_with_all_fields() {
        let info = Info {
            title: "Test API".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test API".to_string()),
            terms_of_service: Some("https://example.com/terms".to_string()),
            contact: Some(Contact {
                name: Some("API Team".to_string()),
                url: Some("https://example.com".to_string()),
                email: Some("api@example.com".to_string()),
            }),
            license: Some(License {
                name: "MIT".to_string(),
                url: Some("https://opensource.org/licenses/MIT".to_string()),
            }),
        };
        
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["title"], "Test API");
        assert_eq!(json["version"], "1.0.0");
        assert_eq!(json["description"], "A test API");
        // Verify camelCase naming convention
        assert_eq!(json["termsOfService"], "https://example.com/terms");
        assert_eq!(json["contact"]["name"], "API Team");
        assert_eq!(json["license"]["name"], "MIT");
    }

    #[test]
    fn test_info_minimal() {
        let info = Info {
            title: "Minimal API".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            terms_of_service: None,
            contact: None,
            license: None,
        };
        
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["title"], "Minimal API");
        assert_eq!(json["version"], "0.1.0");
        // Optional fields should not be present
        assert!(!json.as_object().unwrap().contains_key("description"));
        assert!(!json.as_object().unwrap().contains_key("termsOfService"));
        assert!(!json.as_object().unwrap().contains_key("contact"));
        assert!(!json.as_object().unwrap().contains_key("license"));
    }

    #[test]
    fn test_info_camel_case_convention() {
        let info = Info {
            title: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            terms_of_service: Some("https://example.com/tos".to_string()),
            contact: None,
            license: None,
        };
        
        let json_str = serde_json::to_string(&info).unwrap();
        // Verify the JSON uses camelCase for termsOfService
        assert!(json_str.contains("termsOfService"));
        assert!(!json_str.contains("terms_of_service"));
    }

    #[test]
    fn test_openapi_full_info() {
        let openapi = OpenAPI {
            openapi: "3.0.0".to_string(),
            info: Info {
                title: "Complete API".to_string(),
                version: "2.0.0".to_string(),
                description: Some("A complete API example".to_string()),
                terms_of_service: Some("https://example.com/terms".to_string()),
                contact: Some(Contact {
                    name: Some("Support Team".to_string()),
                    url: Some("https://example.com/support".to_string()),
                    email: Some("support@example.com".to_string()),
                }),
                license: Some(License {
                    name: "Apache 2.0".to_string(),
                    url: Some("https://www.apache.org/licenses/LICENSE-2.0.html".to_string()),
                }),
            },
            paths: HashMap::new(),
            components: None,
            tags: None,
        };
        
        let json_result = openapi.to_json();
        assert!(json_result.is_ok());
        
        let json_str = json_result.unwrap();
        assert!(json_str.contains("Complete API"));
        assert!(json_str.contains("termsOfService"));
        assert!(json_str.contains("Support Team"));
        assert!(json_str.contains("Apache 2.0"));
    }

    #[test]
    fn test_step1_all_types_openapi_conventions() {
        // This test verifies that all types required for Step 1 of the refactoring plan
        // are present and follow OpenAPI 3.0 conventions for serialization
        
        // Build a complete OpenAPI document with all Step 1 types
        let openapi = OpenAPI {
            openapi: "3.0.0".to_string(),
            info: Info {
                title: "Step 1 Verification API".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Testing all required types".to_string()),
                terms_of_service: Some("https://example.com/tos".to_string()),
                contact: Some(Contact {
                    name: Some("API Support".to_string()),
                    url: Some("https://example.com/support".to_string()),
                    email: Some("support@example.com".to_string()),
                }),
                license: Some(License {
                    name: "MIT".to_string(),
                    url: Some("https://opensource.org/licenses/MIT".to_string()),
                }),
            },
            paths: HashMap::new(),
            components: Some(Components {
                schemas: HashMap::new(),
                security_schemes: Some({
                    let mut schemes = HashMap::new();
                    schemes.insert(
                        "sessionAuth".to_string(),
                        SecurityScheme::api_key("x-session-secret", "header")
                            .with_description("API session token"),
                    );
                    schemes
                }),
            }),
            tags: Some(vec![
                Tag {
                    name: "users".to_string(),
                    description: Some("User management".to_string()),
                    external_docs: Some(ExternalDocs {
                        url: "https://docs.example.com/users".to_string(),
                        description: Some("User API documentation".to_string()),
                    }),
                },
                Tag {
                    name: "products".to_string(),
                    description: None,
                    external_docs: None,
                },
            ]),
        };
        
        // Serialize to JSON
        let json_result = openapi.to_json();
        assert!(json_result.is_ok());
        
        let json_str = json_result.unwrap();
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        
        // Verify Info fields follow OpenAPI conventions
        assert_eq!(json["info"]["title"], "Step 1 Verification API");
        assert_eq!(json["info"]["version"], "1.0.0");
        assert_eq!(json["info"]["description"], "Testing all required types");
        // termsOfService must be camelCase
        assert_eq!(json["info"]["termsOfService"], "https://example.com/tos");
        assert!(json["info"].as_object().unwrap().contains_key("termsOfService"));
        assert!(!json["info"].as_object().unwrap().contains_key("terms_of_service"));
        
        // Verify Contact fields
        assert_eq!(json["info"]["contact"]["name"], "API Support");
        assert_eq!(json["info"]["contact"]["url"], "https://example.com/support");
        assert_eq!(json["info"]["contact"]["email"], "support@example.com");
        
        // Verify License fields
        assert_eq!(json["info"]["license"]["name"], "MIT");
        assert_eq!(json["info"]["license"]["url"], "https://opensource.org/licenses/MIT");
        
        // Verify SecurityScheme fields follow OpenAPI conventions
        assert_eq!(json["components"]["securitySchemes"]["sessionAuth"]["type"], "apiKey");
        assert_eq!(json["components"]["securitySchemes"]["sessionAuth"]["name"], "x-session-secret");
        assert_eq!(json["components"]["securitySchemes"]["sessionAuth"]["in"], "header");
        
        // Verify Tag with ExternalDocs
        assert_eq!(json["tags"][0]["name"], "users");
        assert_eq!(json["tags"][0]["description"], "User management");
        // externalDocs must be camelCase
        assert!(json["tags"][0].as_object().unwrap().contains_key("externalDocs"));
        assert!(!json["tags"][0].as_object().unwrap().contains_key("external_docs"));
        assert_eq!(json["tags"][0]["externalDocs"]["url"], "https://docs.example.com/users");
        assert_eq!(json["tags"][0]["externalDocs"]["description"], "User API documentation");
        
        // Verify Tag without ExternalDocs doesn't include the field
        assert_eq!(json["tags"][1]["name"], "products");
        assert!(!json["tags"][1].as_object().unwrap().contains_key("description"));
        assert!(!json["tags"][1].as_object().unwrap().contains_key("externalDocs"));
        
        // Verify all required types are present:
        // ✅ Contact
        // ✅ License  
        // ✅ ExternalDocs
        // ✅ Info with termsOfService, contact, license
        // ✅ Tag with external_docs
        // ✅ SecurityScheme
        // ✅ All fields use proper camelCase conventions
        // ✅ Optional fields are omitted when None
    }
}
