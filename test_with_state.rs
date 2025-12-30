use axum::{Router, routing::get};

#[derive(Clone)]
struct AppState {
    value: i32,
}

async fn handler() -> &'static str {
    "hello"
}

fn main() {
    let router: Router<()> = Router::new().route("/", get(handler));
    let app_state = AppState { value: 42 };
    let app: Router<AppState> = router.with_state(app_state);
    println!("Success!");
}
