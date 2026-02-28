//! Domain events for the Rules & Resolution context.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Emitted when a player intent has been resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResolved {
    /// The resolution identifier.
    pub resolution_id: Uuid,
    /// The intent that was resolved.
    pub intent_id: Uuid,
}

/// Emitted when a skill/combat check is performed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckPerformed {
    /// The resolution identifier.
    pub resolution_id: Uuid,
    /// The check identifier.
    pub check_id: Uuid,
}

/// Emitted when effects are produced from a resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsProduced {
    /// The resolution identifier.
    pub resolution_id: Uuid,
}
