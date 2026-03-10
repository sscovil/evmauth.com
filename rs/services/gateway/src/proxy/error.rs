use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Service '{0}' not found")]
    ServiceNotFound(String),

    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid path")]
    InvalidPath,

    #[error("Bad gateway: {0}")]
    BadGateway(String),
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ProxyError::ServiceNotFound(service) => (
                StatusCode::NOT_FOUND,
                format!("Service '{}' not found", service),
            ),
            ProxyError::RequestFailed(e) => {
                tracing::error!("Backend request failed: {:?}", e);
                (
                    StatusCode::BAD_GATEWAY,
                    "Backend service unavailable".to_string(),
                )
            }
            ProxyError::InvalidPath => {
                (StatusCode::BAD_REQUEST, "Invalid request path".to_string())
            }
            ProxyError::BadGateway(msg) => (StatusCode::BAD_GATEWAY, msg),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}
