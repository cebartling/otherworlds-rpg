//! Command handlers for the Narrative Orchestration context.
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
use tracing::instrument;
use uuid::Uuid;

use crate::domain::aggregates::NarrativeSession;
use crate::domain::commands::{
    AdvanceBeat, ArchiveSession, EnterScene, PresentChoice, SelectChoice,
};
use crate::domain::events::{NarrativeEvent, NarrativeEventKind};

fn to_stored_event(event: &NarrativeEvent) -> StoredEvent {
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

/// Reconstitutes a `NarrativeSession` from stored events.
///
/// # Errors
///
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub(crate) fn reconstitute(
    session_id: Uuid,
    existing_events: &[StoredEvent],
) -> Result<NarrativeSession, DomainError> {
    let mut session = NarrativeSession::new(session_id);
    for stored in existing_events {
        let kind: NarrativeEventKind =
            serde_json::from_value(stored.payload.clone()).map_err(|e| {
                DomainError::Infrastructure(format!("event deserialization failed: {e}"))
            })?;
        let event = NarrativeEvent {
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
        session.apply(&event);
    }
    Ok(session)
}

/// Handles the `AdvanceBeat` command: reconstitutes the aggregate, advances
/// the beat, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
#[instrument(skip(clock, rng, repo), fields(session_id = %command.session_id, correlation_id = %command.correlation_id))]
pub async fn handle_advance_beat(
    command: &AdvanceBeat,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.session_id).await?;
    let mut session = reconstitute(command.session_id, &existing_events)?;

    if session.archived {
        return Err(DomainError::Validation("session is archived".into()));
    }

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        session.advance_beat(command.correlation_id, clock, &mut *rng_guard);
    }

    let stored_events: Vec<StoredEvent> = session
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.session_id, session.version, &stored_events)
        .await?;

    Ok(stored_events)
}

/// Handles the `PresentChoice` command: reconstitutes the aggregate, presents
/// a choice, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
#[instrument(skip(clock, rng, repo), fields(session_id = %command.session_id, correlation_id = %command.correlation_id))]
pub async fn handle_present_choice(
    command: &PresentChoice,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.session_id).await?;
    let mut session = reconstitute(command.session_id, &existing_events)?;

    if session.archived {
        return Err(DomainError::Validation("session is archived".into()));
    }

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        session.present_choice(command.correlation_id, clock, &mut *rng_guard);
    }

    let stored_events: Vec<StoredEvent> = session
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.session_id, session.version, &stored_events)
        .await?;

    Ok(stored_events)
}

/// Handles the `ArchiveSession` command: reconstitutes the aggregate,
/// archives it (soft-delete), and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the session.
/// Returns `DomainError::Validation` if the session is already archived.
/// Returns `DomainError` if event appending fails.
#[instrument(skip(clock, rng, repo), fields(session_id = %command.session_id, correlation_id = %command.correlation_id))]
pub async fn handle_archive_session(
    command: &ArchiveSession,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.session_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.session_id));
    }
    let mut session = reconstitute(command.session_id, &existing_events)?;

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        session.archive(command.correlation_id, clock, &mut *rng_guard)?;
    }

    let stored_events: Vec<StoredEvent> = session
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.session_id, session.version, &stored_events)
        .await?;

    Ok(stored_events)
}

/// Handles the `EnterScene` command: reconstitutes the aggregate, enters
/// the scene, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading, validation, or appending fails.
#[instrument(skip(clock, rng, repo), fields(session_id = %command.session_id, correlation_id = %command.correlation_id))]
pub async fn handle_enter_scene(
    command: &EnterScene,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.session_id).await?;
    let mut session = reconstitute(command.session_id, &existing_events)?;

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        session.enter_scene(
            &command.scene_data,
            command.correlation_id,
            clock,
            &mut *rng_guard,
        )?;
    }

    let stored_events: Vec<StoredEvent> = session
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.session_id, session.version, &stored_events)
        .await?;

    Ok(stored_events)
}

