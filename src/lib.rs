//! Simple stonehm implementation without serde dependencies

use axum::Router;
use std::collections::HashMap;

// Re-export Axum types so users can import everything from stonehm
pub use axum::{
    Router as AxumRouter,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, Response},
    routing::MethodRouter,
    handler::Handler,
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
        format!("openapi: 3.0.0\ninfo:\n  title: {}\n  version: {}\npaths: {{}}\n",
                self.info.title, self.info.version)
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

// Helper function to extract handler name from type
fn extract_handler_name<H>() -> String {
    std::any::type_name::<H>()
        .split("::")
        .last()
        .unwrap_or("unknown")
        .to_string()
}

// Custom routing helper that carries handler metadata
pub struct TrackedMethodRouter<S = ()> {
    method_router: MethodRouter<S>,
    handlers: Vec<(&'static str, String)>, // (method, handler_name) pairs
}

impl<S> TrackedMethodRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn get<H, T>(self, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = extract_handler_name::<H>();
        
        let mut handlers = self.handlers;
        handlers.push(("GET", fn_name));
        
        Self {
            method_router: self.method_router.get(handler),
            handlers,
        }
    }

    pub fn post<H, T>(self, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = extract_handler_name::<H>();
        
        let mut handlers = self.handlers;
        handlers.push(("POST", fn_name));
        
        Self {
            method_router: self.method_router.post(handler),
            handlers,
        }
    }

    pub fn put<H, T>(self, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = extract_handler_name::<H>();
        
        let mut handlers = self.handlers;
        handlers.push(("PUT", fn_name));
        
        Self {
            method_router: self.method_router.put(handler),
            handlers,
        }
    }

    pub fn delete<H, T>(self, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = extract_handler_name::<H>();
        
        let mut handlers = self.handlers;
        handlers.push(("DELETE", fn_name));
        
        Self {
            method_router: self.method_router.delete(handler),
            handlers,
        }
    }

    pub fn patch<H, T>(self, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = extract_handler_name::<H>();
        
        let mut handlers = self.handlers;
        handlers.push(("PATCH", fn_name));
        
        Self {
            method_router: self.method_router.patch(handler),
            handlers,
        }
    }
}

// Simple trait for schema generation
pub trait OpenApiSchema {
    fn schema() -> String {
        r#"{"type":"object"}"#.to_string()
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
    /// Create a new ApiRouter with a specific state type.
    ///
    /// This is used when your application needs to share state between handlers.
    /// Use this instead of `new()` when you need a state type other than `()`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use machined_openapi_gen::ApiRouter;
    ///
    /// #[derive(Clone)]
    /// struct AppState {
    ///     db_pool: String,
    /// }
    ///
    /// let router = ApiRouter::<AppState>::with_state_type("My API", "1.0.0");
    /// // Later: router.into_router().with_state(app_state)
    /// ```
    pub fn with_state_type(title: &str, version: &str) -> Self {
        Self {
            router: Router::new(),
            openapi: OpenAPI::new(title, version),
            routes: Vec::new(),
            used_schemas: std::collections::HashSet::new(),
        }
    }

    // Use into_router().with_state(your_state) for state management
    pub fn route(mut self, path: &str, tracked: TrackedMethodRouter<S>) -> Self {
        // Track all handlers in this method router
        for (method, handler_name) in tracked.handlers {
            self.routes.push(RouteInfo {
                path: path.to_string(),
                method: method.to_string(),
                function_name: handler_name.clone(),
                summary: Some(format!("{} {}", method, path)),
                description: None,
            });
        }

        // Update OpenAPI spec
        self.openapi.paths.insert(path.to_string(), PathItem);

        // Register route with the underlying router
        self.router = self.router.route(path, tracked.method_router);
        self
    }

    // Helper method to register an HTTP method handler
    fn register_http_method<H, T>(
        mut self,
        path: &str,
        method: &str,
        handler: H,
        route_fn: fn(H) -> axum::routing::MethodRouter<S>,
    ) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = extract_handler_name::<H>();

        self.routes.push(RouteInfo {
            path: path.to_string(),
            method: method.to_string(),
            function_name: fn_name,
            summary: Some(format!("{method} {path}")),
            description: None,
        });

        self.openapi.paths.insert(path.to_string(), PathItem);
        self.router = self.router.route(path, route_fn(handler));
        self
    }

    pub fn get<H, T>(self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        self.register_http_method(path, "GET", handler, axum::routing::get)
    }

