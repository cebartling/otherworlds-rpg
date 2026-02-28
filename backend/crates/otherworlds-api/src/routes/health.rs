//! Health check endpoint.

use axum::{Json, Router, routing::get};
use serde::Serialize;

use crate::state::AppState;

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    /// Service status.
    pub status: String,
    /// Service version.
    pub version: String,
}

/// GET /health
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Returns the health check router.
pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}
