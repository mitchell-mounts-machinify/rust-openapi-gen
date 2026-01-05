use axum::{
    extract::Path,
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use machined_openapi_gen::{api_router, api_handler, OpenApiSchema, api_error, get, post};

#[derive(Serialize, OpenApiSchema)]
struct HelloResponse {
    message: String,
}

#[derive(Deserialize, OpenApiSchema)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Deserialize, OpenApiSchema)]
struct GreetRequest {
    name: Option<String>,
    style: Option<String>,
}

#[derive(Serialize, OpenApiSchema)]
struct GreetResponse {
    message: String,
    style: String,
}

#[derive(Debug, Serialize, OpenApiSchema)]
struct UserResponse {
    id: u32,
    name: String,
    email: String,
}

#[api_error]
#[derive(Serialize)]
#[serde(tag = "error", content = "details")]
enum GetUserError {
    /// 404: User not found for the given ID
    #[serde(rename = "user_not_found")]
    UserNotFound { id: u32 },

    /// 400: Invalid user ID format
    #[serde(rename = "invalid_user_id")]
    InvalidUserId { id: u32 },
}

#[api_error]
#[derive(Serialize)]
#[serde(tag = "error", content = "details")]
enum GreetError {
    /// 400: Invalid request format
    #[serde(rename = "invalid_request")]
    InvalidRequest { message: String },
}

#[api_error]
#[derive(Serialize)]
#[serde(tag = "error", content = "details")]
enum CreateUserError {
    /// 400: Invalid input data provided
    #[serde(rename = "invalid_input")]
    InvalidInput { message: String },

    /// 500: Internal server error occurred
    #[serde(rename = "server_error")]
    ServerError { message: String },
}

#[api_error]
#[derive(Serialize)]
#[serde(tag = "error", content = "details")]
enum DeleteUserError {
    /// 404: User not found
    #[serde(rename = "user_not_found")]
    UserNotFound { id: u32 },

    /// 403: Insufficient permissions to delete user
    #[serde(rename = "insufficient_permissions")]
    InsufficientPermissions { id: u32 },
}

/// Simple hello world endpoint
///
/// Returns a basic greeting message to verify the API is working.
/// This is typically used for health checks or testing connectivity.
///
/// # Responses
/// - 200: Returns a hello world message
#[api_handler("health")]
async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello, World!".to_string(),
    })
}

/// Greeting endpoint with custom message
///
/// Provides a personalized greeting response. This endpoint can be used
/// to test POST request handling and basic message functionality.
///
/// # Request Body
/// Content-Type: application/json
/// Optional greeting customization parameters:
/// - name (string): Name to include in the greeting
/// - style (string): Greeting style (formal, casual, friendly)
///
/// # Responses
/// - 200: Returns a personalized GreetResponse message
/// - 400: Invalid request format
#[api_handler("greeting")]
async fn greet(Json(request): Json<GreetRequest>) -> Result<Json<GreetResponse>, GreetError> {
    let name = request.name.unwrap_or_else(|| "friend".to_string());
    let style = request.style.unwrap_or_else(|| "friendly".to_string());
    let message = match style.to_lowercase().as_str() {
        "formal" => format!("Good day, {}! Welcome to our API.", name),
        "casual" => format!("Hey {}! Great to see you!", name),
        "friendly" => format!("Hello there, {}! Welcome to our API!", name),
        _ => {
            return Err(GreetError::InvalidRequest {
                message: format!("Unknown greeting style: '{}'. Supported styles: formal, casual, friendly", style)
            });
        }
    };

    Ok(Json(GreetResponse {
        message,
        style: style.to_lowercase(),
    }))
}

/// Get user information by ID
///
/// Retrieves user information for the specified user ID.
/// Returns user data including name and email address.
///
/// # Parameters
/// - id (path): The unique identifier of the user to retrieve
///
/// # Responses
/// - 200: Successfully retrieved UserResponse information
/// - 404: User not found for the given ID GetUserError
/// - 400: Invalid user ID format GetUserError
#[api_handler("user")]
async fn get_user(Path(id): Path<u32>) -> Result<Json<UserResponse>, GetUserError> {
    match id {
        1 => {
            let user = UserResponse {
                id: 1,
                name: "John Doe".to_string(),
                email: "john.doe@example.com".to_string(),
            };
            Ok(Json(user))
        },
        2 => {
            let user = UserResponse {
                id: 2,
                name: "Jane Smith".to_string(),
                email: "jane.smith@example.com".to_string(),
            };
            Ok(Json(user))
        },
        999 => {
            Err(GetUserError::InvalidUserId { id })
        },
        _ => {
            Err(GetUserError::UserNotFound { id })
        }
    }
}

