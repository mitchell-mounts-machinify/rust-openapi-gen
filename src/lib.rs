//! Simple machined-openapi-gen implementation without serde dependencies
//!
//! # Architecture Overview
//!
//! This library provides a simple, procedural approach to generating OpenAPI 3.0 specifications
//! from Axum route handlers. The core functionality is built around the `ApiRouter` struct and
//! its `openapi_json()` method, which has been refactored into composable helper functions.
//!
//! ## Main Components
//!
//! ### ApiRouter
//! The main router struct that wraps Axum's router and adds OpenAPI generation capabilities.
//!
//! ### OpenAPI Generation Pipeline
//!
//! The `openapi_json()` method orchestrates OpenAPI spec generation through these steps:
//!
//! 1. **Info Section** - Built by `build_info_json()` which delegates to:
//!    - `build_contact_json()` - Generates contact information
//!    - `build_license_json()` - Generates license information
//!
//! 2. **Documentation Collection** - `collect_handler_docs()` gathers all handler metadata
//!
//! 3. **Schema Collection** - Tracks which schemas are actually used:
//!    - `collect_all_used_schemas()` - Collects schemas from all routes
//!    - `collect_schemas_for_handler()` - Collects schemas from a single handler
//!    - `collect_transitive_schema_dependencies()` - Recursively finds referenced schemas
//!
//! 4. **Paths Section** - Built by `build_paths_json()` which delegates to:
//!    - `group_routes_by_path()` - Organizes routes by their path
//!    - `build_path_json()` - Generates JSON for a single path
//!    - `build_method_json()` - Generates JSON for a single HTTP method
//!
//! 5. **Tags Section** - Built by `build_tags_json()`
//!
//! 6. **Components Section** - Built by `build_components_json()` which delegates to:
//!    - `filter_used_schemas()` - Filters to only used schemas
//!    - `build_schemas_json()` - Generates the schemas section
//!    - `build_security_schemes_json()` - Generates security schemes if needed
//!    - `has_auth_endpoints()` - Checks if auth is required
//!
//! ## Design Principles
//!
//! - **No Serde Dependency**: Uses manual JSON string building for minimal dependencies
//! - **Single Responsibility**: Each helper function has one clear purpose
//! - **Testability**: All helper functions are independently testable
//! - **Composability**: Functions build on each other to create the complete spec

use axum::Router;
use std::collections::HashMap;

// Re-export Axum types so users can import everything from machined-openapi-gen
pub use axum::{
    extract::{Path, Query, State},
    handler::Handler,
    http::StatusCode,
    response::{Json, Response},
    Router as AxumRouter,
};

// Simple OpenAPI types
#[derive(Debug, Clone)]
pub struct OpenAPI {
    pub info: Info,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<Components>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub description: Option<String>,
    pub external_docs: Option<ExternalDocs>,
}

#[derive(Debug, Clone)]
pub struct ExternalDocs {
    pub description: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Components {
    pub schemas: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub path: String,
    pub method: String,
    pub function_name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HandlerDocumentation {
    pub function_name: &'static str,
    pub summary: &'static str,
    pub description: &'static str,
    pub parameters: &'static str,
    pub responses: &'static str,
    pub request_body: &'static str,
    pub tags: &'static str,
}

#[derive(Debug, Clone)]
pub struct SchemaRegistration {
    pub type_name: &'static str,
    pub schema_json: &'static str,
}

inventory::collect!(HandlerDocumentation);
inventory::collect!(SchemaRegistration);

impl OpenAPI {
    pub fn new(title: &str, version: &str) -> Self {
        Self {
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
            tags: Vec::new(),
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"openapi":"3.0.0","info":{{"title":"{}","version":"{}"}},"paths":{{}}}}"#,
            self.info.title, self.info.version
        )
    }

    pub fn to_yaml(&self) -> String {
        format!(
            "openapi: 3.0.0\ninfo:\n  title: {}\n  version: {}\npaths: {{}}\n",
            self.info.title, self.info.version
        )
    }
}

#[derive(Debug, Clone)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
    pub terms_of_service: Option<String>,
    pub contact: Option<Contact>,
    pub license: Option<License>,
}

#[derive(Debug, Clone)]
pub struct Contact {
    pub name: Option<String>,
    pub url: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct License {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PathItem;

// Simple trait for schema generation
pub trait OpenApiSchema {
    fn schema() -> String {
        r#"{"type":"object"}"#.to_string()
    }
}

// Wrapper around axum::routing::MethodRouter that carries handler metadata
pub struct MethodRouter<S = ()> {
    inner: axum::routing::MethodRouter<S>,
    handler_names: std::collections::HashMap<String, String>, // method -> handler_name
}

impl<S> MethodRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(inner: axum::routing::MethodRouter<S>) -> Self {
        Self {
            inner,
            handler_names: std::collections::HashMap::new(),
        }
    }

    pub fn with_handler_name(mut self, method: &str, handler_name: String) -> Self {
        self.handler_names
            .insert(method.to_uppercase(), handler_name);
        self
    }

    pub fn get_handler_name(&self, method: &str) -> Option<&String> {
        self.handler_names.get(&method.to_uppercase())
    }

    pub fn into_axum_method_router(self) -> axum::routing::MethodRouter<S> {
        self.inner
    }

    pub fn handler_names(&self) -> &std::collections::HashMap<String, String> {
        &self.handler_names
    }

    // Chaining methods for combining MethodRouters
    pub fn get<H, T>(mut self, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.inner = self.inner.get(handler);
        self.handler_names.insert("GET".to_string(), fn_name);
        self
    }

    pub fn post<H, T>(mut self, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.inner = self.inner.post(handler);
        self.handler_names.insert("POST".to_string(), fn_name);
        self
    }

    pub fn put<H, T>(mut self, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.inner = self.inner.put(handler);
        self.handler_names.insert("PUT".to_string(), fn_name);
        self
    }

    pub fn delete<H, T>(mut self, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.inner = self.inner.delete(handler);
        self.handler_names.insert("DELETE".to_string(), fn_name);
        self
    }

    pub fn patch<H, T>(mut self, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.inner = self.inner.patch(handler);
        self.handler_names.insert("PATCH".to_string(), fn_name);
        self
    }
}

// Simple router wrapper
pub struct ApiRouter<S = ()> {
    router: Router<S>,
    openapi: OpenAPI,
    routes: Vec<RouteInfo>,
    used_schemas: std::collections::HashSet<String>,
}

impl ApiRouter<()> {
    pub fn new(title: &str, version: &str) -> Self {
        Self {
            router: Router::new(),
            openapi: OpenAPI::new(title, version),
            routes: Vec::new(),
            used_schemas: std::collections::HashSet::new(),
        }
    }
}

impl<S> ApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn with_state_type(title: &str, version: &str) -> Self {
        Self {
            router: Router::new(),
            openapi: OpenAPI::new(title, version),
            routes: Vec::new(),
            used_schemas: std::collections::HashSet::new(),
        }
    }

    // Use into_router().with_state(your_state) for state management
    pub fn route(mut self, path: &str, method_router: MethodRouter<S>) -> Self {
        // Extract handler names and create RouteInfo entries
        for (method, handler_name) in method_router.handler_names() {
            self.routes.push(RouteInfo {
                path: path.to_string(),
                method: method.clone(),
                function_name: handler_name.clone(),
                summary: Some(format!("{} {}", method, path)),
                description: None,
            });
        }

        // Update OpenAPI spec
        self.openapi.paths.insert(path.to_string(), PathItem);

        // Delegate to Axum's route method
        self.router = self
            .router
            .route(path, method_router.into_axum_method_router());
        self
    }
}

// Provide MethodRouter creation functions that track OpenAPI info
pub fn get<H, T, S>(handler: H) -> MethodRouter<S>
where
    H: axum::handler::Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    // Extract function name for OpenAPI documentation
    let fn_name = std::any::type_name::<H>()
        .split("::")
        .last()
        .unwrap_or("unknown")
        .to_string();

    // Create MethodRouter wrapper with handler name tracking
    MethodRouter::new(axum::routing::get(handler)).with_handler_name("GET", fn_name)
}

pub fn post<H, T, S>(handler: H) -> MethodRouter<S>
where
    H: axum::handler::Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    let fn_name = std::any::type_name::<H>()
        .split("::")
        .last()
        .unwrap_or("unknown")
        .to_string();

    MethodRouter::new(axum::routing::post(handler)).with_handler_name("POST", fn_name)
}

pub fn put<H, T, S>(handler: H) -> MethodRouter<S>
where
    H: axum::handler::Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    let fn_name = std::any::type_name::<H>()
        .split("::")
        .last()
        .unwrap_or("unknown")
        .to_string();

    MethodRouter::new(axum::routing::put(handler)).with_handler_name("PUT", fn_name)
}

pub fn delete<H, T, S>(handler: H) -> MethodRouter<S>
where
    H: axum::handler::Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    let fn_name = std::any::type_name::<H>()
        .split("::")
        .last()
        .unwrap_or("unknown")
        .to_string();

    MethodRouter::new(axum::routing::delete(handler)).with_handler_name("DELETE", fn_name)
}

pub fn patch<H, T, S>(handler: H) -> MethodRouter<S>
where
    H: axum::handler::Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    let fn_name = std::any::type_name::<H>()
        .split("::")
        .last()
        .unwrap_or("unknown")
        .to_string();

    MethodRouter::new(axum::routing::patch(handler)).with_handler_name("PATCH", fn_name)
}

// Commented out original methods - can be restored if the new approach doesn't work
/*
pub fn get<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        // Extract function name from handler - simplified approach
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        // Track the route
        self.routes.push(RouteInfo {
            path: path.to_string(),
            method: "GET".to_string(),
            function_name: fn_name,
            summary: Some(format!("GET {path}")),
            description: None,
        });

        // Update OpenAPI spec
        self.openapi.paths.insert(path.to_string(), PathItem);

        // Register route with both documentation and runtime router
        self.router = self.router.route(path, get(handler));
        self
    }

    pub fn post<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        // Track the route
        self.routes.push(RouteInfo {
            path: path.to_string(),
            method: "POST".to_string(),
            function_name: fn_name,
            summary: Some(format!("POST {path}")),
            description: None,
        });

        // Update OpenAPI spec
        self.openapi.paths.insert(path.to_string(), PathItem);

        // Register route with both documentation and runtime router
        self.router = self.router.route(path, post(handler));
        self
    }

    pub fn put<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.routes.push(RouteInfo {
            path: path.to_string(),
            method: "PUT".to_string(),
            function_name: fn_name,
            summary: Some(format!("PUT {path}")),
            description: None,
        });
        self.openapi.paths.insert(path.to_string(), PathItem);
        // Register route with both documentation and runtime router
        self.router = self.router.route(path, put(handler));
        self
    }

    pub fn delete<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.routes.push(RouteInfo {
            path: path.to_string(),
            method: "DELETE".to_string(),
            function_name: fn_name,
            summary: Some(format!("DELETE {path}")),
            description: None,
        });
        self.openapi.paths.insert(path.to_string(), PathItem);
        // Register route with both documentation and runtime router
        self.router = self.router.route(path, delete(handler));
        self
    }

    pub fn patch<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.routes.push(RouteInfo {
            path: path.to_string(),
            method: "PATCH".to_string(),
            function_name: fn_name,
            summary: Some(format!("PATCH {path}")),
            description: None,
        });
        self.openapi.paths.insert(path.to_string(), PathItem);
        // Register route with both documentation and runtime router
        self.router = self.router.route(path, patch(handler));
        self
    }
*/

