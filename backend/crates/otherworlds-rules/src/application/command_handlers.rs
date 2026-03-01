//! Command handlers for the Rules & Resolution context.
//!
//! This module contains application-level command handler functions that
//! orchestrate domain logic: load aggregate, execute command, persist events.

use std::sync::Mutex;

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::event::DomainEvent;
use otherworlds_core::repository::{EventRepository, StoredEvent};
use otherworlds_core::rng::DeterministicRng;
use uuid::Uuid;

use crate::domain::aggregates::Resolution;
use crate::domain::commands::{DeclareIntent, EffectSpec, ProduceEffects, ResolveCheck};
use crate::domain::events::{RulesEvent, RulesEventKind};

fn to_stored_event(event: &RulesEvent) -> StoredEvent {
    let meta = event.metadata();
    StoredEvent {
        event_id: meta.event_id,
        aggregate_id: meta.aggregate_id,
        event_type: event.event_type().to_owned(),
        payload: event.to_payload(),
        sequence_number: meta.sequence_number,
        correlation_id: meta.correlation_id,
        causation_id: meta.causation_id,
        occurred_at: meta.occurred_at,
    }
}

/// Reconstitutes a `Resolution` from stored events.
///
/// # Errors
///
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub(crate) fn reconstitute(
    resolution_id: Uuid,
    existing_events: &[StoredEvent],
) -> Result<Resolution, DomainError> {
    let mut resolution = Resolution::new(resolution_id);
    for stored in existing_events {
        let kind: RulesEventKind = serde_json::from_value(stored.payload.clone()).map_err(|e| {
            DomainError::Infrastructure(format!("event deserialization failed: {e}"))
        })?;
        let event = RulesEvent {
            metadata: otherworlds_core::event::EventMetadata {
                event_id: stored.event_id,
                event_type: stored.event_type.clone(),
                aggregate_id: stored.aggregate_id,
                sequence_number: stored.sequence_number,
                correlation_id: stored.correlation_id,
                causation_id: stored.causation_id,
                occurred_at: stored.occurred_at,
            },
            kind,
        };
        resolution.apply(&event);
    }
    Ok(resolution)
}

/// Handles the `DeclareIntent` command: reconstitutes the aggregate, declares
/// the intent, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading, validation, or appending fails.
pub async fn handle_declare_intent(
    command: &DeclareIntent,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.resolution_id).await?;
    let mut resolution = reconstitute(command.resolution_id, &existing_events)?;

    resolution.declare_intent(
        command.intent_id,
        command.action_type.clone(),
        command.skill.clone(),
        command.target_id,
        command.difficulty_class,
        command.modifier,
        command.correlation_id,
        clock,
    )?;

    let stored_events: Vec<StoredEvent> = resolution
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.resolution_id, resolution.version, &stored_events)
        .await?;

    Ok(stored_events)
}

/// Handles the `ResolveCheck` command: reconstitutes the aggregate, resolves
/// the check using the RNG, and persists the resulting events.
///
/// The `Mutex` is locked only around the synchronous domain method call to
/// avoid holding a `MutexGuard` across await points.
///
/// # Errors
///
/// Returns `DomainError` if event loading, validation, or appending fails.
pub async fn handle_resolve_check(
    command: &ResolveCheck,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.resolution_id).await?;
    let mut resolution = reconstitute(command.resolution_id, &existing_events)?;

    // Lock RNG only for the synchronous domain method — never across an await.
    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        resolution.resolve_check(command.correlation_id, clock, &mut *rng_guard)?;
    }

    let stored_events: Vec<StoredEvent> = resolution
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.resolution_id, resolution.version, &stored_events)
        .await?;

    Ok(stored_events)
}

