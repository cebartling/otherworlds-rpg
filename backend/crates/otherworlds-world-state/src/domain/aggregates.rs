//! Aggregate roots for the World State context.

use uuid::Uuid;

/// The aggregate root for a world snapshot.
#[derive(Debug)]
pub struct WorldSnapshot {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub version: i64,
}

impl WorldSnapshot {
    /// Creates a new world snapshot.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self { id, version: 0 }
    }
}
