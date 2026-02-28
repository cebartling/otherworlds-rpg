//! Domain events for the Inventory & Economy context.

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
