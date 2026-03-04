//! Routes for the cross-context play loop orchestration.
//!
//! This module coordinates the manifesto's play loop across bounded contexts:
//! Intent → Check → Effects → World State → Narrative.
//! See ADR-0014 for rationale.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use uuid::Uuid;

use otherworlds_core::repository::StoredEvent;
use otherworlds_narrative::application::command_handlers as narrative_handlers;
use otherworlds_narrative::domain::commands as narrative_commands;
use otherworlds_rules::application::command_handlers as rules_handlers;
use otherworlds_rules::domain::commands as rules_commands;
use otherworlds_world_state::application::command_handlers as world_state_handlers;
use otherworlds_world_state::domain::commands as world_state_commands;

use crate::error::ApiError;
use crate::state::AppState;

/// Specification for a single effect to produce.
#[derive(Debug, Deserialize)]
pub struct EffectSpec {
    /// The type of effect (e.g., "`damage`", "`heal`", "`status_apply`").
    pub effect_type: String,
    /// Optional target of the effect.
    pub target_id: Option<Uuid>,
    /// Campaign-specific payload.
    pub payload: serde_json::Value,
}

/// Request body for POST /resolve-action.
#[derive(Debug, Deserialize)]
pub struct ResolveActionRequest {
    /// The narrative session to advance.
    pub session_id: Uuid,
    /// The world snapshot to apply effects to.
    pub world_id: Uuid,
    /// The type of action (e.g., "`skill_check`", "`attack`", "`save`").
    pub action_type: String,
    /// Optional skill being used.
    pub skill: Option<String>,
    /// Optional target of the action.
    pub target_id: Option<Uuid>,
    /// The difficulty class to beat.
    pub difficulty_class: i32,
    /// The modifier applied to the roll.
    pub modifier: i32,
    /// The effects to produce on success.
    pub effects: Vec<EffectSpec>,
}

/// Response from a resolved action showing all events across contexts.
#[derive(Debug, Serialize)]
pub struct ResolveActionResponse {
    /// The correlation ID threading all events together.
    pub correlation_id: Uuid,
    /// The resolution aggregate ID (rules context).
    pub resolution_id: Uuid,
    /// Event IDs from declaring intent.
    pub intent_event_ids: Vec<Uuid>,
    /// Event IDs from resolving the check.
    pub check_event_ids: Vec<Uuid>,
    /// Event IDs from producing effects.
    pub effects_event_ids: Vec<Uuid>,
    /// Event IDs from applying effects to world state.
    pub world_state_event_ids: Vec<Uuid>,
    /// Event IDs from advancing the narrative beat.
    pub narrative_event_ids: Vec<Uuid>,
}

/// Extracts a correlation ID from the `X-Correlation-ID` header, falling back
/// to a new v4 UUID if the header is absent or not a valid UUID.
fn extract_correlation_id(headers: &HeaderMap) -> Uuid {
    headers
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4)
}

fn collect_event_ids(events: &[StoredEvent]) -> Vec<Uuid> {
    events.iter().map(|e| e.event_id).collect()
}

