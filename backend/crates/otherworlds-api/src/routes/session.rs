//! Routes for the Session & Progress bounded context.

use axum::extract::{Path, State};
use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use uuid::Uuid;

use otherworlds_session::application::query_handlers::CampaignRunView;
use otherworlds_session::application::{command_handlers, query_handlers};
use otherworlds_session::domain::commands;

use crate::error::ApiError;
use crate::state::AppState;

/// Request body for POST /start-campaign-run.
#[derive(Debug, Deserialize)]
pub struct StartCampaignRunRequest {
    /// The campaign to start a run for.
    pub campaign_id: Uuid,
}

/// Request body for POST /create-checkpoint.
#[derive(Debug, Deserialize)]
pub struct CreateCheckpointRequest {
    /// The campaign run to create a checkpoint for.
    pub run_id: Uuid,
}

/// Request body for POST /branch-timeline.
#[derive(Debug, Deserialize)]
pub struct BranchTimelineRequest {
    /// The campaign run to branch from.
    pub source_run_id: Uuid,
    /// The checkpoint to branch from.
    pub from_checkpoint_id: Uuid,
}

/// Response body returned after a command is successfully handled.
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    /// The aggregate ID affected or created by the command.
    pub aggregate_id: Uuid,
    /// IDs of the domain events produced and persisted.
    pub event_ids: Vec<Uuid>,
}

/// POST /start-campaign-run
#[instrument(skip(state, request), fields(campaign_id = %request.campaign_id))]
async fn start_campaign_run(
    State(state): State<AppState>,
    Json(request): Json<StartCampaignRunRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::StartCampaignRun {
        correlation_id: Uuid::new_v4(),
        campaign_id: request.campaign_id,
    };

    info!(correlation_id = %command.correlation_id, "handling start_campaign_run command");

    let result = command_handlers::handle_start_campaign_run(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = result.stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse {
        aggregate_id: result.aggregate_id,
        event_ids,
    }))
}

/// POST /create-checkpoint
#[instrument(skip(state, request), fields(run_id = %request.run_id))]
async fn create_checkpoint(
    State(state): State<AppState>,
    Json(request): Json<CreateCheckpointRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::CreateCheckpoint {
        correlation_id: Uuid::new_v4(),
        run_id: request.run_id,
    };

    info!(correlation_id = %command.correlation_id, "handling create_checkpoint command");

    let result = command_handlers::handle_create_checkpoint(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = result.stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse {
        aggregate_id: result.aggregate_id,
        event_ids,
    }))
}

/// POST /branch-timeline
#[instrument(skip(state, request), fields(source_run_id = %request.source_run_id))]
async fn branch_timeline(
    State(state): State<AppState>,
    Json(request): Json<BranchTimelineRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::BranchTimeline {
        correlation_id: Uuid::new_v4(),
        source_run_id: request.source_run_id,
        from_checkpoint_id: request.from_checkpoint_id,
    };

    info!(correlation_id = %command.correlation_id, "handling branch_timeline command");

    let result = command_handlers::handle_branch_timeline(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = result.stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse {
        aggregate_id: result.aggregate_id,
        event_ids,
    }))
}

