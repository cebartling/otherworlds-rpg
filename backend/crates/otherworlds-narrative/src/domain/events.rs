//! Domain events for the Narrative Orchestration context.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Emitted when a new scene begins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneStarted {
    /// The scene identifier.
    pub scene_id: Uuid,
}

/// Emitted when the narrative advances to the next beat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatAdvanced {
    /// The session this beat belongs to.
    pub session_id: Uuid,
    /// The new beat identifier.
    pub beat_id: Uuid,
}

/// Emitted when a choice is presented to the player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoicePresented {
    /// The session this choice belongs to.
    pub session_id: Uuid,
    /// The choice identifier.
    pub choice_id: Uuid,
}