/// POST /resolve-action
///
/// Orchestrates the full play loop:
/// 1. Rules: declare intent
/// 2. Rules: resolve check (d20 roll)
/// 3. Rules: produce effects
/// 4. World State: apply each effect as a world fact
/// 5. Narrative: advance the beat
#[instrument(skip(state, headers, request), fields(session_id = %request.session_id, world_id = %request.world_id))]
async fn resolve_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ResolveActionRequest>,
) -> Result<Json<ResolveActionResponse>, ApiError> {
    let correlation_id = extract_correlation_id(&headers);
    let resolution_id = Uuid::new_v4();
    let intent_id = Uuid::new_v4();

    info!(%correlation_id, %resolution_id, "orchestrating play loop");

    // Step 1: Declare intent (rules context)
    let declare_intent_cmd = rules_commands::DeclareIntent {
        correlation_id,
        resolution_id,
        intent_id,
        action_type: request.action_type,
        skill: request.skill,
        target_id: request.target_id,
        difficulty_class: request.difficulty_class,
        modifier: request.modifier,
    };

    let intent_events = rules_handlers::handle_declare_intent(
        &declare_intent_cmd,
        state.clock.as_ref(),
        &state.rng,
        &*state.event_repository,
    )
    .await?;

    // Step 2: Resolve check (rules context)
    let resolve_check_cmd = rules_commands::ResolveCheck {
        correlation_id,
        resolution_id,
    };

    let check_events = rules_handlers::handle_resolve_check(
        &resolve_check_cmd,
        state.clock.as_ref(),
        &state.rng,
        &*state.event_repository,
    )
    .await?;

    // Step 3: Produce effects (rules context)
    let effects = request
        .effects
        .into_iter()
        .map(|e| rules_commands::EffectSpec {
            effect_type: e.effect_type,
            target_id: e.target_id,
            payload: e.payload,
        })
        .collect();

    let produce_effects_cmd = rules_commands::ProduceEffects {
        correlation_id,
        resolution_id,
        effects,
    };

    let effects_events = rules_handlers::handle_produce_effects(
        &produce_effects_cmd,
        state.clock.as_ref(),
        &state.rng,
        &*state.event_repository,
    )
    .await?;

    // Step 4: Apply effects to world state
    let mut world_state_events = Vec::new();
    // Extract produced effects from the EffectsProduced event payload
    for stored in &effects_events {
        if stored.event_type == "rules.effects_produced"
            && let Ok(otherworlds_rules::domain::events::RulesEventKind::EffectsProduced(ep)) =
                serde_json::from_value::<otherworlds_rules::domain::events::RulesEventKind>(
                    stored.payload.clone(),
                )
        {
            for effect in &ep.effects {
                let apply_effect_cmd = world_state_commands::ApplyEffect {
                    correlation_id,
                    world_id: request.world_id,
                    fact_key: format!("{}:{}", effect.effect_type, effect.payload),
                };
                let ws_events = world_state_handlers::handle_apply_effect(
                    &apply_effect_cmd,
                    state.clock.as_ref(),
                    &state.rng,
                    &*state.event_repository,
                )
                .await?;
                world_state_events.extend(ws_events);
            }
        }
    }

    // Step 5: Advance narrative beat
    let advance_beat_cmd = narrative_commands::AdvanceBeat {
        correlation_id,
        session_id: request.session_id,
    };

    let narrative_events = narrative_handlers::handle_advance_beat(
        &advance_beat_cmd,
        state.clock.as_ref(),
        &state.rng,
        &*state.event_repository,
    )
    .await?;

    Ok(Json(ResolveActionResponse {
        correlation_id,
        resolution_id,
        intent_event_ids: collect_event_ids(&intent_events),
        check_event_ids: collect_event_ids(&check_events),
        effects_event_ids: collect_event_ids(&effects_events),
        world_state_event_ids: collect_event_ids(&world_state_events),
        narrative_event_ids: collect_event_ids(&narrative_events),
    }))
}

