use std::env;
use std::error::Error;
use std::fmt;
use std::net::{SocketAddr, ToSocketAddrs};
use std::num::ParseIntError;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub app_env: String,
    pub host: String,
    pub port: u16,
    pub site_addr: SocketAddr,
    pub database_url: String,
    pub rust_log: String,
}

impl AppConfig {
    pub fn from_environment() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let app_env = optional_env("APP_ENV").unwrap_or_else(|| "development".to_string());
        let host = optional_env("HOST").unwrap_or_else(|| "0.0.0.0".to_string());
        let port = optional_env("PORT")
            .map(|value| parse_port("PORT", &value))
            .transpose()?
            .unwrap_or(8080);
        let database_url = required_env("DATABASE_URL")?;
        let rust_log = optional_env("RUST_LOG")
            .unwrap_or_else(|| "pgtest011=info,tower_http=info,axum::rejection=trace".to_string());
        let site_addr = resolve_socket_addr(&host, port)?;

        Ok(Self {
            app_env,
            host,
            port,
            site_addr,
            database_url,
            rust_log,
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MissingEnvVar {
        key: &'static str,
    },
    InvalidPort {
        key: &'static str,
        value: String,
        source: ParseIntError,
    },
    InvalidSocketAddress {
        host: String,
        port: u16,
        source: std::io::Error,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnvVar { key } => {
                write!(f, "missing required environment variable `{key}`")
            }
            Self::InvalidPort { key, value, .. } => {
                write!(
                    f,
                    "invalid value `{value}` for environment variable `{key}`"
                )
            }
            Self::InvalidSocketAddress { host, port, .. } => {
                write!(f, "could not resolve bind address from `{host}:{port}`")
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingEnvVar { .. } => None,
            Self::InvalidPort { source, .. } => Some(source),
            Self::InvalidSocketAddress { source, .. } => Some(source),
        }
    }
}

fn optional_env(key: &'static str) -> Option<String> {
    env::var(key).ok().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn required_env(key: &'static str) -> Result<String, ConfigError> {
    optional_env(key).ok_or(ConfigError::MissingEnvVar { key })
}

fn parse_port(key: &'static str, value: &str) -> Result<u16, ConfigError> {
    value
        .parse::<u16>()
        .map_err(|source| ConfigError::InvalidPort {
            key,
            value: value.to_string(),
            source,
        })
}

fn resolve_socket_addr(host: &str, port: u16) -> Result<SocketAddr, ConfigError> {
    (host, port)
        .to_socket_addrs()
        .map_err(|source| ConfigError::InvalidSocketAddress {
            host: host.to_string(),
            port,
            source,
        })?
        .next()
        .ok_or_else(|| ConfigError::InvalidSocketAddress {
            host: host.to_string(),
            port,
            source: std::io::Error::new(
                std::io::ErrorKind::AddrNotAvailable,
                "no socket addresses resolved",
            ),
        })
}
