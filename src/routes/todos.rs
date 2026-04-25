use axum::{
    Json, Router,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, QueryRejection},
    },
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{db::AppState, error::AppError};
use pgtest011::{
    models::todo::{NewTodo, Todo, TodoStatus, UpdateTodo},
    repo::todo_repo::TodoRepository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/todos",
            get(list_todos).post(create_todo).delete(delete_todos),
        )
        .route(
            "/api/todos/toggle-all",
            axum::routing::post(toggle_all_todos),
        )
        .route(
            "/api/todos/:id",
            get(get_todo).patch(update_todo).delete(delete_todo),
        )
}

#[derive(Debug, Deserialize)]
struct ListTodosQuery {
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateTodoRequest {
    title: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTodoRequest {
    title: Option<String>,
    completed: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct DeleteTodosQuery {
    completed: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ToggleAllRequest {
    completed: bool,
}

async fn list_todos(
    State(pool): State<PgPool>,
    query: Result<Query<ListTodosQuery>, QueryRejection>,
) -> Result<Json<Vec<Todo>>, AppError> {
    let Query(query) = query.map_err(AppError::from_query_rejection)?;
    let repository = TodoRepository::new(pool);

    let todos = match query.status.as_deref().unwrap_or("all") {
        "all" => repository.list().await?,
        "active" => repository.list_by_status(TodoStatus::Active).await?,
        "completed" => repository.list_by_status(TodoStatus::Completed).await?,
        other => {
            return Err(AppError::validation(format!(
                "unsupported status filter `{other}`"
            )));
        }
    };

    Ok(Json(todos))
}

async fn create_todo(
    State(pool): State<PgPool>,
    payload: Result<Json<CreateTodoRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let Json(payload) = payload.map_err(AppError::from_json_rejection)?;
    let title = payload.title.trim();
    if title.is_empty() {
        return Err(AppError::validation("title must not be empty"));
    }

    let next_position = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(MAX(position), -1) + 1
        FROM todos
        "#,
    )
    .fetch_one(&pool)
    .await?;

    let repository = TodoRepository::new(pool);
    let todo = repository
        .create(NewTodo {
            title: title.to_string(),
            position: next_position,
            completed: false,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(todo)))
}

async fn delete_todos(
    State(pool): State<PgPool>,
    query: Result<Query<DeleteTodosQuery>, QueryRejection>,
) -> Result<StatusCode, AppError> {
    let Query(query) = query.map_err(AppError::from_query_rejection)?;
    if query.completed != Some(true) {
        return Err(AppError::validation(
            "bulk delete requires `completed=true`",
        ));
    }

    let repository = TodoRepository::new(pool);
    repository.delete_completed().await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn update_todo(
    State(pool): State<PgPool>,
    Path(todo_id): Path<String>,
    payload: Result<Json<UpdateTodoRequest>, JsonRejection>,
) -> Result<Json<Todo>, AppError> {
    let Json(payload) = payload.map_err(AppError::from_json_rejection)?;
    let todo_id = Uuid::parse_str(&todo_id).map_err(|_| AppError::validation("invalid todo id"))?;
    let repository = TodoRepository::new(pool);

    let title = match payload.title {
        Some(title) => {
            let trimmed = title.trim();
            if trimmed.is_empty() {
                return Err(AppError::validation("title must not be empty"));
            }
            Some(trimmed.to_string())
        }
        None => None,
    };

    let todo = repository
        .update(
            todo_id,
            UpdateTodo {
                title,
                completed: payload.completed,
                position: None,
            },
        )
        .await?
        .ok_or_else(|| AppError::not_found(format!("todo `{todo_id}` was not found")))?;

    Ok(Json(todo))
}

async fn toggle_all_todos(
    State(pool): State<PgPool>,
    payload: Result<Json<ToggleAllRequest>, JsonRejection>,
) -> Result<StatusCode, AppError> {
    let Json(payload) = payload.map_err(AppError::from_json_rejection)?;
    let repository = TodoRepository::new(pool);
    repository.set_all_completed(payload.completed).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_todo(
    State(pool): State<PgPool>,
    Path(todo_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let todo_id = Uuid::parse_str(&todo_id).map_err(|_| AppError::validation("invalid todo id"))?;
    let repository = TodoRepository::new(pool);

    let deleted = repository.delete(todo_id).await?;
    if !deleted {
        return Err(AppError::not_found(format!(
            "todo `{todo_id}` was not found"
        )));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn get_todo(
    State(pool): State<PgPool>,
    Path(todo_id): Path<String>,
) -> Result<Json<Todo>, AppError> {
    let todo_id = Uuid::parse_str(&todo_id).map_err(|_| AppError::validation("invalid todo id"))?;
    let repository = TodoRepository::new(pool);

    let todo = repository
        .get(todo_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("todo `{todo_id}` was not found")))?;

    Ok(Json(todo))
}
