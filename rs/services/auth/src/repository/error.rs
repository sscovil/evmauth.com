use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("resource not found")]
    NotFound,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("invalid cursor: {0}")]
    InvalidCursor(String),

    #[error("constraint violation: {0}")]
    ConstraintViolation(String),
}

impl From<pagination::PaginationError> for RepositoryError {
    fn from(err: pagination::PaginationError) -> Self {
        match err {
            pagination::PaginationError::InvalidCursor(msg) => RepositoryError::InvalidCursor(msg),
            pagination::PaginationError::InvalidParameters(msg) => {
                RepositoryError::InvalidCursor(format!("invalid pagination parameters: {}", msg))
            }
        }
    }
}