/// Returns the router for the play orchestration context.
pub fn router() -> Router<AppState> {
    Router::new().route("/resolve-action", post(resolve_action))
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
    use otherworlds_test_support::{FailingEventRepository, FixedClock, SequenceRng};
    use serde_json::Value;
    use sqlx::PgPool;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    /// In-memory repository that accumulates appended events and returns them
    /// on subsequent `load_events` calls. This enables the play loop's
    /// multi-step orchestration where each step reads events from prior steps.
    #[derive(Debug)]
    struct InMemoryEventRepository {
        events: Mutex<Vec<StoredEvent>>,
    }

    impl InMemoryEventRepository {
        fn new() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl EventRepository for InMemoryEventRepository {
        async fn load_events(&self, aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
            let events = self.events.lock().unwrap();
            Ok(events
                .iter()
                .filter(|e| e.aggregate_id == aggregate_id)
                .cloned()
                .collect())
        }

        async fn append_events(
            &self,
            _aggregate_id: Uuid,
            _expected_version: i64,
            new_events: &[StoredEvent],
        ) -> Result<(), DomainError> {
            let mut events = self.events.lock().unwrap();
            events.extend_from_slice(new_events);
            Ok(())
        }

        async fn list_aggregate_ids(
            &self,
            _event_types: &[&str],
        ) -> Result<Vec<Uuid>, DomainError> {
            Ok(vec![])
        }
    }

    fn app_state_with(
        event_repository: Arc<dyn EventRepository>,
        rng: Arc<Mutex<dyn DeterministicRng + Send>>,
    ) -> AppState {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let clock: Arc<dyn Clock + Send + Sync> = Arc::new(FixedClock(
            Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        ));
        AppState::new(pool, clock, rng, event_repository)
    }

    fn test_app_state() -> AppState {
        // Each next_uuid() consumes 4 u32 values, and resolve_check uses 1
        // for the d20 roll. The full play loop (5 phases, each producing at
        // least 1 event) needs ~21+ values. Provide a generous pool.
        let mut values = vec![15]; // first value: d20 natural roll
        values.extend(std::iter::repeat_n(42, 63)); // fill remaining slots
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> =
            Arc::new(Mutex::new(SequenceRng::new(values)));
        app_state_with(Arc::new(InMemoryEventRepository::new()), rng)
    }

    #[tokio::test]
    async fn test_resolve_action_returns_200_with_all_phases() {
        // Arrange
        let app = router().with_state(test_app_state());
        let session_id = Uuid::new_v4();
        let world_id = Uuid::new_v4();
        let body = serde_json::json!({
            "session_id": session_id,
            "world_id": world_id,
            "action_type": "skill_check",
            "skill": "perception",
            "difficulty_class": 15,
            "modifier": 3,
            "effects": [{
                "effect_type": "reveal",
                "payload": { "area": "hidden_passage" }
            }]
        });

        let request = Request::builder()
            .method("POST")
            .uri("/resolve-action")
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

        // Verify correlation_id is present and valid
        Uuid::parse_str(json["correlation_id"].as_str().unwrap()).unwrap();
        Uuid::parse_str(json["resolution_id"].as_str().unwrap()).unwrap();

        // Verify all phases produced events
        assert!(!json["intent_event_ids"].as_array().unwrap().is_empty());
        assert!(!json["check_event_ids"].as_array().unwrap().is_empty());
        assert!(!json["effects_event_ids"].as_array().unwrap().is_empty());
        assert!(!json["world_state_event_ids"].as_array().unwrap().is_empty());
        assert!(!json["narrative_event_ids"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_resolve_action_threads_correlation_id_from_header() {
        // Arrange
        let app = router().with_state(test_app_state());
        let correlation_id = Uuid::new_v4();
        let body = serde_json::json!({
            "session_id": Uuid::new_v4(),
            "world_id": Uuid::new_v4(),
            "action_type": "skill_check",
            "difficulty_class": 10,
            "modifier": 0,
            "effects": [{
                "effect_type": "damage",
                "payload": { "amount": 5 }
            }]
        });

        let request = Request::builder()
            .method("POST")
            .uri("/resolve-action")
            .header("content-type", "application/json")
            .header("x-correlation-id", correlation_id.to_string())
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

        let returned_correlation_id =
            Uuid::parse_str(json["correlation_id"].as_str().unwrap()).unwrap();
        assert_eq!(returned_correlation_id, correlation_id);
    }

    #[tokio::test]
    async fn test_resolve_action_returns_422_for_missing_body() {
        // Arrange
        let app = router().with_state(test_app_state());

        let request = Request::builder()
            .method("POST")
            .uri("/resolve-action")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_resolve_action_returns_500_when_repository_fails() {
        // Arrange
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> =
            Arc::new(Mutex::new(SequenceRng::new(vec![15, 42, 99])));
        let app = router().with_state(app_state_with(Arc::new(FailingEventRepository), rng));
        let body = serde_json::json!({
            "session_id": Uuid::new_v4(),
            "world_id": Uuid::new_v4(),
            "action_type": "skill_check",
            "difficulty_class": 15,
            "modifier": 3,
            "effects": []
        });

        let request = Request::builder()
            .method("POST")
            .uri("/resolve-action")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_resolve_action_with_no_effects() {
        // Arrange — action with empty effects list
        let app = router().with_state(test_app_state());
        let body = serde_json::json!({
            "session_id": Uuid::new_v4(),
            "world_id": Uuid::new_v4(),
            "action_type": "perception",
            "difficulty_class": 12,
            "modifier": 2,
            "effects": []
        });

        let request = Request::builder()
            .method("POST")
            .uri("/resolve-action")
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

        // Rules phases still produce events
        assert!(!json["intent_event_ids"].as_array().unwrap().is_empty());
        assert!(!json["check_event_ids"].as_array().unwrap().is_empty());
        // Empty effects still produces an EffectsProduced event
        assert!(!json["effects_event_ids"].as_array().unwrap().is_empty());
        // No world state changes
        assert!(json["world_state_event_ids"].as_array().unwrap().is_empty());
        // Narrative still advances
        assert!(!json["narrative_event_ids"].as_array().unwrap().is_empty());
    }
}