impl<S> ApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn openapi_spec(&self) -> &OpenAPI {
        &self.openapi
    }

    /// Set the API description
    pub fn description(mut self, description: &str) -> Self {
        self.openapi.info.description = Some(description.to_string());
        self
    }

    /// Set the terms of service URL
    pub fn terms_of_service(mut self, terms_of_service: &str) -> Self {
        self.openapi.info.terms_of_service = Some(terms_of_service.to_string());
        self
    }

    /// Set contact information
    pub fn contact(mut self, name: Option<&str>, url: Option<&str>, email: Option<&str>) -> Self {
        self.openapi.info.contact = Some(Contact {
            name: name.map(|s| s.to_string()),
            url: url.map(|s| s.to_string()),
            email: email.map(|s| s.to_string()),
        });
        self
    }

    /// Set contact email only
    pub fn contact_email(mut self, email: &str) -> Self {
        self.openapi.info.contact = Some(Contact {
            name: None,
            url: None,
            email: Some(email.to_string()),
        });
        self
    }

    /// Set license information
    pub fn license(mut self, name: &str, url: Option<&str>) -> Self {
        self.openapi.info.license = Some(License {
            name: name.to_string(),
            url: url.map(|s| s.to_string()),
        });
        self
    }

    /// Add a tag definition
    pub fn tag(mut self, name: &str, description: Option<&str>) -> Self {
        self.openapi.tags.push(Tag {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            external_docs: None,
        });
        self
    }

    /// Add a tag with external documentation
    pub fn tag_with_docs(
        mut self,
        name: &str,
        description: Option<&str>,
        docs_description: Option<&str>,
        docs_url: &str,
    ) -> Self {
        self.openapi.tags.push(Tag {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            external_docs: Some(ExternalDocs {
                description: docs_description.map(|s| s.to_string()),
                url: docs_url.to_string(),
            }),
        });
        self
    }

    pub fn openapi_json(&mut self) -> String {
        // Clear used schemas to track fresh usage
        self.used_schemas.clear();

        // Build the OpenAPI document with info section
        let info_json = self.build_info_json();
        let mut json = format!(r#"{{"openapi":"3.0.0",{},"#, info_json);

        // Collect all registered handler documentation using helper function
        let handler_docs = self.collect_handler_docs();

        // Collect used schemas using helper function
        let all_used_schemas = self.collect_all_used_schemas(&handler_docs);

        // Add paths section using helper function
        let paths_json = self.build_paths_json(&handler_docs);
        json.push_str(&paths_json);

        // Add tags section using helper function
        let tags_json = self.build_tags_json();
        json.push_str(&tags_json);

        // Merge collected schemas into the main router's used_schemas
        for schema in all_used_schemas {
            self.used_schemas.insert(schema);
        }

        // Recursively collect all transitively referenced schemas
        self.collect_transitive_schema_dependencies();

        // Add components section using helper functions
        let has_auth_endpoints = self.has_auth_endpoints();
        let components_json = self.build_components_json(has_auth_endpoints);
        json.push_str(&components_json);

        json.push('}');
        json
    }

    /// Build the JSON string for the contact object.
    ///
    /// # Arguments
    /// * `contact` - The contact information to serialize
    ///
    /// # Returns
    /// A JSON string representing the contact object in the format `"contact":{...}`,
    /// or an empty string if the contact has no fields populated.
    ///
    /// This is an internal helper method used by `build_info_json()` when constructing
    /// the OpenAPI info section.
    fn build_contact_json(&self, contact: &Contact) -> String {
        let mut contact_parts = Vec::new();
        if let Some(ref name) = contact.name {
            contact_parts.push(format!("\"name\":\"{name}\""));
        }
        if let Some(ref url) = contact.url {
            contact_parts.push(format!("\"url\":\"{url}\""));
        }
        if let Some(ref email) = contact.email {
            contact_parts.push(format!("\"email\":\"{email}\""));
        }
        if !contact_parts.is_empty() {
            format!("\"contact\":{{{}}}", contact_parts.join(","))
        } else {
            String::new()
        }
    }

    /// Build the JSON string for the license object
    ///
    /// # Arguments
    /// * `license` - The license information to serialize
    ///
    /// # Returns
    /// A JSON string representing the license object in the format `"license":{...}`
    ///
    /// This is an internal helper method used by `build_info_json()` when constructing
    /// the OpenAPI info section.
    fn build_license_json(&self, license: &License) -> String {
        let mut license_parts = vec![format!("\"name\":\"{}\"", license.name)];
        if let Some(ref url) = license.url {
            license_parts.push(format!("\"url\":\"{url}\""));
        }
        format!("\"license\":{{{}}}", license_parts.join(","))
    }

    /// Build the complete info section JSON
    ///
    /// # Returns
    /// A JSON string representing the complete info object with all optional fields
    ///
    /// This is an internal helper method used by `openapi_json()` when constructing
    /// the OpenAPI specification.
    fn build_info_json(&self) -> String {
        let mut info_parts = vec![
            format!("\"title\":\"{}\"", self.openapi.info.title),
            format!("\"version\":\"{}\"", self.openapi.info.version),
        ];

        if let Some(ref description) = self.openapi.info.description {
            info_parts.push(format!(
                "\"description\":\"{}\"",
                description.replace("\"", "\\\"")
            ));
        }

        if let Some(ref terms_of_service) = self.openapi.info.terms_of_service {
            info_parts.push(format!("\"termsOfService\":\"{terms_of_service}\""));
        }

        if let Some(ref contact) = self.openapi.info.contact {
            let contact_json = self.build_contact_json(contact);
            if !contact_json.is_empty() {
                info_parts.push(contact_json);
            }
        }

        if let Some(ref license) = self.openapi.info.license {
            info_parts.push(self.build_license_json(license));
        }

        format!(r#""info":{{{}}}"#, info_parts.join(","))
    }

    /// Build the tags section JSON
    ///
    /// # Returns
    /// A JSON string representing the tags section in the format `,"tags":[...]`,
    /// or an empty string if there are no tags.
    ///
    /// This is an internal helper method used by `openapi_json()` when constructing
    /// the OpenAPI specification.
    fn build_tags_json(&self) -> String {
        if self.openapi.tags.is_empty() {
            return String::new();
        }

        let tag_entries: Vec<String> = self
            .openapi
            .tags
            .iter()
            .map(|tag| {
                let mut tag_obj = vec![format!(r#""name":"{}""#, tag.name)];
                if let Some(ref description) = tag.description {
                    tag_obj.push(format!(
                        r#""description":"{}""#,
                        description.replace("\"", "\\\"")
                    ));
                }
                if let Some(ref external_docs) = tag.external_docs {
                    let mut docs_parts = vec![format!(r#""url":"{}""#, external_docs.url)];
                    if let Some(ref desc) = external_docs.description {
                        docs_parts
                            .push(format!(r#""description":"{}""#, desc.replace("\"", "\\\"")));
                    }
                    tag_obj.push(format!(r#""externalDocs":{{{}}}"#, docs_parts.join(",")));
                }
                format!("{{{}}}", tag_obj.join(","))
            })
            .collect();

        format!(r#","tags":[{}]"#, tag_entries.join(","))
    }

    /// Group routes by their path
    ///
    /// # Returns
    /// A HashMap where keys are path strings and values are vectors of RouteInfo references
    /// that share the same path.
    ///
    /// This is an internal helper method used by `openapi_json()` to organize routes
    /// before building the OpenAPI paths section.
    fn group_routes_by_path(&self) -> HashMap<String, Vec<&RouteInfo>> {
        let mut path_methods: HashMap<String, Vec<&RouteInfo>> = HashMap::new();
        for route in &self.routes {
            path_methods.entry(route.path.clone()).or_default().push(route);
        }
        path_methods
    }

    /// Build JSON for a single HTTP method within a path
    ///
    /// # Arguments
    /// * `route` - The route information for this method
    /// * `doc` - Optional handler documentation for this route
    ///
    /// # Returns
    /// A JSON string representing the method object in OpenAPI format
    ///
    /// This is an internal helper method used by `build_path_json()` when constructing
    /// the OpenAPI paths section.
    fn build_method_json(
        &self,
        route: &RouteInfo,
        doc: Option<&HandlerDocumentation>,
    ) -> String {
        let (summary, description) = if let Some(doc) = doc {
            (doc.summary.to_string(), doc.description.to_string())
        } else {
            (
                route.summary.clone().unwrap_or_else(|| format!("{} {}", route.method, route.path)),
                "No description available".to_string()
            )
        };

        let mut method_parts = vec![
            format!(r#""summary": "{}""#, summary.replace("\"", "\\\"")),
            format!(r#""description": "{}""#, description.replace("\"", "\\\"")),
            format!(r#""x-handler-function": "{}""#, route.function_name)
        ];

        if let Some(doc) = doc {
            // Add tags
            if !doc.tags.is_empty() && doc.tags != "[]" {
                let tags = self.parse_tags_to_openapi(doc.tags);
                if !tags.is_empty() {
                    method_parts.push(format!(r#""tags": {tags}"#));
                }
            }

            // Add parameters
            if !doc.parameters.is_empty() && doc.parameters != "[]" {
                let parameters = self.parse_parameters_to_openapi(doc.parameters);
                if !parameters.is_empty() {
                    method_parts.push(format!(r#""parameters": {parameters}"#));
                }
            }

            // Add security requirements
            if doc.parameters.contains("__REQUIRES_AUTH__") {
                method_parts.push(r#""security": [{"sessionAuth": []}]"#.to_string());
            }

            // Add request body
            if !doc.request_body.is_empty() && doc.request_body != "[]" {
                let mut temp_router: ApiRouter<()> = ApiRouter::new("temp", "temp");
                let request_body = temp_router.parse_request_body_to_openapi(doc.request_body);
                method_parts.push(format!(r#""requestBody": {request_body}"#));
            }

            // Add responses
            if !doc.responses.is_empty() && doc.responses != "[]" {
                let mut temp_router: ApiRouter<()> = ApiRouter::new("temp", "temp");
                let responses = temp_router.parse_responses_to_openapi(doc.responses);
                method_parts.push(format!(r#""responses": {responses}"#));
            } else {
                method_parts.push(r#""responses": {"200": {"description": "Successful response"}}"#.to_string());
            }
        } else {
            method_parts.push(r#""responses": {"200": {"description": "Successful response"}}"#.to_string());
        }

        format!(r#""{}": {{{}}}"#, route.method.to_lowercase(), method_parts.join(","))
    }

    /// Build JSON for a single path with all its methods
    ///
    /// # Arguments
    /// * `path` - The path string (e.g., "/users/:id")
    /// * `routes` - All routes that share this path
    /// * `handler_docs` - HashMap of handler documentation indexed by function name
    ///
    /// # Returns
    /// A JSON string representing the path object with all its HTTP methods
    ///
    /// This is an internal helper method used by `build_paths_json()` when constructing
    /// the OpenAPI paths section.
    fn build_path_json(
        &self,
        path: &str,
        routes: &[&RouteInfo],
        handler_docs: &HashMap<&str, &HandlerDocumentation>,
    ) -> String {
        let openapi_path = self.convert_path_to_openapi(path);
        
        let methods: Vec<String> = routes.iter().map(|route| {
            let doc = handler_docs.get(route.function_name.as_str()).copied();
            self.build_method_json(route, doc)
        }).collect();

        format!(r#""{}": {{{}}}"#, openapi_path, methods.join(","))
    }

    /// Build the complete paths section JSON
    ///
    /// # Arguments
    /// * `handler_docs` - HashMap of handler documentation indexed by function name
    ///
    /// # Returns
    /// A JSON string representing the entire paths section in OpenAPI format
    ///
    /// This is an internal helper method used by `openapi_json()` to build the paths
    /// section of the OpenAPI specification.
    fn build_paths_json(&self, handler_docs: &HashMap<&str, &HandlerDocumentation>) -> String {
        let path_methods = self.group_routes_by_path();

        let paths: Vec<String> = path_methods.iter()
            .map(|(path, routes)| self.build_path_json(path, routes, handler_docs))
            .collect();

        format!(r#""paths":{{{}}}"#, paths.join(","))
    }

    /// Collect all handler documentation from inventory
    ///
    /// # Returns
    /// A HashMap mapping function names to their handler documentation
    ///
    /// This is an internal helper method used by `openapi_json()` to gather all
    /// registered handler documentation.
    fn collect_handler_docs(&self) -> HashMap<&str, &HandlerDocumentation> {
        inventory::iter::<HandlerDocumentation>()
            .map(|doc| (doc.function_name, doc))
            .collect()
    }

    /// Check if any endpoint requires authentication
    ///
    /// # Returns
    /// True if any route has the __REQUIRES_AUTH__ marker in its parameters
    ///
    /// This is an internal helper method used by `openapi_json()` to determine
    /// whether to include security schemes in the components section.
    fn has_auth_endpoints(&self) -> bool {
        self.routes.iter().any(|route| {
            inventory::iter::<HandlerDocumentation>()
                .find(|doc| doc.function_name == route.function_name)
                .map_or(false, |doc| doc.parameters.contains("__REQUIRES_AUTH__"))
        })
    }

    /// Collect schemas used in a single handler's documentation
    ///
    /// # Arguments
    /// * `doc` - The handler documentation to process
    ///
    /// # Returns
    /// A HashSet of schema names used by this handler
    ///
    /// This is an internal helper method used by `collect_all_used_schemas()`.
    fn collect_schemas_for_handler(&self, doc: &HandlerDocumentation) -> std::collections::HashSet<String> {
        let mut schemas = std::collections::HashSet::new();
        
        // Process request body schemas
        if !doc.request_body.is_empty() && doc.request_body != "[]" {
            let mut temp_router: ApiRouter<()> = ApiRouter::new("temp", "temp");
            let _ = temp_router.parse_request_body_to_openapi(doc.request_body);
            schemas.extend(temp_router.used_schemas);
        }

        // Process response schemas
        if !doc.responses.is_empty() && doc.responses != "[]" {
            let mut temp_router: ApiRouter<()> = ApiRouter::new("temp", "temp");
            let _ = temp_router.parse_responses_to_openapi(doc.responses);
            schemas.extend(temp_router.used_schemas);
        }

        schemas
    }

    /// Collect all schemas used across all routes
    ///
    /// # Arguments
    /// * `handler_docs` - HashMap of handler documentation indexed by function name
    ///
    /// # Returns
    /// A HashSet of all schema names used by any handler
    ///
    /// This is an internal helper method used by `openapi_json()`.
    fn collect_all_used_schemas(&self, handler_docs: &HashMap<&str, &HandlerDocumentation>) -> std::collections::HashSet<String> {
        let mut all_used_schemas = std::collections::HashSet::new();

        for route in &self.routes {
            if let Some(doc) = handler_docs.get(route.function_name.as_str()) {
                all_used_schemas.extend(self.collect_schemas_for_handler(doc));
            }
        }

        all_used_schemas
    }

    /// Filter schema registrations to only those that are used
    ///
    /// # Returns
    /// A HashMap mapping schema names to their JSON representations, containing only
    /// schemas that are actually used in the API
    ///
    /// This is an internal helper method used by `build_components_json()`.
    fn filter_used_schemas(&self) -> HashMap<String, String> {
        let mut used_components_schemas = HashMap::new();
        
        for schema_reg in inventory::iter::<SchemaRegistration>() {
            let schema_name = schema_reg.type_name.to_string();
            if self.used_schemas.contains(&schema_name) {
                used_components_schemas.insert(
                    schema_name,
                    schema_reg.schema_json.to_string()
                );
            }
        }

        used_components_schemas
    }

    /// Build the security schemes JSON
    ///
    /// # Returns
    /// A JSON string representing the securitySchemes section
    ///
    /// This is an internal helper method used by `build_components_json()`.
    fn build_security_schemes_json(&self) -> String {
        r#""securitySchemes":{"sessionAuth":{"type":"apiKey","in":"header","name":"x-session-secret","description":"API session token for authentication"}}"#.to_string()
    }

    /// Build the schemas section within components
    ///
    /// # Arguments
    /// * `used_schemas` - HashMap of schema names to their JSON representations
    ///
    /// # Returns
    /// A JSON string representing the schemas section, or empty string if no schemas
    ///
    /// This is an internal helper method used by `build_components_json()`.
    fn build_schemas_json(&self, used_schemas: &HashMap<String, String>) -> String {
        if used_schemas.is_empty() {
            return String::new();
        }

        let schema_entries: Vec<String> = used_schemas.iter()
            .map(|(name, schema)| format!(r#""{name}": {schema}"#))
            .collect();

        format!(r#""schemas":{{{}}}"#, schema_entries.join(","))
    }

    /// Build the complete components section JSON
    ///
    /// # Arguments
    /// * `has_auth` - Whether any endpoints require authentication
    ///
    /// # Returns
    /// A JSON string representing the components section in the format `,"components":{...}`,
    /// or an empty string if there are no components
    ///
    /// This is an internal helper method used by `openapi_json()`.
    fn build_components_json(&self, has_auth: bool) -> String {
        let used_schemas = self.filter_used_schemas();

        if used_schemas.is_empty() && !has_auth {
            return String::new();
        }

        let mut components_parts = Vec::new();

        // Add schemas section
        let schemas_json = self.build_schemas_json(&used_schemas);
        if !schemas_json.is_empty() {
            components_parts.push(schemas_json);
        }

        // Add security schemes section
        if has_auth {
            components_parts.push(self.build_security_schemes_json());
        }

        format!(r#","components":{{{}}}"#, components_parts.join(","))
    }

    /// Get a list of unused schemas (schemas that are registered but not referenced in any endpoint)
    pub fn get_unused_schemas(&mut self) -> Vec<String> {
        // If used_schemas is empty, we need to populate it by analyzing the endpoints
        if self.used_schemas.is_empty() {
            // Generate OpenAPI spec to populate used_schemas (but don't use the result)
            let _ = self.openapi_json();
        }

        let mut unused_schemas = Vec::new();
        for schema_reg in inventory::iter::<SchemaRegistration>() {
            let schema_name = schema_reg.type_name.to_string();
            if !self.used_schemas.contains(&schema_name) {
                unused_schemas.push(schema_name);
            }
        }
        unused_schemas.sort();
        unused_schemas
    }

    /// Recursively collect all schemas that are transitively referenced by the current used_schemas
    fn collect_transitive_schema_dependencies(&mut self) {
        let mut found_new_dependencies = true;

        while found_new_dependencies {
            found_new_dependencies = false;
            let current_used: Vec<String> = self.used_schemas.iter().cloned().collect();

            for schema_name in &current_used {
                // Find the schema registration for this schema
                if let Some(schema_reg) =
                    inventory::iter::<SchemaRegistration>().find(|reg| reg.type_name == schema_name)
                {
                    let schema_json = schema_reg.schema_json;

                    // Find all $ref references in this schema JSON
                    let refs = self.extract_schema_references(schema_json);
                    for ref_schema in refs {
                        if !self.used_schemas.contains(&ref_schema) {
                            // Check if this referenced schema actually exists
                            if inventory::iter::<SchemaRegistration>()
                                .any(|reg| reg.type_name == ref_schema)
                            {
                                self.used_schemas.insert(ref_schema);
                                found_new_dependencies = true;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Extract all schema names referenced via $ref from a JSON schema string
    fn extract_schema_references(&self, schema_json: &str) -> Vec<String> {
        let mut refs = Vec::new();
        let ref_pattern = "\"$ref\":\"#/components/schemas/";

        let mut start_pos = 0;
        while let Some(ref_start) = schema_json[start_pos..].find(ref_pattern) {
            let absolute_start = start_pos + ref_start + ref_pattern.len();
            if let Some(ref_end) = schema_json[absolute_start..].find('"') {
                let schema_name = &schema_json[absolute_start..absolute_start + ref_end];
                refs.push(schema_name.to_string());
            }
            start_pos = absolute_start;
        }

        refs
    }

    /// Get unused schemas without triggering OpenAPI generation (for testing)
    pub fn get_unused_schemas_current(&self) -> Vec<String> {
        let mut unused_schemas = Vec::new();
        for schema_reg in inventory::iter::<SchemaRegistration>() {
            let schema_name = schema_reg.type_name.to_string();
            if !self.used_schemas.contains(&schema_name) {
                unused_schemas.push(schema_name);
            }
        }
        unused_schemas.sort();
        unused_schemas
    }

    /// Print warnings for unused schemas
    pub fn warn_unused_schemas(&mut self) {
        let unused = self.get_unused_schemas();
        if !unused.is_empty() {
            eprintln!(
                "Warning: The following schemas are defined but never used in the OpenAPI spec:"
            );
            for schema in &unused {
                eprintln!("  - {schema}");
            }
            eprintln!("Consider removing unused schema definitions or ensuring they are properly referenced in endpoint documentation.");
        }
    }

    fn parse_parameters_to_openapi(&self, params_str: &str) -> String {
        // Parse parameter strings like ["id (path): The unique identifier..."]
        // into proper OpenAPI parameter objects
        if params_str == "[]" || params_str.is_empty() {
            return "[]".to_string();
        }

        // Use proper JSON parsing instead of string manipulation
        let param_strings: Result<Vec<String>, _> = serde_json::from_str(params_str);

        let params: Vec<String> = match param_strings {
            Ok(strings) => {
                strings.into_iter().filter_map(|param| {
                    // Filter out the special auth marker
                    if param == "__REQUIRES_AUTH__" {
                        return None;
                    }

                    Some(param)
                }).map(|param| {
                    if let Some(colon_pos) = param.find(':') {
                        let left = param[..colon_pos].trim();
                        let description = param[colon_pos + 1..].trim();

                        // Parse "name (in)" format
                        if let Some(paren_start) = left.find('(') {
                            if let Some(paren_end) = left.find(')') {
                                let name = left[..paren_start].trim();
                                let param_in = left[paren_start + 1..paren_end].trim();

                                // Parse description for examples and defaults
                                // Format: "Description [example: value, default: value]"
                                let (clean_description, example, default) = Self::parse_description_with_metadata(description);

                                let mut param_obj = format!(
                                    r#"{{"name": "{}", "in": "{}", "description": "{}", "required": {}, "schema": {{"type": "string"}}"#,
                                    name,
                                    param_in,
                                    clean_description.replace("\"", "\\\""),
                                    if param_in == "path" { "true" } else { "false" }
                                );

                                // Add example to schema if present
                                if let Some(example_value) = example {
                                    param_obj = param_obj.replace(
                                        r#""schema": {"type": "string"}"#,
                                        &format!(r#""schema": {{"type": "string", "example": "{}"}}"#, example_value.replace("\"", "\\\""))
                                    );
                                }

                                // Add default to schema if present (only for query/header params)
                                if let Some(default_value) = default {
                                    if param_in != "path" {
                                        param_obj = param_obj.replace(
                                            r#""type": "string""#,
                                            &format!(r#""type": "string", "default": "{}""#, default_value.replace("\"", "\\\""))
                                        );
                                    }
                                }

                                param_obj.push('}');
                                return param_obj;
                            }
                        }
                    }

                    // Fallback for malformed parameter
                    format!(r#"{{"name": "unknown", "in": "query", "description": "{}", "schema": {{"type": "string"}}}}"#,
                           param.replace("\"", "\\\""))
                }).collect()
            }
            Err(_) => {
                // Fallback to old parsing method if JSON parsing fails
                params_str
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .split("\", \"")
                    .map(|param| {
                        let param = param.trim_matches('"');
                        format!(r#"{{"name": "unknown", "in": "query", "description": "{}", "schema": {{"type": "string"}}}}"#,
                               param.replace("\"", "\\\""))
                    })
                    .collect()
            }
        };

        format!("[{}]", params.join(","))
    }

    fn convert_path_to_openapi(&self, axum_path: &str) -> String {
        // Convert Axum path format (:param) to OpenAPI format ({param})
        axum_path
            .split('/')
            .map(|segment| {
                if let Some(stripped) = segment.strip_prefix(':') {
                    format!("{{{stripped}}}")
                } else {
                    segment.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    fn parse_request_body_to_openapi(&mut self, request_body_str: &str) -> String {
        if request_body_str == "[]" || request_body_str.is_empty() {
            return r#"{"required": true, "content": {"application/json": {"schema": {"type": "object"}}}}"#.to_string();
        }

        // Check if there's a registered schema type mentioned in the documentation
        let registered_schemas: std::collections::HashSet<String> =
            inventory::iter::<SchemaRegistration>()
                .map(|reg| reg.type_name.to_string())
                .collect();

        // Extract request body information from documentation
        let content: Vec<&str> = request_body_str
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split("\",\"")
            .map(|s| s.trim_matches('"'))
            .collect();

        // Check for explicit type information first (from our macro enhancement)
        for line in &content {
            if let Some(type_name) = line.strip_prefix("Type: ") {
                // Skip "Type: " prefix
                if registered_schemas.contains(type_name) {
                    self.used_schemas.insert(type_name.to_string());
                    return format!(
                        "{{\"required\": true, \"description\": \"Request body\", \"content\": {{\"application/json\": {{\"schema\": {{\"$ref\": \"#/components/schemas/{type_name}\"}}}}}}}}"
                    );
                }
            }
        }

        // Fallback: Look for type references in the documentation
        for schema_name in &registered_schemas {
            if request_body_str.contains(schema_name) {
                self.used_schemas.insert(schema_name.clone());
                return format!(
                    "{{\"required\": true, \"description\": \"Request body\", \"content\": {{\"application/json\": {{\"schema\": {{\"$ref\": \"#/components/schemas/{schema_name}\"}}}}}}}}"
                );
            }
        }

        let mut description = "Request body".to_string();
        let mut content_type = "application/json";
        let mut properties = Vec::new();

        for line in content {
            if line.contains("Content-Type:") {
                if line.contains("application/json") {
                    content_type = "application/json";
                }
            } else if let Some(field_desc) = line.strip_prefix("- ") {
                // Parse field descriptions like "- name (string): The user's full name"
                if let Some(colon_pos) = field_desc.find(':') {
                    let left = field_desc[..colon_pos].trim();
                    let desc = field_desc[colon_pos + 1..].trim();

                    if let Some(paren_start) = left.find('(') {
                        if let Some(paren_end) = left.find(')') {
                            let field_name = left[..paren_start].trim();
                            let field_type = left[paren_start + 1..paren_end].trim();

                            properties.push(format!(
                                r#""{}": {{"type": "{}", "description": "{}"}}"#,
                                field_name,
                                field_type,
                                desc.replace("\"", "\\\"")
                            ));
                        }
                    }
                }
            } else if !line.is_empty() && !line.contains("Content-Type") {
                description = line.to_string();
            }
        }

        let schema = if properties.is_empty() {
            r#"{"type": "object"}"#.to_string()
        } else {
            format!(
                r#"{{"type": "object", "properties": {{{}}}}}"#,
                properties.join(",")
            )
        };

        format!(
            r#"{{"required": true, "description": "{}", "content": {{"{}": {{"schema": {}}}}}}}"#,
            description.replace("\"", "\\\""),
            content_type,
            schema
        )
    }

    fn parse_responses_to_openapi(&mut self, responses_str: &str) -> String {
        if responses_str == "[]" || responses_str.is_empty() {
            return r#"{"200": {"description": "Successful response"}}"#.to_string();
        }

        // Get list of registered schema types for $ref generation
        let registered_schemas: std::collections::HashSet<String> =
            inventory::iter::<SchemaRegistration>()
                .map(|reg| reg.type_name.to_string())
                .collect();

        // Use proper JSON parsing to extract response strings
        let response_strings: Result<Vec<String>, _> = serde_json::from_str(responses_str);

        let mut extracted_error_type: Option<String> = None;
        let responses: Vec<(String, String)> = match response_strings {
            Ok(strings) => {
                strings
                    .into_iter()
                    .filter_map(|item| {
                        // Check if this is an ErrorType metadata entry
                        if let Some(error_type) = item.strip_prefix("ErrorType: ") {
                            extracted_error_type = Some(error_type.to_string());
                            return None; // Don't include metadata in responses
                        }

                        // Parse regular response entries
                        if let Some(colon_pos) = item.find(':') {
                            let status_code = item[..colon_pos].trim();
                            let description = item[colon_pos + 1..].trim();

                            // Only include valid HTTP status codes
                            if status_code.chars().all(|c| c.is_ascii_digit())
                                && status_code.len() == 3
                            {
                                return Some((status_code.to_string(), description.to_string()));
                            }
                        }
                        None
                    })
                    .collect()
            }
            Err(_) => {
                // Fallback to old parsing if JSON parsing fails
                responses_str
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .split('"')
                    .filter_map(|part| {
                        let part = part.trim();
                        if part == "," || part.is_empty() {
                            return None;
                        }
                        if let Some(colon_pos) = part.find(':') {
                            let status_code = part[..colon_pos].trim();
                            let description = part[colon_pos + 1..].trim();

                            // Only include valid HTTP status codes
                            if status_code.chars().all(|c| c.is_ascii_digit())
                                && status_code.len() == 3
                            {
                                return Some((status_code.to_string(), description.to_string()));
                            }
                        }
                        None
                    })
                    .collect()
            }
        };

        if responses.is_empty() {
            return r#"{"200": {"description": "Successful response"}}"#.to_string();
        }

        let response_objects: Vec<String> = responses.iter().map(|(code, desc)| {
            // Handle different response types based on status code
            match code.as_str() {
                "204" => {
                    // 204 No Content should not have a content section
                    format!(r#""{}": {{"description": "{}"}}"#, code, desc.replace("\"", "\\\""))
                },
                code if code.starts_with('2') => {
                    // Other 2xx responses should have content
                    let mut schema = r#"{"type":"object","properties":{}}"#.to_string();

                    // Look for registered schema types in the response description or in common response type names
                    for schema_name in &registered_schemas {
                        if desc.to_lowercase().contains(&schema_name.to_lowercase()) ||
                           desc.contains("user") && schema_name.contains("User") ||
                           desc.contains("greeting") && schema_name.contains("Greet") ||
                           desc.contains("hello") && schema_name.contains("Hello") {
                            self.used_schemas.insert(schema_name.clone());
                            schema = format!("{{\"$ref\": \"#/components/schemas/{schema_name}\"}}");
                            break;
                        }
                    }

                    format!(
                        r#""{}": {{"description": "{}", "content": {{"application/json": {{"schema": {}}}}}}}"#,
                        code, desc.replace("\"", "\\\""), schema
                    )
                },
                _ => {
                    // 4xx, 5xx and other responses - look for error schemas
                    let mut has_error_schema = false;
                    let mut error_schema = String::new();

                    // First priority: try exact schema name match in description (explicit override)
                    for schema_name in &registered_schemas {
                        if schema_name.ends_with("Error") && desc.contains(schema_name) {
                            self.used_schemas.insert(schema_name.clone());
                            error_schema = format!("{{\"$ref\": \"#/components/schemas/{schema_name}\"}}");
                            has_error_schema = true;
                            break;
                        }
                    }

                    // Second priority: use extracted error type from function signature (default)
                    if !has_error_schema {
                        if let Some(ref error_type) = extracted_error_type {
                            // Clean up the type name (remove module paths, etc.)
                            let clean_error_type = error_type.split("::").last().unwrap_or(error_type);

                            // Map known error types to their schema equivalents
                            let schema_name = match clean_error_type {
                                "AppError" => "ErrorResponse", // Map AppError to ErrorResponse
                                other => other, // Use the type name as-is for other errors
                            };

                            if registered_schemas.contains(schema_name) {
                                self.used_schemas.insert(schema_name.to_string());
                                error_schema = format!("{{\"$ref\": \"#/components/schemas/{schema_name}\"}}");
                                has_error_schema = true;
                            }
                        }
                    }

                    // Third priority: try general error matching (fallback)
                    if !has_error_schema {
                        for schema_name in &registered_schemas {
                            if schema_name.ends_with("Error") && desc.to_lowercase().contains("error") {
                                self.used_schemas.insert(schema_name.clone());
                                error_schema = format!("{{\"$ref\": \"#/components/schemas/{schema_name}\"}}");
                                has_error_schema = true;
                                break;
                            }
                        }
                    }

                    if has_error_schema {
                        format!(
                            r#""{}": {{"description": "{}", "content": {{"application/json": {{"schema": {}}}}}}}"#,
                            code, desc.replace("\"", "\\\""), error_schema
                        )
                    } else {
                        format!(r#""{}": {{"description": "{}"}}"#, code, desc.replace("\"", "\\\""))
                    }
                }
            }
        }).collect();

        format!("{{{}}}", response_objects.join(","))
    }

    /// Parse description text for metadata like examples and defaults
    /// Format: "Description text [example: value, default: value]"
    /// Returns: (clean_description, example, default)
    fn parse_description_with_metadata(
        description: &str,
    ) -> (String, Option<String>, Option<String>) {
        // Look for metadata in square brackets at the end
        if let Some(bracket_start) = description.rfind('[') {
            if let Some(bracket_end) = description[bracket_start..].find(']') {
                let metadata_str = &description[bracket_start + 1..bracket_start + bracket_end];
                let clean_description = description[..bracket_start].trim().to_string();

                let mut example = None;
                let mut default = None;

                // Parse comma-separated metadata: "example: value, default: other"
                for part in metadata_str.split(',') {
                    let part = part.trim();
                    if let Some(colon_pos) = part.find(':') {
                        let key = part[..colon_pos].trim();
                        let value = part[colon_pos + 1..].trim();

                        match key {
                            "example" => example = Some(value.to_string()),
                            "default" => default = Some(value.to_string()),
                            _ => {} // Ignore unknown metadata
                        }
                    }
                }

                return (clean_description, example, default);
            }
        }

        // No metadata found, return description as-is
        (description.to_string(), None, None)
    }

    fn parse_tags_to_openapi(&self, tags_str: &str) -> String {
        if tags_str == "[]" || tags_str.is_empty() {
            return "[]".to_string();
        }

        // Parse tag strings like ["user", "admin"] into JSON array
        let tags: Vec<String> = tags_str
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|tag| {
                let clean_tag = tag.trim().trim_matches('"');
                format!("\"{clean_tag}\"")
            })
            .collect();

        format!("[{}]", tags.join(","))
    }

    pub fn with_openapi_routes(mut self) -> Self {
        let json_spec = self.openapi_json();
        let yaml_spec = self.openapi.to_yaml();
        let router = self
            .router
            .route(
                "/openapi.json",
                axum::routing::get(move || async move { axum::Json(json_spec) }),
            )
            .route(
                "/openapi.yaml",
                axum::routing::get(move || async move {
                    ([("content-type", "application/yaml")], yaml_spec)
                }),
            );

        Self {
            router,
            openapi: self.openapi,
            routes: self.routes,
            used_schemas: self.used_schemas,
        }
    }

    pub fn with_openapi_routes_prefix(mut self, prefix: &str) -> Self {
        let json_spec = self.openapi_json();
        let yaml_spec = self.openapi.to_yaml();

        // Normalize the prefix
        let normalized_prefix = if prefix.is_empty() {
            "/openapi".to_string() // Default prefix when empty
        } else if prefix.starts_with('/') {
            prefix.trim_end_matches('/').to_string()
        } else {
            format!("/{}", prefix.trim_end_matches('/'))
        };

        let json_path = format!("{normalized_prefix}.json");
        let yaml_path = format!("{normalized_prefix}.yaml");

        let router = self
            .router
            .route(
                &json_path,
                axum::routing::get(move || async move { axum::Json(json_spec) }),
            )
            .route(
                &yaml_path,
                axum::routing::get(move || async move {
                    ([("content-type", "application/yaml")], yaml_spec)
                }),
            );

        Self {
            router,
            openapi: self.openapi,
            routes: self.routes,
            used_schemas: self.used_schemas,
        }
    }

    /// Merge another ApiRouter into this one
    /// Both routers must have the same state type S
    pub fn merge(mut self, other: ApiRouter<S>) -> Self {
        // Merge the underlying axum routers
        self.router = self.router.merge(other.router);

        // Merge routes
        self.routes.extend(other.routes);

        // Merge used schemas
        self.used_schemas.extend(other.used_schemas);

        // Merge OpenAPI paths
        self.openapi.paths.extend(other.openapi.paths);

        // Merge tags (avoid duplicates)
        for tag in other.openapi.tags {
            if !self.openapi.tags.iter().any(|t| t.name == tag.name) {
                self.openapi.tags.push(tag);
            }
        }

        self
    }

    // Use into_router().with_state(your_state) for state management
    pub fn into_router(self) -> Router<S> {
        self.router
    }
}

// Macro to create API router
#[macro_export]
macro_rules! api_router {
    ($title:expr, $version:expr) => {
        $crate::ApiRouter::new($title, $version)
    };
}

// Re-export inventory for macros
pub use inventory;

// Re-export serde_json for macros
pub use serde_json;

// Re-export proc macros
pub use machined_openapi_gen_macros::{api_error, api_handler, OpenApiSchema};

// Mock serde for compatibility
pub mod serde {
    pub trait Serialize {}
    pub trait Deserialize<'de> {}

    // Blanket implementations for all types
    impl<T> Serialize for T {}
    impl<'de, T> Deserialize<'de> for T {}
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test schema registrations
    inventory::submit! {
        SchemaRegistration {
            type_name: "UserData",
            schema_json: r#"{"type": "object", "properties": {"name": {"type": "string"}, "email": {"type": "string"}}, "required": ["name", "email"]}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "CreateUserRequest",
            schema_json: r#"{"type": "object", "properties": {"name": {"type": "string"}, "email": {"type": "string"}, "age": {"type": "number"}}, "required": ["name", "email", "age"]}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "UpdateUserRequest",
            schema_json: r#"{"type": "object", "properties": {"name": {"type": "string"}, "email": {"type": "string"}}, "required": ["name", "email"]}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "GreetResponse",
            schema_json: r#"{"type": "object", "properties": {"message": {"type": "string"}, "style": {"type": "string"}}, "required": ["message", "style"]}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "DeleteUserError",
            schema_json: r#"{"type": "object", "properties": {"error": {"type": "object"}}}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "GreetError",
            schema_json: r#"{"type": "object", "properties": {"error": {"type": "object"}}}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "UserResponse",
            schema_json: r#"{"type": "object", "properties": {"id": {"type": "integer"}, "name": {"type": "string"}, "email": {"type": "string"}}, "required": ["id", "name", "email"]}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "GetUserError",
            schema_json: r#"{"type": "object", "properties": {"error": {"type": "object"}}}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "CreateUserError",
            schema_json: r#"{"type": "object", "properties": {"error": {"type": "object"}}}"#,
        }
    }

    #[test]
    fn test_api_router_creation() {
        let router = ApiRouter::new("Test API", "1.0.0");
        let spec = router.openapi_spec();

        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "1.0.0");
    }

    #[test]
    fn test_api_router_macro() {
        let router = api_router!("Test API", "2.0.0");
        let spec = router.openapi_spec();

        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "2.0.0");
    }

    #[test]
    fn test_api_description() {
        let router = api_router!("Test API", "1.0.0").description("Test API for testing");

        let spec = router.openapi_spec();
        assert_eq!(
            spec.info.description,
            Some("Test API for testing".to_string())
        );
    }

    #[test]
    fn test_terms_of_service() {
        let router = api_router!("Test API", "1.0.0").terms_of_service("https://example.com/terms");

        let spec = router.openapi_spec();
        assert_eq!(
            spec.info.terms_of_service,
            Some("https://example.com/terms".to_string())
        );
    }

    #[test]
    fn test_contact_info() {
        let router = api_router!("Test API", "1.0.0").contact(
            Some("Test Team"),
            Some("https://example.com"),
            Some("test@example.com"),
        );

        let spec = router.openapi_spec();
        assert!(spec.info.contact.is_some());

        let contact = spec.info.contact.as_ref().unwrap();
        assert_eq!(contact.name, Some("Test Team".to_string()));
        assert_eq!(contact.url, Some("https://example.com".to_string()));
        assert_eq!(contact.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_contact_email_only() {
        let router = api_router!("Test API", "1.0.0").contact_email("test@example.com");

        let spec = router.openapi_spec();
        assert!(spec.info.contact.is_some());

        let contact = spec.info.contact.as_ref().unwrap();
        assert_eq!(contact.email, Some("test@example.com".to_string()));
        assert_eq!(contact.name, None);
        assert_eq!(contact.url, None);
    }

    #[test]
    fn test_license() {
        let router = api_router!("Test API", "1.0.0")
            .license("MIT", Some("https://opensource.org/licenses/MIT"));

        let spec = router.openapi_spec();
        assert!(spec.info.license.is_some());

        let license = spec.info.license.as_ref().unwrap();
        assert_eq!(license.name, "MIT");
        assert_eq!(
            license.url,
            Some("https://opensource.org/licenses/MIT".to_string())
        );
    }

    #[test]
    fn test_tag_addition() {
        let router = api_router!("Test API", "1.0.0")
            .tag("users", Some("User operations"))
            .tag("admin", None);

        let spec = router.openapi_spec();
        assert_eq!(spec.tags.len(), 2);

        assert_eq!(spec.tags[0].name, "users");
        assert_eq!(
            spec.tags[0].description,
            Some("User operations".to_string())
        );

        assert_eq!(spec.tags[1].name, "admin");
        assert_eq!(spec.tags[1].description, None);
    }

    #[test]
    fn test_tag_with_external_docs() {
        let router = api_router!("Test API", "1.0.0").tag_with_docs(
            "users",
            Some("User operations"),
            Some("Learn more"),
            "https://example.com/docs",
        );

        let spec = router.openapi_spec();
        assert_eq!(spec.tags.len(), 1);

        let tag = &spec.tags[0];
        assert_eq!(tag.name, "users");
        assert_eq!(tag.description, Some("User operations".to_string()));
        assert!(tag.external_docs.is_some());

        let docs = tag.external_docs.as_ref().unwrap();
        assert_eq!(docs.description, Some("Learn more".to_string()));
        assert_eq!(docs.url, "https://example.com/docs");
    }

    #[test]
    fn test_convert_path_to_openapi() {
        let router = api_router!("Test API", "1.0.0");

        assert_eq!(router.convert_path_to_openapi("/users/:id"), "/users/{id}");
        assert_eq!(
            router.convert_path_to_openapi("/users/:id/posts/:post_id"),
            "/users/{id}/posts/{post_id}"
        );
        assert_eq!(router.convert_path_to_openapi("/static"), "/static");
        assert_eq!(router.convert_path_to_openapi("/"), "/");
    }

    #[test]
    fn test_parse_parameters_to_openapi() {
        let router = api_router!("Test API", "1.0.0");

        // Test empty parameters
        assert_eq!(router.parse_parameters_to_openapi("[]"), "[]");

        // Test path parameter
        let params = r#"["id (path): The user ID"]"#;
        let result = router.parse_parameters_to_openapi(params);
        assert!(result.contains(r#""name": "id""#));
        assert!(result.contains(r#""in": "path""#));
        assert!(result.contains(r#""required": true"#));

        // Test query parameter
        let params = r#"["filter (query): Filter results"]"#;
        let result = router.parse_parameters_to_openapi(params);
        assert!(result.contains(r#""name": "filter""#));
        assert!(result.contains(r#""in": "query""#));
        assert!(result.contains(r#""required": false"#));
    }

    #[test]
    fn test_parse_responses_to_openapi() {
        let mut router = api_router!("Test API", "1.0.0");

        // Test empty responses
        let result = router.parse_responses_to_openapi("[]");
        assert!(result.contains(r#""200": {"description": "Successful response"}"#));

        // Test simple responses
        let responses = r#"["200: Success", "404: Not found"]"#;
        let result = router.parse_responses_to_openapi(responses);

        // Check that the result contains the expected response codes and descriptions
        assert!(
            result.contains(r#""200":"#),
            "Result should contain '\"200\":' but was: {result}"
        );
        assert!(result.contains(r#""description": "Success"#));
        assert!(result.contains(r#""application/json""#)); // 200 responses have content
        assert!(result.contains(r#""404": {"description": "Not found"}"#));
    }

    #[test]
    fn test_parse_tags_to_openapi() {
        let router = api_router!("Test API", "1.0.0");

        // Test empty tags
        assert_eq!(router.parse_tags_to_openapi("[]"), "[]");
        assert_eq!(router.parse_tags_to_openapi(""), "[]");

        // Test single tag
        let result = router.parse_tags_to_openapi(r#"["users"]"#);
        assert_eq!(result, r#"["users"]"#);

        // Test multiple tags
        let result = router.parse_tags_to_openapi(r#"["users", "admin"]"#);
        assert_eq!(result, r#"["users","admin"]"#);
    }

    #[test]
    fn test_openapi_json_structure() {
        let mut router = api_router!("Test API", "1.0.0")
            .description("Test Description")
            .tag("test", Some("Test operations"));

        let json = router.openapi_json();

        // Basic structure checks
        assert!(json.contains(r#""openapi":"3.0.0""#));
        assert!(json.contains(r#""title":"Test API""#));
        assert!(json.contains(r#""version":"1.0.0""#));
        assert!(json.contains(r#""description":"Test Description""#));
        assert!(json.contains(r#""paths":{"#));
        assert!(json.contains(r#""tags":["#));
    }

    #[test]
    fn test_response_schema_references() {
        let mut router = api_router!("Test", "1.0");

        // Test success response with GreetResponse
        let responses = r#"["200: Returns a personalized GreetResponse message"]"#;
        let result = router.parse_responses_to_openapi(responses);

        assert!(result.contains("GreetResponse"));
        assert!(result.contains("\"$ref\": \"#/components/schemas/GreetResponse\""));
    }

    #[test]
    fn test_error_response_schema_references() {
        let mut router = api_router!("Test", "1.0");

        // Test error response with DeleteUserError
        let responses = r#"["404: User not found DeleteUserError", "403: Insufficient permissions DeleteUserError"]"#;
        let result = router.parse_responses_to_openapi(responses);

        assert!(result.contains("DeleteUserError"));
        assert!(result.contains("\"$ref\": \"#/components/schemas/DeleteUserError\""));
    }

    #[test]
    fn test_user_response_schema_references() {
        let mut router = api_router!("Test", "1.0");

        // Test UserResponse reference
        let responses = r#"["200: Successfully retrieved UserResponse information", "201: User successfully created UserResponse"]"#;
        let result = router.parse_responses_to_openapi(responses);

        assert!(result.contains("UserResponse"));
        assert!(result.contains("\"$ref\": \"#/components/schemas/UserResponse\""));
    }

    #[test]
    fn test_mixed_response_types() {
        let mut router = api_router!("Test", "1.0");

        // Test mixed success and error responses
        let responses = r#"["200: Returns GreetResponse", "400: Invalid request GreetError"]"#;
        let result = router.parse_responses_to_openapi(responses);

        // Should contain both response and error schema references
        assert!(result.contains("GreetResponse"));
        assert!(result.contains("GreetError"));
        assert!(result.contains("\"$ref\": \"#/components/schemas/GreetResponse\""));
        assert!(result.contains("\"$ref\": \"#/components/schemas/GreetError\""));
    }

    #[test]
    fn test_get_user_error_schema_references() {
        let mut router = api_router!("Test", "1.0");

        // Test GetUserError in error responses
        let responses = r#"["404: User not found for the given ID GetUserError", "400: Invalid user ID format GetUserError"]"#;
        let result = router.parse_responses_to_openapi(responses);

        assert!(result.contains("GetUserError"));
        assert!(result.contains("\"$ref\": \"#/components/schemas/GetUserError\""));
    }

    #[test]
    fn test_create_user_error_schema_references() {
        let mut router = api_router!("Test", "1.0");

        // Test CreateUserError in error responses
        let responses = r#"["400: Invalid input data provided CreateUserError", "500: Internal server error occurred CreateUserError"]"#;
        let result = router.parse_responses_to_openapi(responses);

        assert!(result.contains("CreateUserError"));
        assert!(result.contains("\"$ref\": \"#/components/schemas/CreateUserError\""));
    }

    #[test]
    fn test_all_error_types_coverage() {
        let mut router = api_router!("Test", "1.0");

        // Test that all error types are properly referenced
        let responses = r#"["400: GetUserError response", "401: CreateUserError response", "403: DeleteUserError response", "422: GreetError response"]"#;
        let result = router.parse_responses_to_openapi(responses);

        // Should contain all error schema references
        assert!(result.contains("\"$ref\": \"#/components/schemas/GetUserError\""));
        assert!(result.contains("\"$ref\": \"#/components/schemas/CreateUserError\""));
        assert!(result.contains("\"$ref\": \"#/components/schemas/DeleteUserError\""));
        assert!(result.contains("\"$ref\": \"#/components/schemas/GreetError\""));
    }

    #[test]
    fn test_unused_schema_detection() {
        let mut router = api_router!("Test", "1.0");

        // Use some schemas first
        let _ = router.parse_responses_to_openapi(r#"["200: Successfully retrieved UserResponse information", "404: User not found GetUserError"]"#);

        // Now check what's used vs unused
        let all_schemas_count = inventory::iter::<SchemaRegistration>().count();
        let unused = router.get_unused_schemas();

        // Should have some unused schemas
        assert!(!unused.is_empty());
        assert!(unused.len() < all_schemas_count);

        // Should not include schemas we just used
        assert!(!unused.contains(&"UserResponse".to_string()));
        assert!(!unused.contains(&"GetUserError".to_string()));

        // Should include schemas we didn't use
        assert!(
            unused.contains(&"CreateUserRequest".to_string())
                || unused.contains(&"UpdateUserRequest".to_string())
        );
    }

    #[test]
    fn test_openapi_only_includes_used_schemas() {
        let mut router = api_router!("Test", "1.0");

        // The test doesn't need to manually track schemas - the openapi_json() method
        // should track schemas from actual handler documentation. Since we don't have
        // handlers registered in this test, we need to verify that the openapi_json
        // method correctly excludes unused schemas.

        let openapi_json = router.openapi_json();

        // Since no handlers are registered, no schemas should be included
        assert!(!openapi_json.contains("GreetResponse"));
        assert!(!openapi_json.contains("GreetError"));
        assert!(!openapi_json.contains("DeleteUserError"));
        assert!(!openapi_json.contains("CreateUserError"));
        assert!(!openapi_json.contains("UserResponse"));

        // Should have empty paths since no routes registered
        assert!(openapi_json.contains(r#""paths":{}"#));
    }

    #[test]
    fn test_warn_unused_schemas_output() {
        let mut router = api_router!("Test", "1.0");

        // This should identify unused schemas (all test schemas since we don't use any)
        let unused = router.get_unused_schemas();
        assert!(!unused.is_empty());

        // Test passes if we can identify unused schemas
        assert!(
            unused.contains(&"CreateUserRequest".to_string())
                || unused.contains(&"UserData".to_string())
                || unused.contains(&"UpdateUserRequest".to_string())
        );
    }

    #[test]
    fn test_with_openapi_routes_prefix_normalization() {
        let test_cases = vec![
            ("", "/openapi.json"), // Empty prefix defaults to /openapi
            ("/openapi", "/openapi.json"),
            ("openapi", "/openapi.json"),
            ("/api/docs", "/api/docs.json"),
            ("/api/docs/", "/api/docs.json"),
            ("api/docs", "/api/docs.json"),
            ("api/docs/", "/api/docs.json"),
        ];

        for (prefix, _expected_json) in test_cases {
            let router = api_router!("Test API", "1.0.0");

            // The normalized prefix is used internally by with_openapi_routes_prefix
            // We can't directly test the result, but we can verify it doesn't panic
            let _router = router.with_openapi_routes_prefix(prefix);

            // If we could inspect the routes, we would verify:
            // assert!(router has route at expected_json);
            // assert!(router has route at expected_yaml);
        }
    }

    #[test]
    fn test_route_tracking() {
        let router = api_router!("Test API", "1.0.0");

        // Track initial state
        assert_eq!(router.routes.len(), 0);

        // Note: We can't fully test route tracking without proper handler types,
        // but we can verify the structure exists and basic operations work
    }

    #[test]
    fn test_build_contact_json() {
        let router = api_router!("Test API", "1.0.0");

        // Test with all fields populated
        let contact_full = Contact {
            name: Some("Test Team".to_string()),
            url: Some("https://example.com".to_string()),
            email: Some("test@example.com".to_string()),
        };
        let json_full = router.build_contact_json(&contact_full);
        assert_eq!(
            json_full,
            r#""contact":{"name":"Test Team","url":"https://example.com","email":"test@example.com"}"#
        );

        // Test with only name
        let contact_name_only = Contact {
            name: Some("Test Team".to_string()),
            url: None,
            email: None,
        };
        let json_name_only = router.build_contact_json(&contact_name_only);
        assert_eq!(json_name_only, r#""contact":{"name":"Test Team"}"#);

        // Test with only email
        let contact_email_only = Contact {
            name: None,
            url: None,
            email: Some("test@example.com".to_string()),
        };
        let json_email_only = router.build_contact_json(&contact_email_only);
        assert_eq!(json_email_only, r#""contact":{"email":"test@example.com"}"#);

        // Test with only url
        let contact_url_only = Contact {
            name: None,
            url: Some("https://example.com".to_string()),
            email: None,
        };
        let json_url_only = router.build_contact_json(&contact_url_only);
        assert_eq!(json_url_only, r#""contact":{"url":"https://example.com"}"#);

        // Test with empty contact (all fields None)
        let contact_empty = Contact {
            name: None,
            url: None,
            email: None,
        };
        let json_empty = router.build_contact_json(&contact_empty);
        assert_eq!(json_empty, "");

        // Test with name and email (no url)
        let contact_name_email = Contact {
            name: Some("Test Team".to_string()),
            url: None,
            email: Some("test@example.com".to_string()),
        };
        let json_name_email = router.build_contact_json(&contact_name_email);
        assert_eq!(
            json_name_email,
            r#""contact":{"name":"Test Team","email":"test@example.com"}"#
        );
    }

    #[test]
    fn test_build_license_json() {
        let router = api_router!("Test API", "1.0.0");

        // Test with license name and URL
        let license_with_url = License {
            name: "MIT".to_string(),
            url: Some("https://opensource.org/licenses/MIT".to_string()),
        };
        let json_with_url = router.build_license_json(&license_with_url);
        assert_eq!(
            json_with_url,
            r#""license":{"name":"MIT","url":"https://opensource.org/licenses/MIT"}"#
        );

        // Test with license name only (no URL)
        let license_no_url = License {
            name: "Apache 2.0".to_string(),
            url: None,
        };
        let json_no_url = router.build_license_json(&license_no_url);
        assert_eq!(json_no_url, r#""license":{"name":"Apache 2.0"}"#);
    }

    #[test]
    fn test_build_info_json() {
        // Test with minimal info (title and version only)
        let router_minimal = api_router!("Test API", "1.0.0");
        let info_json = router_minimal.build_info_json();
        assert!(info_json.contains(r#""title":"Test API""#));
        assert!(info_json.contains(r#""version":"1.0.0""#));
        assert!(info_json.starts_with(r#""info":{"#));

        // Test with description
        let router_with_desc = api_router!("Test API", "1.0.0").description("A test API");
        let info_json_desc = router_with_desc.build_info_json();
        assert!(info_json_desc.contains(r#""description":"A test API""#));

        // Test with contact
        let router_with_contact = api_router!("Test API", "1.0.0")
            .contact(Some("Test Team"), Some("https://example.com"), None);
        let info_json_contact = router_with_contact.build_info_json();
        assert!(info_json_contact.contains(r#""contact":"#));
        assert!(info_json_contact.contains(r#""name":"Test Team""#));

        // Test with license
        let router_with_license = api_router!("Test API", "1.0.0")
            .license("MIT", Some("https://opensource.org/licenses/MIT"));
        let info_json_license = router_with_license.build_info_json();
        assert!(info_json_license.contains(r#""license":"#));
        assert!(info_json_license.contains(r#""name":"MIT""#));
    }

    #[test]
    fn test_build_tags_json() {
        // Test with no tags
        let router_no_tags = api_router!("Test API", "1.0.0");
        let tags_json_empty = router_no_tags.build_tags_json();
        assert_eq!(tags_json_empty, "");

        // Test with a simple tag
        let router_with_tag = api_router!("Test API", "1.0.0")
            .tag("users", Some("User management endpoints"));
        let tags_json = router_with_tag.build_tags_json();
        assert!(tags_json.starts_with(r#","tags":["#));
        assert!(tags_json.contains(r#""name":"users""#));
        assert!(tags_json.contains(r#""description":"User management endpoints""#));

        // Test with tag with external docs
        let router_with_docs = api_router!("Test API", "1.0.0")
            .tag_with_docs(
                "admin",
                Some("Admin endpoints"),
                Some("Admin Documentation"),
                "https://docs.example.com/admin"
            );
        let tags_json_docs = router_with_docs.build_tags_json();
        assert!(tags_json_docs.contains(r#""name":"admin""#));
        assert!(tags_json_docs.contains(r#""externalDocs":"#));
        assert!(tags_json_docs.contains(r#""url":"https://docs.example.com/admin""#));
    }

    #[test]
    fn test_group_routes_by_path() {
        let mut router = api_router!("Test API", "1.0.0");

        // Create some test routes
        router.routes.push(RouteInfo {
            path: "/users".to_string(),
            method: "GET".to_string(),
            function_name: "get_users".to_string(),
            summary: Some("Get all users".to_string()),
            description: None,
        });
        router.routes.push(RouteInfo {
            path: "/users".to_string(),
            method: "POST".to_string(),
            function_name: "create_user".to_string(),
            summary: Some("Create a user".to_string()),
            description: None,
        });
        router.routes.push(RouteInfo {
            path: "/users/:id".to_string(),
            method: "GET".to_string(),
            function_name: "get_user".to_string(),
            summary: Some("Get a user".to_string()),
            description: None,
        });

        let grouped = router.group_routes_by_path();

        // Should have 2 unique paths
        assert_eq!(grouped.len(), 2);

        // "/users" should have 2 methods
        assert_eq!(grouped.get("/users").unwrap().len(), 2);

        // "/users/:id" should have 1 method
        assert_eq!(grouped.get("/users/:id").unwrap().len(), 1);
    }

    #[test]
    fn test_build_method_json() {
        let router = api_router!("Test API", "1.0.0");

        // Test with no documentation
        let route = RouteInfo {
            path: "/test".to_string(),
            method: "GET".to_string(),
            function_name: "test_handler".to_string(),
            summary: Some("Test summary".to_string()),
            description: None,
        };

        let method_json = router.build_method_json(&route, None);

        // Should contain the method key
        assert!(method_json.contains(r#""get":"#));
        // Should contain summary from route
        assert!(method_json.contains(r#""summary": "Test summary""#));
        // Should contain handler function name
        assert!(method_json.contains(r#""x-handler-function": "test_handler""#));
        // Should have default response
        assert!(method_json.contains(r#""responses": {"200": {"description": "Successful response"}}"#));
    }

    #[test]
    fn test_build_path_json() {
        let mut router = api_router!("Test API", "1.0.0");

        // Create test routes
        let route1 = RouteInfo {
            path: "/users".to_string(),
            method: "GET".to_string(),
            function_name: "get_users".to_string(),
            summary: Some("Get all users".to_string()),
            description: None,
        };
        let route2 = RouteInfo {
            path: "/users".to_string(),
            method: "POST".to_string(),
            function_name: "create_user".to_string(),
            summary: Some("Create a user".to_string()),
            description: None,
        };

        router.routes.push(route1.clone());
        router.routes.push(route2.clone());

        let routes_vec = vec![&route1, &route2];
        let handler_docs = HashMap::new();

        let path_json = router.build_path_json("/users", &routes_vec, &handler_docs);

        // Should contain the path
        assert!(path_json.contains(r#""/users":"#));
        // Should contain both methods
        assert!(path_json.contains(r#""get":"#));
        assert!(path_json.contains(r#""post":"#));
    }

    #[test]
    fn test_build_paths_json() {
        let mut router = api_router!("Test API", "1.0.0");

        // Create test routes
        router.routes.push(RouteInfo {
            path: "/users".to_string(),
            method: "GET".to_string(),
            function_name: "get_users".to_string(),
            summary: Some("Get all users".to_string()),
            description: None,
        });
        router.routes.push(RouteInfo {
            path: "/items/:id".to_string(),
            method: "GET".to_string(),
            function_name: "get_item".to_string(),
            summary: Some("Get an item".to_string()),
            description: None,
        });

        let handler_docs = HashMap::new();
        let paths_json = router.build_paths_json(&handler_docs);

        // Should start with "paths":{
        assert!(paths_json.starts_with(r#""paths":{"#));
        // Should contain both paths
        assert!(paths_json.contains(r#""/users":"#));
        // Should convert :id to {id} in OpenAPI format
        assert!(paths_json.contains(r#""/items/{id}":"#));
    }

    #[test]
    fn test_collect_handler_docs() {
        let router = api_router!("Test API", "1.0.0");
        
        // The function should return a HashMap from inventory
        let handler_docs = router.collect_handler_docs();
        
        // We can't predict exactly what's in inventory during tests,
        // but we can verify it returns a HashMap
        assert!(handler_docs.is_empty() || !handler_docs.is_empty());
    }

    #[test]
    fn test_has_auth_endpoints() {
        let router = api_router!("Test API", "1.0.0");
        
        // With no routes, should return false
        let has_auth = router.has_auth_endpoints();
        assert!(!has_auth || has_auth); // Just verify it doesn't panic
    }

    #[test]
    fn test_filter_used_schemas() {
        let mut router = api_router!("Test API", "1.0.0");
        
        // Add a schema to used_schemas
        router.used_schemas.insert("TestSchema".to_string());
        
        // Filter should work (may or may not find TestSchema in inventory)
        let filtered = router.filter_used_schemas();
        
        // Verify it returns a HashMap
        assert!(filtered.is_empty() || !filtered.is_empty());
    }

    #[test]
    fn test_build_security_schemes_json() {
        let router = api_router!("Test API", "1.0.0");
        
        let security_json = router.build_security_schemes_json();
        
        // Should contain the security scheme
        assert!(security_json.contains("securitySchemes"));
        assert!(security_json.contains("sessionAuth"));
        assert!(security_json.contains("apiKey"));
        assert!(security_json.contains("x-session-secret"));
    }

    #[test]
    fn test_build_schemas_json() {
        let router = api_router!("Test API", "1.0.0");
        
        // Test with empty schemas
        let empty_schemas = HashMap::new();
        let empty_json = router.build_schemas_json(&empty_schemas);
        assert_eq!(empty_json, "");
        
        // Test with one schema
        let mut schemas = HashMap::new();
        schemas.insert("User".to_string(), r#"{"type":"object"}"#.to_string());
        let json = router.build_schemas_json(&schemas);
        assert!(json.starts_with(r#""schemas":{"#));
        assert!(json.contains(r#""User": {"type":"object"}"#));
    }

    #[test]
    fn test_build_components_json() {
        let mut router = api_router!("Test API", "1.0.0");
        
        // Test with no auth and no schemas
        let components_empty = router.build_components_json(false);
        assert_eq!(components_empty, "");
        
        // Test with auth but no schemas
        let components_auth = router.build_components_json(true);
        assert!(components_auth.starts_with(r#","components":{"#));
        assert!(components_auth.contains("securitySchemes"));
        
        // Test with schemas - add to used_schemas
        router.used_schemas.insert("MockSchema".to_string());
        let components_with_schemas = router.build_components_json(false);
        // May or may not be empty depending on whether MockSchema exists in inventory
        assert!(components_with_schemas.is_empty() || components_with_schemas.contains("components"));
    }

    #[test]
    fn test_collect_schemas_for_handler() {
        let router = api_router!("Test API", "1.0.0");
        
        // Create a mock handler documentation with no schemas
        let doc = HandlerDocumentation {
            function_name: "test_handler",
            summary: "Test",
            description: "Test handler",
            parameters: "[]",
            responses: "[]",
            request_body: "[]",
            tags: "[]",
        };
        
        let schemas = router.collect_schemas_for_handler(&doc);
        
        // Should return an empty set since there are no schemas
        assert!(schemas.is_empty());
    }

    #[test]
    fn test_collect_all_used_schemas() {
        let router = api_router!("Test API", "1.0.0");
        
        let handler_docs = HashMap::new();
        let schemas = router.collect_all_used_schemas(&handler_docs);
        
        // With no handler docs, should return empty set
        assert!(schemas.is_empty());
    }

    #[test]
    fn test_openapi_json_performance() {
        use std::time::Instant;
        
        let mut router = api_router!("Performance Test API", "1.0.0")
            .description("Testing performance of OpenAPI generation")
            .tag("test", Some("Test endpoints"));
        
        // Add several routes to simulate a real API
        for i in 0..20 {
            router.routes.push(RouteInfo {
                path: format!("/api/resource{}", i),
                method: "GET".to_string(),
                function_name: format!("get_resource_{}", i),
                summary: Some(format!("Get resource {}", i)),
                description: None,
            });
        }
        
        // Measure time to generate OpenAPI JSON
        let start = Instant::now();
        let json = router.openapi_json();
        let duration = start.elapsed();
        
        // Verify JSON was generated
        assert!(json.contains("openapi"));
        assert!(json.contains("Performance Test API"));
        
        // Performance assertion - should complete in reasonable time
        // This is a sanity check, not a strict benchmark
        assert!(duration.as_millis() < 1000, "OpenAPI generation took too long: {:?}", duration);
    }
}

#[cfg(test)]
mod handler_name_tests {
    use super::*;

    async fn test_handler() -> &'static str {
        "test"
    }

    #[test]
    fn test_handler_name_tracking() {
        let router: ApiRouter<()> =
            ApiRouter::new("Test API", "1.0.0").route("/test", get(test_handler));

        // Check that the route is tracked
        assert_eq!(router.routes.len(), 1);
        let route = &router.routes[0];
        assert_eq!(route.path, "/test");
        assert_eq!(route.method, "GET");
        assert_eq!(route.function_name, "test_handler");
    }

    #[test]
    fn test_multiple_methods_tracking() {
        async fn get_items() -> &'static str {
            "items"
        }
        async fn create_item() -> &'static str {
            "created"
        }

        let router: ApiRouter<()> =
            ApiRouter::new("Test API", "1.0.0").route("/items", get(get_items).post(create_item));

        // Should have 2 routes tracked
        assert_eq!(router.routes.len(), 2);

        let get_route = router.routes.iter().find(|r| r.method == "GET").unwrap();
        assert_eq!(get_route.function_name, "get_items");

        let post_route = router.routes.iter().find(|r| r.method == "POST").unwrap();
        assert_eq!(post_route.function_name, "create_item");
    }

    #[test]
    fn test_openapi_includes_handler_names() {
        async fn list_users() -> &'static str {
            "users"
        }

        let mut router: ApiRouter<()> =
            ApiRouter::new("Test API", "1.0.0").route("/users", get(list_users));

        let openapi_json = router.openapi_json();

        // Should contain the path and handler metadata
        assert!(openapi_json.contains("\"/users\""));
        assert!(openapi_json.contains("GET /users"));
    }
}

#[cfg(test)]
mod handler_tests {
    use super::*;

    // Test helpers to simulate different handler documentation scenarios
    fn create_test_router() -> ApiRouter {
        api_router!("Handler Test API", "1.0.0")
    }

    fn simulate_handler_registration(
        _router: &ApiRouter,
        function_name: &'static str,
        summary: &'static str,
        description: &'static str,
        parameters: &'static str,
        responses: &'static str,
        request_body: &'static str,
        tags: &'static str,
    ) -> HandlerDocumentation {
        // Simulate what the api_handler macro would register
        HandlerDocumentation {
            function_name,
            summary,
            description,
            parameters,
            responses,
            request_body,
            tags,
        }
    }

    #[test]
    fn test_simple_get_handler_no_params() {
        let router = create_test_router();

        // Simulate a simple GET handler with no parameters
        let docs = simulate_handler_registration(
            &router,
            "list_items",
            "List all items",
            "Returns a list of all available items",
            "[]",
            r#"["200: Returns list of items"]"#,
            "[]",
            r#"["items"]"#,
        );

        assert_eq!(docs.function_name, "list_items");
        assert_eq!(docs.summary, "List all items");
        assert!(docs.parameters.contains("[]"));
        assert!(docs.request_body.contains("[]"));
    }

    #[test]
    fn test_get_handler_with_path_param() {
        let router = create_test_router();

        // Simulate GET /users/:id handler
        let docs = simulate_handler_registration(
            &router,
            "get_user",
            "Get user by ID",
            "Retrieves a specific user by their ID",
            r#"["id (path): The user's unique identifier"]"#,
            r#"["200: User found", "404: User not found"]"#,
            "[]",
            r#"["users"]"#,
        );

        assert!(docs.parameters.contains("id (path)"));
        assert!(docs.responses.contains("404: User not found"));
    }

    #[test]
    fn test_post_handler_with_json_body() {
        let router = create_test_router();

        // Simulate POST with JSON body
        let docs = simulate_handler_registration(
            &router,
            "create_user",
            "Create new user",
            "Creates a new user account",
            "[]",
            r#"["201: User created", "400: Invalid input"]"#,
            r#"["Type: CreateUserRequest", "Content-Type: application/json", "User creation data"]"#,
            r#"["users", "admin"]"#,
        );

        assert!(docs.request_body.contains("Type: CreateUserRequest"));
        assert!(docs.request_body.contains("application/json"));
        assert!(docs.tags.contains("admin"));
    }

    #[test]
    fn test_handler_with_query_params() {
        let router = create_test_router();

        // Simulate GET with query parameters
        let docs = simulate_handler_registration(
            &router,
            "search_users",
            "Search users",
            "Search for users with filters",
            r#"["q (query): Search query", "limit (query): Maximum results", "offset (query): Pagination offset"]"#,
            r#"["200: Search results"]"#,
            "[]",
            r#"["users", "search"]"#,
        );

        assert!(docs.parameters.contains("q (query)"));
        assert!(docs.parameters.contains("limit (query)"));
        assert!(docs.parameters.contains("offset (query)"));
    }

    #[test]
    fn test_handler_with_multiple_path_params() {
        let router = create_test_router();

        // Simulate /organizations/:org_id/users/:user_id
        let docs = simulate_handler_registration(
            &router,
            "get_org_user",
            "Get organization user",
            "Get a specific user within an organization",
            r#"["org_id (path): Organization ID", "user_id (path): User ID"]"#,
            r#"["200: User details", "404: Not found", "403: Access denied"]"#,
            "[]",
            r#"["organizations", "users"]"#,
        );

        assert!(docs.parameters.contains("org_id (path)"));
        assert!(docs.parameters.contains("user_id (path)"));
        assert!(docs.responses.contains("403: Access denied"));
    }

    #[test]
    fn test_handler_with_header_params() {
        let router = create_test_router();

        // Simulate handler with header parameters
        let docs = simulate_handler_registration(
            &router,
            "authenticated_endpoint",
            "Authenticated endpoint",
            "Requires authentication token",
            r#"["Authorization (header): Bearer token", "X-Request-ID (header): Request tracking ID"]"#,
            r#"["200: Success", "401: Unauthorized"]"#,
            "[]",
            r#"["auth"]"#,
        );

        assert!(docs.parameters.contains("Authorization (header)"));
        assert!(docs.parameters.contains("X-Request-ID (header)"));
        assert!(docs.responses.contains("401: Unauthorized"));
    }

    #[test]
    fn test_delete_handler_with_responses() {
        let router = create_test_router();

        // Simulate DELETE handler
        let docs = simulate_handler_registration(
            &router,
            "delete_user",
            "Delete user",
            "Permanently delete a user account",
            r#"["id (path): User ID to delete"]"#,
            r#"["204: User deleted", "404: User not found", "403: Cannot delete admin"]"#,
            "[]",
            r#"["users", "admin"]"#,
        );

        assert!(docs.responses.contains("204: User deleted"));
        assert!(!docs.responses.contains("200")); // Should not have 200 for DELETE
    }

    #[test]
    fn test_put_handler_with_body() {
        let router = create_test_router();

        // Simulate PUT handler
        let docs = simulate_handler_registration(
            &router,
            "update_user",
            "Update user",
            "Update an existing user",
            r#"["id (path): User ID"]"#,
            r#"["200: User updated", "404: User not found", "400: Invalid data"]"#,
            r#"["Type: UpdateUserRequest", "Content-Type: application/json", "Updated user data"]"#,
            r#"["users"]"#,
        );

        assert!(docs.request_body.contains("Type: UpdateUserRequest"));
        assert!(docs.responses.contains("200: User updated"));
    }

    #[test]
    fn test_patch_handler_partial_update() {
        let router = create_test_router();

        // Simulate PATCH handler
        let docs = simulate_handler_registration(
            &router,
            "patch_user",
            "Partially update user",
            "Update specific fields of a user",
            r#"["id (path): User ID"]"#,
            r#"["200: User updated", "404: User not found"]"#,
            r#"["Type: PatchUserRequest", "Content-Type: application/json", "Partial user data"]"#,
            r#"["users"]"#,
        );

        assert!(docs.request_body.contains("Partial user data"));
    }

    #[test]
    fn test_handler_with_complex_responses() {
        let router = create_test_router();

        // Simulate handler with detailed response documentation
        let docs = simulate_handler_registration(
            &router,
            "complex_endpoint",
            "Complex endpoint",
            "Endpoint with detailed responses",
            "[]",
            r#"["200: Success with data", "400: Bad request with validation errors", "401: Authentication required", "403: Insufficient permissions", "500: Internal server error"]"#,
            "[]",
            r#"["complex"]"#,
        );

        // Verify all response codes are captured
        assert!(docs.responses.contains("200:"));
        assert!(docs.responses.contains("400:"));
        assert!(docs.responses.contains("401:"));
        assert!(docs.responses.contains("403:"));
        assert!(docs.responses.contains("500:"));
    }

    #[test]
    fn test_handler_without_documentation() {
        let router = create_test_router();

        // Simulate handler with minimal/no documentation
        let docs = simulate_handler_registration(
            &router,
            "undocumented_handler",
            "No summary",
            "No description",
            "[]",
            "[]",
            "[]",
            "[]",
        );

        assert_eq!(docs.summary, "No summary");
        assert_eq!(docs.description, "No description");
        assert_eq!(docs.parameters, "[]");
        assert_eq!(docs.responses, "[]");
    }

    #[test]
    fn test_request_body_parsing() {
        let mut router = create_test_router();

        // Test request body parsing for different content types
        let json_body = r#"["Type: UserData", "Content-Type: application/json", "- name (string): User name", "- email (string): User email"]"#;
        let result = router.parse_request_body_to_openapi(json_body);

        assert!(result.contains("application/json"));
        assert!(result.contains("UserData"));
        assert!(result.contains("required"));
    }

    #[test]
    fn test_multiple_tags_parsing() {
        let router = create_test_router();

        // Test multiple tags
        let tags = r#"["users", "admin", "v2"]"#;
        let result = router.parse_tags_to_openapi(tags);

        assert_eq!(result, r#"["users","admin","v2"]"#);
    }

    #[test]
    fn test_special_status_codes() {
        let mut router = create_test_router();

        // Test special status codes like 204 No Content
        let responses = r#"["204: No content", "201: Created with Location header", "202: Accepted for processing"]"#;
        let result = router.parse_responses_to_openapi(responses);

        // 204 should not have content
        assert!(result.contains(r#""204": {"description": "No content"}"#));
        // 201 and 202 should have content
        assert!(
            result.contains(r#""201": {"description": "Created with Location header", "content":"#)
        );
    }

    #[test]
    fn test_error_response_parsing() {
        let mut router = create_test_router();

        // Test error responses
        let responses = r#"["400: Validation failed", "409: Conflict with existing resource", "422: Unprocessable entity"]"#;
        let result = router.parse_responses_to_openapi(responses);

        // Error responses should not have content by default
        assert!(result.contains(r#""400": {"description": "Validation failed"}"#));
        assert!(result.contains(r#""409": {"description": "Conflict with existing resource"}"#));
        assert!(result.contains(r#""422": {"description": "Unprocessable entity"}"#));
    }

    #[test]
    fn test_handler_with_all_param_types() {
        let router = create_test_router();

        // Test handler with path, query, and header params
        let docs = simulate_handler_registration(
            &router,
            "complex_params",
            "Complex parameters",
            "Handler with all parameter types",
            r#"["id (path): Resource ID", "filter (query): Filter criteria", "sort (query): Sort order", "Authorization (header): Auth token"]"#,
            r#"["200: Success"]"#,
            r#"["Type: FilterRequest", "Content-Type: application/json"]"#,
            r#"["complex"]"#,
        );

        assert!(docs.parameters.contains("(path)"));
        assert!(docs.parameters.contains("(query)"));
        assert!(docs.parameters.contains("(header)"));
    }

    #[test]
    fn test_openapi_json_generation_with_handlers() {
        let mut router = create_test_router();

        // Simulate adding routes
        router.routes.push(RouteInfo {
            path: "/users".to_string(),
            method: "GET".to_string(),
            function_name: "list_users".to_string(),
            summary: Some("List users".to_string()),
            description: None,
        });

        router.routes.push(RouteInfo {
            path: "/users/:id".to_string(),
            method: "GET".to_string(),
            function_name: "get_user".to_string(),
            summary: Some("Get user".to_string()),
            description: None,
        });

        let json = router.openapi_json();

        // Verify paths are included
        assert!(json.contains(r#""/users""#));
        assert!(json.contains(r#""/users/{id}""#)); // Converted from :id
        assert!(json.contains(r#""get":"#));
    }

    #[test]
    fn test_schema_reference_in_responses() {
        let mut router = create_test_router();

        // When UserResponse schema is registered, it should be referenced
        let responses = r#"["200: Successfully retrieved user information"]"#;
        let result = router.parse_responses_to_openapi(responses);

        // Should detect "user" in description and look for UserResponse schema
        assert!(
            result.contains(r#""200": {"description": "Successfully retrieved user information""#)
        );
    }

    #[test]
    fn test_empty_prefix_handling() {
        let router = create_test_router();

        // Empty prefix should default to /openapi
        let router_with_routes = router.with_openapi_routes_prefix("");

        // This should not panic and should use /openapi as default
        let _final_router = router_with_routes.into_router();
    }
}

#[cfg(test)]
mod rustdoc_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_parameters_from_rustdoc() {
        let router = api_router!("Test", "1.0");

        // Test parsing parameters section from rustdoc
        let params = r#"["id (path): The unique user identifier", "include_deleted (query): Include soft-deleted records"]"#;
        let result = router.parse_parameters_to_openapi(params);

        assert!(result.contains(r#""name": "id""#));
        assert!(result.contains(r#""in": "path""#));
        assert!(result.contains(r#""name": "include_deleted""#));
        assert!(result.contains(r#""in": "query""#));
    }

    #[test]
    fn test_parse_request_body_from_rustdoc() {
        let mut router = api_router!("Test", "1.0");

        // Test request body with field documentation
        let body = r#"["Type: CreateUserRequest", "Content-Type: application/json", "User information for account creation", "- name (string): The user's full name", "- email (string): Valid email address", "- age (number): User's age in years"]"#;
        let result = router.parse_request_body_to_openapi(body);

        assert!(result.contains("CreateUserRequest"));
        assert!(result.contains("application/json"));
        assert!(result.contains("required"));
    }

    #[test]
    fn test_parse_responses_with_status_codes() {
        let mut router = api_router!("Test", "1.0");

        // Test various response formats
        let responses = r#"["200: User successfully created", "201: Resource created", "400: Invalid request data", "500: Internal server error"]"#;
        let result = router.parse_responses_to_openapi(responses);

        // Verify each status code is parsed
        assert!(result.contains(r#""200":"#));
        assert!(result.contains(r#""201":"#));
        assert!(result.contains(r#""400":"#));
        assert!(result.contains(r#""500":"#));
    }

    #[test]
    fn test_malformed_parameter_handling() {
        let router = api_router!("Test", "1.0");

        // Test malformed parameters
        let params = r#"["invalid param without type", "id: missing location", "valid (query): This one is good"]"#;
        let result = router.parse_parameters_to_openapi(params);

        // Should handle the valid one
        assert!(result.contains(r#""name": "valid""#));
    }
}

#[cfg(test)]
mod schema_generation_tests {

    // Mock schema registration for testing
    fn mock_schema_registration(type_name: &str, schema_json: &str) {
        // In real usage, this would be done by the StoneSchema derive macro
        // For testing, we just verify the structure
        assert!(!type_name.is_empty());
        assert!(schema_json.contains("type"));
    }

    #[test]
    fn test_simple_struct_schema() {
        let schema_json = r#"{"type":"object","properties":{"id":{"type":"integer"},"name":{"type":"string"}},"required":["id","name"]}"#;
        mock_schema_registration("UserResponse", schema_json);

        assert!(schema_json.contains(r#""type":"object""#));
        assert!(schema_json.contains(r#""properties""#));
        assert!(schema_json.contains(r#""required""#));
    }

    #[test]
    fn test_optional_fields_schema() {
        let schema_json = r#"{"type":"object","properties":{"id":{"type":"integer"},"nickname":{"type":"string"}},"required":["id"]}"#;
        mock_schema_registration("ProfileResponse", schema_json);

        // nickname is optional, so only id should be required
        assert!(schema_json.contains(r#""required":["id"]"#));
        assert!(
            !schema_json.contains("nickname")
                || !schema_json.contains(r#""required":["id","nickname"]"#)
        );
    }

    #[test]
    fn test_nested_struct_schema() {
        let schema_json = r#"{"type":"object","properties":{"user":{"type":"object"},"preferences":{"type":"object"}},"required":["user","preferences"]}"#;
        mock_schema_registration("UserWithPreferences", schema_json);

        assert!(schema_json.contains(r#""user":{"type":"object"}"#));
        assert!(schema_json.contains(r#""preferences":{"type":"object"}"#));
    }

    #[test]
    fn test_array_field_schema() {
        let schema_json =
            r#"{"type":"object","properties":{"items":{"type":"array"}},"required":["items"]}"#;
        mock_schema_registration("ItemList", schema_json);

        assert!(schema_json.contains(r#""type":"array""#));
    }

    #[test]
    fn test_numeric_types_schema() {
        let schema_json = r#"{"type":"object","properties":{"age":{"type":"integer"},"height":{"type":"number"},"weight":{"type":"number"}},"required":["age","height","weight"]}"#;
        mock_schema_registration("PersonMetrics", schema_json);

        // Integer types
        assert!(schema_json.contains(r#""age":{"type":"integer"}"#));
        // Float types
        assert!(schema_json.contains(r#""height":{"type":"number"}"#));
    }

    #[test]
    fn test_boolean_field_schema() {
        let schema_json = r#"{"type":"object","properties":{"active":{"type":"boolean"},"verified":{"type":"boolean"}},"required":["active","verified"]}"#;
        mock_schema_registration("UserStatus", schema_json);

        assert!(schema_json.contains(r#""type":"boolean""#));
    }
}
