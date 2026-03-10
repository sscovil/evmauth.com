use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde_json::json;

use crate::api::error::ApiError;
use crate::dto::request::CreatePerson;
use crate::repository::person::{PersonRepository, PersonRepositoryImpl};
use crate::{AppState, jwt};

/// Deployer signup request
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct SignupRequest {
    /// Display name for the new person
    #[schema(example = "Alice Adams", format = "string")]
    pub display_name: String,
    /// Primary email address
    #[schema(example = "alice.adams@example.com", format = "email")]
    pub primary_email: String,
    /// Authentication provider name (e.g., "turnkey", "google")
    #[schema(example = "turnkey", format = "string")]
    pub auth_provider_name: String,
    /// Authentication provider reference (user ID from provider)
    #[schema(example = "usr_abc123xyz", format = "string")]
    pub auth_provider_ref: String,
}

/// Login request
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    /// Primary email address
    #[schema(example = "alice.adams@example.com", format = "email")]
    pub primary_email: String,
    /// Authentication provider name
    #[schema(example = "turnkey", format = "string")]
    pub auth_provider_name: String,
    /// Authentication provider reference
    #[schema(example = "usr_abc123xyz", format = "string")]
    pub auth_provider_ref: String,
}

/// Token response
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct TokenResponse {
    /// The JWT access token
    #[schema(example = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    /// Token type (always "Bearer")
    #[schema(example = "Bearer")]
    pub token_type: String,
    /// Token lifetime in seconds
    #[schema(example = 28800)]
    pub expires_in: i64,
}

/// Deployer signup
#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "Account created", body = TokenResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn signup(
    State(state): State<AppState>,
    Json(body): Json<SignupRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let jwt_keys = state
        .jwt_keys
        .as_ref()
        .ok_or_else(|| ApiError::Internal("JWT not configured".to_string()))?;

    let repo = PersonRepositoryImpl::new(&state.db);

    let person = repo
        .create(CreatePerson {
            display_name: body.display_name,
            description: None,
            auth_provider_name: body.auth_provider_name,
            auth_provider_ref: body.auth_provider_ref,
            primary_email: body.primary_email,
        })
        .await?;

    let duration_hours = 8;
    let token = jwt::create_session_token(jwt_keys, person.id, duration_hours)
        .map_err(|e| ApiError::Internal(format!("Failed to create token: {e}")))?;

    let cookie = format!(
        "session={token}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
        duration_hours * 3600
    );

    Ok((
        StatusCode::CREATED,
        [(header::SET_COOKIE, cookie)],
        Json(TokenResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: duration_hours * 3600,
        }),
    ))
}

/// Deployer login
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = TokenResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let jwt_keys = state
        .jwt_keys
        .as_ref()
        .ok_or_else(|| ApiError::Internal("JWT not configured".to_string()))?;

    let repo = PersonRepositoryImpl::new(&state.db);

    // Look up person by email and auth provider
    let people = repo
        .list(
            crate::repository::PersonFilter::new()
                .email(&body.primary_email)
                .auth_provider(&body.auth_provider_name),
            crate::repository::Page {
                first: Some(1),
                ..Default::default()
            },
        )
        .await?;

    let person = people.first().ok_or(ApiError::NotFound)?;

    // Verify auth provider ref matches
    if person.auth_provider_ref != body.auth_provider_ref {
        return Err(ApiError::BadRequest("Invalid credentials".to_string()));
    }

    let duration_hours = 8;
    let token = jwt::create_session_token(jwt_keys, person.id, duration_hours)
        .map_err(|e| ApiError::Internal(format!("Failed to create token: {e}")))?;

    let cookie = format!(
        "session={token}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
        duration_hours * 3600
    );

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie)],
        Json(TokenResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: duration_hours * 3600,
        }),
    ))
}

/// Logout - clears the session cookie
#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logged out")
    ),
    tag = "auth"
)]
pub async fn logout() -> impl IntoResponse {
    let cookie = "session=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0";
    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(json!({"status": "logged out"})),
    )
}
