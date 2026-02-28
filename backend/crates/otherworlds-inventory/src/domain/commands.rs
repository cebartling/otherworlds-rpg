//! Commands for the Inventory & Economy context.

use otherworlds_core::command::Command;
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

impl Command for AddItem {
    fn command_type(&self) -> &'static str {
        "inventory.add_item"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
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

impl Command for RemoveItem {
    fn command_type(&self) -> &'static str {
        "inventory.remove_item"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
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

impl Command for EquipItem {
    fn command_type(&self) -> &'static str {
        "inventory.equip_item"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}