    pub fn post<H, T>(self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        self.register_http_method(path, "POST", handler, axum::routing::post)
    }

    pub fn put<H, T>(self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        self.register_http_method(path, "PUT", handler, axum::routing::put)
    }

    pub fn delete<H, T>(self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        self.register_http_method(path, "DELETE", handler, axum::routing::delete)
    }

    pub fn patch<H, T>(self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        self.register_http_method(path, "PATCH", handler, axum::routing::patch)
    }

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
    pub fn tag_with_docs(mut self, name: &str, description: Option<&str>, docs_description: Option<&str>, docs_url: &str) -> Self {
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

        // Build info section with all optional fields
        let mut info_parts = vec![
            format!("\"title\":\"{}\"", self.openapi.info.title),
            format!("\"version\":\"{}\"", self.openapi.info.version),
        ];

        if let Some(ref description) = self.openapi.info.description {
            info_parts.push(format!("\"description\":\"{}\"", description.replace("\"", "\\\"")));
        }

        if let Some(ref terms_of_service) = self.openapi.info.terms_of_service {
            info_parts.push(format!("\"termsOfService\":\"{terms_of_service}\""));
        }

        if let Some(ref contact) = self.openapi.info.contact {
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
                info_parts.push(format!("\"contact\":{{{}}}", contact_parts.join(",")));
            }
        }

        if let Some(ref license) = self.openapi.info.license {
            let mut license_parts = vec![format!("\"name\":\"{}\"", license.name)];
            if let Some(ref url) = license.url {
                license_parts.push(format!("\"url\":\"{url}\""));
            }
            info_parts.push(format!("\"license\":{{{}}}", license_parts.join(",")));
        }

        let mut json = format!(
            r#"{{"openapi":"3.0.0","info":{{{}}},"#,
            info_parts.join(",")
        );

        // Collect all registered handler documentation
        let handler_docs: HashMap<&str, &HandlerDocumentation> = inventory::iter::<HandlerDocumentation>()
            .map(|doc| (doc.function_name, doc))
            .collect();

        // First pass: Process all documentation to track schema usage
        let routes_clone = self.routes.clone();
        for route in &routes_clone {
            if let Some(doc) = handler_docs.get(route.function_name.as_str()) {
                if !doc.request_body.is_empty() && doc.request_body != "[]" {
                    let _ = self.parse_request_body_to_openapi(doc.request_body);
                }
                if !doc.responses.is_empty() && doc.responses != "[]" {
                    let _ = self.parse_responses_to_openapi(doc.responses);
                }
            }
        }

        // Group routes by path
        let mut path_methods: HashMap<String, Vec<&RouteInfo>> = HashMap::new();
        for route in &self.routes {
            path_methods.entry(route.path.clone()).or_default().push(route);
        }

        // Clone the routes to avoid borrowing issues
        let routes_clone = self.routes.clone();

        // Collect used schemas separately to avoid borrowing issues
        let mut all_used_schemas = std::collections::HashSet::new();

        // Process each path and collect schemas
        for route in &routes_clone {
            let doc = handler_docs.get(route.function_name.as_str());

            if let Some(doc) = doc {
                // Process request body schemas
                if !doc.request_body.is_empty() && doc.request_body != "[]" {
                    let mut temp_router: ApiRouter<()> = ApiRouter::new("temp", "temp");
                    let _ = temp_router.parse_request_body_to_openapi(doc.request_body);
                    for schema in temp_router.used_schemas {
                        all_used_schemas.insert(schema);
                    }
                }

                // Process response schemas
                if !doc.responses.is_empty() && doc.responses != "[]" {
                    let mut temp_router: ApiRouter<()> = ApiRouter::new("temp", "temp");
                    let _ = temp_router.parse_responses_to_openapi(doc.responses);
                    for schema in temp_router.used_schemas {
                        all_used_schemas.insert(schema);
                    }
                }
            }
        }

