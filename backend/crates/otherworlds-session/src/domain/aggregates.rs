//! Aggregate roots for the Session & Progress context.

use otherworlds_core::aggregate::AggregateRoot;
use uuid::Uuid;

use super::events::SessionEvent;

/// The aggregate root for a campaign run.
#[derive(Debug)]
pub struct CampaignRun {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub version: i64,
    /// Uncommitted events pending persistence.
    uncommitted_events: Vec<SessionEvent>,
}

impl CampaignRun {
    /// Creates a new campaign run.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
            uncommitted_events: Vec::new(),
        }
    }
}

impl AggregateRoot for CampaignRun {
    type Event = SessionEvent;

    fn aggregate_id(&self) -> Uuid {
        self.id
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn apply(&mut self, _event: &Self::Event) {
        self.version += 1;
    }

    fn uncommitted_events(&self) -> &[Self::Event] {
        &self.uncommitted_events
    }

    fn clear_uncommitted_events(&mut self) {
        self.uncommitted_events.clear();
    }
}
