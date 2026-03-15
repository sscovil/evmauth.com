use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::repository::RepositoryError;
use crate::s3::S3Error;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("resource not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal server error: {0}")]
    Internal(String),
    #[error("payload too large: {0}")]
    PayloadTooLarge(String),
    #[error("unsupported media type: {0}")]
    UnsupportedMediaType(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "resource not found".to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::PayloadTooLarge(msg) => (StatusCode::PAYLOAD_TOO_LARGE, msg),
            ApiError::UnsupportedMediaType(msg) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, msg),
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
                ApiError::BadRequest(format!("invalid cursor: {msg}"))
            }
            RepositoryError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                ApiError::Internal("database error".to_string())
            }
            RepositoryError::ConstraintViolation(msg) => {
                ApiError::BadRequest(format!("constraint violation: {msg}"))
            }
        }
    }
}

impl From<S3Error> for ApiError {
    fn from(err: S3Error) -> Self {
        tracing::error!("S3 error: {:?}", err);
        ApiError::Internal("Storage error".to_string())
    }
}

impl From<pagination::PaginationError> for ApiError {
    fn from(err: pagination::PaginationError) -> Self {
        match err {
            pagination::PaginationError::InvalidCursor(msg) => {
                ApiError::BadRequest(format!("invalid cursor: {msg}"))
            }
            pagination::PaginationError::InvalidParameters(msg) => {
                ApiError::BadRequest(format!("invalid pagination parameters: {msg}"))
            }
        }
    }
}
