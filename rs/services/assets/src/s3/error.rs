use thiserror::Error;

#[derive(Debug, Error)]
pub enum S3Error {
    #[error("Failed to upload file: {0}")]
    UploadFailed(String),

    #[error("Failed to delete file: {0}")]
    DeleteFailed(String),

    #[error("Failed to generate presigned URL: {0}")]
    PresignFailed(String),

    #[error("S3 client configuration error: {0}")]
    ConfigError(String),

    #[error("S3 connection error: {0}")]
    ConnectionError(String),
}
