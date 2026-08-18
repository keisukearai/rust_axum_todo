use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// title の最大文字数（バイトではなく文字数）。
pub const MAX_TITLE_LEN: usize = 200;

#[derive(Debug, Serialize)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub done: bool,
}

#[derive(Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

impl CreateTodo {
    /// 前後の空白を落としたうえで検証した title を返す。
    pub fn validated_title(&self) -> Result<&str, AppError> {
        let title = self.title.trim();

        if title.is_empty() {
            return Err(AppError::Validation("title must not be empty".into()));
        }

        let len = title.chars().count();
        if len > MAX_TITLE_LEN {
            return Err(AppError::Validation(format!(
                "title must be {MAX_TITLE_LEN} characters or fewer (got {len})"
            )));
        }

        Ok(title)
    }
}
