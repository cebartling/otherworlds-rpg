//! Aggregate roots for the World State context.

use std::collections::HashMap;

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::event::EventMetadata;
use uuid::Uuid;

use super::events::{
    DispositionUpdated, FlagSet, WorldFactChanged, WorldStateEvent, WorldStateEventKind,
};

/// The aggregate root for a world snapshot.
#[derive(Debug)]
pub struct WorldSnapshot {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub(crate) version: i64,
    /// Fact keys that have been applied.
    pub(crate) facts: Vec<String>,
    /// Boolean flags set in the world.
    pub(crate) flags: HashMap<String, bool>,
    /// Entity IDs whose dispositions have been updated.
    pub(crate) disposition_entity_ids: Vec<Uuid>,
    /// Uncommitted events pending persistence.
    uncommitted_events: Vec<WorldStateEvent>,
}

impl WorldSnapshot {
    /// Creates a new world snapshot.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
            facts: Vec::new(),
            flags: HashMap::new(),
            disposition_entity_ids: Vec::new(),
            uncommitted_events: Vec::new(),
        }
    }

    /// Returns the next sequence number for a new event.
    #[allow(clippy::cast_possible_wrap)]
    fn next_sequence_number(&self) -> i64 {
        self.version + self.uncommitted_events.len() as i64 + 1
    }

    /// Applies an effect to the world state, producing a `WorldFactChanged` event.
    pub fn apply_effect(&mut self, fact_key: String, correlation_id: Uuid, clock: &dyn Clock) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // Requires extending DeterministicRng to support UUID generation and
        // threading &mut dyn DeterministicRng through all domain methods.
        let event = WorldStateEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "world_state.world_fact_changed".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: WorldStateEventKind::WorldFactChanged(WorldFactChanged {
                world_id: self.id,
                fact_key,
            }),
        };

        self.uncommitted_events.push(event);
    }

    /// Sets a flag in the world state, producing a `FlagSet` event.
    pub fn set_flag(
        &mut self,
        flag_key: String,
        value: bool,
        correlation_id: Uuid,
        clock: &dyn Clock,
    ) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // See TODO on `apply_effect()` for details.
        let event = WorldStateEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "world_state.flag_set".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: WorldStateEventKind::FlagSet(FlagSet {
                world_id: self.id,
                flag_key,
                value,
            }),
        };

        self.uncommitted_events.push(event);
    }

    /// Updates a disposition in the world state, producing a `DispositionUpdated` event.
    pub fn update_disposition(&mut self, entity_id: Uuid, correlation_id: Uuid, clock: &dyn Clock) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // See TODO on `apply_effect()` for details.
        let event = WorldStateEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "world_state.disposition_updated".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: WorldStateEventKind::DispositionUpdated(DispositionUpdated {
                world_id: self.id,
                entity_id,
            }),
        };

        self.uncommitted_events.push(event);
    }
}

impl AggregateRoot for WorldSnapshot {
    type Event = WorldStateEvent;

