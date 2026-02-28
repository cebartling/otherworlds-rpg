//! Domain events for the Narrative Orchestration context.

use otherworlds_core::event::{DomainEvent, EventMetadata};
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

/// Event payload variants for the Narrative Orchestration context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NarrativeEventKind {
    /// A new scene has started.
    SceneStarted(SceneStarted),
    /// The narrative beat has advanced.
    BeatAdvanced(BeatAdvanced),
    /// A choice has been presented to the player.
    ChoicePresented(ChoicePresented),
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
