use axum::{
    Json,
    extract::rejection::{JsonRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    BadRequest { message: String },
    Validation { message: String },
    NotFound { message: String },
    Database(sqlx::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
        }
    }

    pub fn from_json_rejection(rejection: JsonRejection) -> Self {
        Self::bad_request(format!("invalid JSON body: {}", rejection.body_text()))
    }

    pub fn from_query_rejection(rejection: QueryRejection) -> Self {
        Self::bad_request(format!("invalid query string: {}", rejection.body_text()))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => Self::not_found("requested record was not found"),
            other => Self::Database(other),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::BadRequest { message } => (StatusCode::BAD_REQUEST, "bad_request", message),
            Self::Validation { message } => (StatusCode::BAD_REQUEST, "validation_error", message),
            Self::NotFound { message } => (StatusCode::NOT_FOUND, "not_found", message),
            Self::Database(error) => {
                tracing::error!(error = %error, "database request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "database request failed".to_string(),
                )
            }
        };

        (status, Json(ErrorBody { code, message })).into_response()
    }
}
