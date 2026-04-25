use leptos::prelude::*;
use uuid::Uuid;

use crate::models::todo::{NewTodo, Todo, TodoStatus, UpdateTodo as TodoUpdate};

#[cfg(feature = "ssr")]
use crate::repo::todo_repo::TodoRepository;

#[cfg(feature = "ssr")]
use sqlx::{PgPool, postgres::PgPoolOptions};

#[server]
pub async fn list_todos(status: Option<TodoStatus>) -> Result<Vec<Todo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let repo = todo_repository().await?;

        let todos = match status {
            Some(status) => repo.list_by_status(status).await?,
            None => repo.list().await?,
        };

        Ok(todos)
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = status;
        Err(ServerFnError::new(
            "list_todos is only available on the server",
        ))
    }
}

#[server]
pub async fn add_todo(title: String) -> Result<Todo, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = database_pool().await?;
        let repo = TodoRepository::new(pool.clone());
        let next_position = next_position(&pool).await?;
        let title = normalize_title(title)?;

        repo.create(NewTodo {
            title,
            position: next_position,
            completed: false,
        })
        .await
        .map_err(Into::into)
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = title;
        Err(ServerFnError::new(
            "add_todo is only available on the server",
        ))
    }
}

#[server]
pub async fn update_todo(
    id: Uuid,
    title: Option<String>,
    completed: Option<bool>,
) -> Result<Todo, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let repo = todo_repository().await?;
        let title = title.map(normalize_title).transpose()?;

        repo.update(
            id,
            TodoUpdate {
                title,
                completed,
                position: None,
            },
        )
        .await?
        .ok_or_else(|| not_found("todo not found"))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (id, title, completed);
        Err(ServerFnError::new(
            "update_todo is only available on the server",
        ))
    }
}

#[server]
pub async fn toggle_todo(id: Uuid, completed: bool) -> Result<Todo, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let repo = todo_repository().await?;

        repo.update(
            id,
            TodoUpdate {
                title: None,
                completed: Some(completed),
                position: None,
            },
        )
        .await?
        .ok_or_else(|| not_found("todo not found"))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (id, completed);
        Err(ServerFnError::new(
            "toggle_todo is only available on the server",
        ))
    }
}

#[server]
pub async fn delete_todo(id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let repo = todo_repository().await?;

        if repo.delete(id).await? {
            Ok(())
        } else {
            Err(not_found("todo not found"))
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = id;
        Err(ServerFnError::new(
            "delete_todo is only available on the server",
        ))
    }
}

#[server]
pub async fn clear_completed() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let repo = todo_repository().await?;
        repo.delete_completed().await?;
        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(
            "clear_completed is only available on the server",
        ))
    }
}

#[server]
pub async fn toggle_all(completed: bool) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let repo = todo_repository().await?;
        repo.set_all_completed(completed).await?;
        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = completed;
        Err(ServerFnError::new(
            "toggle_all is only available on the server",
        ))
    }
}

fn normalize_title(title: String) -> Result<String, ServerFnError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        Err(ServerFnError::new("todo title cannot be empty"))
    } else {
        Ok(trimmed.to_string())
    }
}

fn not_found(message: &str) -> ServerFnError {
    ServerFnError::new(message)
}

#[cfg(feature = "ssr")]
async fn todo_repository() -> Result<TodoRepository, ServerFnError> {
    Ok(TodoRepository::new(database_pool().await?))
}

#[cfg(feature = "ssr")]
async fn database_pool() -> Result<PgPool, ServerFnError> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|error| ServerFnError::new(format!("DATABASE_URL is not configured: {error}")))?;

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(Into::into)
}

#[cfg(feature = "ssr")]
async fn next_position(pool: &PgPool) -> Result<i64, ServerFnError> {
    sqlx::query_scalar::<_, i64>("SELECT COALESCE(MAX(position), -1) + 1 FROM todos")
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}
