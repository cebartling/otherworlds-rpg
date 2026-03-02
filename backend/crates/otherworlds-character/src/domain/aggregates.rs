//! Aggregate roots for the Character Management context.

use std::collections::HashMap;

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::event::EventMetadata;
use otherworlds_core::rng::DeterministicRng;
use uuid::Uuid;

use otherworlds_core::error::DomainError;

use super::events::{
    AttributeModified, CharacterArchived, CharacterCreated, CharacterEvent, CharacterEventKind,
    ExperienceGained,
};

/// The aggregate root for a character.
#[derive(Debug)]
pub struct Character {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub(crate) version: i64,
    /// The character's name (set on creation).
    pub(crate) name: Option<String>,
    /// Character attributes (e.g., "strength" → 18).
    pub(crate) attributes: HashMap<String, i32>,
    /// Total experience accumulated.
    pub(crate) experience: u32,
    /// Whether this character has been archived (soft-deleted).
    pub(crate) archived: bool,
    /// Uncommitted events pending persistence.
    uncommitted_events: Vec<CharacterEvent>,
}

impl Character {
    /// Creates a new character.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
            name: None,
            attributes: HashMap::new(),
            experience: 0,
            archived: false,
            uncommitted_events: Vec::new(),
        }
    }

    /// Returns the next sequence number for a new event.
    #[allow(clippy::cast_possible_wrap)]
    fn next_sequence_number(&self) -> i64 {
        self.version + self.uncommitted_events.len() as i64 + 1
    }

    /// Creates a character, producing a `CharacterCreated` event.
    pub fn create(
        &mut self,
        name: String,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) {
        let event = CharacterEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
                event_type: "character.character_created".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: CharacterEventKind::CharacterCreated(CharacterCreated {
                character_id: self.id,
                name,
            }),
        };

        self.uncommitted_events.push(event);
    }

    /// Modifies a character attribute, producing an `AttributeModified` event.
    pub fn modify_attribute(
        &mut self,
        attribute: String,
        new_value: i32,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) {
        let event = CharacterEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
                event_type: "character.attribute_modified".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: CharacterEventKind::AttributeModified(AttributeModified {
                character_id: self.id,
                attribute,
                new_value,
            }),
        };

        self.uncommitted_events.push(event);
    }

    /// Archives (soft-deletes) a character, producing a `CharacterArchived` event.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if the character is already archived.
    pub fn archive(
        &mut self,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) -> Result<(), DomainError> {
        if self.archived {
            return Err(DomainError::Validation(
                "character is already archived".into(),
            ));
        }

        let event = CharacterEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
                event_type: "character.character_archived".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: CharacterEventKind::CharacterArchived(CharacterArchived {
                character_id: self.id,
            }),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    /// Awards experience to a character, producing an `ExperienceGained` event.
    pub fn award_experience(
        &mut self,
        amount: u32,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) {
        let event = CharacterEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
                event_type: "character.experience_gained".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: CharacterEventKind::ExperienceGained(ExperienceGained {
                character_id: self.id,
                amount,
            }),
        };

        self.uncommitted_events.push(event);
    }
}

impl AggregateRoot for Character {
    type Event = CharacterEvent;

    fn aggregate_id(&self) -> Uuid {
        self.id
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn apply(&mut self, event: &Self::Event) {
        match &event.kind {
            CharacterEventKind::CharacterCreated(payload) => {
                self.name = Some(payload.name.clone());
            }
            CharacterEventKind::AttributeModified(payload) => {
                self.attributes
                    .insert(payload.attribute.clone(), payload.new_value);
            }
            CharacterEventKind::ExperienceGained(payload) => {
                self.experience += payload.amount;
            }
            CharacterEventKind::CharacterArchived(_) => {
                self.archived = true;
            }
        }
        self.version += 1;
    }

    fn uncommitted_events(&self) -> &[Self::Event] {
        &self.uncommitted_events
    }

    fn clear_uncommitted_events(&mut self) {
        self.uncommitted_events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use otherworlds_core::aggregate::AggregateRoot;
    use otherworlds_core::event::DomainEvent;
    use otherworlds_test_support::{FixedClock, MockRng};

    #[test]
    fn test_create_produces_character_created_event() {
        // Arrange
        let character_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut character = Character::new(character_id);

        // Act
        character.create("Alaric".to_owned(), correlation_id, &clock, &mut MockRng);

        // Assert
        let events = character.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "character.character_created");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, character_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            CharacterEventKind::CharacterCreated(payload) => {
                assert_eq!(payload.character_id, character_id);
                assert_eq!(payload.name, "Alaric");
            }
            other => panic!("expected CharacterCreated, got {other:?}"),
        }
    }

