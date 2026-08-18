use std::sync::Arc;

use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
}

pub type SharedState = Arc<AppState>;
