//! Routes for the World State bounded context.

use axum::extract::{Path, State};
use axum::{Json, Router, routing::{get, post}};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use uuid::Uuid;

use otherworlds_world_state::application::{command_handlers, query_handlers};
use otherworlds_world_state::application::query_handlers::WorldSnapshotView;
use otherworlds_world_state::domain::commands;

use crate::error::ApiError;
use crate::state::AppState;

/// Request body for POST /apply-effect.
#[derive(Debug, Deserialize)]
pub struct ApplyEffectRequest {
    /// The world snapshot identifier.
    pub world_id: Uuid,
    /// The fact key to apply.
    pub fact_key: String,
}

/// Request body for POST /set-flag.
#[derive(Debug, Deserialize)]
pub struct SetFlagRequest {
    /// The world snapshot identifier.
    pub world_id: Uuid,
    /// The flag key.
    pub flag_key: String,
    /// The flag value.
    pub value: bool,
}

/// Request body for POST /update-disposition.
#[derive(Debug, Deserialize)]
pub struct UpdateDispositionRequest {
    /// The world snapshot identifier.
    pub world_id: Uuid,
    /// The entity whose disposition to update.
    pub entity_id: Uuid,
}

/// Response body returned after a command is successfully handled.
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    /// IDs of the domain events produced and persisted.
    pub event_ids: Vec<Uuid>,
}

/// POST /apply-effect
#[instrument(skip(state, request), fields(world_id = %request.world_id))]
async fn apply_effect(
    State(state): State<AppState>,
    Json(request): Json<ApplyEffectRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::ApplyEffect {
        correlation_id: Uuid::new_v4(),
        world_id: request.world_id,
        fact_key: request.fact_key,
    };

    info!(correlation_id = %command.correlation_id, "handling apply_effect command");

    let stored_events = command_handlers::handle_apply_effect(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// POST /set-flag
#[instrument(skip(state, request), fields(world_id = %request.world_id))]
async fn set_flag(
    State(state): State<AppState>,
    Json(request): Json<SetFlagRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::SetFlag {
        correlation_id: Uuid::new_v4(),
        world_id: request.world_id,
        flag_key: request.flag_key,
        value: request.value,
    };

    info!(correlation_id = %command.correlation_id, "handling set_flag command");

    let stored_events =
        command_handlers::handle_set_flag(&command, state.clock.as_ref(), &*state.event_repository)
            .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// POST /update-disposition
#[instrument(skip(state, request), fields(world_id = %request.world_id))]
async fn update_disposition(
    State(state): State<AppState>,
    Json(request): Json<UpdateDispositionRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::UpdateDisposition {
        correlation_id: Uuid::new_v4(),
        world_id: request.world_id,
        entity_id: request.entity_id,
    };

    info!(correlation_id = %command.correlation_id, "handling update_disposition command");

    let stored_events = command_handlers::handle_update_disposition(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// GET /{`world_id`}
#[instrument(skip(state), fields(world_id = %id))]
async fn get_world_snapshot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorldSnapshotView>, ApiError> {
    let view = query_handlers::get_world_snapshot_by_id(id, &*state.event_repository).await?;
    Ok(Json(view))
}

/// Returns the router for the world state context.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{world_id}", get(get_world_snapshot))
        .route("/apply-effect", post(apply_effect))
        .route("/set-flag", post(set_flag))
        .route("/update-disposition", post(update_disposition))
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
    use otherworlds_core::repository::StoredEvent;
    use otherworlds_test_support::{
        EmptyEventRepository, FailingEventRepository, FixedClock, MockRng, RecordingEventRepository,
    };
    use otherworlds_world_state::domain::events::{WorldFactChanged, WorldStateEventKind};
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
    async fn test_apply_effect_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let world_id = Uuid::new_v4();
        let body = serde_json::json!({
            "world_id": world_id,
            "fact_key": "quest_complete"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/apply-effect")
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
    async fn test_set_flag_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let world_id = Uuid::new_v4();
        let body = serde_json::json!({
            "world_id": world_id,
            "flag_key": "door_unlocked",
            "value": true
        });

        let request = Request::builder()
            .method("POST")
            .uri("/set-flag")
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
    async fn test_update_disposition_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let world_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();
        let body = serde_json::json!({
            "world_id": world_id,
            "entity_id": entity_id
        });

        let request = Request::builder()
            .method("POST")
            .uri("/update-disposition")
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
    async fn test_apply_effect_returns_422_for_missing_body() {
        // Arrange
        let app = router().with_state(test_app_state());

        let request = Request::builder()
            .method("POST")
            .uri("/apply-effect")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert — Axum returns 422 for deserialization failures.
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_apply_effect_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let world_id = Uuid::new_v4();
        let body = serde_json::json!({
            "world_id": world_id,
            "fact_key": "quest_complete"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/apply-effect")
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
    async fn test_set_flag_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let world_id = Uuid::new_v4();
        let body = serde_json::json!({
            "world_id": world_id,
            "flag_key": "door_unlocked",
            "value": true
        });

        let request = Request::builder()
            .method("POST")
            .uri("/set-flag")
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
    async fn test_update_disposition_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let world_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();
        let body = serde_json::json!({
            "world_id": world_id,
            "entity_id": entity_id
        });

        let request = Request::builder()
            .method("POST")
            .uri("/update-disposition")
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
    async fn test_apply_effect_returns_400_for_empty_fact_key() {
        // Arrange
        let app = router().with_state(test_app_state());
        let world_id = Uuid::new_v4();
        let body = serde_json::json!({
            "world_id": world_id,
            "fact_key": "  "
        });

        let request = Request::builder()
            .method("POST")
            .uri("/apply-effect")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "validation_error");
    }

    #[tokio::test]
    async fn test_get_world_snapshot_returns_200_with_json() {
        // Arrange
        let world_id = Uuid::new_v4();
        let fixed_now = chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 1, 15, 10, 0, 0).unwrap();
        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: world_id,
            event_type: "world_state.world_fact_changed".to_owned(),
            payload: serde_json::to_value(WorldStateEventKind::WorldFactChanged(
                WorldFactChanged {
                    world_id,
                    fact_key: "quest_complete".to_owned(),
                },
            ))
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
            .uri(format!("/{world_id}"))
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

        assert_eq!(json["world_id"], world_id.to_string());
        assert_eq!(json["facts"], serde_json::json!(["quest_complete"]));
        assert_eq!(json["version"], 1);
    }

    #[tokio::test]
    async fn test_get_world_snapshot_returns_404_when_not_found() {
        // Arrange
        let app = router().with_state(test_app_state());
        let world_id = Uuid::new_v4();

        let request = Request::builder()
            .method("GET")
            .uri(format!("/{world_id}"))
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
    async fn test_get_world_snapshot_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let world_id = Uuid::new_v4();

        let request = Request::builder()
            .method("GET")
            .uri(format!("/{world_id}"))
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

    #[tokio::test]
    async fn test_set_flag_returns_400_for_empty_flag_key() {
        // Arrange
        let app = router().with_state(test_app_state());
        let world_id = Uuid::new_v4();
        let body = serde_json::json!({
            "world_id": world_id,
            "flag_key": "",
            "value": true
        });

        let request = Request::builder()
            .method("POST")
            .uri("/set-flag")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "validation_error");
    }
}
