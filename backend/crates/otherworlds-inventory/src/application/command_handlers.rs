//! Command handlers for the Inventory & Economy context.
//!
//! This module contains application-level command handler functions that
//! orchestrate domain logic: load aggregate, execute command, persist events.

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::event::DomainEvent;
use otherworlds_core::repository::{EventRepository, StoredEvent};
use uuid::Uuid;

use crate::domain::aggregates::Inventory;
use crate::domain::commands::{AddItem, EquipItem, RemoveItem};
use crate::domain::events::{InventoryEvent, InventoryEventKind};

/// Result of a successfully handled command.
#[derive(Debug)]
pub struct InventoryCommandResult {
    /// The aggregate ID affected by the command.
    pub aggregate_id: Uuid,
    /// The stored events produced and persisted.
    pub stored_events: Vec<StoredEvent>,
}

fn to_stored_event(event: &InventoryEvent) -> StoredEvent {
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

/// Reconstitutes an `Inventory` from stored events.
///
/// # Errors
///
/// Returns `DomainError::Infrastructure` if event deserialization fails.
fn reconstitute(
    inventory_id: Uuid,
    existing_events: &[StoredEvent],
) -> Result<Inventory, DomainError> {
    let mut inventory = Inventory::new(inventory_id);
    for stored in existing_events {
        let kind: InventoryEventKind =
            serde_json::from_value(stored.payload.clone()).map_err(|e| {
                DomainError::Infrastructure(format!("event deserialization failed: {e}"))
            })?;
        let event = InventoryEvent {
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
        inventory.apply(&event);
    }
    Ok(inventory)
}

/// Handles the `AddItem` command: loads the aggregate, adds the item, and
/// persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_add_item(
    command: &AddItem,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<InventoryCommandResult, DomainError> {
    let existing_events = repo.load_events(command.inventory_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.inventory_id));
    }
    let mut inventory = reconstitute(command.inventory_id, &existing_events)?;

    inventory.add_item(command.item_id, command.correlation_id, clock);

    let stored_events: Vec<StoredEvent> = inventory
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.inventory_id, inventory.version(), &stored_events)
        .await?;

    Ok(InventoryCommandResult {
        aggregate_id: command.inventory_id,
        stored_events,
    })
}

/// Handles the `RemoveItem` command: loads the aggregate, removes the item,
/// and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_remove_item(
    command: &RemoveItem,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<InventoryCommandResult, DomainError> {
    let existing_events = repo.load_events(command.inventory_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.inventory_id));
    }
    let mut inventory = reconstitute(command.inventory_id, &existing_events)?;

    inventory.remove_item(command.item_id, command.correlation_id, clock)?;

    let stored_events: Vec<StoredEvent> = inventory
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.inventory_id, inventory.version(), &stored_events)
        .await?;

    Ok(InventoryCommandResult {
        aggregate_id: command.inventory_id,
        stored_events,
    })
}

