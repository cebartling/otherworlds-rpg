//! Query handlers for the Narrative Orchestration context.
//!
//! This module contains query handlers that reconstitute aggregates
//! from stored events and return read-only view DTOs.

use otherworlds_core::error::DomainError;
use otherworlds_core::repository::EventRepository;
use serde::Serialize;
use uuid::Uuid;

use crate::application::command_handlers;
use crate::domain::value_objects::ChoiceOption;

/// Read-only view of a narrative session aggregate.
#[derive(Debug, Serialize)]
pub struct NarrativeSessionView {
    /// The session identifier.
    pub session_id: Uuid,
    /// The most recent beat ID.
    pub current_beat_id: Option<Uuid>,
    /// All choice IDs presented in this session.
    pub choice_ids: Vec<Uuid>,
    /// The current scene ID.
    pub current_scene_id: Option<String>,
    /// History of scene IDs visited in order.
    pub scene_history: Vec<String>,
    /// Active choice options for the current scene.
    pub active_choice_options: Vec<ChoiceOption>,
    /// Current version (event count).
    pub version: i64,
}

/// Event types used by the Narrative Orchestration context.
const EVENT_TYPES: &[&str] = &[
    "narrative.beat_advanced",
    "narrative.choice_presented",
    "narrative.choice_selected",
    "narrative.scene_started",
    "narrative.session_archived",
];

/// Summary view for listing narrative sessions.
#[derive(Debug, Serialize)]
pub struct NarrativeSessionSummary {
    /// The session identifier.
    pub session_id: Uuid,
    /// The most recent beat ID.
    pub current_beat_id: Option<Uuid>,
    /// The current scene ID.
    pub current_scene_id: Option<String>,
    /// Current version (event count).
    pub version: i64,
}

/// Lists all narrative sessions.
///
/// # Errors
///
/// Returns `DomainError::Infrastructure` if querying or deserialization fails.
pub async fn list_sessions(
    repo: &dyn EventRepository,
) -> Result<Vec<NarrativeSessionSummary>, DomainError> {
    let ids = repo.list_aggregate_ids(EVENT_TYPES).await?;
    let mut summaries = Vec::with_capacity(ids.len());
    for id in ids {
        let stored_events = repo.load_events(id).await?;
        if stored_events.is_empty() {
            continue;
        }
        let session = command_handlers::reconstitute(id, &stored_events)?;
        if session.archived {
            continue;
        }
        summaries.push(NarrativeSessionSummary {
            session_id: id,
            current_beat_id: session.current_beat_id,
            current_scene_id: session.current_scene_id.clone(),
            version: session.version,
        });
    }
    Ok(summaries)
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
        current_scene_id: session.current_scene_id.clone(),
        scene_history: session.scene_history.clone(),
        active_choice_options: session.active_choice_options.clone(),
        version: session.version,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use uuid::Uuid;

    use crate::application::query_handlers::{get_session_by_id, list_sessions};
    use crate::domain::events::{BeatAdvanced, NarrativeEventKind, SceneStarted, SessionArchived};
    use crate::domain::value_objects::ChoiceOption;
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

    #[tokio::test]
    async fn test_list_sessions_returns_empty_when_no_aggregates() {
        // Arrange
        let repo = EmptyEventRepository;

        // Act
        let result = list_sessions(&repo).await.unwrap();

        // Assert
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_list_sessions_returns_summaries() {
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
        let repo = RecordingEventRepository::with_aggregate_ids(Ok(events), vec![session_id]);

        // Act
        let result = list_sessions(&repo).await.unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].session_id, session_id);
        assert_eq!(result[0].current_beat_id, Some(beat_id));
        assert_eq!(result[0].version, 1);
    }

    #[tokio::test]
    async fn test_list_sessions_excludes_archived() {
        let session_id = Uuid::new_v4();
        let beat_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![
            StoredEvent {
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
            },
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: session_id,
                event_type: "narrative.session_archived".to_owned(),
                payload: serde_json::to_value(NarrativeEventKind::SessionArchived(
                    SessionArchived { session_id },
                ))
                .unwrap(),
                sequence_number: 2,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
        ];
        let repo = RecordingEventRepository::with_aggregate_ids(Ok(events), vec![session_id]);

        let result = list_sessions(&repo).await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_session_by_id_includes_scene_state() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let choices = vec![ChoiceOption {
            label: "Go north".to_owned(),
            target_scene_id: "forest".to_owned(),
        }];

        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: session_id,
            event_type: "narrative.scene_started".to_owned(),
            payload: serde_json::to_value(NarrativeEventKind::SceneStarted(SceneStarted {
                session_id,
                scene_id: "tavern".to_owned(),
                narrative_text: "You enter the tavern.".to_owned(),
                choices: choices.clone(),
                npc_refs: vec!["barkeep".to_owned()],
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
        assert_eq!(view.current_scene_id, Some("tavern".to_owned()));
        assert_eq!(view.scene_history, vec!["tavern"]);
        assert_eq!(view.active_choice_options, choices);
        assert_eq!(view.version, 1);
    }
}
