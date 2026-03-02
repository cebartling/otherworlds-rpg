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
use uuid::Uuid;

use crate::domain::aggregates::NarrativeSession;
use crate::domain::commands::{AdvanceBeat, ArchiveSession, PresentChoice};
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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;

    use crate::application::command_handlers::{
        handle_advance_beat, handle_archive_session, handle_present_choice,
    };
    use crate::domain::commands::{AdvanceBeat, ArchiveSession, PresentChoice};
    use crate::domain::events::{BeatAdvanced, NarrativeEventKind};
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
}
