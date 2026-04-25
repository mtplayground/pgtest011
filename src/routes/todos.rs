use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{db::AppState, error::AppError};
use pgtest011::{
    models::todo::{Todo, TodoStatus},
    repo::todo_repo::TodoRepository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/todos", get(list_todos))
        .route("/api/todos/:id", get(get_todo))
}

#[derive(Debug, Deserialize)]
struct ListTodosQuery {
    status: Option<String>,
}

async fn list_todos(
    State(pool): State<PgPool>,
    Query(query): Query<ListTodosQuery>,
) -> Result<Json<Vec<Todo>>, AppError> {
    let repository = TodoRepository::new(pool);

    let todos = match query.status.as_deref().unwrap_or("all") {
        "all" => repository.list().await?,
        "active" => repository.list_by_status(TodoStatus::Active).await?,
        "completed" => repository.list_by_status(TodoStatus::Completed).await?,
        other => {
            return Err(AppError::bad_request(format!(
                "unsupported status filter `{other}`"
            )));
        }
    };

    Ok(Json(todos))
}

async fn get_todo(
    State(pool): State<PgPool>,
    Path(todo_id): Path<String>,
) -> Result<Json<Todo>, AppError> {
    let todo_id =
        Uuid::parse_str(&todo_id).map_err(|_| AppError::bad_request("invalid todo id"))?;
    let repository = TodoRepository::new(pool);

    let todo = repository
        .get(todo_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("todo `{todo_id}` was not found")))?;

    Ok(Json(todo))
}
