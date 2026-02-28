//! Commands for the Narrative Orchestration context.

use uuid::Uuid;

/// Command to advance the current narrative beat.
#[derive(Debug, Clone)]
pub struct AdvanceBeat {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The session this beat belongs to.
    pub session_id: Uuid,
}

/// Command to present a choice to the player.
#[derive(Debug, Clone)]
pub struct PresentChoice {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The session this choice belongs to.
    pub session_id: Uuid,
}
