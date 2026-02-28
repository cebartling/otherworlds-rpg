//! Domain event abstractions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Metadata attached to every domain event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique event identifier.
    pub event_id: Uuid,
    /// Type name for deserialization routing.
    pub event_type: String,
    /// Aggregate/stream this event belongs to.
    pub aggregate_id: Uuid,
    /// Monotonically increasing version within the aggregate stream.
    pub sequence_number: i64,
    /// Correlation ID for tracing a command through its effects.
    pub correlation_id: Uuid,
    /// Causation ID linking this event to the event/command that caused it.
    pub causation_id: Uuid,
    /// Timestamp of event creation.
    pub occurred_at: DateTime<Utc>,
}

/// Trait that all domain events implement.
pub trait DomainEvent: Send + Sync + std::fmt::Debug {
    /// Returns the event type name (used for serialization routing).
    fn event_type(&self) -> &'static str;

    /// Serializes the event payload to JSON.
    fn to_payload(&self) -> serde_json::Value;

    /// Returns the metadata for this event.
    fn metadata(&self) -> &EventMetadata;
}
