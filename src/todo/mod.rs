mod handler;
mod model;
mod repo;

pub use model::Todo;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::SharedState;

/// `/api/todos` 配下のルート
pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/", get(handler::list).post(handler::create))
        .route("/{id}", post(handler::toggle).delete(handler::delete))
}
