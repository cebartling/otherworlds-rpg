//! Domain events for the World State context.

use otherworlds_core::event::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Emitted when a world fact changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFactChanged {
    /// The world snapshot identifier.
    pub world_id: Uuid,
    /// The fact key that changed.
    pub fact_key: String,
}

/// Emitted when a flag is set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagSet {
    /// The world snapshot identifier.
    pub world_id: Uuid,
    /// The flag key.
    pub flag_key: String,
    /// The flag value.
    pub value: bool,
}

/// Emitted when a disposition is updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispositionUpdated {
    /// The world snapshot identifier.
    pub world_id: Uuid,
    /// The entity whose disposition changed.
    pub entity_id: Uuid,
}

/// Event payload variants for the World State context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldStateEventKind {
    /// A world fact has changed.
    WorldFactChanged(WorldFactChanged),
    /// A flag has been set.
    FlagSet(FlagSet),
    /// A disposition has been updated.
    DispositionUpdated(DispositionUpdated),
}

/// Domain event envelope for the World State context.
#[derive(Debug, Clone)]
pub struct WorldStateEvent {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// Event-specific payload.
    pub kind: WorldStateEventKind,
}

impl DomainEvent for WorldStateEvent {
    fn event_type(&self) -> &'static str {
        match &self.kind {
            WorldStateEventKind::WorldFactChanged(_) => "world_state.world_fact_changed",
            WorldStateEventKind::FlagSet(_) => "world_state.flag_set",
            WorldStateEventKind::DispositionUpdated(_) => "world_state.disposition_updated",
        }
    }

    fn to_payload(&self) -> serde_json::Value {
        // Serialization of derived Serialize types to Value is infallible.
        serde_json::to_value(&self.kind).expect("WorldStateEventKind serialization is infallible")
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
