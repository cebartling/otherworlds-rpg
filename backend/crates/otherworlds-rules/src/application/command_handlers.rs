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

use crate::domain::aggregates::{DeclareIntentParams, Resolution};
use crate::domain::commands::{
    ArchiveResolution, DeclareIntent, EffectSpec, ProduceEffects, ResolveCheck,
};
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
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.resolution_id).await?;
    let mut resolution = reconstitute(command.resolution_id, &existing_events)?;

    if resolution.archived {
        return Err(DomainError::Validation("resolution is archived".into()));
    }

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        resolution.declare_intent(
            DeclareIntentParams {
                intent_id: command.intent_id,
                action_type: command.action_type.clone(),
                skill: command.skill.clone(),
                target_id: command.target_id,
                difficulty_class: command.difficulty_class,
                modifier: command.modifier,
            },
            command.correlation_id,
            clock,
            &mut *rng_guard,
        )?;
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

    if resolution.archived {
        return Err(DomainError::Validation("resolution is archived".into()));
    }

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
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.resolution_id).await?;
    let mut resolution = reconstitute(command.resolution_id, &existing_events)?;

    if resolution.archived {
        return Err(DomainError::Validation("resolution is archived".into()));
    }

    let effects = command
        .effects
        .iter()
        .cloned()
        .map(EffectSpec::into_resolved_effect)
        .collect();

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        resolution.produce_effects(effects, command.correlation_id, clock, &mut *rng_guard)?;
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

