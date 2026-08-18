use std::process::ExitCode;
use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use todo_app::{
    app,
    config::Config,
    logging,
    state::{AppState, SharedState},
};

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    // ログ初期化より前なので、設定エラーは標準エラー出力に直接書く
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("configuration error: {err}");
            return ExitCode::FAILURE;
        }
    };

    // guard を落とすとファイルへの書き出しが止まるので main の最後まで持つ
    let _log_guard = logging::init(&config);

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to postgres");
    tracing::info!("connected to postgres");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");
    tracing::info!("migrations up to date");

    let state: SharedState = Arc::new(AppState { pool });

    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind {}: {err}", config.bind_addr));

    tracing::info!(addr = %config.bind_addr, log_dir = %config.log_dir, "listening");
    axum::serve(listener, app(state)).await.unwrap();

    ExitCode::SUCCESS
}
