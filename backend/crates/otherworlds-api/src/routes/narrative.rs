//! Routes for the Narrative Orchestration bounded context.

use axum::extract::State;
use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use otherworlds_narrative::application::command_handlers;
use otherworlds_narrative::domain::commands;

use crate::error::ApiError;
use crate::state::AppState;

/// Request body for POST /advance-beat.
#[derive(Debug, Deserialize)]
pub struct AdvanceBeatRequest {
    /// The narrative session to advance.
    pub session_id: Uuid,
}

/// Request body for POST /present-choice.
#[derive(Debug, Deserialize)]
pub struct PresentChoiceRequest {
    /// The narrative session to present a choice in.
    pub session_id: Uuid,
}

/// Response body returned after a command is successfully handled.
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    /// IDs of the domain events produced and persisted.
    pub event_ids: Vec<Uuid>,
}

/// POST /advance-beat
async fn advance_beat(
    State(state): State<AppState>,
    Json(request): Json<AdvanceBeatRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::AdvanceBeat {
        correlation_id: Uuid::new_v4(),
        session_id: request.session_id,
    };

    let stored_events = command_handlers::handle_advance_beat(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// POST /present-choice
async fn present_choice(
    State(state): State<AppState>,
    Json(request): Json<PresentChoiceRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::PresentChoice {
        correlation_id: Uuid::new_v4(),
        session_id: request.session_id,
    };

    let stored_events = command_handlers::handle_present_choice(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// Returns the router for the narrative context.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/advance-beat", post(advance_beat))
        .route("/present-choice", post(present_choice))
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use chrono::{DateTime, Utc};
    use otherworlds_core::clock::Clock;
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::{EventRepository, StoredEvent};
    use otherworlds_core::rng::DeterministicRng;
    use serde_json::Value;
    use sqlx::PgPool;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    #[derive(Debug)]
    struct FixedClock(DateTime<Utc>);

    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
        }
    }

    #[derive(Debug)]
    struct MockRng;

    impl DeterministicRng for MockRng {
        fn next_u32_range(&mut self, min: u32, _max: u32) -> u32 {
            min
        }

        fn next_f64(&mut self) -> f64 {
            0.0
        }
    }

    #[derive(Debug)]
    struct MockEventRepository;

    #[async_trait::async_trait]
    impl EventRepository for MockEventRepository {
        async fn load_events(&self, _aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
            Ok(vec![])
        }

        async fn append_events(
            &self,
            _aggregate_id: Uuid,
            _expected_version: i64,
            _events: &[StoredEvent],
        ) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct FailingEventRepository;

    #[async_trait::async_trait]
    impl EventRepository for FailingEventRepository {
        async fn load_events(&self, _aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
            Err(DomainError::Infrastructure("connection refused".into()))
        }

        async fn append_events(
            &self,
            _aggregate_id: Uuid,
            _expected_version: i64,
            _events: &[StoredEvent],
        ) -> Result<(), DomainError> {
            Err(DomainError::Infrastructure("connection refused".into()))
        }
    }

    fn test_app_state() -> AppState {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let clock: Arc<dyn Clock + Send + Sync> = Arc::new(FixedClock(Utc::now()));
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let event_repository: Arc<dyn EventRepository> = Arc::new(MockEventRepository);
        AppState::new(pool, clock, rng, event_repository)
    }

    fn failing_app_state() -> AppState {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let clock: Arc<dyn Clock + Send + Sync> = Arc::new(FixedClock(Utc::now()));
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let event_repository: Arc<dyn EventRepository> = Arc::new(FailingEventRepository);
        AppState::new(pool, clock, rng, event_repository)
    }

    #[tokio::test]
    async fn test_advance_beat_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let session_id = Uuid::new_v4();
        let body = serde_json::json!({ "session_id": session_id });

        let request = Request::builder()
            .method("POST")
            .uri("/advance-beat")
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
    async fn test_present_choice_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let session_id = Uuid::new_v4();
        let body = serde_json::json!({ "session_id": session_id });

        let request = Request::builder()
            .method("POST")
            .uri("/present-choice")
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
    async fn test_advance_beat_returns_422_for_missing_body() {
        // Arrange
        let app = router().with_state(test_app_state());

        let request = Request::builder()
            .method("POST")
            .uri("/advance-beat")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert — Axum returns 422 for deserialization failures.
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_advance_beat_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let session_id = Uuid::new_v4();
        let body = serde_json::json!({ "session_id": session_id });

        let request = Request::builder()
            .method("POST")
            .uri("/advance-beat")
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
    async fn test_present_choice_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let session_id = Uuid::new_v4();
        let body = serde_json::json!({ "session_id": session_id });

        let request = Request::builder()
            .method("POST")
            .uri("/present-choice")
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