/// Handles the `ArchiveResolution` command: loads events, checks for existence,
/// reconstitutes the aggregate, archives it, and persists the resulting event.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the resolution.
/// Returns `DomainError::Validation` if the resolution is already archived.
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_archive_resolution(
    command: &ArchiveResolution,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.resolution_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.resolution_id));
    }
    let mut resolution = reconstitute(command.resolution_id, &existing_events)?;

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        resolution.archive(command.correlation_id, clock, &mut *rng_guard)?;
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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use otherworlds_core::rng::DeterministicRng;
    use std::sync::Mutex;
    use uuid::Uuid;

    use crate::application::command_handlers::{
        handle_archive_resolution, handle_declare_intent, handle_produce_effects,
        handle_resolve_check, reconstitute,
    };
    use crate::domain::commands::{
        ArchiveResolution, DeclareIntent, EffectSpec, ProduceEffects, ResolveCheck,
    };
    use crate::domain::events::{
        CheckOutcome, CheckResolved, IntentDeclared, ResolutionArchived, RulesEventKind,
    };
    use otherworlds_test_support::{
        EmptyEventRepository, FixedClock, MockRng, RecordingEventRepository, SequenceRng,
    };

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

        let rng: Mutex<MockRng> = Mutex::new(MockRng);
        let rng_ref: &Mutex<dyn DeterministicRng + Send> = &rng;

        let result = handle_declare_intent(&command, &clock, rng_ref, &repo).await;
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

        let rng: Mutex<MockRng> = Mutex::new(MockRng);
        let rng_ref: &Mutex<dyn DeterministicRng + Send> = &rng;

        let result = handle_declare_intent(&command, &fixed_clock(), rng_ref, &repo).await;
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
        let rng: Mutex<SequenceRng> =
            Mutex::new(SequenceRng::new(vec![15, 42, 99, 7, 13, 0, 0, 0, 0]));
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

        let rng: Mutex<MockRng> = Mutex::new(MockRng);
        let rng_ref: &Mutex<dyn DeterministicRng + Send> = &rng;

        let result = handle_produce_effects(&command, &fixed_clock(), rng_ref, &repo).await;
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

        let rng: Mutex<MockRng> = Mutex::new(MockRng);
        let rng_ref: &Mutex<dyn DeterministicRng + Send> = &rng;

        let result = handle_produce_effects(&command, &fixed_clock(), rng_ref, &repo).await;
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
        let intent = resolution.intent.as_ref().unwrap();
        assert_eq!(intent.intent_id, intent_id);
        assert_eq!(intent.action_type, "attack");
        assert!(intent.skill.is_none());
        assert_eq!(intent.difficulty_class, 12);
        assert_eq!(intent.modifier, 2);
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
        let check = resolution.check_result.as_ref().unwrap();
        assert_eq!(check.natural_roll, 18);
        assert_eq!(check.total, 21);
        assert_eq!(check.difficulty_class, 15);
        assert_eq!(check.outcome, CheckOutcome::Success);
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
        assert_eq!(resolution.effects[0].effect_type, "damage");
        assert_eq!(resolution.effects[0].payload["amount"], 8);
    }

    // --- archive handler tests ---

    fn intent_declared_event(resolution_id: Uuid, fixed_now: chrono::DateTime<Utc>) -> StoredEvent {
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
            occurred_at: fixed_now,
        }
    }

    fn resolution_archived_event(
        resolution_id: Uuid,
        fixed_now: chrono::DateTime<Utc>,
    ) -> StoredEvent {
        StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: resolution_id,
            event_type: "rules.resolution_archived".to_owned(),
            payload: serde_json::to_value(RulesEventKind::ResolutionArchived(ResolutionArchived {
                resolution_id,
            }))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }
    }

    #[tokio::test]
    async fn test_handle_archive_resolution_persists_resolution_archived_event() {
        let resolution_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = fixed_clock();
        let repo = RecordingEventRepository::new(Ok(vec![intent_declared_event(
            resolution_id,
            fixed_now,
        )]));

        let command = ArchiveResolution {
            correlation_id,
            resolution_id,
        };

        let rng: Mutex<MockRng> = Mutex::new(MockRng);
        let rng_ref: &Mutex<dyn DeterministicRng + Send> = &rng;

        let result = handle_archive_resolution(&command, &clock, rng_ref, &repo).await;
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, resolution_id);
        assert_eq!(*expected_version, 1);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "rules.resolution_archived");
        assert_eq!(stored.aggregate_id, resolution_id);
        assert_eq!(stored.correlation_id, correlation_id);
    }

    #[tokio::test]
    async fn test_handle_archive_resolution_rejects_not_found() {
        let repo = EmptyEventRepository;

        let command = ArchiveResolution {
            correlation_id: Uuid::new_v4(),
            resolution_id: Uuid::new_v4(),
        };

        let rng: Mutex<MockRng> = Mutex::new(MockRng);
        let rng_ref: &Mutex<dyn DeterministicRng + Send> = &rng;

        let result = handle_archive_resolution(&command, &fixed_clock(), rng_ref, &repo).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(_) => {}
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_archive_resolution_rejects_already_archived() {
        let resolution_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let repo = RecordingEventRepository::new(Ok(vec![
            intent_declared_event(resolution_id, fixed_now),
            resolution_archived_event(resolution_id, fixed_now),
        ]));

        let command = ArchiveResolution {
            correlation_id: Uuid::new_v4(),
            resolution_id,
        };

        let rng: Mutex<MockRng> = Mutex::new(MockRng);
        let rng_ref: &Mutex<dyn DeterministicRng + Send> = &rng;

        let result = handle_archive_resolution(&command, &fixed_clock(), rng_ref, &repo).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "resolution is already archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_declare_intent_rejects_archived_resolution() {
        let resolution_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let repo = RecordingEventRepository::new(Ok(vec![
            intent_declared_event(resolution_id, fixed_now),
            resolution_archived_event(resolution_id, fixed_now),
        ]));

        let command = DeclareIntent {
            correlation_id: Uuid::new_v4(),
            resolution_id,
            intent_id: Uuid::new_v4(),
            action_type: "skill_check".to_owned(),
            skill: None,
            target_id: None,
            difficulty_class: 15,
            modifier: 0,
        };

        let rng: Mutex<MockRng> = Mutex::new(MockRng);
        let rng_ref: &Mutex<dyn DeterministicRng + Send> = &rng;

        let result = handle_declare_intent(&command, &fixed_clock(), rng_ref, &repo).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "resolution is archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_resolve_check_rejects_archived_resolution() {
        let resolution_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let repo = RecordingEventRepository::new(Ok(vec![
            intent_declared_event(resolution_id, fixed_now),
            resolution_archived_event(resolution_id, fixed_now),
        ]));
        let rng: Mutex<SequenceRng> = Mutex::new(SequenceRng::new(vec![15, 42, 99, 7, 13]));
        let rng_ref: &Mutex<dyn DeterministicRng + Send> = &rng;

        let command = ResolveCheck {
            correlation_id: Uuid::new_v4(),
            resolution_id,
        };

        let result = handle_resolve_check(&command, &fixed_clock(), rng_ref, &repo).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "resolution is archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_produce_effects_rejects_archived_resolution() {
        let resolution_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let repo = RecordingEventRepository::new(Ok(vec![
            intent_declared_event(resolution_id, fixed_now),
            resolution_archived_event(resolution_id, fixed_now),
        ]));

        let command = ProduceEffects {
            correlation_id: Uuid::new_v4(),
            resolution_id,
            effects: vec![],
        };

        let rng: Mutex<MockRng> = Mutex::new(MockRng);
        let rng_ref: &Mutex<dyn DeterministicRng + Send> = &rng;

        let result = handle_produce_effects(&command, &fixed_clock(), rng_ref, &repo).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "resolution is archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
