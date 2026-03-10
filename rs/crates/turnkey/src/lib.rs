pub mod client;
pub mod signing;
pub mod sub_org;

pub use client::TurnkeyClient;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TurnkeyError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Signing error: {0}")]
    Signing(String),

    #[error("Max retries exceeded after {attempts} attempts: {last_error}")]
    MaxRetriesExceeded { attempts: u32, last_error: String },
}
