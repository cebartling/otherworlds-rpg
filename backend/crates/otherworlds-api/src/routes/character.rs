//! Routes for the Character Management bounded context.

use axum::extract::State;
use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use uuid::Uuid;

use otherworlds_character::application::command_handlers;
use otherworlds_character::domain::commands;

use crate::error::ApiError;
use crate::state::AppState;

/// Request body for POST /create.
#[derive(Debug, Deserialize)]
pub struct CreateCharacterRequest {
    /// The character's name.
    pub name: String,
}

/// Request body for POST /modify-attribute.
#[derive(Debug, Deserialize)]
pub struct ModifyAttributeRequest {
    /// The character to modify.
    pub character_id: Uuid,
    /// The attribute key.
    pub attribute: String,
    /// The new value.
    pub new_value: i32,
}

/// Request body for POST /award-experience.
#[derive(Debug, Deserialize)]
pub struct AwardExperienceRequest {
    /// The character to award experience to.
    pub character_id: Uuid,
    /// The amount of experience to award.
    pub amount: u32,
}

/// Response body returned after a command is successfully handled.
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    /// IDs of the domain events produced and persisted.
    pub event_ids: Vec<Uuid>,
}

/// POST /create
#[instrument(skip(state, request), fields(name = %request.name))]
async fn create_character(
    State(state): State<AppState>,
    Json(request): Json<CreateCharacterRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::CreateCharacter {
        correlation_id: Uuid::new_v4(),
        character_id: Uuid::new_v4(),
        name: request.name,
    };

    info!(correlation_id = %command.correlation_id, "handling create_character command");

    let stored_events = command_handlers::handle_create_character(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// POST /modify-attribute
#[instrument(skip(state, request), fields(character_id = %request.character_id))]
async fn modify_attribute(
    State(state): State<AppState>,
    Json(request): Json<ModifyAttributeRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::ModifyAttribute {
        correlation_id: Uuid::new_v4(),
        character_id: request.character_id,
        attribute: request.attribute,
        new_value: request.new_value,
    };

    info!(correlation_id = %command.correlation_id, "handling modify_attribute command");

    let stored_events = command_handlers::handle_modify_attribute(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// POST /award-experience
#[instrument(skip(state, request), fields(character_id = %request.character_id))]
async fn award_experience(
    State(state): State<AppState>,
    Json(request): Json<AwardExperienceRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::AwardExperience {
        correlation_id: Uuid::new_v4(),
        character_id: request.character_id,
        amount: request.amount,
    };

    info!(correlation_id = %command.correlation_id, "handling award_experience command");

    let stored_events = command_handlers::handle_award_experience(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// Returns the router for the character context.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/create", post(create_character))
        .route("/modify-attribute", post(modify_attribute))
        .route("/award-experience", post(award_experience))
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

    fn app_state_with(event_repository: Arc<dyn EventRepository>) -> AppState {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let clock: Arc<dyn Clock + Send + Sync> = Arc::new(FixedClock(Utc::now()));
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        AppState::new(pool, clock, rng, event_repository)
    }

    fn test_app_state() -> AppState {
        app_state_with(Arc::new(MockEventRepository))
    }

    fn failing_app_state() -> AppState {
        app_state_with(Arc::new(FailingEventRepository))
    }

    #[tokio::test]
    async fn test_create_character_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let body = serde_json::json!({ "name": "Alaric" });

        let request = Request::builder()
            .method("POST")
            .uri("/create")
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
    async fn test_modify_attribute_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let character_id = Uuid::new_v4();
        let body = serde_json::json!({
            "character_id": character_id,
            "attribute": "strength",
            "new_value": 18
        });

        let request = Request::builder()
            .method("POST")
            .uri("/modify-attribute")
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
    async fn test_award_experience_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let character_id = Uuid::new_v4();
        let body = serde_json::json!({
            "character_id": character_id,
            "amount": 250
        });

        let request = Request::builder()
            .method("POST")
            .uri("/award-experience")
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
    async fn test_create_character_returns_422_for_missing_body() {
        // Arrange
        let app = router().with_state(test_app_state());

        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert — Axum returns 422 for deserialization failures.
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_character_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let body = serde_json::json!({ "name": "Alaric" });

        let request = Request::builder()
            .method("POST")
            .uri("/create")
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
    async fn test_modify_attribute_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let character_id = Uuid::new_v4();
        let body = serde_json::json!({
            "character_id": character_id,
            "attribute": "strength",
            "new_value": 18
        });

        let request = Request::builder()
            .method("POST")
            .uri("/modify-attribute")
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
    async fn test_award_experience_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let character_id = Uuid::new_v4();
        let body = serde_json::json!({
            "character_id": character_id,
            "amount": 250
        });

        let request = Request::builder()
            .method("POST")
            .uri("/award-experience")
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
    async fn test_create_character_returns_400_for_empty_name() {
        // Arrange
        let app = router().with_state(test_app_state());
        let body = serde_json::json!({ "name": "  " });

        let request = Request::builder()
            .method("POST")
            .uri("/create")
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
    async fn test_modify_attribute_returns_400_for_empty_attribute() {
        // Arrange
        let app = router().with_state(test_app_state());
        let character_id = Uuid::new_v4();
        let body = serde_json::json!({
            "character_id": character_id,
            "attribute": "",
            "new_value": 18
        });

        let request = Request::builder()
            .method("POST")
            .uri("/modify-attribute")
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
    async fn test_award_experience_returns_400_for_zero_amount() {
        // Arrange
        let app = router().with_state(test_app_state());
        let character_id = Uuid::new_v4();
        let body = serde_json::json!({
            "character_id": character_id,
            "amount": 0
        });

        let request = Request::builder()
            .method("POST")
            .uri("/award-experience")
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