/// Handles the `SelectChoice` command: reconstitutes the aggregate, selects
/// the choice and transitions to the target scene, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading, validation, or appending fails.
#[instrument(skip(clock, rng, repo), fields(session_id = %command.session_id, correlation_id = %command.correlation_id))]
pub async fn handle_select_choice(
    command: &SelectChoice,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.session_id).await?;
    let mut session = reconstitute(command.session_id, &existing_events)?;

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        session.select_choice(
            command.choice_index,
            &command.target_scene_data,
            command.correlation_id,
            clock,
            &mut *rng_guard,
        )?;
    }

    let stored_events: Vec<StoredEvent> = session
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.session_id, session.version, &stored_events)
        .await?;

    Ok(stored_events)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;

    use crate::application::command_handlers::{
        handle_advance_beat, handle_archive_session, handle_enter_scene, handle_present_choice,
        handle_select_choice,
    };
    use crate::domain::commands::{
        AdvanceBeat, ArchiveSession, EnterScene, PresentChoice, SelectChoice,
    };
    use crate::domain::events::{BeatAdvanced, NarrativeEventKind, SceneStarted};
    use crate::domain::value_objects::{ChoiceOption, SceneData};
    use otherworlds_core::rng::DeterministicRng;
    use otherworlds_test_support::{
        ConflictingEventRepository, FixedClock, MockRng, RecordingEventRepository,
    };
    use std::sync::{Arc, Mutex};

    fn beat_advanced_event(session_id: Uuid, fixed_now: chrono::DateTime<Utc>) -> StoredEvent {
        StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: session_id,
            event_type: "narrative.beat_advanced".to_owned(),
            payload: serde_json::to_value(NarrativeEventKind::BeatAdvanced(BeatAdvanced {
                session_id,
                beat_id: Uuid::new_v4(),
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }
    }

    #[tokio::test]
    async fn test_handle_advance_beat_persists_beat_advanced_event() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = AdvanceBeat {
            correlation_id,
            session_id,
        };

        // Act
        let result = handle_advance_beat(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, session_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "narrative.beat_advanced");
        assert_eq!(stored.aggregate_id, session_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_present_choice_persists_choice_presented_event() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = PresentChoice {
            correlation_id,
            session_id,
        };

        // Act
        let result = handle_present_choice(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, session_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "narrative.choice_presented");
        assert_eq!(stored.aggregate_id, session_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_archive_session_persists_session_archived_event() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let existing = vec![beat_advanced_event(session_id, fixed_now)];
        let repo = RecordingEventRepository::new(Ok(existing));

        let command = ArchiveSession {
            correlation_id,
            session_id,
        };

        // Act
        let result = handle_archive_session(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, session_id);
        assert_eq!(*expected_version, 1);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "narrative.session_archived");
        assert_eq!(stored.aggregate_id, session_id);
        assert_eq!(stored.sequence_number, 2);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_archive_session_rejects_not_found() {
        // Arrange
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));
        let session_id = Uuid::new_v4();

        let command = ArchiveSession {
            correlation_id: Uuid::new_v4(),
            session_id,
        };

        // Act
        let result = handle_archive_session(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, session_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_archive_session_rejects_already_archived() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);

        let archived_event = StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: session_id,
            event_type: "narrative.session_archived".to_owned(),
            payload: serde_json::to_value(NarrativeEventKind::SessionArchived(
                crate::domain::events::SessionArchived { session_id },
            ))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        };
        let existing = vec![beat_advanced_event(session_id, fixed_now), archived_event];
        let repo = RecordingEventRepository::new(Ok(existing));

        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));

        let command = ArchiveSession {
            correlation_id: Uuid::new_v4(),
            session_id,
        };

        // Act
        let result = handle_archive_session(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "session is already archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_advance_beat_rejects_archived_session() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);

        let archived_event = StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: session_id,
            event_type: "narrative.session_archived".to_owned(),
            payload: serde_json::to_value(NarrativeEventKind::SessionArchived(
                crate::domain::events::SessionArchived { session_id },
            ))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        };
        let existing = vec![beat_advanced_event(session_id, fixed_now), archived_event];
        let repo = RecordingEventRepository::new(Ok(existing));

        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));

        let command = AdvanceBeat {
            correlation_id: Uuid::new_v4(),
            session_id,
        };

        // Act
        let result = handle_advance_beat(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "session is archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_present_choice_rejects_archived_session() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);

        let archived_event = StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: session_id,
            event_type: "narrative.session_archived".to_owned(),
            payload: serde_json::to_value(NarrativeEventKind::SessionArchived(
                crate::domain::events::SessionArchived { session_id },
            ))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        };
        let existing = vec![beat_advanced_event(session_id, fixed_now), archived_event];
        let repo = RecordingEventRepository::new(Ok(existing));

        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));

        let command = PresentChoice {
            correlation_id: Uuid::new_v4(),
            session_id,
        };

        // Act
        let result = handle_present_choice(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "session is archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_advance_beat_propagates_concurrency_conflict() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = ConflictingEventRepository::new(vec![], session_id, 0, 1);

        let command = AdvanceBeat {
            correlation_id: Uuid::new_v4(),
            session_id,
        };

        // Act
        let result = handle_advance_beat(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ConcurrencyConflict {
                aggregate_id,
                expected,
                actual,
            } => {
                assert_eq!(aggregate_id, session_id);
                assert_eq!(expected, 0);
                assert_eq!(actual, 1);
            }
            other => panic!("expected ConcurrencyConflict, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_present_choice_propagates_concurrency_conflict() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = ConflictingEventRepository::new(vec![], session_id, 0, 1);

        let command = PresentChoice {
            correlation_id: Uuid::new_v4(),
            session_id,
        };

        // Act
        let result = handle_present_choice(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ConcurrencyConflict {
                aggregate_id,
                expected,
                actual,
            } => {
                assert_eq!(aggregate_id, session_id);
                assert_eq!(expected, 0);
                assert_eq!(actual, 1);
            }
            other => panic!("expected ConcurrencyConflict, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_archive_session_propagates_concurrency_conflict() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let existing = vec![beat_advanced_event(session_id, fixed_now)];
        let repo = ConflictingEventRepository::new(existing, session_id, 1, 2);

        let command = ArchiveSession {
            correlation_id: Uuid::new_v4(),
            session_id,
        };

        // Act
        let result = handle_archive_session(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ConcurrencyConflict {
                aggregate_id,
                expected,
                actual,
            } => {
                assert_eq!(aggregate_id, session_id);
                assert_eq!(expected, 1);
                assert_eq!(actual, 2);
            }
            other => panic!("expected ConcurrencyConflict, got {other:?}"),
        }
    }

    fn sample_scene_data(scene_id: &str, choices: Vec<(&str, &str)>) -> SceneData {
        SceneData {
            scene_id: scene_id.to_owned(),
            narrative_text: format!("You are in {scene_id}."),
            choices: choices
                .into_iter()
                .map(|(label, target)| ChoiceOption {
                    label: label.to_owned(),
                    target_scene_id: target.to_owned(),
                })
                .collect(),
            npc_refs: vec![],
        }
    }

    fn scene_started_event(
        session_id: Uuid,
        scene_id: &str,
        choices: Vec<ChoiceOption>,
        fixed_now: chrono::DateTime<Utc>,
        sequence_number: i64,
    ) -> StoredEvent {
        StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: session_id,
            event_type: "narrative.scene_started".to_owned(),
            payload: serde_json::to_value(NarrativeEventKind::SceneStarted(SceneStarted {
                session_id,
                scene_id: scene_id.to_owned(),
                narrative_text: format!("You are in {scene_id}."),
                choices,
                npc_refs: vec![],
            }))
            .unwrap(),
            sequence_number,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }
    }

    #[tokio::test]
    async fn test_handle_enter_scene_persists_scene_started_event() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = EnterScene {
            correlation_id,
            session_id,
            scene_data: sample_scene_data("tavern", vec![("Leave", "street")]),
        };

        // Act
        let result = handle_enter_scene(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, session_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "narrative.scene_started");
        assert_eq!(stored.aggregate_id, session_id);
        assert_eq!(stored.correlation_id, correlation_id);
    }

    #[tokio::test]
    async fn test_handle_enter_scene_rejects_archived_session() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));

        let archived_event = StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: session_id,
            event_type: "narrative.session_archived".to_owned(),
            payload: serde_json::to_value(NarrativeEventKind::SessionArchived(
                crate::domain::events::SessionArchived { session_id },
            ))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        };
        let existing = vec![beat_advanced_event(session_id, fixed_now), archived_event];
        let repo = RecordingEventRepository::new(Ok(existing));

        let command = EnterScene {
            correlation_id: Uuid::new_v4(),
            session_id,
            scene_data: sample_scene_data("tavern", vec![]),
        };

        // Act
        let result = handle_enter_scene(&command, &clock, &*rng, &repo).await;

        // Assert
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert_eq!(msg, "session is archived"),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_select_choice_persists_two_events() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));

        let choices = vec![ChoiceOption {
            label: "Go north".to_owned(),
            target_scene_id: "forest".to_owned(),
        }];
        let existing = vec![scene_started_event(
            session_id, "start", choices, fixed_now, 1,
        )];
        let repo = RecordingEventRepository::new(Ok(existing));

        let command = SelectChoice {
            correlation_id,
            session_id,
            choice_index: 0,
            target_scene_data: sample_scene_data("forest", vec![("Return", "start")]),
        };

        // Act
        let result = handle_select_choice(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (_, expected_version, events) = &appended[0];
        assert_eq!(*expected_version, 1);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, "narrative.choice_selected");
        assert_eq!(events[1].event_type, "narrative.scene_started");
    }

    #[tokio::test]
    async fn test_handle_select_choice_rejects_no_active_scene() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = SelectChoice {
            correlation_id: Uuid::new_v4(),
            session_id,
            choice_index: 0,
            target_scene_data: sample_scene_data("next", vec![]),
        };

        // Act
        let result = handle_select_choice(&command, &clock, &*rng, &repo).await;

        // Assert
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert_eq!(msg, "no active scene"),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_select_choice_rejects_out_of_bounds() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));

        let choices = vec![ChoiceOption {
            label: "Go north".to_owned(),
            target_scene_id: "forest".to_owned(),
        }];
        let existing = vec![scene_started_event(
            session_id, "start", choices, fixed_now, 1,
        )];
        let repo = RecordingEventRepository::new(Ok(existing));

        let command = SelectChoice {
            correlation_id: Uuid::new_v4(),
            session_id,
            choice_index: 5,
            target_scene_data: sample_scene_data("forest", vec![]),
        };

        // Act
        let result = handle_select_choice(&command, &clock, &*rng, &repo).await;

        // Assert
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert!(msg.contains("choice index 5 out of bounds")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_enter_scene_propagates_concurrency_conflict() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = ConflictingEventRepository::new(vec![], session_id, 0, 1);

        let command = EnterScene {
            correlation_id: Uuid::new_v4(),
            session_id,
            scene_data: sample_scene_data("tavern", vec![]),
        };

        // Act
        let result = handle_enter_scene(&command, &clock, &*rng, &repo).await;

        // Assert
        match result.unwrap_err() {
            DomainError::ConcurrencyConflict {
                aggregate_id,
                expected,
                actual,
            } => {
                assert_eq!(aggregate_id, session_id);
                assert_eq!(expected, 0);
                assert_eq!(actual, 1);
            }
            other => panic!("expected ConcurrencyConflict, got {other:?}"),
        }
    }
}
