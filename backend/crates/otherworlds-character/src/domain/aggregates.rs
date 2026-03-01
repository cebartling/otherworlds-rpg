//! Aggregate roots for the Character Management context.

use std::collections::HashMap;

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::event::EventMetadata;
use uuid::Uuid;

use super::events::{
    AttributeModified, CharacterCreated, CharacterEvent, CharacterEventKind, ExperienceGained,
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
            uncommitted_events: Vec::new(),
        }
    }

    /// Returns the next sequence number for a new event.
    #[allow(clippy::cast_possible_wrap)]
    fn next_sequence_number(&self) -> i64 {
        self.version + self.uncommitted_events.len() as i64 + 1
    }

    /// Creates a character, producing a `CharacterCreated` event.
    pub fn create(&mut self, name: String, correlation_id: Uuid, clock: &dyn Clock) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // Requires extending DeterministicRng to support UUID generation and
        // threading &mut dyn DeterministicRng through all domain methods.
        let event = CharacterEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
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
    ) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // See TODO on `create()` for details.
        let event = CharacterEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
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

    /// Awards experience to a character, producing an `ExperienceGained` event.
    pub fn award_experience(&mut self, amount: u32, correlation_id: Uuid, clock: &dyn Clock) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // See TODO on `create()` for details.
        let event = CharacterEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
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
    use otherworlds_test_support::FixedClock;

    #[test]
    fn test_create_produces_character_created_event() {
        // Arrange
        let character_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut character = Character::new(character_id);

        // Act
        character.create("Alaric".to_owned(), correlation_id, &clock);

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
        character.modify_attribute("strength".to_owned(), 18, correlation_id, &clock);

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
    fn test_award_experience_produces_experience_gained_event() {
        // Arrange
        let character_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut character = Character::new(character_id);

        // Act
        character.award_experience(250, correlation_id, &clock);

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
}
