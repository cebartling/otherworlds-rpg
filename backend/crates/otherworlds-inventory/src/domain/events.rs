//! Domain events for the Inventory & Economy context.

use otherworlds_core::event::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Emitted when an item is added to an inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemAdded {
    /// The inventory identifier.
    pub inventory_id: Uuid,
    /// The item identifier.
    pub item_id: Uuid,
}

/// Emitted when an item is removed from an inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRemoved {
    /// The inventory identifier.
    pub inventory_id: Uuid,
    /// The item identifier.
    pub item_id: Uuid,
}

/// Emitted when an item is equipped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemEquipped {
    /// The inventory identifier.
    pub inventory_id: Uuid,
    /// The item identifier.
    pub item_id: Uuid,
}

/// Event type identifier for [`ItemAdded`].
pub const ITEM_ADDED_EVENT_TYPE: &str = "inventory.item_added";

/// Event type identifier for [`ItemRemoved`].
pub const ITEM_REMOVED_EVENT_TYPE: &str = "inventory.item_removed";

/// Event type identifier for [`ItemEquipped`].
pub const ITEM_EQUIPPED_EVENT_TYPE: &str = "inventory.item_equipped";

/// Event payload variants for the Inventory & Economy context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InventoryEventKind {
    /// An item has been added to the inventory.
    ItemAdded(ItemAdded),
    /// An item has been removed from the inventory.
    ItemRemoved(ItemRemoved),
    /// An item has been equipped.
    ItemEquipped(ItemEquipped),
}

/// Domain event envelope for the Inventory & Economy context.
#[derive(Debug, Clone)]
pub struct InventoryEvent {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// Event-specific payload.
    pub kind: InventoryEventKind,
}

impl DomainEvent for InventoryEvent {
    fn event_type(&self) -> &'static str {
        match &self.kind {
            InventoryEventKind::ItemAdded(_) => ITEM_ADDED_EVENT_TYPE,
            InventoryEventKind::ItemRemoved(_) => ITEM_REMOVED_EVENT_TYPE,
            InventoryEventKind::ItemEquipped(_) => ITEM_EQUIPPED_EVENT_TYPE,
        }
    }

    fn to_payload(&self) -> serde_json::Value {
        // Serialization of derived Serialize types to Value is infallible.
        serde_json::to_value(&self.kind).expect("InventoryEventKind serialization is infallible")
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
