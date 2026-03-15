use thiserror::Error;

#[derive(Debug, Error)]
pub enum S3Error {
    #[error("failed to upload file: {0}")]
    UploadFailed(String),

    #[error("failed to delete file: {0}")]
    DeleteFailed(String),

    #[error("failed to generate presigned url: {0}")]
    PresignFailed(String),

    #[error("s3 client configuration error: {0}")]
    ConfigError(String),

    #[error("s3 connection error: {0}")]
    ConnectionError(String),
}
