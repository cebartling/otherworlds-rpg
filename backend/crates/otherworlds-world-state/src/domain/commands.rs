//! Commands for the World State context.

use otherworlds_core::command::Command;
use uuid::Uuid;

/// Command to apply an effect to the world state.
#[derive(Debug, Clone)]
pub struct ApplyEffect {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The world snapshot identifier.
    pub world_id: Uuid,
}

impl Command for ApplyEffect {
    fn command_type(&self) -> &'static str {
        "world_state.apply_effect"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
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

impl Command for SetFlag {
    fn command_type(&self) -> &'static str {
        "world_state.set_flag"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
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

impl Command for UpdateDisposition {
    fn command_type(&self) -> &'static str {
        "world_state.update_disposition"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}
