//! Query handlers for the Narrative Orchestration context.
//!
//! This module contains query handlers that reconstitute aggregates
//! from stored events and return read-only view DTOs.

use otherworlds_core::error::DomainError;
use otherworlds_core::repository::EventRepository;
use serde::Serialize;
use uuid::Uuid;

use crate::application::command_handlers;

/// Read-only view of a narrative session aggregate.
#[derive(Debug, Serialize)]
pub struct NarrativeSessionView {
    /// The session identifier.
    pub session_id: Uuid,
    /// The most recent beat ID.
    pub current_beat_id: Option<Uuid>,
    /// All choice IDs presented in this session.
    pub choice_ids: Vec<Uuid>,
    /// Current version (event count).
    pub version: i64,
}

/// Retrieves a narrative session by its aggregate ID.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the ID.
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub async fn get_session_by_id(
    session_id: Uuid,
    repo: &dyn EventRepository,
) -> Result<NarrativeSessionView, DomainError> {
    let stored_events = repo.load_events(session_id).await?;
    if stored_events.is_empty() {
        return Err(DomainError::AggregateNotFound(session_id));
    }
    let session = command_handlers::reconstitute(session_id, &stored_events)?;
    Ok(NarrativeSessionView {
        session_id,
        current_beat_id: session.current_beat_id,
        choice_ids: session.choice_ids.clone(),
        version: session.version,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use uuid::Uuid;

    use crate::application::query_handlers::get_session_by_id;
    use crate::domain::events::{BeatAdvanced, NarrativeEventKind};
    use otherworlds_test_support::{EmptyEventRepository, RecordingEventRepository};

    #[tokio::test]
    async fn test_get_session_by_id_returns_view_with_state() {
        // Arrange
        let session_id = Uuid::new_v4();
        let beat_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: session_id,
            event_type: "narrative.beat_advanced".to_owned(),
            payload: serde_json::to_value(NarrativeEventKind::BeatAdvanced(BeatAdvanced {
                session_id,
                beat_id,
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }];
        let repo = RecordingEventRepository::new(Ok(events));

        // Act
        let view = get_session_by_id(session_id, &repo).await.unwrap();

        // Assert
        assert_eq!(view.session_id, session_id);
        assert_eq!(view.current_beat_id, Some(beat_id));
        assert!(view.choice_ids.is_empty());
        assert_eq!(view.version, 1);
    }

    #[tokio::test]
    async fn test_get_session_by_id_returns_not_found_when_no_events() {
        // Arrange
        let session_id = Uuid::new_v4();
        let repo = EmptyEventRepository;

        // Act
        let result = get_session_by_id(session_id, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, session_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }
}
