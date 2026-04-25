#[cfg(feature = "ssr")]
mod config;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use axum::Router;
    use leptos::config::get_configuration;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tracing::info;

    use crate::config::AppConfig;
    use pgtest011::app::{App, shell};

    let app_config = AppConfig::from_environment()?;
    initialize_tracing(&app_config)?;

    let conf = get_configuration(Some("Cargo.toml"))?;
    let mut leptos_options = conf.leptos_options;
    leptos_options.site_addr = app_config.site_addr;

    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    info!(
        app_env = %app_config.app_env,
        host = %app_config.host,
        port = app_config.port,
        database_configured = !app_config.database_url.is_empty(),
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