/// Delete a user account
///
/// Permanently deletes a user account and all associated data.
/// This action cannot be undone.
///
/// # Parameters
/// - id (path): The unique identifier of the user to delete
///
/// # Responses
/// - 204: User successfully deleted
/// - 404: User not found DeleteUserError
/// - 403: Insufficient permissions to delete user DeleteUserError
#[api_handler("user")]
async fn delete_user(Path(id): Path<u32>) -> Result<StatusCode, DeleteUserError> {
    match id {
        1 | 2 => {
            println!("Deleting user with ID: {}", id);
            Ok(StatusCode::NO_CONTENT)
        },
        3 => {
            Err(DeleteUserError::InsufficientPermissions { id })
        },
        _ => {
            Err(DeleteUserError::UserNotFound { id })
        }
    }
}

/// Create a new user account
///
/// Creates a new user account with validation and error handling.
///
/// # Request Body
/// Content-Type: application/json
/// User information with name and email fields:
/// - name (string): The user's full name
/// - email (string): The user's email address
///
/// # Responses
/// - 201: User successfully created UserResponse
/// - 400: Invalid input data provided DeleteUserError
/// - 500: Internal server error occurred CreateUserError
#[api_handler("user", "admin")]
async fn create_user_with_errors(Json(request): Json<CreateUserRequest>) -> Result<(StatusCode, Json<UserResponse>), CreateUserError> {
    // Validate the input data
    if request.name.trim().is_empty() {
        return Err(CreateUserError::InvalidInput {
            message: "Name cannot be empty".to_string()
        });
    }

    if !request.email.contains('@') || !request.email.contains('.') {
        return Err(CreateUserError::InvalidInput {
            message: "Invalid email format".to_string()
        });
    }

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut hasher = DefaultHasher::new();
    timestamp.hash(&mut hasher);
    let random_outcome = hasher.finish() % 100;

    match random_outcome {
        0..=79 => {
            let user_response = UserResponse {
                id: 123,
                name: request.name,
                email: request.email,
            };
            Ok((StatusCode::CREATED, Json(user_response)))
        },
        _ => {
            Err(CreateUserError::ServerError {
                message: "Database connection failed".to_string()
            })
        }
    }
}


#[tokio::main]
async fn main() {
    if std::env::args().any(|arg| arg == "--test-schemas") {
        println!("HelloResponse schema: {}", HelloResponse::schema());
        println!("UserResponse schema: {}", UserResponse::schema());
        println!("GreetRequest schema: {}", GreetRequest::schema());
        println!("GreetResponse schema: {}", GreetResponse::schema());
        return;
    }
    if std::env::args().any(|arg| arg == "--test-schema") {
        let mut router = api_router!("Hello World API", "1.0.0")
            .description("A comprehensive example API demonstrating machined-openapi-gen's automatic OpenAPI generation capabilities. This API showcases various endpoint types, request/response schemas, error handling, and documentation features.")
            .terms_of_service("https://example.com/terms")
            .contact(Some("API Support Team"), Some("https://example.com/support"), Some("support@example.com"))
            .license("MIT", Some("https://opensource.org/licenses/MIT"))
            .tag("health", Some("Health check and status endpoints"))
            .tag_with_docs("user", Some("User management operations"), Some("Find out more about user management"), "https://example.com/docs/users")
            .tag("greeting", Some("Greeting and message endpoints"))
            .tag("admin", Some("Administrative operations requiring elevated permissions"))
            .route("/", get(hello))
            .route("/greet", post(greet))
            .route("/users/{id}", get(get_user).delete(delete_user))
            .route("/users", post(create_user_with_errors));

        println!("{}", router.openapi_json());
        return;
    }

    let router = axum::Router::new()
        .route("/", axum::routing::get(hello))
        .route("/greet", axum::routing::post(greet))
        .route("/users/{id}", axum::routing::get(get_user))
        .route("/users/{id}", axum::routing::delete(delete_user))
        .route("/users", axum::routing::post(create_user_with_errors));

    let app = router
        .route("/openapi.json", axum::routing::get(|| async {
            r#"{"openapi":"3.0.0","info":{"title":"Hello World API","version":"1.0.0"},"paths":{}}"#
        }));

    run_server(app).await;
}

async fn run_server(app: axum::Router) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");
    println!("OpenAPI spec available at: http://127.0.0.1:3000/openapi.json");
    println!();
    println!("Available endpoints:");
    println!("  - GET /");
    println!("  - POST /greet");
    println!("  - GET /users/:id");
    println!("  - DELETE /users/:id");
    println!("  - POST /users");
    println!();
    println!("Usage:");
    println!("  cargo run                 # Uses custom prefix /api/docs");
    println!("  cargo run -- --default    # Uses default prefix /openapi");

    axum::serve(listener, app).await.unwrap();
}
