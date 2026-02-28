//! Aggregate roots for the Narrative Orchestration context.

use otherworlds_core::aggregate::AggregateRoot;
use uuid::Uuid;

use super::events::NarrativeEvent;

/// The aggregate root for a narrative session.
#[derive(Debug)]
pub struct NarrativeSession {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub version: i64,
    /// Uncommitted events pending persistence.
    uncommitted_events: Vec<NarrativeEvent>,
}

impl NarrativeSession {
    /// Creates a new narrative session.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
            uncommitted_events: Vec::new(),
        }
    }
}

impl AggregateRoot for NarrativeSession {
    type Event = NarrativeEvent;

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