    fn aggregate_id(&self) -> Uuid {
        self.id
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn apply(&mut self, event: &Self::Event) {
        match &event.kind {
            WorldStateEventKind::WorldFactChanged(payload) => {
                self.facts.push(payload.fact_key.clone());
            }
            WorldStateEventKind::FlagSet(payload) => {
                self.flags.insert(payload.flag_key.clone(), payload.value);
            }
            WorldStateEventKind::DispositionUpdated(payload) => {
                self.disposition_entity_ids.push(payload.entity_id);
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
    fn test_apply_effect_produces_world_fact_changed_event() {
        // Arrange
        let world_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut snapshot = WorldSnapshot::new(world_id);

        // Act
        snapshot.apply_effect("quest_complete".to_owned(), correlation_id, &clock);

        // Assert
        let events = snapshot.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "world_state.world_fact_changed");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, world_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            WorldStateEventKind::WorldFactChanged(payload) => {
                assert_eq!(payload.world_id, world_id);
                assert_eq!(payload.fact_key, "quest_complete");
            }
            other => panic!("expected WorldFactChanged, got {other:?}"),
        }
    }

    #[test]
    fn test_apply_world_fact_changed_pushes_fact() {
        // Arrange
        let world_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut snapshot = WorldSnapshot::new(world_id);
        let event = WorldStateEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "world_state.world_fact_changed".to_owned(),
                aggregate_id: world_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: WorldStateEventKind::WorldFactChanged(WorldFactChanged {
                world_id,
                fact_key: "quest_complete".to_owned(),
            }),
        };

        // Act
        snapshot.apply(&event);

        // Assert
        assert_eq!(snapshot.facts, vec!["quest_complete".to_owned()]);
        assert_eq!(snapshot.version, 1);
    }

    #[test]
    fn test_apply_flag_set_inserts_flag() {
        // Arrange
        let world_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut snapshot = WorldSnapshot::new(world_id);
        let event = WorldStateEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "world_state.flag_set".to_owned(),
                aggregate_id: world_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: WorldStateEventKind::FlagSet(FlagSet {
                world_id,
                flag_key: "door_unlocked".to_owned(),
                value: true,
            }),
        };

        // Act
        snapshot.apply(&event);

        // Assert
        assert_eq!(snapshot.flags.get("door_unlocked"), Some(&true));
        assert_eq!(snapshot.version, 1);
    }

    #[test]
    fn test_apply_disposition_updated_pushes_entity_id() {
        // Arrange
        let world_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut snapshot = WorldSnapshot::new(world_id);
        let event = WorldStateEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "world_state.disposition_updated".to_owned(),
                aggregate_id: world_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: WorldStateEventKind::DispositionUpdated(DispositionUpdated {
                world_id,
                entity_id,
            }),
        };

        // Act
        snapshot.apply(&event);

        // Assert
        assert_eq!(snapshot.disposition_entity_ids, vec![entity_id]);
        assert_eq!(snapshot.version, 1);
    }

    #[test]
    fn test_set_flag_produces_flag_set_event() {
        // Arrange
        let world_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut snapshot = WorldSnapshot::new(world_id);

        // Act
        snapshot.set_flag("door_unlocked".to_owned(), true, correlation_id, &clock);

        // Assert
        let events = snapshot.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "world_state.flag_set");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, world_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            WorldStateEventKind::FlagSet(payload) => {
                assert_eq!(payload.world_id, world_id);
                assert_eq!(payload.flag_key, "door_unlocked");
                assert!(payload.value);
            }
            other => panic!("expected FlagSet, got {other:?}"),
        }
    }

    #[test]
    fn test_set_flag_produces_flag_set_event_with_false_value() {
        // Arrange
        let world_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut snapshot = WorldSnapshot::new(world_id);

        // Act
        snapshot.set_flag("door_unlocked".to_owned(), false, correlation_id, &clock);

        // Assert
        let events = snapshot.uncommitted_events();
        assert_eq!(events.len(), 1);

        match &events[0].kind {
            WorldStateEventKind::FlagSet(payload) => {
                assert_eq!(payload.world_id, world_id);
                assert_eq!(payload.flag_key, "door_unlocked");
                assert!(!payload.value);
            }
            other => panic!("expected FlagSet, got {other:?}"),
        }
    }

    #[test]
    fn test_update_disposition_produces_disposition_updated_event() {
        // Arrange
        let world_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut snapshot = WorldSnapshot::new(world_id);

        // Act
        snapshot.update_disposition(entity_id, correlation_id, &clock);

        // Assert
        let events = snapshot.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "world_state.disposition_updated");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, world_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            WorldStateEventKind::DispositionUpdated(payload) => {
                assert_eq!(payload.world_id, world_id);
                assert_eq!(payload.entity_id, entity_id);
            }
            other => panic!("expected DispositionUpdated, got {other:?}"),
        }
    }
}
