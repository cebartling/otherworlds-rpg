//! Aggregate roots for the Narrative Orchestration context.

use uuid::Uuid;

/// The aggregate root for a narrative session.
#[derive(Debug)]
pub struct NarrativeSession {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub version: i64,
}

impl NarrativeSession {
    /// Creates a new narrative session.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self { id, version: 0 }
    }
}
