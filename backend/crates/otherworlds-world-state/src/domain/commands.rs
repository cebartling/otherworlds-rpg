//! Commands for the World State context.

use uuid::Uuid;

/// Command to apply an effect to the world state.
#[derive(Debug, Clone)]
pub struct ApplyEffect {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The world snapshot identifier.
    pub world_id: Uuid,
}

/// Command to set a flag in the world state.
#[derive(Debug, Clone)]
pub struct SetFlag {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The world snapshot identifier.
    pub world_id: Uuid,
    /// The flag key.
    pub flag_key: String,
    /// The flag value.
    pub value: bool,
}

/// Command to update a disposition.
#[derive(Debug, Clone)]
pub struct UpdateDisposition {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The world snapshot identifier.
    pub world_id: Uuid,
    /// The entity whose disposition to update.
    pub entity_id: Uuid,
}
