use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Clone, Serialize)]
struct Todo {
    id: u64,
    title: String,
    done: bool,
}

#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Default)]
struct AppState {
    todos: Mutex<Vec<Todo>>,
    next_id: Mutex<u64>,
}

type SharedState = Arc<AppState>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("todo_app=debug,tower_http=debug")
        .init();

    let state: SharedState = Arc::new(AppState::default());

    let app = Router::new()
        .route("/api/todos", get(list_todos).post(create_todo))
        .route("/api/todos/{id}", post(toggle_todo).delete(delete_todo))
        .with_state(state)
        .fallback_service(ServeDir::new("static"))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind 127.0.0.1:3000");

    tracing::info!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

/// GET /api/todos — 一覧を返す
async fn list_todos(State(state): State<SharedState>) -> Json<Vec<Todo>> {
    let todos = state.todos.lock().unwrap();
    Json(todos.clone())
}

/// POST /api/todos — 追加する
async fn create_todo(
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
async fn toggle_todo(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> Result<Json<Todo>, StatusCode> {
    let mut todos = state.todos.lock().unwrap();
    let todo = todos.iter_mut().find(|t| t.id == id).ok_or(StatusCode::NOT_FOUND)?;
    todo.done = !todo.done;
    Ok(Json(todo.clone()))
}

/// DELETE /api/todos/{id} — 削除する
async fn delete_todo(State(state): State<SharedState>, Path(id): Path<u64>) -> StatusCode {
    let mut todos = state.todos.lock().unwrap();
    let before = todos.len();
    todos.retain(|t| t.id != id);

    if todos.len() == before {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::NO_CONTENT
    }
}
