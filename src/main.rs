use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use todo_app::{
    app, logging,
    state::{AppState, SharedState},
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // guard を落とすとファイルへの書き出しが止まるので main の最後まで持つ
    let _log_guard = logging::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to postgres");
    tracing::info!("connected to postgres");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");
    tracing::info!("migrations up to date");

    let state: SharedState = Arc::new(AppState { pool });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind 127.0.0.1:3000");

    tracing::info!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app(state)).await.unwrap();
}
