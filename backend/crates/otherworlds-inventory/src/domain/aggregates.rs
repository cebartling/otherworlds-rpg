//! Aggregate roots for the Inventory & Economy context.

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::event::EventMetadata;
use uuid::Uuid;

use super::events::{
    ITEM_ADDED_EVENT_TYPE, ITEM_EQUIPPED_EVENT_TYPE, ITEM_REMOVED_EVENT_TYPE, InventoryEvent,
    InventoryEventKind, ItemAdded, ItemEquipped, ItemRemoved,
};

/// The aggregate root for an inventory.
#[derive(Debug)]
pub struct Inventory {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub version: i64,
    /// Uncommitted events pending persistence.
    uncommitted_events: Vec<InventoryEvent>,
}

impl Inventory {
    /// Creates a new inventory.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
            uncommitted_events: Vec::new(),
        }
    }

    /// Returns the next sequence number for a new event.
    #[allow(clippy::cast_possible_wrap)]
    fn next_sequence_number(&self) -> i64 {
        self.version + self.uncommitted_events.len() as i64 + 1
    }

    /// Adds an item to the inventory, producing an `ItemAdded` event.
    pub fn add_item(&mut self, item_id: Uuid, correlation_id: Uuid, clock: &dyn Clock) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // Requires extending DeterministicRng to support UUID generation and
        // threading &mut dyn DeterministicRng through all domain methods.
        let event = InventoryEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: ITEM_ADDED_EVENT_TYPE.to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: InventoryEventKind::ItemAdded(ItemAdded {
                inventory_id: self.id,
                item_id,
            }),
        };

        self.uncommitted_events.push(event);
    }

    /// Removes an item from the inventory, producing an `ItemRemoved` event.
    pub fn remove_item(&mut self, item_id: Uuid, correlation_id: Uuid, clock: &dyn Clock) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // See TODO on `add_item()` for details.
        let event = InventoryEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: ITEM_REMOVED_EVENT_TYPE.to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: InventoryEventKind::ItemRemoved(ItemRemoved {
                inventory_id: self.id,
                item_id,
            }),
        };

        self.uncommitted_events.push(event);
    }

    /// Equips an item, producing an `ItemEquipped` event.
    pub fn equip_item(&mut self, item_id: Uuid, correlation_id: Uuid, clock: &dyn Clock) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // See TODO on `add_item()` for details.
        let event = InventoryEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: ITEM_EQUIPPED_EVENT_TYPE.to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: InventoryEventKind::ItemEquipped(ItemEquipped {
                inventory_id: self.id,
                item_id,
            }),
        };

        self.uncommitted_events.push(event);
    }
}

impl AggregateRoot for Inventory {
    type Event = InventoryEvent;

    fn aggregate_id(&self) -> Uuid {
        self.id
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn apply(&mut self, _event: &Self::Event) {
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
    use chrono::{DateTime, TimeZone, Utc};
    use otherworlds_core::aggregate::AggregateRoot;
    use otherworlds_core::clock::Clock;
    use otherworlds_core::event::DomainEvent;

    #[derive(Debug)]
    struct FixedClock(DateTime<Utc>);

    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
        }
    }

    #[test]
    fn test_add_item_produces_item_added_event() {
        // Arrange
        let inventory_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut inventory = Inventory::new(inventory_id);

        // Act
        inventory.add_item(item_id, correlation_id, &clock);

        // Assert
        let events = inventory.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "inventory.item_added");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, inventory_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            InventoryEventKind::ItemAdded(payload) => {
                assert_eq!(payload.inventory_id, inventory_id);
                assert_eq!(payload.item_id, item_id);
            }
            other => panic!("expected ItemAdded, got {other:?}"),
        }
    }

    #[test]
    fn test_remove_item_produces_item_removed_event() {
        // Arrange
        let inventory_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut inventory = Inventory::new(inventory_id);

        // Act
        inventory.remove_item(item_id, correlation_id, &clock);

        // Assert
        let events = inventory.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "inventory.item_removed");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, inventory_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            InventoryEventKind::ItemRemoved(payload) => {
                assert_eq!(payload.inventory_id, inventory_id);
                assert_eq!(payload.item_id, item_id);
            }
            other => panic!("expected ItemRemoved, got {other:?}"),
        }
    }

    #[test]
    fn test_equip_item_produces_item_equipped_event() {
        // Arrange
        let inventory_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut inventory = Inventory::new(inventory_id);

        // Act
        inventory.equip_item(item_id, correlation_id, &clock);

        // Assert
        let events = inventory.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "inventory.item_equipped");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, inventory_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            InventoryEventKind::ItemEquipped(payload) => {
                assert_eq!(payload.inventory_id, inventory_id);
                assert_eq!(payload.item_id, item_id);
            }
            other => panic!("expected ItemEquipped, got {other:?}"),
        }
    }
}