    #[test]
    fn test_modify_attribute_produces_attribute_modified_event() {
        // Arrange
        let character_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut character = Character::new(character_id);

        // Act
        character.modify_attribute(
            "strength".to_owned(),
            18,
            correlation_id,
            &clock,
            &mut MockRng,
        );

        // Assert
        let events = character.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "character.attribute_modified");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, character_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            CharacterEventKind::AttributeModified(payload) => {
                assert_eq!(payload.character_id, character_id);
                assert_eq!(payload.attribute, "strength");
                assert_eq!(payload.new_value, 18);
            }
            other => panic!("expected AttributeModified, got {other:?}"),
        }
    }

    #[test]
    fn test_apply_character_created_sets_name() {
        // Arrange
        let character_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut character = Character::new(character_id);
        let event = CharacterEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "character.character_created".to_owned(),
                aggregate_id: character_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: CharacterEventKind::CharacterCreated(CharacterCreated {
                character_id,
                name: "Alaric".to_owned(),
            }),
        };

        // Act
        character.apply(&event);

        // Assert
        assert_eq!(character.name, Some("Alaric".to_owned()));
        assert_eq!(character.version, 1);
    }

    #[test]
    fn test_apply_attribute_modified_updates_attributes() {
        // Arrange
        let character_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut character = Character::new(character_id);
        let event = CharacterEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "character.attribute_modified".to_owned(),
                aggregate_id: character_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: CharacterEventKind::AttributeModified(AttributeModified {
                character_id,
                attribute: "strength".to_owned(),
                new_value: 18,
            }),
        };

        // Act
        character.apply(&event);

        // Assert
        assert_eq!(character.attributes.get("strength"), Some(&18));
        assert_eq!(character.version, 1);
    }

    #[test]
    fn test_apply_experience_gained_accumulates_experience() {
        // Arrange
        let character_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut character = Character::new(character_id);
        let event1 = CharacterEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "character.experience_gained".to_owned(),
                aggregate_id: character_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: CharacterEventKind::ExperienceGained(ExperienceGained {
                character_id,
                amount: 100,
            }),
        };
        let event2 = CharacterEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "character.experience_gained".to_owned(),
                aggregate_id: character_id,
                sequence_number: 2,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: CharacterEventKind::ExperienceGained(ExperienceGained {
                character_id,
                amount: 150,
            }),
        };

        // Act
        character.apply(&event1);
        character.apply(&event2);

        // Assert
        assert_eq!(character.experience, 250);
        assert_eq!(character.version, 2);
    }

    #[test]
    fn test_award_experience_produces_experience_gained_event() {
        // Arrange
        let character_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut character = Character::new(character_id);

        // Act
        character.award_experience(250, correlation_id, &clock, &mut MockRng);

        // Assert
        let events = character.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "character.experience_gained");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, character_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            CharacterEventKind::ExperienceGained(payload) => {
                assert_eq!(payload.character_id, character_id);
                assert_eq!(payload.amount, 250);
            }
            other => panic!("expected ExperienceGained, got {other:?}"),
        }
    }

    #[test]
    fn test_archive_produces_character_archived_event() {
        // Arrange
        let character_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut character = Character::new(character_id);

        // Act
        let result = character.archive(correlation_id, &clock, &mut MockRng);

        // Assert
        assert!(result.is_ok());

        let events = character.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "character.character_archived");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, character_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            CharacterEventKind::CharacterArchived(payload) => {
                assert_eq!(payload.character_id, character_id);
            }
            other => panic!("expected CharacterArchived, got {other:?}"),
        }
    }

    #[test]
    fn test_apply_character_archived_sets_flag() {
        // Arrange
        let character_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut character = Character::new(character_id);
        let event = CharacterEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "character.character_archived".to_owned(),
                aggregate_id: character_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: CharacterEventKind::CharacterArchived(CharacterArchived { character_id }),
        };

        // Act
        character.apply(&event);

        // Assert
        assert!(character.archived);
        assert_eq!(character.version, 1);
    }

    #[test]
    fn test_archive_already_archived_returns_error() {
        // Arrange
        let character_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut character = Character::new(character_id);
        character.archived = true;

        // Act
        let result = character.archive(correlation_id, &clock, &mut MockRng);

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "character is already archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