/// Handles the `ProduceEffects` command: reconstitutes the aggregate, produces
/// effects, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading, validation, or appending fails.
pub async fn handle_produce_effects(
    command: &ProduceEffects,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.resolution_id).await?;
    let mut resolution = reconstitute(command.resolution_id, &existing_events)?;

    let effects = command
        .effects
        .iter()
        .cloned()
        .map(EffectSpec::into_resolved_effect)
        .collect();

    resolution.produce_effects(effects, command.correlation_id, clock)?;

    let stored_events: Vec<StoredEvent> = resolution
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.resolution_id, resolution.version, &stored_events)
        .await?;

    Ok(stored_events)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use otherworlds_core::repository::StoredEvent;
    use otherworlds_core::rng::DeterministicRng;
    use std::sync::Mutex;
    use uuid::Uuid;

    use crate::application::command_handlers::{
        handle_declare_intent, handle_produce_effects, handle_resolve_check, reconstitute,
    };
    use crate::domain::commands::{DeclareIntent, EffectSpec, ProduceEffects, ResolveCheck};
    use crate::domain::events::{CheckOutcome, CheckResolved, IntentDeclared, RulesEventKind};
    use otherworlds_test_support::{FixedClock, RecordingEventRepository, SequenceRng};

    fn fixed_clock() -> FixedClock {
        FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap())
    }

    #[tokio::test]
    async fn test_handle_declare_intent_persists_event() {
        let resolution_id = Uuid::new_v4();
        let intent_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let clock = fixed_clock();
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = DeclareIntent {
            correlation_id,
            resolution_id,
            intent_id,
            action_type: "skill_check".to_owned(),
            skill: Some("perception".to_owned()),
            target_id: None,
            difficulty_class: 15,
            modifier: 3,
        };

        let result = handle_declare_intent(&command, &clock, &repo).await;
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, resolution_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "rules.intent_declared");
        assert_eq!(stored.aggregate_id, resolution_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
    }

    #[tokio::test]
    async fn test_handle_declare_intent_with_existing_events_validates_phase() {
        let resolution_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        // Pre-load an IntentDeclared event — phase will be IntentDeclared, not Created
        let existing = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: resolution_id,
            event_type: "rules.intent_declared".to_owned(),
            payload: serde_json::to_value(RulesEventKind::IntentDeclared(IntentDeclared {
                resolution_id,
                intent_id: Uuid::new_v4(),
                action_type: "attack".to_owned(),
                skill: None,
                target_id: None,
                difficulty_class: 12,
                modifier: 0,
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }];
        let repo = RecordingEventRepository::new(Ok(existing));

        let command = DeclareIntent {
            correlation_id: Uuid::new_v4(),
            resolution_id,
            intent_id: Uuid::new_v4(),
            action_type: "save".to_owned(),
            skill: None,
            target_id: None,
            difficulty_class: 10,
            modifier: 0,
        };

        let result = handle_declare_intent(&command, &fixed_clock(), &repo).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_resolve_check_persists_event() {
        let resolution_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        // Pre-load IntentDeclared so aggregate is in correct phase
        let existing = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: resolution_id,
            event_type: "rules.intent_declared".to_owned(),
            payload: serde_json::to_value(RulesEventKind::IntentDeclared(IntentDeclared {
                resolution_id,
                intent_id: Uuid::new_v4(),
                action_type: "skill_check".to_owned(),
                skill: None,
                target_id: None,
                difficulty_class: 15,
                modifier: 3,
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }];
        let repo = RecordingEventRepository::new(Ok(existing));
        // RNG: d20 roll = 15, then four values for check_id
        let rng: Mutex<SequenceRng> = Mutex::new(SequenceRng::new(vec![15, 42, 99, 7, 13]));
        let rng_ref: &Mutex<dyn DeterministicRng + Send> = &rng;

        let command = ResolveCheck {
            correlation_id,
            resolution_id,
        };

        let result = handle_resolve_check(&command, &fixed_clock(), rng_ref, &repo).await;
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);
        let stored = &appended[0].2[0];
        assert_eq!(stored.event_type, "rules.check_resolved");
    }

    #[tokio::test]
    async fn test_handle_resolve_check_on_fresh_aggregate_returns_error() {
        let repo = RecordingEventRepository::new(Ok(Vec::new()));
        let rng: Mutex<SequenceRng> = Mutex::new(SequenceRng::new(vec![10, 1, 1]));
        let rng_ref: &Mutex<dyn DeterministicRng + Send> = &rng;

        let command = ResolveCheck {
            correlation_id: Uuid::new_v4(),
            resolution_id: Uuid::new_v4(),
        };

        let result = handle_resolve_check(&command, &fixed_clock(), rng_ref, &repo).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_produce_effects_persists_event() {
        let resolution_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        // Pre-load IntentDeclared + CheckResolved so aggregate is in correct phase
        let existing = vec![
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: resolution_id,
                event_type: "rules.intent_declared".to_owned(),
                payload: serde_json::to_value(RulesEventKind::IntentDeclared(IntentDeclared {
                    resolution_id,
                    intent_id: Uuid::new_v4(),
                    action_type: "skill_check".to_owned(),
                    skill: None,
                    target_id: None,
                    difficulty_class: 15,
                    modifier: 3,
                }))
                .unwrap(),
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
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
                occurred_at: fixed_now,
            },
        ];
        let repo = RecordingEventRepository::new(Ok(existing));

        let command = ProduceEffects {
            correlation_id,
            resolution_id,
            effects: vec![EffectSpec {
                effect_type: "damage".to_owned(),
                target_id: None,
                payload: serde_json::json!({ "amount": 8 }),
            }],
        };

        let result = handle_produce_effects(&command, &fixed_clock(), &repo).await;
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);
        let stored = &appended[0].2[0];
        assert_eq!(stored.event_type, "rules.effects_produced");
    }

    #[tokio::test]
    async fn test_handle_produce_effects_on_fresh_aggregate_returns_error() {
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = ProduceEffects {
            correlation_id: Uuid::new_v4(),
            resolution_id: Uuid::new_v4(),
            effects: vec![],
        };

        let result = handle_produce_effects(&command, &fixed_clock(), &repo).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reconstitute_intent_declared_event() {
        let resolution_id = Uuid::new_v4();
        let intent_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: resolution_id,
            event_type: "rules.intent_declared".to_owned(),
            payload: serde_json::to_value(RulesEventKind::IntentDeclared(IntentDeclared {
                resolution_id,
                intent_id,
                action_type: "attack".to_owned(),
                skill: None,
                target_id: None,
                difficulty_class: 12,
                modifier: 2,
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }];

        let resolution = reconstitute(resolution_id, &events).unwrap();
        assert_eq!(resolution.version, 1);
        assert!(resolution.intent.is_some());
    }

    #[tokio::test]
    async fn test_reconstitute_check_resolved_event() {
        let resolution_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: resolution_id,
                event_type: "rules.intent_declared".to_owned(),
                payload: serde_json::to_value(RulesEventKind::IntentDeclared(IntentDeclared {
                    resolution_id,
                    intent_id: Uuid::new_v4(),
                    action_type: "skill_check".to_owned(),
                    skill: None,
                    target_id: None,
                    difficulty_class: 15,
                    modifier: 3,
                }))
                .unwrap(),
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: resolution_id,
                event_type: "rules.check_resolved".to_owned(),
                payload: serde_json::to_value(RulesEventKind::CheckResolved(CheckResolved {
                    resolution_id,
                    check_id: Uuid::new_v4(),
                    natural_roll: 18,
                    modifier: 3,
                    total: 21,
                    difficulty_class: 15,
                    outcome: CheckOutcome::Success,
                }))
                .unwrap(),
                sequence_number: 2,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
        ];

        let resolution = reconstitute(resolution_id, &events).unwrap();
        assert_eq!(resolution.version, 2);
        assert!(resolution.check_result.is_some());
    }

    #[tokio::test]
    async fn test_reconstitute_effects_produced_event() {
        let resolution_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: resolution_id,
                event_type: "rules.intent_declared".to_owned(),
                payload: serde_json::to_value(RulesEventKind::IntentDeclared(IntentDeclared {
                    resolution_id,
                    intent_id: Uuid::new_v4(),
                    action_type: "skill_check".to_owned(),
                    skill: None,
                    target_id: None,
                    difficulty_class: 15,
                    modifier: 3,
                }))
                .unwrap(),
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
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
                occurred_at: fixed_now,
            },
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: resolution_id,
                event_type: "rules.effects_produced".to_owned(),
                payload: serde_json::to_value(RulesEventKind::EffectsProduced(
                    crate::domain::events::EffectsProduced {
                        resolution_id,
                        effects: vec![crate::domain::events::ResolvedEffect {
                            effect_type: "damage".to_owned(),
                            target_id: None,
                            payload: serde_json::json!({ "amount": 8 }),
                        }],
                    },
                ))
                .unwrap(),
                sequence_number: 3,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
        ];

        let resolution = reconstitute(resolution_id, &events).unwrap();
        assert_eq!(resolution.version, 3);
        assert_eq!(resolution.effects.len(), 1);
    }
}
