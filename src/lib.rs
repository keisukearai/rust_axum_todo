pub mod error;
pub mod logging;
pub mod state;
pub mod todo;

use axum::Router;
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::state::SharedState;

/// ルーターを組み立てる。テストからも同じ関数を使う。
pub fn app(state: SharedState) -> Router {
    Router::new()
        .nest("/api/todos", todo::router())
        .with_state(state)
        .fallback_service(ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
}
