//! Aggregate roots for the World State context.

use otherworlds_core::aggregate::AggregateRoot;
use uuid::Uuid;

use super::events::WorldStateEvent;

/// The aggregate root for a world snapshot.
#[derive(Debug)]
pub struct WorldSnapshot {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub version: i64,
    /// Uncommitted events pending persistence.
    uncommitted_events: Vec<WorldStateEvent>,
}

impl WorldSnapshot {
    /// Creates a new world snapshot.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
            uncommitted_events: Vec::new(),
        }
    }
}

impl AggregateRoot for WorldSnapshot {
    type Event = WorldStateEvent;

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
