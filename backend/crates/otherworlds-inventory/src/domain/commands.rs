//! Commands for the Inventory & Economy context.

use uuid::Uuid;

/// Command to add an item to an inventory.
#[derive(Debug, Clone)]
pub struct AddItem {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The inventory identifier.
    pub inventory_id: Uuid,
    /// The item identifier.
    pub item_id: Uuid,
}

/// Command to remove an item from an inventory.
#[derive(Debug, Clone)]
pub struct RemoveItem {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The inventory identifier.
    pub inventory_id: Uuid,
    /// The item identifier.
    pub item_id: Uuid,
}

/// Command to equip an item.
#[derive(Debug, Clone)]
pub struct EquipItem {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The inventory identifier.
    pub inventory_id: Uuid,
    /// The item identifier.
    pub item_id: Uuid,
}