/// GET /{`run_id`}
#[instrument(skip(state), fields(run_id = %id))]
async fn get_campaign_run(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CampaignRunView>, ApiError> {
    let view = query_handlers::get_campaign_run_by_id(id, &*state.event_repository).await?;
    Ok(Json(view))
}

/// Returns the router for the session context.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{run_id}", get(get_campaign_run))
        .route("/start-campaign-run", post(start_campaign_run))
        .route("/create-checkpoint", post(create_checkpoint))
        .route("/branch-timeline", post(branch_timeline))
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use chrono::{TimeZone, Utc};
    use otherworlds_core::clock::Clock;
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::{EventRepository, StoredEvent};
    use otherworlds_core::rng::DeterministicRng;
    use otherworlds_session::domain::events::{CampaignRunStarted, SessionEventKind};
    use otherworlds_test_support::{
        EmptyEventRepository, FailingEventRepository, FixedClock, MockRng, RecordingEventRepository,
    };
    use serde_json::Value;
    use sqlx::PgPool;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    #[derive(Debug)]
    struct MockEventRepository;

    #[async_trait::async_trait]
    impl EventRepository for MockEventRepository {
        async fn load_events(&self, aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
            // Return a dummy event so existence checks pass.
            Ok(vec![StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                event_type: "session.campaign_run_started".to_owned(),
                payload: serde_json::to_value(SessionEventKind::CampaignRunStarted(
                    CampaignRunStarted {
                        run_id: aggregate_id,
                        campaign_id: Uuid::new_v4(),
                    },
                ))
                .unwrap(),
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            }])
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

    fn app_state_with(event_repository: Arc<dyn EventRepository>) -> AppState {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let clock: Arc<dyn Clock + Send + Sync> = Arc::new(FixedClock(
            Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        ));
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        AppState::new(pool, clock, rng, event_repository)
    }

    fn test_app_state() -> AppState {
        app_state_with(Arc::new(MockEventRepository))
    }

    fn empty_app_state() -> AppState {
        app_state_with(Arc::new(EmptyEventRepository))
    }

    fn failing_app_state() -> AppState {
        app_state_with(Arc::new(FailingEventRepository))
    }

    #[tokio::test]
    async fn test_start_campaign_run_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let campaign_id = Uuid::new_v4();
        let body = serde_json::json!({ "campaign_id": campaign_id });

        let request = Request::builder()
            .method("POST")
            .uri("/start-campaign-run")
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

        // Verify aggregate_id is a valid UUID (the newly created run_id).
        Uuid::parse_str(json["aggregate_id"].as_str().unwrap()).unwrap();

        let event_ids = json["event_ids"].as_array().unwrap();
        assert_eq!(event_ids.len(), 1);
        for id in event_ids {
            Uuid::parse_str(id.as_str().unwrap()).unwrap();
        }
    }

    #[tokio::test]
    async fn test_create_checkpoint_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let run_id = Uuid::new_v4();
        let body = serde_json::json!({ "run_id": run_id });

        let request = Request::builder()
            .method("POST")
            .uri("/create-checkpoint")
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

        // Verify aggregate_id matches the run_id from the request.
        let returned_id = Uuid::parse_str(json["aggregate_id"].as_str().unwrap()).unwrap();
        assert_eq!(returned_id, run_id);

        let event_ids = json["event_ids"].as_array().unwrap();
        assert_eq!(event_ids.len(), 1);
        for id in event_ids {
            Uuid::parse_str(id.as_str().unwrap()).unwrap();
        }
    }

    #[tokio::test]
    async fn test_branch_timeline_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let source_run_id = Uuid::new_v4();
        let from_checkpoint_id = Uuid::new_v4();
        let body = serde_json::json!({
            "source_run_id": source_run_id,
            "from_checkpoint_id": from_checkpoint_id
        });

        let request = Request::builder()
            .method("POST")
            .uri("/branch-timeline")
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

        // Verify aggregate_id is a valid UUID (the newly created branch_run_id).
        Uuid::parse_str(json["aggregate_id"].as_str().unwrap()).unwrap();

        let event_ids = json["event_ids"].as_array().unwrap();
        assert_eq!(event_ids.len(), 1);
        for id in event_ids {
            Uuid::parse_str(id.as_str().unwrap()).unwrap();
        }
    }

    #[tokio::test]
    async fn test_start_campaign_run_returns_422_for_missing_body() {
        // Arrange
        let app = router().with_state(test_app_state());

        let request = Request::builder()
            .method("POST")
            .uri("/start-campaign-run")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert — Axum returns 422 for deserialization failures.
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_checkpoint_returns_404_when_run_not_found() {
        // Arrange
        let app = router().with_state(empty_app_state());
        let run_id = Uuid::new_v4();
        let body = serde_json::json!({ "run_id": run_id });

        let request = Request::builder()
            .method("POST")
            .uri("/create-checkpoint")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
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
    async fn test_branch_timeline_returns_404_when_source_run_not_found() {
        // Arrange
        let app = router().with_state(empty_app_state());
        let source_run_id = Uuid::new_v4();
        let from_checkpoint_id = Uuid::new_v4();
        let body = serde_json::json!({
            "source_run_id": source_run_id,
            "from_checkpoint_id": from_checkpoint_id
        });

        let request = Request::builder()
            .method("POST")
            .uri("/branch-timeline")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
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
    async fn test_create_checkpoint_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let run_id = Uuid::new_v4();
        let body = serde_json::json!({ "run_id": run_id });

        let request = Request::builder()
            .method("POST")
            .uri("/create-checkpoint")
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
    async fn test_get_campaign_run_returns_200_with_json() {
        // Arrange
        let run_id = Uuid::new_v4();
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: run_id,
            event_type: "session.campaign_run_started".to_owned(),
            payload: serde_json::to_value(SessionEventKind::CampaignRunStarted(
                CampaignRunStarted {
                    run_id,
                    campaign_id,
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
            .uri(format!("/{run_id}"))
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

        assert_eq!(json["run_id"], run_id.to_string());
        assert_eq!(json["campaign_id"], campaign_id.to_string());
        assert_eq!(json["version"], 1);
    }

    #[tokio::test]
    async fn test_get_campaign_run_returns_404_when_not_found() {
        // Arrange
        let app = router().with_state(empty_app_state());
        let run_id = Uuid::new_v4();

        let request = Request::builder()
            .method("GET")
            .uri(format!("/{run_id}"))
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
    async fn test_get_campaign_run_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let run_id = Uuid::new_v4();

        let request = Request::builder()
            .method("GET")
            .uri(format!("/{run_id}"))
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
