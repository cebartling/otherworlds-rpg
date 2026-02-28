//! Domain events for the World State context.

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
