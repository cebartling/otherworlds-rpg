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
use crate::domain::events::CharacterEvent;

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
fn reconstitute(character_id: Uuid, existing_events: &[StoredEvent]) -> Character {
    let mut character = Character::new(character_id);
    #[allow(clippy::cast_possible_wrap)]
    let version = existing_events.len() as i64;
    character.version = version;
    character
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
    let character_id = Uuid::new_v4();
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
    let existing_events = repo.load_events(command.character_id).await?;
    let mut character = reconstitute(command.character_id, &existing_events);

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
    let existing_events = repo.load_events(command.character_id).await?;
    let mut character = reconstitute(command.character_id, &existing_events);

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
    use chrono::{DateTime, TimeZone, Utc};
    use otherworlds_core::clock::Clock;
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::{EventRepository, StoredEvent};
    use std::sync::Mutex;
    use uuid::Uuid;

    use crate::application::command_handlers::{
        handle_award_experience, handle_create_character, handle_modify_attribute,
    };
    use crate::domain::commands::{AwardExperience, CreateCharacter, ModifyAttribute};

    #[derive(Debug)]
    struct FixedClock(DateTime<Utc>);

    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
        }
    }

    #[derive(Debug)]
    struct MockEventRepository {
        load_result: Mutex<Option<Result<Vec<StoredEvent>, DomainError>>>,
        appended: Mutex<Vec<(Uuid, i64, Vec<StoredEvent>)>>,
    }

    impl MockEventRepository {
        fn new(load_result: Result<Vec<StoredEvent>, DomainError>) -> Self {
            Self {
                load_result: Mutex::new(Some(load_result)),
                appended: Mutex::new(Vec::new()),
            }
        }

        fn appended_events(&self) -> Vec<(Uuid, i64, Vec<StoredEvent>)> {
            self.appended.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl EventRepository for MockEventRepository {
        async fn load_events(&self, _aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
            self.load_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Ok(Vec::new()))
        }

        async fn append_events(
            &self,
            aggregate_id: Uuid,
            expected_version: i64,
            events: &[StoredEvent],
        ) -> Result<(), DomainError> {
            self.appended
                .lock()
                .unwrap()
                .push((aggregate_id, expected_version, events.to_vec()));
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_handle_create_character_persists_character_created_event() {
        // Arrange
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = MockEventRepository::new(Ok(Vec::new()));

        let command = CreateCharacter {
            correlation_id,
            name: "Alaric".to_owned(),
        };

        // Act
        let result = handle_create_character(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (_, expected_version, events) = &appended[0];
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "character.character_created");
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
        let repo = MockEventRepository::new(Ok(Vec::new()));

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
        let repo = MockEventRepository::new(Ok(Vec::new()));

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
}
