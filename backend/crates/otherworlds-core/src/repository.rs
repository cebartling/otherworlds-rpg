//! Event repository abstraction.

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;

/// Stored representation of a domain event.
#[derive(Debug, Clone)]
pub struct StoredEvent {
    /// Unique event identifier.
    pub event_id: Uuid,
    /// Aggregate this event belongs to.
    pub aggregate_id: Uuid,
    /// Event type name for deserialization routing.
    pub event_type: String,
    /// Serialized event payload.
    pub payload: serde_json::Value,
    /// Sequence number within the aggregate stream.
    pub sequence_number: i64,
    /// Correlation ID for tracing.
    pub correlation_id: Uuid,
    /// Causation ID linking to the causing event/command.
    pub causation_id: Uuid,
    /// Timestamp of event creation.
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

/// Repository trait for loading and appending domain events.
#[async_trait]
pub trait EventRepository: Send + Sync {
    /// Load all events for a given aggregate, ordered by sequence number.
    async fn load_events(&self, aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError>;

    /// Append new events to an aggregate stream with optimistic concurrency.
    /// `expected_version` is the last known sequence number.
    async fn append_events(
        &self,
        aggregate_id: Uuid,
        expected_version: i64,
        events: Vec<StoredEvent>,
    ) -> Result<(), DomainError>;
}
