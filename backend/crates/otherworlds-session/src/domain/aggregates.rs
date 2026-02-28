//! Aggregate roots for the Session & Progress context.

use uuid::Uuid;

/// The aggregate root for a campaign run.
#[derive(Debug)]
pub struct CampaignRun {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub version: i64,
}

impl CampaignRun {
    /// Creates a new campaign run.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self { id, version: 0 }
    }
}
