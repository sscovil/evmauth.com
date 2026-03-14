pub mod client;
pub mod signing;
pub mod sub_org;

pub use client::TurnkeyClient;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TurnkeyError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("api error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("signing error: {0}")]
    Signing(String),

    #[error("max retries exceeded after {attempts} attempts: {last_error}")]
    MaxRetriesExceeded { attempts: u32, last_error: String },
}
