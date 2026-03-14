use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, Method},
    response::Response,
};
use std::sync::Arc;

use super::error::ProxyError;
use crate::config::Config;

const SKIP_HEADERS: &[&str] = &["host", "connection", "transfer-encoding"];

pub struct AppState {
    pub config: Config,
    pub client: reqwest::Client,
}

pub async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response, ProxyError> {
    let path = req.uri().path();
    let query = req.uri().query();

    // Route root and OpenAPI spec to docs service
    let (service_name, backend_path): (&str, String) = if path == "/" {
        ("docs", "/".to_string())
    } else if path == "/openapi.json" {
        ("docs", "/openapi.json".to_string())
    } else {
        // Extract service name from path: /auth/people -> service="auth", remaining="/people"
        let parts: Vec<&str> = path.trim_start_matches('/').splitn(2, '/').collect();

        if parts.is_empty() {
            return Err(ProxyError::InvalidPath);
        }

        let service = parts[0];
        let backend = if parts.len() > 1 {
            format!("/{}", parts[1])
        } else {
            "/".to_string()
        };

        (service, backend)
    };

    // Look up service
    let service = state
        .config
        .services
        .iter()
        .find(|s| s.name == service_name)
        .ok_or_else(|| ProxyError::ServiceNotFound(service_name.to_string()))?;

    // Build backend URL
    let mut backend_url = format!("{}{}", service.base_url, backend_path);
    if let Some(q) = query {
        backend_url.push('?');
        backend_url.push_str(q);
    }

    tracing::debug!("Proxying {} {} -> {}", req.method(), path, backend_url);

    // Forward request
    let method = req.method().clone();
    let headers = copy_headers(req.headers());
    let body = req.into_body();

    let backend_response =
        forward_request(&state.client, method, &backend_url, headers, body).await?;

    // Convert backend response to axum Response
    Ok(backend_response)
}

fn copy_headers(headers: &HeaderMap) -> HeaderMap {
    let mut new_headers = HeaderMap::new();

    // Copy all headers except those that should be set by reqwest
    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if !SKIP_HEADERS.contains(&key_str.as_str()) {
            new_headers.insert(key.clone(), value.clone());
        }
    }

    new_headers
}

async fn forward_request(
    client: &reqwest::Client,
    method: Method,
    url: &str,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, ProxyError> {
    // No body size limit: gateway proxies requests transparently to backend services
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|e| ProxyError::BadGateway(format!("Failed to read request body: {}", e)))?;

    // Build reqwest request
    let mut req_builder = match method {
        Method::GET => client.get(url),
        Method::POST => client.post(url),
        Method::PUT => client.put(url),
        Method::PATCH => client.patch(url),
        Method::DELETE => client.delete(url),
        Method::HEAD => client.head(url),
        Method::OPTIONS => client.request(Method::OPTIONS, url),
        _ => return Err(ProxyError::InvalidPath),
    };

    // Add headers
    for (key, value) in headers.iter() {
        req_builder = req_builder.header(key, value);
    }

    // Add body if present
    if !body_bytes.is_empty() {
        req_builder = req_builder.body(body_bytes.to_vec());
    }

    // Send request
    let backend_resp = req_builder.send().await?;

    // Convert reqwest response to axum response
    let status = backend_resp.status();
    let headers = backend_resp.headers().clone();
    let body_bytes = backend_resp.bytes().await?;

    let mut response = Response::builder().status(status);

    // Copy response headers
    for (key, value) in headers.iter() {
        response = response.header(key, value);
    }

    response
        .body(Body::from(body_bytes))
        .map_err(|e| ProxyError::BadGateway(format!("Failed to build response: {e}")))
}
