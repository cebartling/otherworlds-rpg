//! Commands for the Narrative Orchestration context.

use otherworlds_core::command::Command;
use uuid::Uuid;

use super::value_objects::SceneData;

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

/// Command to enter a scene in the narrative session.
#[derive(Debug, Clone)]
pub struct EnterScene {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The session to enter the scene in.
    pub session_id: Uuid,
    /// The scene data to enter.
    pub scene_data: SceneData,
}

impl Command for EnterScene {
    fn command_type(&self) -> &'static str {
        "narrative.enter_scene"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}

/// Command to select a choice and transition to the next scene.
#[derive(Debug, Clone)]
pub struct SelectChoice {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The session to select the choice in.
    pub session_id: Uuid,
    /// The index of the choice to select.
    pub choice_index: usize,
    /// The target scene data to transition to.
    pub target_scene_data: SceneData,
}

impl Command for SelectChoice {
    fn command_type(&self) -> &'static str {
        "narrative.select_choice"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}

/// Command to archive (soft-delete) a narrative session.
#[derive(Debug, Clone)]
pub struct ArchiveSession {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The session identifier.
    pub session_id: Uuid,
}

impl Command for ArchiveSession {
    fn command_type(&self) -> &'static str {
        "narrative.archive_session"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}
