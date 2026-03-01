//! Routes for the Rules & Resolution bounded context.

use axum::extract::State;
use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use uuid::Uuid;

use otherworlds_rules::application::command_handlers;
use otherworlds_rules::domain::commands;

use crate::error::ApiError;
use crate::state::AppState;

/// Request body for POST /resolve-intent.
#[derive(Debug, Deserialize)]
pub struct ResolveIntentRequest {
    /// The intent to resolve.
    pub intent_id: Uuid,
}

/// Request body for POST /perform-check.
#[derive(Debug, Deserialize)]
pub struct PerformCheckRequest {
    /// The resolution this check belongs to.
    pub resolution_id: Uuid,
}

/// Response body returned after a command is successfully handled.
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    /// IDs of the domain events produced and persisted.
    pub event_ids: Vec<Uuid>,
}

/// POST /resolve-intent
#[instrument(skip(state, request), fields(intent_id = %request.intent_id))]
async fn resolve_intent(
    State(state): State<AppState>,
    Json(request): Json<ResolveIntentRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::ResolveIntent {
        correlation_id: Uuid::new_v4(),
        intent_id: request.intent_id,
    };

    info!(correlation_id = %command.correlation_id, "handling resolve_intent command");

    let stored_events = command_handlers::handle_resolve_intent(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// POST /perform-check
#[instrument(skip(state, request), fields(resolution_id = %request.resolution_id))]
async fn perform_check(
    State(state): State<AppState>,
    Json(request): Json<PerformCheckRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::PerformCheck {
        correlation_id: Uuid::new_v4(),
        resolution_id: request.resolution_id,
    };

    info!(correlation_id = %command.correlation_id, "handling perform_check command");

    let stored_events = command_handlers::handle_perform_check(
        &command,
        state.clock.as_ref(),
        &state.rng,
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// Returns the router for the rules context.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/resolve-intent", post(resolve_intent))
        .route("/perform-check", post(perform_check))
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use chrono::Utc;
    use otherworlds_core::clock::Clock;
    use otherworlds_core::repository::EventRepository;
    use otherworlds_core::rng::DeterministicRng;
    use otherworlds_test_support::{
        EmptyEventRepository, FailingEventRepository, FixedClock, MockRng,
    };
    use serde_json::Value;
    use sqlx::PgPool;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    fn app_state_with(event_repository: Arc<dyn EventRepository>) -> AppState {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let clock: Arc<dyn Clock + Send + Sync> = Arc::new(FixedClock(Utc::now()));
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        AppState::new(pool, clock, rng, event_repository)
    }

    fn test_app_state() -> AppState {
        app_state_with(Arc::new(EmptyEventRepository))
    }

    fn failing_app_state() -> AppState {
        app_state_with(Arc::new(FailingEventRepository))
    }

    #[tokio::test]
    async fn test_resolve_intent_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let intent_id = Uuid::new_v4();
        let body = serde_json::json!({ "intent_id": intent_id });

        let request = Request::builder()
            .method("POST")
            .uri("/resolve-intent")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        let event_ids = json["event_ids"].as_array().unwrap();
        assert_eq!(event_ids.len(), 1);
        // Verify each event_id is a valid UUID.
        for id in event_ids {
            Uuid::parse_str(id.as_str().unwrap()).unwrap();
        }
    }

    #[tokio::test]
    async fn test_perform_check_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let resolution_id = Uuid::new_v4();
        let body = serde_json::json!({ "resolution_id": resolution_id });

        let request = Request::builder()
            .method("POST")
            .uri("/perform-check")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        let event_ids = json["event_ids"].as_array().unwrap();
        assert_eq!(event_ids.len(), 1);
        for id in event_ids {
            Uuid::parse_str(id.as_str().unwrap()).unwrap();
        }
    }

    #[tokio::test]
    async fn test_resolve_intent_returns_422_for_missing_body() {
        // Arrange
        let app = router().with_state(test_app_state());

        let request = Request::builder()
            .method("POST")
            .uri("/resolve-intent")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert — Axum returns 422 for deserialization failures.
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_resolve_intent_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let intent_id = Uuid::new_v4();
        let body = serde_json::json!({ "intent_id": intent_id });

        let request = Request::builder()
            .method("POST")
            .uri("/resolve-intent")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "infrastructure_error");
    }

    #[tokio::test]
    async fn test_perform_check_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let resolution_id = Uuid::new_v4();
        let body = serde_json::json!({ "resolution_id": resolution_id });

        let request = Request::builder()
            .method("POST")
            .uri("/perform-check")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "infrastructure_error");
    }
}
