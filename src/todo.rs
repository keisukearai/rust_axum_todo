use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::state::SharedState;

#[derive(Clone, Serialize)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub done: bool,
}

#[derive(Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

/// `/api/todos` 配下のルート
pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", post(toggle).delete(delete))
}

/// GET /api/todos — 一覧を返す
async fn list(State(state): State<SharedState>) -> Json<Vec<Todo>> {
    let todos = state.todos.lock().unwrap();
    Json(todos.clone())
}

/// POST /api/todos — 追加する
async fn create(
    State(state): State<SharedState>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), StatusCode> {
    let title = payload.title.trim().to_string();
    if title.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut next_id = state.next_id.lock().unwrap();
    *next_id += 1;
    let todo = Todo {
        id: *next_id,
        title,
        done: false,
    };

    state.todos.lock().unwrap().push(todo.clone());
    Ok((StatusCode::CREATED, Json(todo)))
}

/// POST /api/todos/{id} — 完了状態を反転する
async fn toggle(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> Result<Json<Todo>, StatusCode> {
    let mut todos = state.todos.lock().unwrap();
    let todo = todos.iter_mut().find(|t| t.id == id).ok_or(StatusCode::NOT_FOUND)?;
    todo.done = !todo.done;
    Ok(Json(todo.clone()))
}

/// DELETE /api/todos/{id} — 削除する
async fn delete(State(state): State<SharedState>, Path(id): Path<u64>) -> StatusCode {
    let mut todos = state.todos.lock().unwrap();
    let before = todos.len();
    todos.retain(|t| t.id != id);

    if todos.len() == before {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::NO_CONTENT
    }
}
