mod state;
mod todo;

use axum::Router;
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::state::{AppState, SharedState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("todo_app=debug,tower_http=debug")
        .init();

    let state: SharedState = std::sync::Arc::new(AppState::default());

    let app = Router::new()
        .nest("/api/todos", todo::router())
        .with_state(state)
        .fallback_service(ServeDir::new("static"))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind 127.0.0.1:3000");

    tracing::info!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
