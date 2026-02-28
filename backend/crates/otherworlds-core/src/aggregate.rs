//! Aggregate root abstraction.

use uuid::Uuid;

use crate::event::DomainEvent;

/// Trait for aggregate roots that reconstitute from event history.
pub trait AggregateRoot: Send + Sync {
    /// The event type this aggregate produces and consumes.
    type Event: DomainEvent;

    /// Returns the aggregate identifier.
    fn aggregate_id(&self) -> Uuid;

    /// Returns the current version (number of events applied).
    fn version(&self) -> i64;

    /// Apply an event to mutate internal state (used during reconstitution).
    fn apply(&mut self, event: &Self::Event);

    /// Returns uncommitted events produced by command handling.
    fn uncommitted_events(&self) -> &[Self::Event];

    /// Clears uncommitted events after persistence.
    fn clear_uncommitted_events(&mut self);
}
