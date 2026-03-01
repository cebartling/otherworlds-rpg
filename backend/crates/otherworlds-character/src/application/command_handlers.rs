//! Command handlers for the Character Management context.
//!
//! This module contains application-level command handler functions that
//! orchestrate domain logic: load aggregate, execute command, persist events.

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::event::DomainEvent;
use otherworlds_core::repository::{EventRepository, StoredEvent};
use uuid::Uuid;

use crate::domain::aggregates::Character;
use crate::domain::commands::{AwardExperience, CreateCharacter, ModifyAttribute};
use crate::domain::events::{CharacterEvent, CharacterEventKind};

fn to_stored_event(event: &CharacterEvent) -> StoredEvent {
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

/// Reconstitutes a `Character` from stored events.
///
/// # Errors
///
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub(crate) fn reconstitute(
    character_id: Uuid,
    existing_events: &[StoredEvent],
) -> Result<Character, DomainError> {
    let mut character = Character::new(character_id);
    for stored in existing_events {
        let kind: CharacterEventKind =
            serde_json::from_value(stored.payload.clone()).map_err(|e| {
                DomainError::Infrastructure(format!("event deserialization failed: {e}"))
            })?;
        let event = CharacterEvent {
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
        character.apply(&event);
    }
    Ok(character)
}

/// Handles the `CreateCharacter` command: creates a fresh aggregate, applies
/// the create domain method, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event appending fails.
pub async fn handle_create_character(
    command: &CreateCharacter,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    if command.name.trim().is_empty() {
        return Err(DomainError::Validation(
            "character name must not be empty".into(),
        ));
    }

    let character_id = command.character_id;
    let mut character = Character::new(character_id);

    character.create(command.name.clone(), command.correlation_id, clock);

    let stored_events: Vec<StoredEvent> = character
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(character_id, character.version, &stored_events)
        .await?;

    Ok(stored_events)
}

/// Handles the `ModifyAttribute` command: reconstitutes the aggregate, modifies
/// the attribute, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_modify_attribute(
    command: &ModifyAttribute,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    if command.attribute.trim().is_empty() {
        return Err(DomainError::Validation(
            "attribute name must not be empty".into(),
        ));
    }

    let existing_events = repo.load_events(command.character_id).await?;
    let mut character = reconstitute(command.character_id, &existing_events)?;

    character.modify_attribute(
        command.attribute.clone(),
        command.new_value,
        command.correlation_id,
        clock,
    );

    let stored_events: Vec<StoredEvent> = character
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.character_id, character.version, &stored_events)
        .await?;

    Ok(stored_events)
}

/// Handles the `AwardExperience` command: reconstitutes the aggregate, awards
/// experience, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_award_experience(
    command: &AwardExperience,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    if command.amount == 0 {
        return Err(DomainError::Validation(
            "experience amount must be greater than zero".into(),
        ));
    }

    let existing_events = repo.load_events(command.character_id).await?;
    let mut character = reconstitute(command.character_id, &existing_events)?;

    character.award_experience(command.amount, command.correlation_id, clock);

    let stored_events: Vec<StoredEvent> = character
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.character_id, character.version, &stored_events)
        .await?;

    Ok(stored_events)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use uuid::Uuid;

    use crate::application::command_handlers::{
        handle_award_experience, handle_create_character, handle_modify_attribute,
    };
    use crate::domain::commands::{AwardExperience, CreateCharacter, ModifyAttribute};
    use otherworlds_test_support::{FixedClock, RecordingEventRepository};

    #[tokio::test]
    async fn test_handle_create_character_persists_character_created_event() {
        // Arrange
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let character_id = Uuid::new_v4();
        let command = CreateCharacter {
            correlation_id,
            character_id,
            name: "Alaric".to_owned(),
        };

        // Act
        let result = handle_create_character(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, character_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "character.character_created");
        assert_eq!(stored.aggregate_id, character_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_modify_attribute_persists_attribute_modified_event() {
        // Arrange
        let character_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = ModifyAttribute {
            correlation_id,
            character_id,
            attribute: "strength".to_owned(),
            new_value: 18,
        };

        // Act
        let result = handle_modify_attribute(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, character_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "character.attribute_modified");
        assert_eq!(stored.aggregate_id, character_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_award_experience_persists_experience_gained_event() {
        // Arrange
        let character_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = AwardExperience {
            correlation_id,
            character_id,
            amount: 250,
        };

        // Act
        let result = handle_award_experience(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, character_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "character.experience_gained");
        assert_eq!(stored.aggregate_id, character_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_create_character_rejects_empty_name() {
        // Arrange
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = CreateCharacter {
            correlation_id: Uuid::new_v4(),
            character_id: Uuid::new_v4(),
            name: "  ".to_owned(),
        };

        // Act
        let result = handle_create_character(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "character name must not be empty");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_modify_attribute_rejects_empty_attribute() {
        // Arrange
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = ModifyAttribute {
            correlation_id: Uuid::new_v4(),
            character_id: Uuid::new_v4(),
            attribute: String::new(),
            new_value: 18,
        };

        // Act
        let result = handle_modify_attribute(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "attribute name must not be empty");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_award_experience_rejects_zero_amount() {
        // Arrange
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = AwardExperience {
            correlation_id: Uuid::new_v4(),
            character_id: Uuid::new_v4(),
            amount: 0,
        };

        // Act
        let result = handle_award_experience(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "experience amount must be greater than zero");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
