//! Query handlers for the Rules & Resolution context.
//!
//! This module contains query handlers that reconstitute aggregates
//! from stored events and return read-only view DTOs.

use otherworlds_core::error::DomainError;
use otherworlds_core::repository::EventRepository;
use serde::Serialize;
use uuid::Uuid;

use crate::application::command_handlers;

/// Read-only view of a resolution's declared intent.
#[derive(Debug, Serialize)]
pub struct IntentView {
    /// The intent identifier.
    pub intent_id: Uuid,
    /// The type of action.
    pub action_type: String,
    /// Optional skill being used.
    pub skill: Option<String>,
    /// Optional target of the action.
    pub target_id: Option<Uuid>,
    /// The difficulty class.
    pub difficulty_class: i32,
    /// The roll modifier.
    pub modifier: i32,
}

/// Read-only view of a resolution's check result.
#[derive(Debug, Serialize)]
pub struct CheckResultView {
    /// The check identifier.
    pub check_id: Uuid,
    /// The raw d20 result.
    pub natural_roll: u32,
    /// The modifier applied.
    pub modifier: i32,
    /// The total (`natural_roll` + modifier).
    pub total: i32,
    /// The difficulty class.
    pub difficulty_class: i32,
    /// The outcome as a string.
    pub outcome: String,
}

/// Read-only view of a single effect.
#[derive(Debug, Serialize)]
pub struct EffectView {
    /// The type of effect.
    pub effect_type: String,
    /// Optional target of the effect.
    pub target_id: Option<Uuid>,
    /// Campaign-specific payload.
    pub payload: serde_json::Value,
}

/// Read-only view of a resolution aggregate.
#[derive(Debug, Serialize)]
pub struct ResolutionView {
    /// The resolution identifier.
    pub resolution_id: Uuid,
    /// Current phase as a string.
    pub phase: String,
    /// Declared intent, if any.
    pub intent: Option<IntentView>,
    /// Check result, if any.
    pub check_result: Option<CheckResultView>,
    /// Produced effects.
    pub effects: Vec<EffectView>,
    /// Current version (event count).
    pub version: i64,
}

/// Retrieves a resolution by its aggregate ID.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the ID.
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub async fn get_resolution_by_id(
    resolution_id: Uuid,
    repo: &dyn EventRepository,
) -> Result<ResolutionView, DomainError> {
    let stored_events = repo.load_events(resolution_id).await?;
    if stored_events.is_empty() {
        return Err(DomainError::AggregateNotFound(resolution_id));
    }
    let resolution = command_handlers::reconstitute(resolution_id, &stored_events)?;

    let phase = resolution.phase_name().to_owned();

    let intent = resolution.intent.map(|i| IntentView {
        intent_id: i.intent_id,
        action_type: i.action_type,
        skill: i.skill,
        target_id: i.target_id,
        difficulty_class: i.difficulty_class,
        modifier: i.modifier,
    });

    let check_result = resolution.check_result.map(|c| CheckResultView {
        check_id: c.check_id,
        natural_roll: c.natural_roll,
        modifier: c.modifier,
        total: c.total,
        difficulty_class: c.difficulty_class,
        outcome: c.outcome.to_string(),
    });

    let effects = resolution
        .effects
        .into_iter()
        .map(|e| EffectView {
            effect_type: e.effect_type,
            target_id: e.target_id,
            payload: e.payload,
        })
        .collect();

    Ok(ResolutionView {
        resolution_id,
        phase,
        intent,
        check_result,
        effects,
        version: resolution.version,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use uuid::Uuid;

    use crate::application::query_handlers::get_resolution_by_id;
    use crate::domain::events::{
        CheckOutcome, CheckResolved, EffectsProduced, IntentDeclared, ResolvedEffect,
        RulesEventKind,
    };
    use otherworlds_test_support::{EmptyEventRepository, RecordingEventRepository};

    fn fixed_now() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap()
    }

    fn intent_declared_event(resolution_id: Uuid) -> StoredEvent {
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

    fn check_resolved_event(resolution_id: Uuid) -> StoredEvent {
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

    fn effects_produced_event(resolution_id: Uuid) -> StoredEvent {
        StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: resolution_id,
            event_type: "rules.effects_produced".to_owned(),
            payload: serde_json::to_value(RulesEventKind::EffectsProduced(EffectsProduced {
                resolution_id,
                effects: vec![ResolvedEffect {
                    effect_type: "damage".to_owned(),
                    target_id: None,
                    payload: serde_json::json!({ "amount": 8 }),
                }],
            }))
            .unwrap(),
            sequence_number: 3,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now(),
        }
    }

    #[tokio::test]
    async fn test_view_from_intent_declared() {
        let resolution_id = Uuid::new_v4();
        let repo = RecordingEventRepository::new(Ok(vec![intent_declared_event(resolution_id)]));

        let view = get_resolution_by_id(resolution_id, &repo).await.unwrap();

        assert_eq!(view.resolution_id, resolution_id);
        assert_eq!(view.phase, "intent_declared");
        assert!(view.intent.is_some());
        assert!(view.check_result.is_none());
        assert!(view.effects.is_empty());
        assert_eq!(view.version, 1);
    }

    #[tokio::test]
    async fn test_view_from_intent_and_check() {
        let resolution_id = Uuid::new_v4();
        let repo = RecordingEventRepository::new(Ok(vec![
            intent_declared_event(resolution_id),
            check_resolved_event(resolution_id),
        ]));

        let view = get_resolution_by_id(resolution_id, &repo).await.unwrap();

        assert_eq!(view.phase, "check_resolved");
        assert!(view.intent.is_some());
        assert!(view.check_result.is_some());
        let check = view.check_result.unwrap();
        assert_eq!(check.outcome, "success");
        assert!(view.effects.is_empty());
    }

    #[tokio::test]
    async fn test_view_from_full_lifecycle() {
        let resolution_id = Uuid::new_v4();
        let repo = RecordingEventRepository::new(Ok(vec![
            intent_declared_event(resolution_id),
            check_resolved_event(resolution_id),
            effects_produced_event(resolution_id),
        ]));

        let view = get_resolution_by_id(resolution_id, &repo).await.unwrap();

        assert_eq!(view.phase, "effects_produced");
        assert!(view.intent.is_some());
        assert!(view.check_result.is_some());
        assert_eq!(view.effects.len(), 1);
        assert_eq!(view.effects[0].effect_type, "damage");
        assert_eq!(view.version, 3);
    }

    #[tokio::test]
    async fn test_phase_string_for_each_phase() {
        let resolution_id = Uuid::new_v4();

        // Intent declared phase
        let repo = RecordingEventRepository::new(Ok(vec![intent_declared_event(resolution_id)]));
        let view = get_resolution_by_id(resolution_id, &repo).await.unwrap();
        assert_eq!(view.phase, "intent_declared");
    }

    #[tokio::test]
    async fn test_not_found_returns_aggregate_not_found() {
        let resolution_id = Uuid::new_v4();
        let repo = EmptyEventRepository;

        let result = get_resolution_by_id(resolution_id, &repo).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, resolution_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }
}
