//! Aggregate roots for the Rules & Resolution context.

use uuid::Uuid;

/// The aggregate root for a resolution.
#[derive(Debug)]
pub struct Resolution {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub version: i64,
}

impl Resolution {
    /// Creates a new resolution.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self { id, version: 0 }
    }
}
