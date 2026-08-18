//! HTTP の入口。リクエストを検証して repo を呼ぶ。エラー変換は AppError に任せる。

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use super::{model::CreateTodo, repo};
use crate::error::AppError;
use crate::state::SharedState;
use crate::todo::Todo;

/// GET /api/todos — 一覧を返す
pub async fn list(State(state): State<SharedState>) -> Result<Json<Vec<Todo>>, AppError> {
    let todos = repo::list(&state.pool).await?;
    tracing::debug!(count = todos.len(), "listed todos");
    Ok(Json(todos))
}

/// POST /api/todos — 追加する
pub async fn create(
    State(state): State<SharedState>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let title = payload.validated_title()?;

    let todo = repo::insert(&state.pool, title).await?;
    tracing::info!(id = todo.id, title = %todo.title, "created todo");

    Ok((StatusCode::CREATED, Json(todo)))
}

/// POST /api/todos/{id} — 完了状態を反転する
pub async fn toggle(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<Todo>, AppError> {
    let todo = repo::toggle(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound(id))?;

    tracing::info!(id = todo.id, done = todo.done, "toggled todo");
    Ok(Json(todo))
}

/// DELETE /api/todos/{id} — 削除する
pub async fn delete(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let deleted = repo::delete(&state.pool, id).await?;

    if deleted == 0 {
        return Err(AppError::NotFound(id));
    }

    tracing::info!(id, "deleted todo");
    Ok(StatusCode::NO_CONTENT)
}
