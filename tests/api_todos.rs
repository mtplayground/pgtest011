#[allow(dead_code)]
#[path = "../src/db.rs"]
mod db;
#[allow(dead_code)]
#[path = "../src/error.rs"]
mod error;
#[path = "../src/routes/todos.rs"]
mod todos_routes;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use leptos::config::get_configuration;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

use pgtest011::{
    models::todo::{NewTodo, Todo, TodoStatus},
    repo::todo_repo::TodoRepository,
};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    code: String,
}

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

fn app(pool: PgPool) -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
    let conf = get_configuration(Some("Cargo.toml"))?;
    let state = db::AppState::new(conf.leptos_options, pool);

    Ok(Router::<db::AppState>::new()
        .merge(todos_routes::router())
        .with_state(state))
}

async fn request(
    app: Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
) -> axum::response::Response {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(match body {
            Some(value) => Body::from(value.to_string()),
            None => Body::empty(),
        })
        .expect("request should build");

    app.oneshot(request).await.expect("request should succeed")
}

async fn parse_json<T: DeserializeOwned>(response: axum::response::Response) -> T {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    serde_json::from_slice(&bytes).expect("response body should be valid JSON")
}

#[tokio::test]
async fn api_get_routes_respect_filters_and_not_found()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (pool, admin_pool, schema) = setup_test_pool("api_get").await?;
    let repository = TodoRepository::new(pool.clone());
    let active = repository
        .create(NewTodo {
            title: "active".to_string(),
            position: 0,
            completed: false,
        })
        .await?;
    repository
        .create(NewTodo {
            title: "done".to_string(),
            position: 1,
            completed: true,
        })
        .await?;

    let response = request(
        app(pool.clone())?,
        Method::GET,
        "/api/todos?status=active",
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let todos: Vec<Todo> = parse_json(response).await;
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].id, active.id);

    let response = request(
        app(pool.clone())?,
        Method::GET,
        &format!("/api/todos/{}", active.id),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let todo: Todo = parse_json(response).await;
    assert_eq!(todo.id, active.id);

    let missing_response = request(
        app(pool.clone())?,
        Method::GET,
        &format!("/api/todos/{}", Uuid::new_v4()),
        None,
    )
    .await;
    assert_eq!(missing_response.status(), StatusCode::NOT_FOUND);
    let error: ErrorResponse = parse_json(missing_response).await;
    assert_eq!(error.code, "not_found");

    cleanup_schema(&admin_pool, &schema).await?;
    Ok(())
}

#[tokio::test]
async fn api_create_and_update_validate_titles()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (pool, admin_pool, schema) = setup_test_pool("api_write").await?;

    let create_response = request(
        app(pool.clone())?,
        Method::POST,
        "/api/todos",
        Some(json!({ "title": "  new todo  " })),
    )
    .await;
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created: Todo = parse_json(create_response).await;
    assert_eq!(created.title, "new todo");
    assert_eq!(created.position, 0);

    let invalid_create = request(
        app(pool.clone())?,
        Method::POST,
        "/api/todos",
        Some(json!({ "title": "   " })),
    )
    .await;
    assert_eq!(invalid_create.status(), StatusCode::BAD_REQUEST);
    let error: ErrorResponse = parse_json(invalid_create).await;
    assert_eq!(error.code, "validation_error");

    let update_response = request(
        app(pool.clone())?,
        Method::PATCH,
        &format!("/api/todos/{}", created.id),
        Some(json!({ "title": "  updated todo  ", "completed": true })),
    )
    .await;
    assert_eq!(update_response.status(), StatusCode::OK);
    let updated: Todo = parse_json(update_response).await;
    assert_eq!(updated.title, "updated todo");
    assert!(updated.completed);

    let invalid_update = request(
        app(pool.clone())?,
        Method::PATCH,
        &format!("/api/todos/{}", created.id),
        Some(json!({ "title": "   " })),
    )
    .await;
    assert_eq!(invalid_update.status(), StatusCode::BAD_REQUEST);
    let error: ErrorResponse = parse_json(invalid_update).await;
    assert_eq!(error.code, "validation_error");

    cleanup_schema(&admin_pool, &schema).await?;
    Ok(())
}

#[tokio::test]
async fn api_delete_bulk_delete_and_toggle_all()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (pool, admin_pool, schema) = setup_test_pool("api_delete").await?;
    let repository = TodoRepository::new(pool.clone());
    let first = repository
        .create(NewTodo {
            title: "first".to_string(),
            position: 0,
            completed: false,
        })
        .await?;
    repository
        .create(NewTodo {
            title: "second".to_string(),
            position: 1,
            completed: true,
        })
        .await?;
    repository
        .create(NewTodo {
            title: "third".to_string(),
            position: 2,
            completed: false,
        })
        .await?;

    let toggle_response = request(
        app(pool.clone())?,
        Method::POST,
        "/api/todos/toggle-all",
        Some(json!({ "completed": true })),
    )
    .await;
    assert_eq!(toggle_response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        repository
            .list_by_status(TodoStatus::Completed)
            .await?
            .len(),
        3
    );

    let bulk_delete_response = request(
        app(pool.clone())?,
        Method::DELETE,
        "/api/todos?completed=true",
        None,
    )
    .await;
    assert_eq!(bulk_delete_response.status(), StatusCode::NO_CONTENT);
    assert!(repository.list().await?.is_empty());

    let recreated = repository
        .create(NewTodo {
            title: "recreated".to_string(),
            position: 0,
            completed: false,
        })
        .await?;

    let single_delete_response = request(
        app(pool.clone())?,
        Method::DELETE,
        &format!("/api/todos/{}", recreated.id),
        None,
    )
    .await;
    assert_eq!(single_delete_response.status(), StatusCode::NO_CONTENT);

    let missing_delete_response = request(
        app(pool.clone())?,
        Method::DELETE,
        &format!("/api/todos/{}", first.id),
        None,
    )
    .await;
    assert_eq!(missing_delete_response.status(), StatusCode::NOT_FOUND);
    let error: ErrorResponse = parse_json(missing_delete_response).await;
    assert_eq!(error.code, "not_found");

    cleanup_schema(&admin_pool, &schema).await?;
    Ok(())
}