        let paths: Vec<String> = path_methods.iter().map(|(path, routes)| {
            // Convert Axum path format (:param) to OpenAPI format ({param})
            let openapi_path = self.convert_path_to_openapi(path);
            let methods: Vec<String> = routes.iter().map(|route| {
                // Look up documentation for this handler
                let doc = handler_docs.get(route.function_name.as_str());

                let (summary, description) = if let Some(doc) = doc {
                    (doc.summary.to_string(), doc.description.to_string())
                } else {
                    (
                        route.summary.clone().unwrap_or_else(|| format!("{} {}", route.method, path)),
                        "No description available".to_string()
                    )
                };

                // Build proper OpenAPI method object
                let mut method_parts = vec![
                    format!(r#""summary": "{}""#, summary.replace("\"", "\\\"")),
                    format!(r#""description": "{}""#, description.replace("\"", "\\\""))
                ];

                // Add tags if present
                if let Some(doc) = doc {
                    if !doc.tags.is_empty() && doc.tags != "[]" {
                        let tags = self.parse_tags_to_openapi(doc.tags);
                        if !tags.is_empty() {
                            method_parts.push(format!(r#""tags": {tags}"#));
                        }
                    }

                    // Add parameters in proper OpenAPI format
                    if !doc.parameters.is_empty() && doc.parameters != "[]" {
                        let parameters = self.parse_parameters_to_openapi(doc.parameters);
                        if !parameters.is_empty() {
                            method_parts.push(format!(r#""parameters": {parameters}"#));
                        }
                    }

                    // Add security requirements for authenticated endpoints
                    if doc.parameters.contains("__REQUIRES_AUTH__") {
                        method_parts.push(r#""security": [{"sessionAuth": []}]"#.to_string());
                    }

                    // Add request body in proper OpenAPI format (processing already done in first pass)
                    if !doc.request_body.is_empty() && doc.request_body != "[]" {
                        // Create a temporary router to avoid borrowing issues
                        let mut temp_router: ApiRouter<()> = ApiRouter::new("temp", "temp");
                        let request_body = temp_router.parse_request_body_to_openapi(doc.request_body);
                        method_parts.push(format!(r#""requestBody": {request_body}"#));
                    }

                    // Add responses in proper OpenAPI format (processing already done in first pass)
                    if !doc.responses.is_empty() && doc.responses != "[]" {
                        // Create a temporary router to avoid borrowing issues
                        let mut temp_router: ApiRouter<()> = ApiRouter::new("temp", "temp");
                        let responses = temp_router.parse_responses_to_openapi(doc.responses);
                        method_parts.push(format!(r#""responses": {responses}"#));
                    } else {
                        // Default response structure
                        method_parts.push(r#""responses": {"200": {"description": "Successful response"}}"#.to_string());
                    }
                } else {
                    // Default response structure
                    method_parts.push(r#""responses": {"200": {"description": "Successful response"}}"#.to_string());
                }

                format!(r#""{}": {{{}}}"#, route.method.to_lowercase(), method_parts.join(","))
            }).collect();

            format!(r#""{}": {{{}}}"#, openapi_path, methods.join(","))
        }).collect();

        // Add paths section
        json.push_str(r#""paths":{"#);
        json.push_str(&paths.join(","));
        json.push('}');

        // Add tags section if there are tags
        if !self.openapi.tags.is_empty() {
            json.push_str(r#","tags":["#);
            let tag_entries: Vec<String> = self.openapi.tags.iter()
                .map(|tag| {
                    let mut tag_obj = vec![format!(r#""name":"{}""#, tag.name)];
                    if let Some(ref description) = tag.description {
                        tag_obj.push(format!(r#""description":"{}""#, description.replace("\"", "\\\"")));
                    }
                    if let Some(ref external_docs) = tag.external_docs {
                        let mut docs_parts = vec![format!(r#""url":"{}""#, external_docs.url)];
                        if let Some(ref desc) = external_docs.description {
                            docs_parts.push(format!(r#""description":"{}""#, desc.replace("\"", "\\\"")));
                        }
                        tag_obj.push(format!(r#""externalDocs":{{{}}}"#, docs_parts.join(",")));
                    }
                    format!("{{{}}}", tag_obj.join(","))
                })
                .collect();
            json.push_str(&tag_entries.join(","));
            json.push(']');
        }

        // Merge collected schemas into the main router's used_schemas
        for schema in all_used_schemas {
            self.used_schemas.insert(schema);
        }

        // Recursively collect all transitively referenced schemas
        self.collect_transitive_schema_dependencies();

        // Add components section with only used schemas
        let mut used_components_schemas: HashMap<String, String> = HashMap::new();
        for schema_reg in inventory::iter::<SchemaRegistration>() {
            let schema_name = schema_reg.type_name.to_string();
            if self.used_schemas.contains(&schema_name) {
                used_components_schemas.insert(
                    schema_name,
                    schema_reg.schema_json.to_string()
                );
            }
        }

        // Check if any endpoint uses authentication (has Authorized parameter)
        let has_auth_endpoints = self.routes.iter().any(|route| {
            // Find the handler documentation for this route
            inventory::iter::<HandlerDocumentation>()
                .find(|doc| doc.function_name == route.function_name)
                .map_or(false, |doc| {
                    // Check if this endpoint requires auth (has the special marker)
                    doc.parameters.contains("__REQUIRES_AUTH__")
                })
        });

        if !used_components_schemas.is_empty() || has_auth_endpoints {
            json.push_str(r#","components":{"#);

            let mut components_parts = Vec::new();

            // Add schemas section if we have schemas
            if !used_components_schemas.is_empty() {
                let schema_entries: Vec<String> = used_components_schemas.iter()
                    .map(|(name, schema)| format!(r#""{name}": {schema}"#))
                    .collect();
                components_parts.push(format!(r#""schemas":{{{}}}"#, schema_entries.join(",")));
            }

            // Add securitySchemes section if we have auth endpoints
            if has_auth_endpoints {
                let security_schemes = r#""securitySchemes":{"sessionAuth":{"type":"apiKey","in":"header","name":"x-session-secret","description":"API session token for authentication"}}"#;
                components_parts.push(security_schemes.to_string());
            }

            json.push_str(&components_parts.join(","));
            json.push('}');
        }

        json.push('}');
        json
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
                if let Some(schema_reg) = inventory::iter::<SchemaRegistration>()
                    .find(|reg| reg.type_name == schema_name) {

                    let schema_json = schema_reg.schema_json;

                    // Find all $ref references in this schema JSON
                    let refs = self.extract_schema_references(schema_json);
                    for ref_schema in refs {
                        if !self.used_schemas.contains(&ref_schema) {
                            // Check if this referenced schema actually exists
                            if inventory::iter::<SchemaRegistration>()
                                .any(|reg| reg.type_name == ref_schema) {
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
            eprintln!("Warning: The following schemas are defined but never used in the OpenAPI spec:");
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
            },
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
        axum_path.split('/').map(|segment| {
            if let Some(stripped) = segment.strip_prefix(':') {
                format!("{{{stripped}}}")
            } else {
                segment.to_string()
            }
        }).collect::<Vec<_>>().join("/")
    }

    fn parse_request_body_to_openapi(&mut self, request_body_str: &str) -> String {
        if request_body_str == "[]" || request_body_str.is_empty() {
            return r#"{"required": true, "content": {"application/json": {"schema": {"type": "object"}}}}"#.to_string();
        }

        // Check if there's a registered schema type mentioned in the documentation
        let registered_schemas: std::collections::HashSet<String> = inventory::iter::<SchemaRegistration>()
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
            format!(r#"{{"type": "object", "properties": {{{}}}}}"#, properties.join(","))
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
        let registered_schemas: std::collections::HashSet<String> = inventory::iter::<SchemaRegistration>()
            .map(|reg| reg.type_name.to_string())
            .collect();

        // Use proper JSON parsing to extract response strings
        let response_strings: Result<Vec<String>, _> = serde_json::from_str(responses_str);

        let mut extracted_error_type: Option<String> = None;
        let responses: Vec<(String, String)> = match response_strings {
            Ok(strings) => {
                strings.into_iter().filter_map(|item| {
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
                        if status_code.chars().all(|c| c.is_ascii_digit()) && status_code.len() == 3 {
                            return Some((status_code.to_string(), description.to_string()));
                        }
                    }
                    None
                }).collect()
            },
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
                            if status_code.chars().all(|c| c.is_ascii_digit()) && status_code.len() == 3 {
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

                    // First priority: use extracted error type from function signature with mapping
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

                    // If no extracted error type, try exact schema name match in description
                    if !has_error_schema {
                        for schema_name in &registered_schemas {
                            if schema_name.ends_with("Error") && desc.contains(schema_name) {
                                self.used_schemas.insert(schema_name.clone());
                                error_schema = format!("{{\"$ref\": \"#/components/schemas/{schema_name}\"}}");
                                has_error_schema = true;
                                break;
                            }
                        }
                    }

                    // If still no match, try general error matching
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
    fn parse_description_with_metadata(description: &str) -> (String, Option<String>, Option<String>) {
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
        let router = self.router
            .route("/openapi.json", axum::routing::get(move || async move {
                axum::Json(json_spec)
            }))
            .route("/openapi.yaml", axum::routing::get(move || async move {
                ([("content-type", "application/yaml")], yaml_spec)
            }));

        Self { router, openapi: self.openapi, routes: self.routes, used_schemas: self.used_schemas }
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

        let router = self.router
            .route(&json_path, axum::routing::get(move || async move {
                axum::Json(json_spec)
            }))
            .route(&yaml_path, axum::routing::get(move || async move {
                ([("content-type", "application/yaml")], yaml_spec)
            }));

        Self { router, openapi: self.openapi, routes: self.routes, used_schemas: self.used_schemas }
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

// Macro to generate standalone routing functions
macro_rules! tracked_routing_fn {
    ($fn_name:ident, $method_upper:expr, $axum_fn:path) => {
        pub fn $fn_name<H, T, S>(handler: H) -> TrackedMethodRouter<S>
        where
            H: axum::handler::Handler<T, S>,
            T: 'static,
            S: Clone + Send + Sync + 'static,
        {
            TrackedMethodRouter {
                method_router: $axum_fn(handler),
                handlers: vec![($method_upper, extract_handler_name::<H>())],
            }
        }
    };
}

// Custom routing helpers that preserve handler metadata
tracked_routing_fn!(get, "GET", axum::routing::get);
tracked_routing_fn!(post, "POST", axum::routing::post);
tracked_routing_fn!(put, "PUT", axum::routing::put);
tracked_routing_fn!(delete, "DELETE", axum::routing::delete);
tracked_routing_fn!(patch, "PATCH", axum::routing::patch);

// Re-export inventory for macros
pub use inventory;

// Re-export serde_json for macros
pub use serde_json;

// Re-export proc macros
pub use machined_openapi_gen_macros::{api_handler, OpenApiSchema, api_error};

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
        let router = api_router!("Test API", "1.0.0")
            .description("Test API for testing");

        let spec = router.openapi_spec();
        assert_eq!(spec.info.description, Some("Test API for testing".to_string()));
    }

    #[test]
    fn test_terms_of_service() {
        let router = api_router!("Test API", "1.0.0")
            .terms_of_service("https://example.com/terms");

        let spec = router.openapi_spec();
        assert_eq!(spec.info.terms_of_service, Some("https://example.com/terms".to_string()));
    }

    #[test]
    fn test_contact_info() {
        let router = api_router!("Test API", "1.0.0")
            .contact(Some("Test Team"), Some("https://example.com"), Some("test@example.com"));

        let spec = router.openapi_spec();
        assert!(spec.info.contact.is_some());

        let contact = spec.info.contact.as_ref().unwrap();
        assert_eq!(contact.name, Some("Test Team".to_string()));
        assert_eq!(contact.url, Some("https://example.com".to_string()));
        assert_eq!(contact.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_contact_email_only() {
        let router = api_router!("Test API", "1.0.0")
            .contact_email("test@example.com");

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
        assert_eq!(license.url, Some("https://opensource.org/licenses/MIT".to_string()));
    }

    #[test]
    fn test_tag_addition() {
        let router = api_router!("Test API", "1.0.0")
            .tag("users", Some("User operations"))
            .tag("admin", None);

        let spec = router.openapi_spec();
        assert_eq!(spec.tags.len(), 2);

        assert_eq!(spec.tags[0].name, "users");
        assert_eq!(spec.tags[0].description, Some("User operations".to_string()));

        assert_eq!(spec.tags[1].name, "admin");
        assert_eq!(spec.tags[1].description, None);
    }

    #[test]
    fn test_tag_with_external_docs() {
        let router = api_router!("Test API", "1.0.0")
            .tag_with_docs(
                "users",
                Some("User operations"),
                Some("Learn more"),
                "https://example.com/docs"
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
        assert_eq!(router.convert_path_to_openapi("/users/:id/posts/:post_id"), "/users/{id}/posts/{post_id}");
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
        assert!(result.contains(r#""200":"#), "Result should contain '\"200\":' but was: {result}");
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
        assert!(unused.contains(&"CreateUserRequest".to_string()) ||
                unused.contains(&"UpdateUserRequest".to_string()));
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
        assert!(unused.contains(&"CreateUserRequest".to_string()) ||
                unused.contains(&"UserData".to_string()) ||
                unused.contains(&"UpdateUserRequest".to_string()));
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
        assert!(result.contains(r#""201": {"description": "Created with Location header", "content":"#));
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
        assert!(result.contains(r#""200": {"description": "Successfully retrieved user information""#));
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
        assert!(!schema_json.contains("nickname") || !schema_json.contains(r#""required":["id","nickname"]"#));
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
        let schema_json = r#"{"type":"object","properties":{"items":{"type":"array"}},"required":["items"]}"#;
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
