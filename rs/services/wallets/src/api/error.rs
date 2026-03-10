use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::repository::RepositoryError;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl From<RepositoryError> for ApiError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => ApiError::NotFound,
            RepositoryError::InvalidCursor(msg) => {
                ApiError::BadRequest(format!("Invalid cursor: {msg}"))
            }
            RepositoryError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                ApiError::Internal("Database error".to_string())
            }
            RepositoryError::ConstraintViolation(msg) => {
                ApiError::BadRequest(format!("Constraint violation: {msg}"))
            }
        }
    }
}

impl From<pagination::PaginationError> for ApiError {
    fn from(err: pagination::PaginationError) -> Self {
        match err {
            pagination::PaginationError::InvalidCursor(msg) => {
                ApiError::BadRequest(format!("Invalid cursor: {msg}"))
            }
            pagination::PaginationError::InvalidParameters(msg) => {
                ApiError::BadRequest(format!("Invalid pagination parameters: {msg}"))
            }
        }
    }
}

impl From<turnkey::TurnkeyError> for ApiError {
    fn from(err: turnkey::TurnkeyError) -> Self {
        tracing::error!("Turnkey API error: {:?}", err);
        match err {
            turnkey::TurnkeyError::Config(msg) => {
                ApiError::Internal(format!("Turnkey configuration error: {msg}"))
            }
            turnkey::TurnkeyError::Api { status, message } => {
                ApiError::Internal(format!("Turnkey API error ({status}): {message}"))
            }
            _ => ApiError::Internal("Turnkey operation failed".to_string()),
        }
    }
}
