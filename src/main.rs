#[cfg(feature = "ssr")]
mod config;
#[cfg(feature = "ssr")]
mod db;
#[cfg(feature = "ssr")]
mod error;
#[cfg(feature = "ssr")]
mod routes {
    pub mod health;
    pub mod todos;
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use axum::Router;
    use leptos::config::get_configuration;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tracing::info;

    use crate::config::AppConfig;
    use crate::db::{AppState, DEFAULT_MAX_DB_CONNECTIONS, create_pool};
    use pgtest011::app::{App, shell};

    let app_config = AppConfig::from_environment()?;
    initialize_tracing(&app_config)?;

    let pool = create_pool(&app_config.database_url).await?;
    sqlx::migrate!().run(&pool).await?;

    let conf = get_configuration(Some("Cargo.toml"))?;
    let mut leptos_options = conf.leptos_options;
    leptos_options.site_addr = app_config.site_addr;

    let addr = leptos_options.site_addr;
    let route_list = generate_route_list(App);
    let state = AppState::new(leptos_options.clone(), pool);

    let app = Router::<AppState>::new()
        .merge(routes::health::router())
        .merge(routes::todos::router())
        .leptos_routes(&state, route_list, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(state);

    info!(
        app_env = %app_config.app_env,
        host = %app_config.host,
        port = app_config.port,
        database_pool_ready = true,
        database_pool_max_connections = DEFAULT_MAX_DB_CONNECTIONS,
        database_migrations_ready = true,
        "starting pgtest011 server"
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(feature = "ssr")]
fn initialize_tracing(
    config: &config::AppConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let env_filter = tracing_subscriber::EnvFilter::try_new(config.rust_log.as_str())?;

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .compact()
        .try_init()?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
