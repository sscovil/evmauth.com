use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Resource not found")]
    NotFound,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid cursor: {0}")]
    InvalidCursor(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

impl From<pagination::PaginationError> for RepositoryError {
    fn from(err: pagination::PaginationError) -> Self {
        match err {
            pagination::PaginationError::InvalidCursor(msg) => RepositoryError::InvalidCursor(msg),
            pagination::PaginationError::InvalidParameters(msg) => {
                RepositoryError::InvalidCursor(format!("Invalid pagination parameters: {}", msg))
            }
        }
    }
}
