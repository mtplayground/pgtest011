use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::todo::{NewTodo, Todo, TodoStatus, UpdateTodo};

#[derive(Clone)]
pub struct TodoRepository {
    pool: PgPool,
}

impl TodoRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<Todo>, sqlx::Error> {
        sqlx::query_as::<_, Todo>(
            r#"
            SELECT id, title, completed, position, created_at, updated_at
            FROM todos
            ORDER BY position ASC, created_at ASC NULLS LAST, id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_by_status(&self, status: TodoStatus) -> Result<Vec<Todo>, sqlx::Error> {
        sqlx::query_as::<_, Todo>(
            r#"
            SELECT id, title, completed, position, created_at, updated_at
            FROM todos
            WHERE completed = $1
            ORDER BY position ASC, created_at ASC NULLS LAST, id ASC
            "#,
        )
        .bind(status.completed())
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Todo>, sqlx::Error> {
        sqlx::query_as::<_, Todo>(
            r#"
            SELECT id, title, completed, position, created_at, updated_at
            FROM todos
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(&self, new_todo: NewTodo) -> Result<Todo, sqlx::Error> {
        let now = Utc::now();
        let id = Uuid::new_v4();

        sqlx::query_as::<_, Todo>(
            r#"
            INSERT INTO todos (id, title, completed, position, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, title, completed, position, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(new_todo.title)
        .bind(new_todo.completed)
        .bind(new_todo.position)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(&self, id: Uuid, update: UpdateTodo) -> Result<Option<Todo>, sqlx::Error> {
        sqlx::query_as::<_, Todo>(
            r#"
            UPDATE todos
            SET
                title = COALESCE($2, title),
                completed = COALESCE($3, completed),
                position = COALESCE($4, position),
                updated_at = $5
            WHERE id = $1
            RETURNING id, title, completed, position, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(update.title)
        .bind(update.completed)
        .bind(update.position)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM todos
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_completed(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM todos
            WHERE completed = TRUE
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn set_all_completed(&self, completed: bool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE todos
            SET completed = $1, updated_at = $2
            WHERE completed IS DISTINCT FROM $1
            "#,
        )
        .bind(completed)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
