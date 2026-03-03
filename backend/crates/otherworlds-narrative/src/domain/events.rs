//! Domain events for the Narrative Orchestration context.

use otherworlds_core::event::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::value_objects::ChoiceOption;

/// Emitted when a new scene begins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneStarted {
    /// The session this scene belongs to.
    pub session_id: Uuid,
    /// The scene identifier (author-defined).
    pub scene_id: String,
    /// The narrative text displayed to the player.
    pub narrative_text: String,
    /// The choices available in this scene.
    pub choices: Vec<ChoiceOption>,
    /// NPC references present in this scene.
    pub npc_refs: Vec<String>,
}

/// Emitted when a player selects a choice, transitioning between scenes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceSelected {
    /// The session this choice belongs to.
    pub session_id: Uuid,
    /// The label of the selected choice.
    pub choice_label: String,
    /// The scene the player was in when they chose.
    pub from_scene_id: String,
    /// The scene the player transitions to.
    pub to_scene_id: String,
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

/// Emitted when a session is archived (soft-deleted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArchived {
    /// The session identifier.
    pub session_id: Uuid,
}

/// Event payload variants for the Narrative Orchestration context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NarrativeEventKind {
    /// A new scene has started.
    SceneStarted(SceneStarted),
    /// The narrative beat has advanced.
    BeatAdvanced(BeatAdvanced),
    /// A choice has been presented to the player.
    ChoicePresented(ChoicePresented),
    /// A player has selected a choice, transitioning between scenes.
    ChoiceSelected(ChoiceSelected),
    /// A session has been archived (soft-deleted).
    SessionArchived(SessionArchived),
}

/// Domain event envelope for the Narrative Orchestration context.
#[derive(Debug, Clone)]
pub struct NarrativeEvent {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// Event-specific payload.
    pub kind: NarrativeEventKind,
}

impl DomainEvent for NarrativeEvent {
    fn event_type(&self) -> &'static str {
        match &self.kind {
            NarrativeEventKind::SceneStarted(_) => "narrative.scene_started",
            NarrativeEventKind::BeatAdvanced(_) => "narrative.beat_advanced",
            NarrativeEventKind::ChoicePresented(_) => "narrative.choice_presented",
            NarrativeEventKind::ChoiceSelected(_) => "narrative.choice_selected",
            NarrativeEventKind::SessionArchived(_) => "narrative.session_archived",
        }
    }

    fn to_payload(&self) -> serde_json::Value {
        // Serialization of derived Serialize types to Value is infallible.
        serde_json::to_value(&self.kind).expect("NarrativeEventKind serialization is infallible")
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
