//! Routes for the Rules & Resolution bounded context.

use axum::extract::{Path, State};
use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use uuid::Uuid;

use otherworlds_rules::application::query_handlers::ResolutionView;
use otherworlds_rules::application::{command_handlers, query_handlers};
use otherworlds_rules::domain::commands;

use crate::error::ApiError;
use crate::state::AppState;

/// Request body for POST /declare-intent.
#[derive(Debug, Deserialize)]
pub struct DeclareIntentRequest {
    /// The resolution this intent belongs to.
    pub resolution_id: Uuid,
    /// The intent identifier.
    pub intent_id: Uuid,
    /// The type of action (e.g., "`skill_check`", "attack", "save").
    pub action_type: String,
    /// Optional skill being used.
    pub skill: Option<String>,
    /// Optional target of the action.
    pub target_id: Option<Uuid>,
    /// The difficulty class to beat.
    pub difficulty_class: i32,
    /// The modifier applied to the roll.
    pub modifier: i32,
}

/// Request body for POST /resolve-check.
#[derive(Debug, Deserialize)]
pub struct ResolveCheckRequest {
    /// The resolution this check belongs to.
    pub resolution_id: Uuid,
}

/// Request body for a single effect specification.
#[derive(Debug, Deserialize)]
pub struct EffectSpecRequest {
    /// The type of effect (e.g., "damage", "heal", "`status_apply`").
    pub effect_type: String,
    /// Optional target of the effect.
    pub target_id: Option<Uuid>,
    /// Campaign-specific payload.
    pub payload: serde_json::Value,
}

/// Request body for POST /produce-effects.
#[derive(Debug, Deserialize)]
pub struct ProduceEffectsRequest {
    /// The resolution this effect production belongs to.
    pub resolution_id: Uuid,
    /// The effects to produce.
    pub effects: Vec<EffectSpecRequest>,
}

/// Response body returned after a command is successfully handled.
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    /// IDs of the domain events produced and persisted.
    pub event_ids: Vec<Uuid>,
}

