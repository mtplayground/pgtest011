use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use uuid::Uuid;

use pgtest011::{
    models::todo::{NewTodo, TodoStatus, UpdateTodo},
    repo::todo_repo::TodoRepository,
};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

async fn setup_test_pool(prefix: &str) -> Result<(PgPool, PgPool, String), sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let admin_pool = PgPool::connect(&database_url).await?;

    let schema = format!("{}_{}", prefix, Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin_pool)
        .await?;

    let search_path = format!("SET search_path TO {schema}");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .after_connect(move |connection, _meta| {
            let search_path = search_path.clone();
            Box::pin(async move {
                sqlx::query(&search_path).execute(connection).await?;
                Ok(())
            })
        })
        .connect(&database_url)
        .await?;
    MIGRATOR.run(&pool).await?;

    Ok((pool, admin_pool, schema))
}

async fn cleanup_schema(admin_pool: &PgPool, schema: &str) -> Result<(), sqlx::Error> {
    sqlx::query(&format!("DROP SCHEMA IF EXISTS {schema} CASCADE"))
        .execute(admin_pool)
        .await?;
    Ok(())
}

#[tokio::test]
async fn repository_create_list_get_and_filter() -> Result<(), sqlx::Error> {
    let (pool, admin_pool, schema) = setup_test_pool("repo_list").await?;
    let repository = TodoRepository::new(pool);

    let first = repository
        .create(NewTodo {
            title: "first".to_string(),
            position: 0,
            completed: false,
        })
        .await?;
    let second = repository
        .create(NewTodo {
            title: "second".to_string(),
            position: 1,
            completed: true,
        })
        .await?;

    let all = repository.list().await?;
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].id, first.id);
    assert_eq!(all[1].id, second.id);

    let active = repository.list_by_status(TodoStatus::Active).await?;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, first.id);

    let completed = repository.list_by_status(TodoStatus::Completed).await?;
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].id, second.id);

    let fetched = repository.get(first.id).await?;
    assert_eq!(fetched.as_ref().map(|todo| todo.id), Some(first.id));

    let missing = repository.get(Uuid::new_v4()).await?;
    assert!(missing.is_none());

    cleanup_schema(&admin_pool, &schema).await?;
    Ok(())
}

#[tokio::test]
async fn repository_update_updates_fields_and_timestamp() -> Result<(), sqlx::Error> {
    let (pool, admin_pool, schema) = setup_test_pool("repo_update").await?;
    let repository = TodoRepository::new(pool);
    let created = repository
        .create(NewTodo {
            title: "original".to_string(),
            position: 0,
            completed: false,
        })
        .await?;

    let updated = repository
        .update(
            created.id,
            UpdateTodo {
                title: Some("updated".to_string()),
                completed: Some(true),
                position: None,
            },
        )
        .await?;

    let updated = updated.expect("todo should exist");
    assert_eq!(updated.title, "updated");
    assert!(updated.completed);
    assert_eq!(updated.position, 0);
    assert!(updated.updated_at.is_some());

    let missing = repository
        .update(
            Uuid::new_v4(),
            UpdateTodo {
                title: Some("missing".to_string()),
                completed: None,
                position: None,
            },
        )
        .await?;
    assert!(missing.is_none());

    cleanup_schema(&admin_pool, &schema).await?;
    Ok(())
}

#[tokio::test]
async fn repository_delete_and_delete_completed() -> Result<(), sqlx::Error> {
    let (pool, admin_pool, schema) = setup_test_pool("repo_delete").await?;
    let repository = TodoRepository::new(pool);

    let active = repository
        .create(NewTodo {
            title: "active".to_string(),
            position: 0,
            completed: false,
        })
        .await?;
    repository
        .create(NewTodo {
            title: "done one".to_string(),
            position: 1,
            completed: true,
        })
        .await?;
    repository
        .create(NewTodo {
            title: "done two".to_string(),
            position: 2,
            completed: true,
        })
        .await?;

    assert!(repository.delete(active.id).await?);
    assert!(!repository.delete(Uuid::new_v4()).await?);

    let deleted_completed = repository.delete_completed().await?;
    assert_eq!(deleted_completed, 2);
    assert!(repository.list().await?.is_empty());

    cleanup_schema(&admin_pool, &schema).await?;
    Ok(())
}

#[tokio::test]
async fn repository_set_all_completed_updates_matching_rows() -> Result<(), sqlx::Error> {
    let (pool, admin_pool, schema) = setup_test_pool("repo_toggle").await?;
    let repository = TodoRepository::new(pool);

    repository
        .create(NewTodo {
            title: "one".to_string(),
            position: 0,
            completed: false,
        })
        .await?;
    repository
        .create(NewTodo {
            title: "two".to_string(),
            position: 1,
            completed: false,
        })
        .await?;
    repository
        .create(NewTodo {
            title: "three".to_string(),
            position: 2,
            completed: true,
        })
        .await?;

    let first_toggle = repository.set_all_completed(true).await?;
    assert_eq!(first_toggle, 2);
    assert_eq!(
        repository
            .list_by_status(TodoStatus::Completed)
            .await?
            .len(),
        3
    );

    let second_toggle = repository.set_all_completed(false).await?;
    assert_eq!(second_toggle, 3);
    assert_eq!(
        repository.list_by_status(TodoStatus::Active).await?.len(),
        3
    );

    cleanup_schema(&admin_pool, &schema).await?;
    Ok(())
}
