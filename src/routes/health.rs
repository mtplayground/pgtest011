use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use sqlx::PgPool;

use crate::db::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/healthz/db", get(database_healthz))
}

async fn healthz() -> impl IntoResponse {
    StatusCode::OK
}

async fn database_healthz(
    State(pool): State<PgPool>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|_| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "database connectivity check failed",
            )
        })
}
