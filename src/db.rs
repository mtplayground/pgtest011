use axum::extract::FromRef;
use leptos::config::LeptosOptions;
use sqlx::{PgPool, postgres::PgPoolOptions};

pub const DEFAULT_MAX_DB_CONNECTIONS: u32 = 5;

#[derive(Clone)]
pub struct AppState {
    leptos_options: LeptosOptions,
    pool: PgPool,
}

impl AppState {
    pub fn new(leptos_options: LeptosOptions, pool: PgPool) -> Self {
        Self {
            leptos_options,
            pool,
        }
    }
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(DEFAULT_MAX_DB_CONNECTIONS)
        .connect(database_url)
        .await
}