/// Handles the `EquipItem` command: loads the aggregate, equips the item,
/// and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_equip_item(
    command: &EquipItem,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<InventoryCommandResult, DomainError> {
    let existing_events = repo.load_events(command.inventory_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.inventory_id));
    }
    let mut inventory = reconstitute(command.inventory_id, &existing_events)?;

    inventory.equip_item(command.item_id, command.correlation_id, clock)?;

    let stored_events: Vec<StoredEvent> = inventory
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.inventory_id, inventory.version(), &stored_events)
        .await?;

    Ok(InventoryCommandResult {
        aggregate_id: command.inventory_id,
        stored_events,
    })
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
        handle_add_item, handle_equip_item, handle_remove_item,
    };
    use crate::domain::commands::{AddItem, EquipItem, RemoveItem};
    use crate::domain::events::{InventoryEventKind, ItemAdded};

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

    fn dummy_stored_event(
        aggregate_id: Uuid,
        item_id: Uuid,
        fixed_now: DateTime<Utc>,
    ) -> StoredEvent {
        StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            event_type: "inventory.item_added".to_owned(),
            payload: serde_json::to_value(InventoryEventKind::ItemAdded(ItemAdded {
                inventory_id: aggregate_id,
                item_id,
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }
    }

    #[tokio::test]
    async fn test_handle_add_item_persists_item_added_event() {
        // Arrange
        let inventory_id = Uuid::new_v4();
        let existing_item_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let existing_event = dummy_stored_event(inventory_id, existing_item_id, fixed_now);
        let repo = MockEventRepository::new(Ok(vec![existing_event]));

        let command = AddItem {
            correlation_id,
            inventory_id,
            item_id,
        };

        // Act
        let result = handle_add_item(&command, &clock, &repo).await;

        // Assert
        let cmd_result = result.unwrap();
        assert_eq!(cmd_result.aggregate_id, inventory_id);
        assert_eq!(cmd_result.stored_events.len(), 1);

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, inventory_id);
        assert_eq!(*expected_version, 1);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "inventory.item_added");
        assert_eq!(stored.aggregate_id, inventory_id);
        assert_eq!(stored.sequence_number, 2);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);

        let payload: InventoryEventKind = serde_json::from_value(stored.payload.clone()).unwrap();
        match payload {
            InventoryEventKind::ItemAdded(added) => {
                assert_eq!(added.inventory_id, inventory_id);
                assert_eq!(added.item_id, item_id);
            }
            other => panic!("expected ItemAdded payload, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_remove_item_persists_item_removed_event() {
        // Arrange — the existing event must add the item we intend to remove.
        let inventory_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let existing_event = dummy_stored_event(inventory_id, item_id, fixed_now);
        let repo = MockEventRepository::new(Ok(vec![existing_event]));

        let command = RemoveItem {
            correlation_id,
            inventory_id,
            item_id,
        };

        // Act
        let result = handle_remove_item(&command, &clock, &repo).await;

        // Assert
        let cmd_result = result.unwrap();
        assert_eq!(cmd_result.aggregate_id, inventory_id);
        assert_eq!(cmd_result.stored_events.len(), 1);

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, inventory_id);
        assert_eq!(*expected_version, 1);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "inventory.item_removed");
        assert_eq!(stored.aggregate_id, inventory_id);
        assert_eq!(stored.sequence_number, 2);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);

        let payload: InventoryEventKind = serde_json::from_value(stored.payload.clone()).unwrap();
        match payload {
            InventoryEventKind::ItemRemoved(removed) => {
                assert_eq!(removed.inventory_id, inventory_id);
                assert_eq!(removed.item_id, item_id);
            }
            other => panic!("expected ItemRemoved payload, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_equip_item_persists_item_equipped_event() {
        // Arrange — the existing event must add the item we intend to equip.
        let inventory_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let existing_event = dummy_stored_event(inventory_id, item_id, fixed_now);
        let repo = MockEventRepository::new(Ok(vec![existing_event]));

        let command = EquipItem {
            correlation_id,
            inventory_id,
            item_id,
        };

        // Act
        let result = handle_equip_item(&command, &clock, &repo).await;

        // Assert
        let cmd_result = result.unwrap();
        assert_eq!(cmd_result.aggregate_id, inventory_id);
        assert_eq!(cmd_result.stored_events.len(), 1);

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, inventory_id);
        assert_eq!(*expected_version, 1);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "inventory.item_equipped");
        assert_eq!(stored.aggregate_id, inventory_id);
        assert_eq!(stored.sequence_number, 2);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);

        let payload: InventoryEventKind = serde_json::from_value(stored.payload.clone()).unwrap();
        match payload {
            InventoryEventKind::ItemEquipped(equipped) => {
                assert_eq!(equipped.inventory_id, inventory_id);
                assert_eq!(equipped.item_id, item_id);
            }
            other => panic!("expected ItemEquipped payload, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_add_item_returns_error_when_inventory_not_found() {
        // Arrange
        let inventory_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = MockEventRepository::new(Ok(Vec::new()));

        let command = AddItem {
            correlation_id,
            inventory_id,
            item_id,
        };

        // Act
        let result = handle_add_item(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, inventory_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_remove_item_returns_error_when_inventory_not_found() {
        // Arrange
        let inventory_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = MockEventRepository::new(Ok(Vec::new()));

        let command = RemoveItem {
            correlation_id,
            inventory_id,
            item_id,
        };

        // Act
        let result = handle_remove_item(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, inventory_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_remove_item_returns_error_when_item_not_in_inventory() {
        // Arrange — inventory exists but does not contain the item being removed.
        let inventory_id = Uuid::new_v4();
        let existing_item_id = Uuid::new_v4();
        let missing_item_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let existing_event = dummy_stored_event(inventory_id, existing_item_id, fixed_now);
        let repo = MockEventRepository::new(Ok(vec![existing_event]));

        let command = RemoveItem {
            correlation_id,
            inventory_id,
            item_id: missing_item_id,
        };

        // Act
        let result = handle_remove_item(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains(&missing_item_id.to_string()));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_equip_item_returns_error_when_item_not_in_inventory() {
        // Arrange — inventory exists but does not contain the item being equipped.
        let inventory_id = Uuid::new_v4();
        let existing_item_id = Uuid::new_v4();
        let missing_item_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let existing_event = dummy_stored_event(inventory_id, existing_item_id, fixed_now);
        let repo = MockEventRepository::new(Ok(vec![existing_event]));

        let command = EquipItem {
            correlation_id,
            inventory_id,
            item_id: missing_item_id,
        };

        // Act
        let result = handle_equip_item(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains(&missing_item_id.to_string()));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
