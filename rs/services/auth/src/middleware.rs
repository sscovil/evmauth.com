use axum::{
    Json,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use uuid::Uuid;

use crate::{AppState, jwt};

/// Extension type to store the authenticated person ID and wallet address
#[derive(Debug, Clone)]
pub struct AuthenticatedPerson {
    pub person_id: Uuid,
    pub wallet_address: String,
}

/// Middleware that validates the session JWT from the cookie or Authorization header
pub async fn require_session(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let jwt_keys = match &state.jwt_keys {
        Some(keys) => keys,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "JWT not configured"})),
            )
                .into_response();
        }
    };

    // Extract token from cookie or Authorization header
    let token = extract_token(&request);

    let token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Authentication required"})),
            )
                .into_response();
        }
    };

    match jwt::verify_session_token(jwt_keys, &token) {
        Ok(claims) => {
            let person_id = match claims.sub.parse::<Uuid>() {
                Ok(id) => id,
                Err(_) => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "Invalid token subject"})),
                    )
                        .into_response();
                }
            };

            request.extensions_mut().insert(AuthenticatedPerson {
                person_id,
                wallet_address: claims.wallet,
            });

            next.run(request).await
        }
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid or expired token"})),
        )
            .into_response(),
    }
}

fn extract_token(request: &Request) -> Option<String> {
    // Try Authorization header first
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION)
        && let Ok(auth_str) = auth_header.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        return Some(token.to_string());
    }

    // Try cookie
    if let Some(cookie_header) = request.headers().get(header::COOKIE)
        && let Ok(cookies) = cookie_header.to_str()
    {
        for cookie in cookies.split(';') {
            let cookie = cookie.trim();
            if let Some(token) = cookie.strip_prefix("session=") {
                return Some(token.to_string());
            }
        }
    }

    None
}
