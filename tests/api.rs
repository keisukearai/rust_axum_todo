//! API の統合テスト。
//!
//! `#[sqlx::test]` がテストごとに使い捨ての DB を作り、migrations/ を適用し、
//! 終了後に落とす。テスト間で状態が混ざらないので順序に依存しない。

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use todo_app::{
    app,
    state::{AppState, SharedState},
};
use tower::ServiceExt;

fn build(pool: PgPool) -> Router {
    let state: SharedState = Arc::new(AppState { pool });
    app(state)
}

/// リクエストを1本投げて (ステータス, JSON ボディ) を返す。ボディが空なら Null。
async fn call(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let builder = Request::builder().method(method).uri(uri);

    let request = match body {
        Some(value) => builder
            .header("content-type", "application/json")
            .body(Body::from(value.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();

    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };

    (status, json)
}

async fn create_todo(app: &Router, title: &str) -> Value {
    let (status, body) = call(app, "POST", "/api/todos", Some(json!({ "title": title }))).await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    body
}

#[sqlx::test]
async fn list_is_empty_initially(pool: PgPool) {
    let app = build(pool);

    let (status, body) = call(&app, "GET", "/api/todos", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));
}

#[sqlx::test]
async fn create_returns_the_created_todo(pool: PgPool) {
    let app = build(pool);

    let body = create_todo(&app, "Rustを学ぶ").await;

    assert_eq!(body["title"], "Rustを学ぶ");
    assert_eq!(body["done"], false);
    assert!(body["id"].as_i64().unwrap() > 0);
}

#[sqlx::test]
async fn created_todos_are_listed_in_id_order(pool: PgPool) {
    let app = build(pool);

    create_todo(&app, "1つ目").await;
    create_todo(&app, "2つ目").await;

    let (status, body) = call(&app, "GET", "/api/todos", None).await;

    assert_eq!(status, StatusCode::OK);
    let titles: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, vec!["1つ目", "2つ目"]);
}

#[sqlx::test]
async fn create_trims_surrounding_whitespace(pool: PgPool) {
    let app = build(pool);

    let body = create_todo(&app, "  余白つき  ").await;

    assert_eq!(body["title"], "余白つき");
}

#[sqlx::test]
async fn create_rejects_blank_title(pool: PgPool) {
    let app = build(pool);

    let (status, body) = call(&app, "POST", "/api/todos", Some(json!({ "title": "   " }))).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "title must not be empty");
}

#[sqlx::test]
async fn create_rejects_too_long_title(pool: PgPool) {
    let app = build(pool);

    let title = "あ".repeat(201);
    let (status, body) = call(&app, "POST", "/api/todos", Some(json!({ "title": title }))).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body["error"],
        "title must be 200 characters or fewer (got 201)"
    );
}

#[sqlx::test]
async fn title_at_the_length_limit_is_accepted(pool: PgPool) {
    let app = build(pool);

    let title = "あ".repeat(200);
    let body = create_todo(&app, &title).await;

    assert_eq!(body["title"], title);
}

#[sqlx::test]
async fn toggle_flips_done_back_and_forth(pool: PgPool) {
    let app = build(pool);
    let id = create_todo(&app, "切り替える").await["id"]
        .as_i64()
        .unwrap();

    let (status, body) = call(&app, "POST", &format!("/api/todos/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["done"], true);

    let (_, body) = call(&app, "POST", &format!("/api/todos/{id}"), None).await;
    assert_eq!(body["done"], false);
}

#[sqlx::test]
async fn toggle_unknown_id_returns_not_found(pool: PgPool) {
    let app = build(pool);

    let (status, body) = call(&app, "POST", "/api/todos/999", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "todo 999 not found");
}

#[sqlx::test]
async fn delete_removes_the_todo(pool: PgPool) {
    let app = build(pool);
    let id = create_todo(&app, "消す").await["id"].as_i64().unwrap();

    let (status, _) = call(&app, "DELETE", &format!("/api/todos/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, body) = call(&app, "GET", "/api/todos", None).await;
    assert_eq!(body, json!([]));
}

#[sqlx::test]
async fn delete_unknown_id_returns_not_found(pool: PgPool) {
    let app = build(pool);

    let (status, body) = call(&app, "DELETE", "/api/todos/999", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "todo 999 not found");
}

#[sqlx::test]
async fn index_html_is_served(pool: PgPool) {
    let app = build(pool);

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["content-type"],
        "text/html",
        "静的ファイル配信が壊れている"
    );
}
