use thiserror::Error;

/// Errors that can occur during cursor-based pagination operations.
#[derive(Debug, Error)]
pub enum PaginationError {
    /// The provided cursor string could not be decoded (malformed base64 or JSON).
    #[error("invalid cursor: {0}")]
    InvalidCursor(String),
    /// The pagination parameters violate Relay spec rules (e.g., using both `first` and `last`).
    #[error("invalid pagination parameters: {0}")]
    InvalidParameters(String),
}
