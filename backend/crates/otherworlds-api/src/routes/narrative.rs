//! Routes for the Narrative Orchestration bounded context.

use axum::extract::{Path, State};
use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use uuid::Uuid;

use otherworlds_narrative::application::query_handlers::NarrativeSessionView;
use otherworlds_narrative::application::{command_handlers, query_handlers};
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
#[instrument(skip(state, request), fields(session_id = %request.session_id))]
async fn advance_beat(
    State(state): State<AppState>,
    Json(request): Json<AdvanceBeatRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::AdvanceBeat {
        correlation_id: Uuid::new_v4(),
        session_id: request.session_id,
    };

    info!(correlation_id = %command.correlation_id, "handling advance_beat command");

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
#[instrument(skip(state, request), fields(session_id = %request.session_id))]
async fn present_choice(
    State(state): State<AppState>,
    Json(request): Json<PresentChoiceRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::PresentChoice {
        correlation_id: Uuid::new_v4(),
        session_id: request.session_id,
    };

    info!(correlation_id = %command.correlation_id, "handling present_choice command");

    let stored_events = command_handlers::handle_present_choice(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// GET /{`session_id`}
#[instrument(skip(state), fields(session_id = %id))]
async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<NarrativeSessionView>, ApiError> {
    let view = query_handlers::get_session_by_id(id, &*state.event_repository).await?;
    Ok(Json(view))
}

/// Returns the router for the narrative context.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{session_id}", get(get_session))
        .route("/advance-beat", post(advance_beat))
        .route("/present-choice", post(present_choice))
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use chrono::{TimeZone, Utc};
    use otherworlds_core::clock::Clock;
    use otherworlds_core::repository::EventRepository;
    use otherworlds_core::repository::StoredEvent;
    use otherworlds_core::rng::DeterministicRng;
    use otherworlds_narrative::domain::events::{BeatAdvanced, NarrativeEventKind};
    use otherworlds_test_support::{
        EmptyEventRepository, FailingEventRepository, FixedClock, MockRng, RecordingEventRepository,
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

    #[tokio::test]
    async fn test_get_session_returns_200_with_json() {
        // Arrange
        let session_id = Uuid::new_v4();
        let beat_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: session_id,
            event_type: "narrative.beat_advanced".to_owned(),
            payload: serde_json::to_value(NarrativeEventKind::BeatAdvanced(BeatAdvanced {
                session_id,
                beat_id,
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }];
        let repo = RecordingEventRepository::new(Ok(events));
        let app = router().with_state(app_state_with(Arc::new(repo)));

        let request = Request::builder()
            .method("GET")
            .uri(format!("/{session_id}"))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["session_id"], session_id.to_string());
        assert_eq!(json["current_beat_id"], beat_id.to_string());
        assert_eq!(json["version"], 1);
    }

    #[tokio::test]
    async fn test_get_session_returns_404_when_not_found() {
        // Arrange
        let app = router().with_state(test_app_state());
        let session_id = Uuid::new_v4();

        let request = Request::builder()
            .method("GET")
            .uri(format!("/{session_id}"))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "aggregate_not_found");
    }

    #[tokio::test]
    async fn test_get_session_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let session_id = Uuid::new_v4();

        let request = Request::builder()
            .method("GET")
            .uri(format!("/{session_id}"))
            .body(Body::empty())
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
