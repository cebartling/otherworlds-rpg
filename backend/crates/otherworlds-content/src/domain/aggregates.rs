//! Aggregate roots for the Content Authoring context.

use uuid::Uuid;

/// The aggregate root for a campaign.
#[derive(Debug)]
pub struct Campaign {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub version: i64,
}

impl Campaign {
    /// Creates a new campaign.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self { id, version: 0 }
    }
}
