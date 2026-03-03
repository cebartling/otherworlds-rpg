//! Command handlers for the Character Management context.
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

use crate::domain::aggregates::Character;
use crate::domain::commands::{
    ArchiveCharacter, AwardExperience, CreateCharacter, ModifyAttribute,
};
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
#[instrument(skip(clock, rng, repo), fields(character_id = %command.character_id, correlation_id = %command.correlation_id))]
pub async fn handle_create_character(
    command: &CreateCharacter,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    if command.name.trim().is_empty() {
        return Err(DomainError::Validation(
            "character name must not be empty".into(),
        ));
    }

    let character_id = command.character_id;
    let mut character = Character::new(character_id);

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        character.create(
            command.name.clone(),
            command.correlation_id,
            clock,
            &mut *rng_guard,
        );
    }

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
#[instrument(skip(clock, rng, repo), fields(character_id = %command.character_id, correlation_id = %command.correlation_id))]
pub async fn handle_modify_attribute(
    command: &ModifyAttribute,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    if command.attribute.trim().is_empty() {
        return Err(DomainError::Validation(
            "attribute name must not be empty".into(),
        ));
    }

    let existing_events = repo.load_events(command.character_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.character_id));
    }
    let mut character = reconstitute(command.character_id, &existing_events)?;

    if character.archived {
        return Err(DomainError::Validation("character is archived".into()));
    }

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        character.modify_attribute(
            command.attribute.clone(),
            command.new_value,
            command.correlation_id,
            clock,
            &mut *rng_guard,
        );
    }

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
#[instrument(skip(clock, rng, repo), fields(character_id = %command.character_id, correlation_id = %command.correlation_id))]
pub async fn handle_award_experience(
    command: &AwardExperience,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    if command.amount == 0 {
        return Err(DomainError::Validation(
            "experience amount must be greater than zero".into(),
        ));
    }

    let existing_events = repo.load_events(command.character_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.character_id));
    }
    let mut character = reconstitute(command.character_id, &existing_events)?;

    if character.archived {
        return Err(DomainError::Validation("character is archived".into()));
    }

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        character.award_experience(
            command.amount,
            command.correlation_id,
            clock,
            &mut *rng_guard,
        );
    }

    let stored_events: Vec<StoredEvent> = character
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.character_id, character.version, &stored_events)
        .await?;

    Ok(stored_events)
}

