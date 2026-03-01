//! Command handlers for the Narrative Orchestration context.
//!
//! This module contains application-level command handler functions that
//! orchestrate domain logic: load aggregate, execute command, persist events.

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::event::DomainEvent;
use otherworlds_core::repository::{EventRepository, StoredEvent};
use uuid::Uuid;

use crate::domain::aggregates::NarrativeSession;
use crate::domain::commands::{AdvanceBeat, PresentChoice};
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
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.session_id).await?;
    let mut session = reconstitute(command.session_id, &existing_events)?;

    session.advance_beat(command.correlation_id, clock);

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
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.session_id).await?;
    let mut session = reconstitute(command.session_id, &existing_events)?;

    session.present_choice(command.correlation_id, clock);

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

    use crate::application::command_handlers::{handle_advance_beat, handle_present_choice};
    use crate::domain::commands::{AdvanceBeat, PresentChoice};
    use otherworlds_test_support::{FixedClock, RecordingEventRepository};

    #[tokio::test]
    async fn test_handle_advance_beat_persists_beat_advanced_event() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = AdvanceBeat {
            correlation_id,
            session_id,
        };

        // Act
        let result = handle_advance_beat(&command, &clock, &repo).await;

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
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = PresentChoice {
            correlation_id,
            session_id,
        };

        // Act
        let result = handle_present_choice(&command, &clock, &repo).await;

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
}
