//! Aggregate roots for the Character Management context.

use uuid::Uuid;

/// The aggregate root for a character.
#[derive(Debug)]
pub struct Character {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub version: i64,
}

impl Character {
    /// Creates a new character.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self { id, version: 0 }
    }
}
