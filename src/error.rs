//! アプリ全体のエラー型。HTTP ステータスと JSON ボディへの変換をここに集約する。

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 入力が不正。メッセージはそのままクライアントに返す。
    #[error("{0}")]
    Validation(String),

    #[error("todo {0} not found")]
    NotFound(i64),

    /// DB エラー。詳細はログにだけ残し、クライアントには伏せる。
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();

        // 内部エラーの中身は漏らさない
        let message = match &self {
            AppError::Database(err) => {
                tracing::error!(error = %err, "database error");
                "internal server error".to_string()
            }
            other => {
                tracing::warn!(error = %other, status = status.as_u16(), "request rejected");
                other.to_string()
            }
        };

        (status, Json(ErrorBody { error: message })).into_response()
    }
}
