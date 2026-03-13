use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::AppState;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy"),
        (status = 503, description = "Service is unhealthy")
    ),
    tag = "health"
)]
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let db_healthy = sqlx::query("SELECT 1").fetch_one(&state.db).await.is_ok();

    let redis_healthy = redis::cmd("PING")
        .query_async::<String>(&mut state.redis.clone())
        .await
        .is_ok();

    let status = if db_healthy && redis_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(json!({
            "status": if status == StatusCode::OK { "healthy" } else { "unhealthy" },
            "database": if db_healthy { "connected" } else { "disconnected" },
            "redis": if redis_healthy { "connected" } else { "disconnected" },
        })),
    )
}