/// POST /declare-intent
#[instrument(skip(state, request), fields(resolution_id = %request.resolution_id))]
async fn declare_intent(
    State(state): State<AppState>,
    Json(request): Json<DeclareIntentRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::DeclareIntent {
        correlation_id: Uuid::new_v4(),
        resolution_id: request.resolution_id,
        intent_id: request.intent_id,
        action_type: request.action_type,
        skill: request.skill,
        target_id: request.target_id,
        difficulty_class: request.difficulty_class,
        modifier: request.modifier,
    };

    info!(correlation_id = %command.correlation_id, "handling declare_intent command");

    let stored_events = command_handlers::handle_declare_intent(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// POST /resolve-check
#[instrument(skip(state, request), fields(resolution_id = %request.resolution_id))]
async fn resolve_check(
    State(state): State<AppState>,
    Json(request): Json<ResolveCheckRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::ResolveCheck {
        correlation_id: Uuid::new_v4(),
        resolution_id: request.resolution_id,
    };

    info!(correlation_id = %command.correlation_id, "handling resolve_check command");

    let stored_events = command_handlers::handle_resolve_check(
        &command,
        state.clock.as_ref(),
        &state.rng,
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// POST /produce-effects
#[instrument(skip(state, request), fields(resolution_id = %request.resolution_id))]
async fn produce_effects(
    State(state): State<AppState>,
    Json(request): Json<ProduceEffectsRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let effects = request
        .effects
        .into_iter()
        .map(|e| commands::EffectSpec {
            effect_type: e.effect_type,
            target_id: e.target_id,
            payload: e.payload,
        })
        .collect();

    let command = commands::ProduceEffects {
        correlation_id: Uuid::new_v4(),
        resolution_id: request.resolution_id,
        effects,
    };

    info!(correlation_id = %command.correlation_id, "handling produce_effects command");

    let stored_events = command_handlers::handle_produce_effects(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse { event_ids }))
}

/// GET /{`resolution_id`}
#[instrument(skip(state), fields(resolution_id = %id))]
async fn get_resolution(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ResolutionView>, ApiError> {
    let view = query_handlers::get_resolution_by_id(id, &*state.event_repository).await?;
    Ok(Json(view))
}

/// Returns the router for the rules context.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{resolution_id}", get(get_resolution))
        .route("/declare-intent", post(declare_intent))
        .route("/resolve-check", post(resolve_check))
        .route("/produce-effects", post(produce_effects))
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
    use otherworlds_rules::domain::events::{
        CheckOutcome, CheckResolved, IntentDeclared, RulesEventKind,
    };
    use otherworlds_test_support::{
        EmptyEventRepository, FailingEventRepository, FixedClock, MockRng,
        RecordingEventRepository, SequenceRng,
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

    fn app_state_with_rng(
        event_repository: Arc<dyn EventRepository>,
        rng: Arc<Mutex<dyn DeterministicRng + Send>>,
    ) -> AppState {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let clock: Arc<dyn Clock + Send + Sync> = Arc::new(FixedClock(Utc::now()));
        AppState::new(pool, clock, rng, event_repository)
    }

    fn test_app_state() -> AppState {
        app_state_with(Arc::new(EmptyEventRepository))
    }

    fn failing_app_state() -> AppState {
        app_state_with(Arc::new(FailingEventRepository))
    }

    fn fixed_now() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap()
    }

    fn intent_declared_stored_event(resolution_id: Uuid) -> StoredEvent {
        StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: resolution_id,
            event_type: "rules.intent_declared".to_owned(),
            payload: serde_json::to_value(RulesEventKind::IntentDeclared(IntentDeclared {
                resolution_id,
                intent_id: Uuid::new_v4(),
                action_type: "skill_check".to_owned(),
                skill: Some("perception".to_owned()),
                target_id: None,
                difficulty_class: 15,
                modifier: 3,
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now(),
        }
    }

    fn check_resolved_stored_event(resolution_id: Uuid) -> StoredEvent {
        StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: resolution_id,
            event_type: "rules.check_resolved".to_owned(),
            payload: serde_json::to_value(RulesEventKind::CheckResolved(CheckResolved {
                resolution_id,
                check_id: Uuid::new_v4(),
                natural_roll: 15,
                modifier: 3,
                total: 18,
                difficulty_class: 15,
                outcome: CheckOutcome::Success,
            }))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now(),
        }
    }

    // --- POST /declare-intent tests ---

    #[tokio::test]
    async fn test_declare_intent_returns_200_with_event_ids() {
        let app = router().with_state(test_app_state());
        let body = serde_json::json!({
            "resolution_id": Uuid::new_v4(),
            "intent_id": Uuid::new_v4(),
            "action_type": "skill_check",
            "difficulty_class": 15,
            "modifier": 3
        });

        let request = Request::builder()
            .method("POST")
            .uri("/declare-intent")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
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
    async fn test_declare_intent_returns_422_for_missing_body() {
        let app = router().with_state(test_app_state());

        let request = Request::builder()
            .method("POST")
            .uri("/declare-intent")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_declare_intent_returns_500_when_repository_fails() {
        let app = router().with_state(failing_app_state());
        let body = serde_json::json!({
            "resolution_id": Uuid::new_v4(),
            "intent_id": Uuid::new_v4(),
            "action_type": "skill_check",
            "difficulty_class": 15,
            "modifier": 3
        });

        let request = Request::builder()
            .method("POST")
            .uri("/declare-intent")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["error"], "infrastructure_error");
    }

    #[tokio::test]
    async fn test_declare_intent_returns_400_for_phase_validation_error() {
        let resolution_id = Uuid::new_v4();
        // Pre-load an IntentDeclared event so the aggregate is no longer in Created phase
        let repo =
            RecordingEventRepository::new(Ok(vec![intent_declared_stored_event(resolution_id)]));
        let app = router().with_state(app_state_with(Arc::new(repo)));

        let body = serde_json::json!({
            "resolution_id": resolution_id,
            "intent_id": Uuid::new_v4(),
            "action_type": "attack",
            "difficulty_class": 12,
            "modifier": 0
        });

        let request = Request::builder()
            .method("POST")
            .uri("/declare-intent")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["error"], "validation_error");
    }

    // --- POST /resolve-check tests ---

    #[tokio::test]
    async fn test_resolve_check_returns_200_with_event_ids() {
        let resolution_id = Uuid::new_v4();
        let repo =
            RecordingEventRepository::new(Ok(vec![intent_declared_stored_event(resolution_id)]));
        // RNG: d20 roll, then two values for check_id
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> =
            Arc::new(Mutex::new(SequenceRng::new(vec![15, 42, 99])));
        let app = router().with_state(app_state_with_rng(Arc::new(repo), rng));

        let body = serde_json::json!({ "resolution_id": resolution_id });

        let request = Request::builder()
            .method("POST")
            .uri("/resolve-check")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();
        let event_ids = json["event_ids"].as_array().unwrap();
        assert_eq!(event_ids.len(), 1);
    }

    #[tokio::test]
    async fn test_resolve_check_returns_422_for_missing_body() {
        let app = router().with_state(test_app_state());

        let request = Request::builder()
            .method("POST")
            .uri("/resolve-check")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_resolve_check_returns_500_when_repository_fails() {
        let app = router().with_state(failing_app_state());
        let body = serde_json::json!({ "resolution_id": Uuid::new_v4() });

        let request = Request::builder()
            .method("POST")
            .uri("/resolve-check")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["error"], "infrastructure_error");
    }

    #[tokio::test]
    async fn test_resolve_check_returns_400_for_phase_validation_error() {
        // Fresh aggregate (Created phase) — resolve_check requires IntentDeclared
        let app = router().with_state(test_app_state());
        let body = serde_json::json!({ "resolution_id": Uuid::new_v4() });

        let request = Request::builder()
            .method("POST")
            .uri("/resolve-check")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["error"], "validation_error");
    }

    // --- POST /produce-effects tests ---

    #[tokio::test]
    async fn test_produce_effects_returns_200_with_event_ids() {
        let resolution_id = Uuid::new_v4();
        let repo = RecordingEventRepository::new(Ok(vec![
            intent_declared_stored_event(resolution_id),
            check_resolved_stored_event(resolution_id),
        ]));
        let app = router().with_state(app_state_with(Arc::new(repo)));

        let body = serde_json::json!({
            "resolution_id": resolution_id,
            "effects": [{
                "effect_type": "damage",
                "payload": { "amount": 8 }
            }]
        });

        let request = Request::builder()
            .method("POST")
            .uri("/produce-effects")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();
        let event_ids = json["event_ids"].as_array().unwrap();
        assert_eq!(event_ids.len(), 1);
    }

    #[tokio::test]
    async fn test_produce_effects_returns_422_for_missing_body() {
        let app = router().with_state(test_app_state());

        let request = Request::builder()
            .method("POST")
            .uri("/produce-effects")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_produce_effects_returns_500_when_repository_fails() {
        let app = router().with_state(failing_app_state());
        let body = serde_json::json!({
            "resolution_id": Uuid::new_v4(),
            "effects": []
        });

        let request = Request::builder()
            .method("POST")
            .uri("/produce-effects")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // --- GET /{resolution_id} tests ---

    #[tokio::test]
    async fn test_get_resolution_returns_200_with_enriched_json() {
        let resolution_id = Uuid::new_v4();
        let repo =
            RecordingEventRepository::new(Ok(vec![intent_declared_stored_event(resolution_id)]));
        let app = router().with_state(app_state_with(Arc::new(repo)));

        let request = Request::builder()
            .method("GET")
            .uri(format!("/{resolution_id}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["resolution_id"], resolution_id.to_string());
        assert_eq!(json["phase"], "intent_declared");
        assert!(json["intent"].is_object());
        assert!(json["check_result"].is_null());
        assert!(json["effects"].as_array().unwrap().is_empty());
        assert_eq!(json["version"], 1);
    }

    #[tokio::test]
    async fn test_get_resolution_returns_404_when_not_found() {
        let app = router().with_state(test_app_state());
        let resolution_id = Uuid::new_v4();

        let request = Request::builder()
            .method("GET")
            .uri(format!("/{resolution_id}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["error"], "aggregate_not_found");
    }
}
