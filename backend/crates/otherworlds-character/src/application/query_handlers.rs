//! Query handlers for the Character Management context.
//!
//! This module contains query handlers that reconstitute aggregates
//! from stored events and return read-only view DTOs.

use std::collections::HashMap;

use otherworlds_core::error::DomainError;
use otherworlds_core::repository::EventRepository;
use serde::Serialize;
use uuid::Uuid;

use crate::application::command_handlers;

/// Read-only view of a character aggregate.
#[derive(Debug, Serialize)]
pub struct CharacterView {
    /// The character identifier.
    pub character_id: Uuid,
    /// The character's name.
    pub name: Option<String>,
    /// Character attributes (e.g., "strength" → 18).
    pub attributes: HashMap<String, i32>,
    /// Total experience accumulated.
    pub experience: u32,
    /// Current version (event count).
    pub version: i64,
}

/// Event types used by the Character Management context.
const EVENT_TYPES: &[&str] = &[
    "character.character_created",
    "character.attribute_modified",
    "character.experience_gained",
];

/// Summary view for listing characters.
#[derive(Debug, Serialize)]
pub struct CharacterSummary {
    /// The character identifier.
    pub character_id: Uuid,
    /// The character's name.
    pub name: Option<String>,
    /// Total experience accumulated.
    pub experience: u32,
    /// Current version (event count).
    pub version: i64,
}

/// Lists all characters.
///
/// # Errors
///
/// Returns `DomainError::Infrastructure` if querying or deserialization fails.
pub async fn list_characters(
    repo: &dyn EventRepository,
) -> Result<Vec<CharacterSummary>, DomainError> {
    let ids = repo.list_aggregate_ids(EVENT_TYPES).await?;
    let mut summaries = Vec::with_capacity(ids.len());
    for id in ids {
        let stored_events = repo.load_events(id).await?;
        if stored_events.is_empty() {
            continue;
        }
        let character = command_handlers::reconstitute(id, &stored_events)?;
        summaries.push(CharacterSummary {
            character_id: id,
            name: character.name.clone(),
            experience: character.experience,
            version: character.version,
        });
    }
    Ok(summaries)
}

/// Retrieves a character by its aggregate ID.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the ID.
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub async fn get_character_by_id(
    character_id: Uuid,
    repo: &dyn EventRepository,
) -> Result<CharacterView, DomainError> {
    let stored_events = repo.load_events(character_id).await?;
    if stored_events.is_empty() {
        return Err(DomainError::AggregateNotFound(character_id));
    }
    let character = command_handlers::reconstitute(character_id, &stored_events)?;
    Ok(CharacterView {
        character_id,
        name: character.name.clone(),
        attributes: character.attributes.clone(),
        experience: character.experience,
        version: character.version,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use uuid::Uuid;

    use crate::application::query_handlers::{get_character_by_id, list_characters};
    use crate::domain::events::{CharacterCreated, CharacterEventKind};
    use otherworlds_test_support::{EmptyEventRepository, RecordingEventRepository};

    #[tokio::test]
    async fn test_get_character_by_id_returns_view_with_state() {
        // Arrange
        let character_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: character_id,
            event_type: "character.character_created".to_owned(),
            payload: serde_json::to_value(CharacterEventKind::CharacterCreated(CharacterCreated {
                character_id,
                name: "Alaric".to_owned(),
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }];
        let repo = RecordingEventRepository::new(Ok(events));

        // Act
        let view = get_character_by_id(character_id, &repo).await.unwrap();

        // Assert
        assert_eq!(view.character_id, character_id);
        assert_eq!(view.name, Some("Alaric".to_owned()));
        assert!(view.attributes.is_empty());
        assert_eq!(view.experience, 0);
        assert_eq!(view.version, 1);
    }

    #[tokio::test]
    async fn test_get_character_by_id_returns_not_found_when_no_events() {
        // Arrange
        let character_id = Uuid::new_v4();
        let repo = EmptyEventRepository;

        // Act
        let result = get_character_by_id(character_id, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, character_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_list_characters_returns_empty_when_no_aggregates() {
        let repo = EmptyEventRepository;

        let result = list_characters(&repo).await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_list_characters_returns_summaries() {
        let character_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: character_id,
            event_type: "character.character_created".to_owned(),
            payload: serde_json::to_value(CharacterEventKind::CharacterCreated(CharacterCreated {
                character_id,
                name: "Alaric".to_owned(),
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }];
        let repo = RecordingEventRepository::with_aggregate_ids(Ok(events), vec![character_id]);

        let result = list_characters(&repo).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].character_id, character_id);
        assert_eq!(result[0].name, Some("Alaric".to_owned()));
        assert_eq!(result[0].experience, 0);
        assert_eq!(result[0].version, 1);
    }
}
