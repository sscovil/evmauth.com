use thiserror::Error;

#[derive(Debug, Error)]
pub enum PaginationError {
    #[error("Invalid cursor: {0}")]
    InvalidCursor(String),
    #[error("Invalid pagination parameters: {0}")]
    InvalidParameters(String),
}
