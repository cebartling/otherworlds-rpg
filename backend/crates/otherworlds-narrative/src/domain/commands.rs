//! Commands for the Narrative Orchestration context.

use otherworlds_core::command::Command;
use uuid::Uuid;

/// Command to advance the current narrative beat.
#[derive(Debug, Clone)]
pub struct AdvanceBeat {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The session this beat belongs to.
    pub session_id: Uuid,
}

impl Command for AdvanceBeat {
    fn command_type(&self) -> &'static str {
        "narrative.advance_beat"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}

/// Command to present a choice to the player.
#[derive(Debug, Clone)]
pub struct PresentChoice {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The session this choice belongs to.
    pub session_id: Uuid,
}

impl Command for PresentChoice {
    fn command_type(&self) -> &'static str {
        "narrative.present_choice"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}