/// Handles the `ArchiveCharacter` command: reconstitutes the aggregate,
/// archives it (soft-delete), and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the character.
/// Returns `DomainError::Validation` if the character is already archived.
/// Returns `DomainError` if event appending fails.
#[instrument(skip(clock, rng, repo), fields(character_id = %command.character_id, correlation_id = %command.correlation_id))]
pub async fn handle_archive_character(
    command: &ArchiveCharacter,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.character_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.character_id));
    }
    let mut character = reconstitute(command.character_id, &existing_events)?;

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        character.archive(command.correlation_id, clock, &mut *rng_guard)?;
    }

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
    use std::sync::{Arc, Mutex};

    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::rng::DeterministicRng;
    use uuid::Uuid;

    use otherworlds_core::repository::StoredEvent;

    use crate::application::command_handlers::{
        handle_archive_character, handle_award_experience, handle_create_character,
        handle_modify_attribute,
    };
    use crate::domain::commands::{
        ArchiveCharacter, AwardExperience, CreateCharacter, ModifyAttribute,
    };
    use crate::domain::events::{CharacterCreated, CharacterEventKind};
    use otherworlds_test_support::{FixedClock, MockRng, RecordingEventRepository};

    fn character_created_event(
        character_id: Uuid,
        fixed_now: chrono::DateTime<Utc>,
    ) -> StoredEvent {
        StoredEvent {
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
        }
    }

    #[tokio::test]
    async fn test_handle_create_character_persists_character_created_event() {
        // Arrange
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let character_id = Uuid::new_v4();
        let command = CreateCharacter {
            correlation_id,
            character_id,
            name: "Alaric".to_owned(),
        };

        // Act
        let result = handle_create_character(&command, &clock, &*rng, &repo).await;

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
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let existing = vec![character_created_event(character_id, fixed_now)];
        let repo = RecordingEventRepository::new(Ok(existing));

        let command = ModifyAttribute {
            correlation_id,
            character_id,
            attribute: "strength".to_owned(),
            new_value: 18,
        };

        // Act
        let result = handle_modify_attribute(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, character_id);
        assert_eq!(*expected_version, 1);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "character.attribute_modified");
        assert_eq!(stored.aggregate_id, character_id);
        assert_eq!(stored.sequence_number, 2);
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
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let existing = vec![character_created_event(character_id, fixed_now)];
        let repo = RecordingEventRepository::new(Ok(existing));

        let command = AwardExperience {
            correlation_id,
            character_id,
            amount: 250,
        };

        // Act
        let result = handle_award_experience(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, character_id);
        assert_eq!(*expected_version, 1);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "character.experience_gained");
        assert_eq!(stored.aggregate_id, character_id);
        assert_eq!(stored.sequence_number, 2);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_create_character_rejects_empty_name() {
        // Arrange
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = CreateCharacter {
            correlation_id: Uuid::new_v4(),
            character_id: Uuid::new_v4(),
            name: "  ".to_owned(),
        };

        // Act
        let result = handle_create_character(&command, &clock, &*rng, &repo).await;

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
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = ModifyAttribute {
            correlation_id: Uuid::new_v4(),
            character_id: Uuid::new_v4(),
            attribute: String::new(),
            new_value: 18,
        };

        // Act
        let result = handle_modify_attribute(&command, &clock, &*rng, &repo).await;

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
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = AwardExperience {
            correlation_id: Uuid::new_v4(),
            character_id: Uuid::new_v4(),
            amount: 0,
        };

        // Act
        let result = handle_award_experience(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "experience amount must be greater than zero");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_archive_character_persists_character_archived_event() {
        // Arrange
        let character_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let existing = vec![character_created_event(character_id, fixed_now)];
        let repo = RecordingEventRepository::new(Ok(existing));

        let command = ArchiveCharacter {
            correlation_id,
            character_id,
        };

        // Act
        let result = handle_archive_character(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, character_id);
        assert_eq!(*expected_version, 1);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "character.character_archived");
        assert_eq!(stored.aggregate_id, character_id);
        assert_eq!(stored.sequence_number, 2);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_archive_character_rejects_not_found() {
        // Arrange
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));
        let character_id = Uuid::new_v4();

        let command = ArchiveCharacter {
            correlation_id: Uuid::new_v4(),
            character_id,
        };

        // Act
        let result = handle_archive_character(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, character_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_archive_character_rejects_already_archived() {
        // Arrange
        let character_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));

        let archived_event = StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: character_id,
            event_type: "character.character_archived".to_owned(),
            payload: serde_json::to_value(CharacterEventKind::CharacterArchived(
                crate::domain::events::CharacterArchived { character_id },
            ))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        };
        let existing = vec![
            character_created_event(character_id, fixed_now),
            archived_event,
        ];
        let repo = RecordingEventRepository::new(Ok(existing));

        let command = ArchiveCharacter {
            correlation_id: Uuid::new_v4(),
            character_id,
        };

        // Act
        let result = handle_archive_character(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "character is already archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_modify_attribute_rejects_archived_character() {
        // Arrange
        let character_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));

        let archived_event = StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: character_id,
            event_type: "character.character_archived".to_owned(),
            payload: serde_json::to_value(CharacterEventKind::CharacterArchived(
                crate::domain::events::CharacterArchived { character_id },
            ))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        };
        let existing = vec![
            character_created_event(character_id, fixed_now),
            archived_event,
        ];
        let repo = RecordingEventRepository::new(Ok(existing));

        let command = ModifyAttribute {
            correlation_id: Uuid::new_v4(),
            character_id,
            attribute: "strength".to_owned(),
            new_value: 18,
        };

        // Act
        let result = handle_modify_attribute(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "character is archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_award_experience_rejects_archived_character() {
        // Arrange
        let character_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));

        let archived_event = StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: character_id,
            event_type: "character.character_archived".to_owned(),
            payload: serde_json::to_value(CharacterEventKind::CharacterArchived(
                crate::domain::events::CharacterArchived { character_id },
            ))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        };
        let existing = vec![
            character_created_event(character_id, fixed_now),
            archived_event,
        ];
        let repo = RecordingEventRepository::new(Ok(existing));

        let command = AwardExperience {
            correlation_id: Uuid::new_v4(),
            character_id,
            amount: 250,
        };

        // Act
        let result = handle_award_experience(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "character is archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_modify_attribute_rejects_not_found() {
        // Arrange
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));
        let character_id = Uuid::new_v4();

        let command = ModifyAttribute {
            correlation_id: Uuid::new_v4(),
            character_id,
            attribute: "strength".to_owned(),
            new_value: 18,
        };

        // Act
        let result = handle_modify_attribute(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, character_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_award_experience_rejects_not_found() {
        // Arrange
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));
        let character_id = Uuid::new_v4();

        let command = AwardExperience {
            correlation_id: Uuid::new_v4(),
            character_id,
            amount: 250,
        };

        // Act
        let result = handle_award_experience(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, character_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }
}
