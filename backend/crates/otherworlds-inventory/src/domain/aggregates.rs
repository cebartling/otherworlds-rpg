//! Aggregate roots for the Inventory & Economy context.

use uuid::Uuid;

/// The aggregate root for an inventory.
#[derive(Debug)]
pub struct Inventory {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub version: i64,
}

impl Inventory {
    /// Creates a new inventory.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self { id, version: 0 }
    }
}
