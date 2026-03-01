//! Query handlers for the Inventory & Economy context.
//!
//! This module contains query handlers that reconstitute aggregates
//! from stored events and return read-only view DTOs.

use otherworlds_core::error::DomainError;
use otherworlds_core::repository::EventRepository;
use serde::Serialize;
use uuid::Uuid;

use crate::application::command_handlers;

/// Read-only view of an inventory aggregate.
#[derive(Debug, Serialize)]
pub struct InventoryView {
    /// The inventory identifier.
    pub inventory_id: Uuid,
    /// Items currently in the inventory (sorted for determinism).
    pub items: Vec<Uuid>,
    /// Current version (event count).
    pub version: i64,
}

/// Retrieves an inventory by its aggregate ID.
///
/// Loads all stored events for the aggregate, reconstitutes the inventory,
/// and returns a serializable view.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the ID.
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub async fn get_inventory_by_id(
    inventory_id: Uuid,
    repo: &dyn EventRepository,
) -> Result<InventoryView, DomainError> {
    let stored_events = repo.load_events(inventory_id).await?;
    if stored_events.is_empty() {
        return Err(DomainError::AggregateNotFound(inventory_id));
    }
    let inventory = command_handlers::reconstitute(inventory_id, &stored_events)?;
    let mut items: Vec<Uuid> = inventory.items.iter().copied().collect();
    items.sort();
    Ok(InventoryView {
        inventory_id,
        items,
        version: inventory.version,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use uuid::Uuid;

    use crate::application::query_handlers::get_inventory_by_id;
    use crate::domain::events::{ITEM_ADDED_EVENT_TYPE, InventoryEventKind, ItemAdded};
    use otherworlds_test_support::{EmptyEventRepository, RecordingEventRepository};

    #[tokio::test]
    async fn test_get_inventory_by_id_returns_view_with_items() {
        // Arrange
        let inventory_id = Uuid::new_v4();
        let item_id_a = Uuid::new_v4();
        let item_id_b = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: inventory_id,
                event_type: ITEM_ADDED_EVENT_TYPE.to_owned(),
                payload: serde_json::to_value(InventoryEventKind::ItemAdded(ItemAdded {
                    inventory_id,
                    item_id: item_id_a,
                }))
                .unwrap(),
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: inventory_id,
                event_type: ITEM_ADDED_EVENT_TYPE.to_owned(),
                payload: serde_json::to_value(InventoryEventKind::ItemAdded(ItemAdded {
                    inventory_id,
                    item_id: item_id_b,
                }))
                .unwrap(),
                sequence_number: 2,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
        ];
        let repo = RecordingEventRepository::new(Ok(events));

        // Act
        let view = get_inventory_by_id(inventory_id, &repo).await.unwrap();

        // Assert
        assert_eq!(view.inventory_id, inventory_id);
        assert_eq!(view.version, 2);
        assert_eq!(view.items.len(), 2);
        assert!(view.items.contains(&item_id_a));
        assert!(view.items.contains(&item_id_b));
    }

    #[tokio::test]
    async fn test_get_inventory_by_id_returns_not_found_when_no_events() {
        // Arrange
        let inventory_id = Uuid::new_v4();
        let repo = EmptyEventRepository;

        // Act
        let result = get_inventory_by_id(inventory_id, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, inventory_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }
}
