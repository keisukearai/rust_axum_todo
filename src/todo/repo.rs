//! todos テーブルへのアクセス。SQL はここだけに閉じ込める。

use sqlx::PgPool;

use super::model::Todo;

pub async fn list(pool: &PgPool) -> sqlx::Result<Vec<Todo>> {
    sqlx::query_as!(Todo, "select id, title, done from todos order by id")
        .fetch_all(pool)
        .await
}

pub async fn insert(pool: &PgPool, title: &str) -> sqlx::Result<Todo> {
    sqlx::query_as!(
        Todo,
        "insert into todos (title) values ($1) returning id, title, done",
        title
    )
    .fetch_one(pool)
    .await
}

/// 完了状態を反転する。該当行がなければ `None`。
pub async fn toggle(pool: &PgPool, id: i64) -> sqlx::Result<Option<Todo>> {
    sqlx::query_as!(
        Todo,
        "update todos set done = not done where id = $1 returning id, title, done",
        id
    )
    .fetch_optional(pool)
    .await
}

/// 削除した行数を返す。
pub async fn delete(pool: &PgPool, id: i64) -> sqlx::Result<u64> {
    let result = sqlx::query!("delete from todos where id = $1", id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
